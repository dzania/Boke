/**
 * Feed HTML to sanitized hast, ready for rendering.
 *
 * Feed content is HTML, so it is parsed as HTML once. It used to go through a Markdown parser
 * first, which needed whitespace hacks and broke <pre> indentation.
 */

import type { Element, Root, Text } from "hast";
import rehypeParse from "rehype-parse";
import rehypeSanitize, { defaultSchema, type Options as Schema } from "rehype-sanitize";
import { rehypeInlineCodeProperty } from "react-shiki";
import { unified } from "unified";
import { SKIP, visit } from "unist-util-visit";
import { resolveTreeUrls } from "../../lib/rehype/resolveUrls";

const SVG_TAGS = [
  "svg",
  "g",
  "path",
  "circle",
  "ellipse",
  "rect",
  "line",
  "polyline",
  "polygon",
  "text",
  "tspan",
  "defs",
  "use",
  "symbol",
  "clipPath",
  "linearGradient",
  "radialGradient",
  "stop",
  "marker",
  "pattern",
  "mask",
  "image",
  "foreignObject",
  "desc",
  "title",
  "animate",
  "animateTransform",
  "animateMotion",
  "set",
];

const SVG_ATTRS = [
  "viewBox",
  "xmlns",
  "fill",
  "stroke",
  "strokeWidth",
  "strokeLinecap",
  "strokeLinejoin",
  "strokeDasharray",
  "strokeDashoffset",
  "opacity",
  "d",
  "cx",
  "cy",
  "r",
  "rx",
  "ry",
  "x",
  "y",
  "x1",
  "y1",
  "x2",
  "y2",
  "width",
  "height",
  "transform",
  "points",
  "textAnchor",
  "dominantBaseline",
  "fontFamily",
  "fontSize",
  "fontWeight",
  "letterSpacing",
  "dx",
  "dy",
  "id",
  "clipPath",
  "clipRule",
  "fillRule",
  "fillOpacity",
  "strokeOpacity",
  "markerEnd",
  "markerStart",
  "markerMid",
  "gradientUnits",
  "gradientTransform",
  "offset",
  "stopColor",
  "stopOpacity",
  "patternUnits",
  "patternTransform",
  "href",
  "xlinkHref",
  "preserveAspectRatio",
  "role",
  "ariaLabel",
  "ariaHidden",
];

const sanitizeSchema: Schema = {
  ...defaultSchema,
  tagNames: [
    ...(defaultSchema.tagNames || []),
    ...SVG_TAGS,
    "figure",
    "figcaption",
    "picture",
    "source",
    "video",
    "audio",
    "iframe",
  ],
  attributes: {
    ...defaultSchema.attributes,
    "*": [...(defaultSchema.attributes?.["*"] || []), "className", "style"],
    code: ["className", "inline", "dataLang", "dataLanguage"],
    pre: ["className", "dataLang", "dataLanguage"],
    // The default schema keeps only `src` on images and `srcSet` on sources, which drops
    // responsive candidates and breaks <picture>.
    img: [...(defaultSchema.attributes?.img || []), "srcSet", "sizes"],
    iframe: ["src", "width", "height", "frameBorder", "allow", "allowFullScreen"],
    video: ["src", "width", "height", "controls", "poster"],
    audio: ["src", "controls"],
    source: ["src", "srcSet", "sizes", "type"],
    ...Object.fromEntries(SVG_TAGS.map((tag) => [tag, SVG_ATTRS])),
  },
};

/**
 * Rehype plugin: strip inline styles from <pre> and its descendants.
 * Blogs (e.g. Astro) embed theme-specific colors as inline styles which override our styling.
 */
function rehypeCleanCodeBlocks() {
  return (tree: Root) => {
    visit(tree, "element", (node: Element) => {
      if (node.tagName !== "pre") return;
      delete node.properties.style;
      delete node.properties.className;
      visit(node, "element", (child: Element) => {
        delete child.properties.style;
      });
    });
  };
}

/**
 * Rehype plugin: copy data-language from <pre> to its child <code> as a className.
 * Handles sites like steveklabnik.com (Astro) that put language info on <pre data-language="rust">.
 */
function rehypeCopyLanguage() {
  return (tree: Root) => {
    visit(tree, "element", (node: Element) => {
      if (node.tagName !== "pre") return;

      const lang =
        (node.properties?.dataLanguage as string) || (node.properties?.dataLang as string);
      if (!lang) return;

      // Find child <code> element
      const codeChild = node.children.find(
        (c): c is Element => c.type === "element" && c.tagName === "code"
      );
      if (!codeChild) return;

      // Only add if code doesn't already have a language class
      const existing = (codeChild.properties?.className as string[] | string) || [];
      const classStr = Array.isArray(existing) ? existing.join(" ") : existing;
      if (/language-/.test(classStr)) return;

      // Set the language class on the code element
      const langClass = `language-${lang}`;
      if (Array.isArray(codeChild.properties?.className)) {
        (codeChild.properties.className as string[]).push(langClass);
      } else {
        codeChild.properties = codeChild.properties || {};
        codeChild.properties.className = [langClass];
      }
    });
  };
}

/**
 * Social embed detection patterns.
 * RSS feeds include <blockquote class="bluesky-embed"> (etc.) with a <script>
 * to load the embed widget. The script is stripped by sanitize, so we detect
 * the blockquote class and render a styled card instead.
 */
const EMBED_PLATFORMS: Array<{
  className: string;
  platform: string;
  label: string;
  urlPattern: RegExp;
}> = [
  { className: "bluesky-embed", platform: "bluesky", label: "Bluesky", urlPattern: /bsky\.app/ },
  {
    className: "twitter-tweet",
    platform: "twitter",
    label: "X (Twitter)",
    urlPattern: /(?:twitter|x)\.com/,
  },
  {
    className: "instagram-media",
    platform: "instagram",
    label: "Instagram",
    urlPattern: /instagram\.com/,
  },
];

/**
 * Rehype plugin: detect social-embed elements (both <blockquote> and <div>
 * with embed classes) and replace them with native embed iframes or styled cards.
 *
 * IMPORTANT: must run BEFORE rehypeSanitize so we can read data-bluesky-uri
 * (which sanitize would strip).
 */
function rehypeEmbeds() {
  return (tree: Root) => {
    visit(tree, "element", (node: Element) => {
      if (node.tagName !== "blockquote" && node.tagName !== "div") return;

      const classes = Array.isArray(node.properties?.className)
        ? (node.properties.className as string[])
        : typeof node.properties?.className === "string"
          ? [node.properties.className]
          : [];

      const match = EMBED_PLATFORMS.find((p) => classes.includes(p.className));
      if (!match) return;

      // Extract the post URL from the last matching <a>
      let embedUrl = "";
      visit(node, "element", (child: Element) => {
        if (child.tagName === "a" && typeof child.properties?.href === "string") {
          if (match.urlPattern.test(child.properties.href)) {
            embedUrl = child.properties.href;
          }
        }
      });

      // Bluesky: use native embed iframe
      if (match.platform === "bluesky") {
        let iframeSrc = "";

        // Prefer AT URI from data-bluesky-uri (available before sanitize strips it)
        const atUri = node.properties?.dataBlueskyUri as string | undefined;
        if (atUri?.startsWith("at://")) {
          iframeSrc = `https://embed.bsky.app/embed/${atUri.slice(5)}`;
        } else if (embedUrl) {
          // Fallback: construct from post URL
          const m = embedUrl.match(/bsky\.app\/profile\/([^/?#]+)\/post\/([^/?#]+)/);
          if (m) {
            iframeSrc = `https://embed.bsky.app/embed/${m[1]}/app.bsky.feed.post/${m[2]}`;
          }
        }

        if (iframeSrc) {
          node.tagName = "div";
          node.properties = { className: ["social-embed", "social-embed--bluesky"] };
          const iframe: Element = {
            type: "element",
            tagName: "iframe",
            properties: {
              src: iframeSrc,
              width: "100%",
              height: "400",
              frameBorder: "0",
              style: "border: none;",
            },
            children: [],
          };
          node.children = [iframe];
          return SKIP;
        }
      }

      // Fallback: styled card with original content
      node.tagName = "div";
      node.properties = { className: ["social-embed", `social-embed--${match.platform}`] };

      const contentWrapper: Element = {
        type: "element",
        tagName: "div",
        properties: { className: ["social-embed__content"] },
        children: [...node.children],
      };

      node.children = [contentWrapper];

      if (embedUrl) {
        const linkText: Text = { type: "text", value: `View on ${match.label} \u2192` };
        const linkEl: Element = {
          type: "element",
          tagName: "a",
          properties: { href: embedUrl, className: ["social-embed__link"] },
          children: [linkText],
        };
        node.children.push(linkEl);
      }

      return SKIP;
    });
  };
}

const parser = unified().use(rehypeParse, { fragment: true });

const transformer = unified()
  // Embeds must run before sanitize so they can read data-bluesky-uri.
  .use(rehypeEmbeds)
  .use(rehypeSanitize, sanitizeSchema)
  .use(rehypeCleanCodeBlocks)
  .use(rehypeCopyLanguage)
  .use(rehypeInlineCodeProperty)
  .freeze();

/**
 * Parses and sanitizes article HTML. Relative URLs are resolved against `baseUrl` (the article's
 * own address) before sanitizing; with a `null` base they are left as they are.
 */
export function buildArticleTree(html: string, baseUrl: URL | null): Root {
  const tree = parser.parse(html);
  resolveTreeUrls(tree, baseUrl);
  return transformer.runSync(tree);
}
