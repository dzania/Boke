import { describe, expect, it } from "vitest";
import type { Element, Root } from "hast";
import rehypeParse from "rehype-parse";
import { unified } from "unified";
import { visit } from "unist-util-visit";
import { parseBaseUrl, resolveSrcset, resolveTreeUrls, resolveUrl } from "./resolveUrls";

const BASE = new URL("https://example.com/blog/post/");

function firstElement(tree: Root, tagName: string): Element {
  let found: Element | undefined;
  visit(tree, "element", (node: Element) => {
    if (!found && node.tagName === tagName) found = node;
  });
  if (!found) throw new Error(`no <${tagName}> in tree`);
  return found;
}

function resolved(html: string, base: URL | null = BASE): Root {
  const tree = unified().use(rehypeParse, { fragment: true }).parse(html);
  resolveTreeUrls(tree, base);
  return tree;
}

describe("resolveUrl", () => {
  it("resolves root-relative, path-relative and protocol-relative URLs", () => {
    expect(resolveUrl("/a.png", BASE)).toBe("https://example.com/a.png");
    expect(resolveUrl("img/a.png", BASE)).toBe("https://example.com/blog/post/img/a.png");
    expect(resolveUrl("//cdn.example.org/a.png", BASE)).toBe("https://cdn.example.org/a.png");
  });

  it("leaves absolute and special URLs unchanged", () => {
    for (const value of [
      "https://cdn.example.org/a.png",
      "#section",
      "data:image/png;base64,AAAA",
      "mailto:me@example.com",
      "JavaScript:void(0)",
      "tel:+1",
      "",
    ]) {
      expect(resolveUrl(value, BASE)).toBe(value);
    }
  });
});

describe("resolveSrcset", () => {
  it("resolves every candidate and keeps descriptors", () => {
    expect(resolveSrcset("/a.png 1x, b.png 2x", BASE)).toBe(
      "https://example.com/a.png 1x, https://example.com/blog/post/b.png 2x"
    );
  });

  it("keeps commas inside CDN URLs", () => {
    expect(resolveSrcset("/w_300,h_200/a.png 300w, /b.png 2x", BASE)).toBe(
      "https://example.com/w_300,h_200/a.png 300w, https://example.com/b.png 2x"
    );
  });

  it("accepts candidates that are already split", () => {
    expect(resolveSrcset(["/a.png 1x", "/b.png 2x"], BASE)).toBe(
      "https://example.com/a.png 1x, https://example.com/b.png 2x"
    );
  });

  it("handles candidates without descriptors and empty input", () => {
    expect(resolveSrcset("/a.png, /b.png", BASE)).toBe(
      "https://example.com/a.png, https://example.com/b.png"
    );
    // Per the HTML grammar a URL runs to whitespace, so this is a single candidate.
    expect(resolveSrcset("/a.png,/b.png", BASE)).toBe("https://example.com/a.png,/b.png");
    expect(resolveSrcset("", BASE)).toBe("");
    expect(resolveSrcset(" , ", BASE)).toBe("");
  });
});

describe("parseBaseUrl", () => {
  it("picks the first absolute http(s) candidate", () => {
    expect(parseBaseUrl([null, "not a url", "https://example.com/x"])?.href).toBe(
      "https://example.com/x"
    );
    expect(parseBaseUrl(["ftp://example.com/", "javascript:alert(1)"])).toBeNull();
    expect(parseBaseUrl([])).toBeNull();
  });
});

describe("resolveTreeUrls", () => {
  it("resolves img src and a href", () => {
    const tree = resolved('<a href="/page"><img src="pic.png"></a>');
    expect(firstElement(tree, "a").properties.href).toBe("https://example.com/page");
    expect(firstElement(tree, "img").properties.src).toBe("https://example.com/blog/post/pic.png");
  });

  it("promotes lazy-loading attributes over placeholders", () => {
    const tree = resolved(
      '<img src="data:image/gif;base64,R0lGOD" data-src="/real.jpg" data-srcset="/real.jpg 1x, /real@2x.jpg 2x">'
    );
    const img = firstElement(tree, "img");
    expect(img.properties.src).toBe("https://example.com/real.jpg");
    expect(img.properties.srcSet).toBe(
      "https://example.com/real.jpg 1x, https://example.com/real@2x.jpg 2x"
    );
  });

  it("keeps a real src over data-src", () => {
    const tree = resolved('<img src="/real.jpg" data-src="/other.jpg">');
    expect(firstElement(tree, "img").properties.src).toBe("https://example.com/real.jpg");
  });

  it("resolves picture sources and video posters", () => {
    const tree = resolved(
      '<picture><source srcset="/a.webp" type="image/webp"><img src="/a.jpg"></picture><video poster="p.jpg"></video>'
    );
    expect(firstElement(tree, "source").properties.srcSet).toBe("https://example.com/a.webp");
    expect(firstElement(tree, "video").properties.poster).toBe(
      "https://example.com/blog/post/p.jpg"
    );
  });

  it("still promotes lazy images when there is no base", () => {
    const tree = resolved('<img data-src="/real.jpg">', null);
    expect(firstElement(tree, "img").properties.src).toBe("/real.jpg");
  });
});
