import type { Element, Properties, Root } from "hast";
import { visit } from "unist-util-visit";

/** URL values that must never be joined with a base URL. */
const UNRESOLVED_URL = /^\s*(#|data:|javascript:|mailto:|tel:)/i;

/** Properties holding a single URL. */
const URL_PROPERTIES = ["src", "href", "poster"] as const;

/**
 * Properties where lazy-loading scripts park the real image URL, in priority order. The scripts
 * never run in the reader, so without promotion the reader shows the placeholder.
 */
const LAZY_SRC_PROPERTIES = ["dataSrc", "dataLazySrc", "dataOriginal"] as const;
const LAZY_SRCSET_PROPERTIES = ["dataSrcset", "dataLazySrcset"] as const;

/**
 * Parses the base URL for an article. Only absolute http(s) URLs are usable; anything else
 * returns `null` and leaves the content untouched.
 */
export function parseBaseUrl(candidates: ReadonlyArray<string | null | undefined>): URL | null {
  for (const candidate of candidates) {
    if (!candidate) continue;
    try {
      const url = new URL(candidate);
      if (url.protocol === "http:" || url.protocol === "https:") return url;
    } catch {
      // Not an absolute URL; try the next candidate.
    }
  }
  return null;
}

/** Resolves `value` against `base`, leaving absolute, fragment and special URLs unchanged. */
export function resolveUrl(value: string, base: URL): string {
  const trimmed = value.trim();
  if (trimmed === "" || UNRESOLVED_URL.test(trimmed)) return value;
  try {
    return new URL(trimmed, base).href;
  } catch {
    return value;
  }
}

/**
 * Resolves each candidate of a `srcset`, keeping width and density descriptors.
 *
 * Parses with the HTML candidate grammar: a URL runs to the next whitespace and its descriptors
 * run to the next comma. Naive comma splitting would break image-CDN URLs such as
 * `/w_300,h_200/a.jpg`. Array input (already split into candidates) is joined with ", ".
 */
export function resolveSrcset(value: string | ReadonlyArray<string | number>, base: URL): string {
  const text = typeof value === "string" ? value : value.join(", ");
  const resolved: string[] = [];
  let index = 0;
  while (index < text.length) {
    while (index < text.length && /[\s,]/.test(text[index])) index += 1;
    if (index >= text.length) break;

    let urlEnd = index;
    while (urlEnd < text.length && !/\s/.test(text[urlEnd])) urlEnd += 1;
    let url = text.slice(index, urlEnd);
    let descriptor = "";
    let next = urlEnd;
    if (url.endsWith(",")) {
      // A URL ending in commas is a candidate without descriptors.
      url = url.replace(/,+$/, "");
    } else {
      const comma = text.indexOf(",", urlEnd);
      next = comma === -1 ? text.length : comma;
      descriptor = text.slice(urlEnd, next).trim();
    }

    resolved.push(descriptor ? `${resolveUrl(url, base)} ${descriptor}` : resolveUrl(url, base));
    index = next;
  }
  return resolved.join(", ");
}

function isPlaceholderSrc(src: Properties[string]): boolean {
  return typeof src !== "string" || src.trim() === "" || /^\s*data:/i.test(src);
}

function firstString(properties: Properties, names: ReadonlyArray<string>): string | undefined {
  for (const name of names) {
    const value = properties[name];
    if (typeof value === "string" && value.trim() !== "") return value;
  }
  return undefined;
}

/** Copies lazy-loading `data-*` URLs into `src`/`srcSet` when the real ones are placeholders. */
function promoteLazySources(node: Element) {
  if (node.tagName !== "img" && node.tagName !== "source") return;
  const properties = node.properties;

  const lazySrc = firstString(properties, LAZY_SRC_PROPERTIES);
  if (lazySrc !== undefined && isPlaceholderSrc(properties.src)) {
    properties.src = lazySrc;
  }
  const lazySrcset = firstString(properties, LAZY_SRCSET_PROPERTIES);
  if (lazySrcset !== undefined) {
    properties.srcSet = lazySrcset;
  }
}

/**
 * Makes every URL in an article tree absolute, so images and links work when the content is
 * rendered from the app's own origin instead of the blog's.
 *
 * Also promotes lazy-loading attributes (`data-src`, `data-srcset`, …) to `src`/`srcSet`. Must
 * run before sanitizing, which strips `data-*` attributes. Mutates `tree` in place.
 */
export function resolveTreeUrls(tree: Root, base: URL | null): void {
  visit(tree, "element", (node: Element) => {
    promoteLazySources(node);
    if (base === null) return;

    const properties = node.properties;
    for (const name of URL_PROPERTIES) {
      const value = properties[name];
      if (typeof value === "string") properties[name] = resolveUrl(value, base);
    }
    const srcSet = properties.srcSet;
    if (typeof srcSet === "string" || Array.isArray(srcSet)) {
      properties.srcSet = resolveSrcset(srcSet, base);
    }
  });
}
