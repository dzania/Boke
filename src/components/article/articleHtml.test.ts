import { describe, expect, it } from "vitest";
import type { Element, Root } from "hast";
import { visit } from "unist-util-visit";
import { buildArticleTree } from "./articleHtml";

const BASE = new URL("https://example.com/2024/post.html");

function elements(tree: Root, tagName: string): Element[] {
  const found: Element[] = [];
  visit(tree, "element", (node: Element) => {
    if (node.tagName === tagName) found.push(node);
  });
  return found;
}

describe("buildArticleTree", () => {
  it("resolves relative image URLs and keeps responsive candidates through sanitize", () => {
    const tree = buildArticleTree(
      '<p>Hi</p><img src="/a.jpg" srcset="/a.jpg 1x, /a@2x.jpg 2x" sizes="100vw" alt="A">',
      BASE
    );
    const [img] = elements(tree, "img");
    expect(img.properties.src).toBe("https://example.com/a.jpg");
    expect(img.properties.srcSet).toBe(
      "https://example.com/a.jpg 1x, https://example.com/a@2x.jpg 2x"
    );
    expect(img.properties.sizes).toBe("100vw");
    expect(img.properties.alt).toBe("A");
  });

  it("renders lazy-loaded images and strips the data attributes afterwards", () => {
    const tree = buildArticleTree(
      '<img src="data:image/gif;base64,R0lG" data-src="/real.jpg">',
      BASE
    );
    const [img] = elements(tree, "img");
    expect(img.properties.src).toBe("https://example.com/real.jpg");
    expect(img.properties.dataSrc).toBeUndefined();
  });

  it("keeps <picture> sources", () => {
    const tree = buildArticleTree(
      '<picture><source srcset="a.webp" type="image/webp"><img src="a.jpg"></picture>',
      BASE
    );
    const [source] = elements(tree, "source");
    expect(source.properties.srcSet).toBe("https://example.com/2024/a.webp");
  });

  it("preserves indentation inside <pre>", () => {
    const html = '<pre><code>fn main() {\n    println!("hi");\n}</code></pre>';
    const tree = buildArticleTree(html, BASE);
    let text = "";
    visit(tree, "text", (node: { value: string }) => {
      text += node.value;
    });
    expect(text).toContain('\n    println!("hi");');
  });

  it("drops scripts and javascript: links", () => {
    const tree = buildArticleTree(
      '<script>alert(1)</script><a href="javascript:alert(1)">x</a><img src="x" onerror="alert(1)">',
      BASE
    );
    expect(elements(tree, "script")).toHaveLength(0);
    const [link] = elements(tree, "a");
    expect(link.properties.href).toBeUndefined();
    const [img] = elements(tree, "img");
    expect(img.properties.onError).toBeUndefined();
  });

  it("handles empty input", () => {
    expect(buildArticleTree("", BASE).children).toHaveLength(0);
  });
});
