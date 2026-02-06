import Markdown from "react-markdown";
import rehypeRaw from "rehype-raw";
import rehypeSanitize, { defaultSchema } from "rehype-sanitize";
import remarkGfm from "remark-gfm";
import { rehypeInlineCodeProperty } from "react-shiki";
import type { Root, Element } from "hast";
import { visit } from "unist-util-visit";
import CodeBlock from "./CodeBlock";

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

const sanitizeSchema = {
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
    iframe: ["src", "width", "height", "frameBorder", "allow", "allowFullScreen"],
    video: ["src", "width", "height", "controls", "poster"],
    audio: ["src", "controls"],
    source: ["src", "type"],
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

interface ArticleContentProps {
  content: string;
  theme: "light" | "dark";
}

/**
 * Strip leading whitespace from each line so that indented HTML from RSS feeds
 * isn't treated as a CommonMark indented code block (4+ spaces) by the markdown
 * parser before rehype-raw gets a chance to process it.
 */
function dedentHtml(html: string): string {
  return html.replace(/^[ \t]+/gm, "");
}

export default function ArticleContent({ content, theme }: ArticleContentProps) {
  const isDark = theme === "dark";

  return (
    <Markdown
      remarkPlugins={[remarkGfm]}
      rehypePlugins={[
        rehypeRaw,
        [rehypeSanitize, sanitizeSchema],
        rehypeCleanCodeBlocks,
        rehypeCopyLanguage,
        rehypeInlineCodeProperty,
      ]}
      components={{
        code: (props) => <CodeBlock {...props} isDark={isDark} />,
        img: ({ node: _, ...props }) => <img loading="lazy" {...props} />,
      }}
    >
      {dedentHtml(content)}
    </Markdown>
  );
}
