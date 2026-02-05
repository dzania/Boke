import Markdown from "react-markdown";
import rehypeRaw from "rehype-raw";
import rehypeSanitize, { defaultSchema } from "rehype-sanitize";
import remarkGfm from "remark-gfm";
import { rehypeInlineCodeProperty } from "react-shiki";
import CodeBlock from "./CodeBlock";

const sanitizeSchema = {
  ...defaultSchema,
  attributes: {
    ...defaultSchema.attributes,
    code: [...(defaultSchema.attributes?.code || []), "className"],
  },
};

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
