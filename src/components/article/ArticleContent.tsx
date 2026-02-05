import Markdown from "react-markdown";
import rehypeRaw from "rehype-raw";
import rehypeSanitize, { defaultSchema } from "rehype-sanitize";
import remarkGfm from "remark-gfm";
import { rehypeInlineCodeProperty } from "react-shiki";
import type { Root, Element } from "hast";
import { visit } from "unist-util-visit";
import CodeBlock from "./CodeBlock";

const sanitizeSchema = {
  ...defaultSchema,
  attributes: {
    ...defaultSchema.attributes,
    code: ["className", "inline", "dataLang", "dataLanguage"],
    pre: ["className", "dataLang", "dataLanguage"],
    span: [...(defaultSchema.attributes?.span || []), "className", "style"],
  },
};

/**
 * Rehype plugin: copy data-language from <pre> to its child <code> as a className.
 * Handles sites like steveklabnik.com (Astro) that put language info on <pre data-language="rust">.
 */
function rehypeCopyLanguage() {
  return (tree: Root) => {
    visit(tree, "element", (node: Element) => {
      if (node.tagName !== "pre") return;

      const lang =
        (node.properties?.dataLanguage as string) ||
        (node.properties?.dataLang as string);
      if (!lang) return;

      // Find child <code> element
      const codeChild = node.children.find(
        (c): c is Element => c.type === "element" && c.tagName === "code",
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
}

export default function ArticleContent({ content }: ArticleContentProps) {
  return (
    <Markdown
      remarkPlugins={[remarkGfm]}
      rehypePlugins={[
        rehypeRaw,
        [rehypeSanitize, sanitizeSchema],
        rehypeCopyLanguage,
        rehypeInlineCodeProperty,
      ]}
      components={{
        code: CodeBlock,
        img: ({ node: _, ...props }) => <img loading="lazy" {...props} />,
      }}
    >
      {content}
    </Markdown>
  );
}
