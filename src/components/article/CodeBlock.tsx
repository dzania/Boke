import { ShikiHighlighter, isInlineCode, type Element } from "react-shiki";

interface CodeBlockProps extends React.ComponentPropsWithoutRef<"code"> {
  node?: Element;
}

export default function CodeBlock({ className, children, node, ...props }: CodeBlockProps) {
  if (node && isInlineCode(node)) {
    return (
      <code className={className} {...props}>
        {children}
      </code>
    );
  }

  const match = /language-(\w+)/.exec(className || "");
  const lang = match ? match[1] : undefined;
  const code = String(children).replace(/\n$/, "");

  return (
    <ShikiHighlighter
      language={lang}
      theme={{ light: "github-light", dark: "github-dark" }}
    >
      {code}
    </ShikiHighlighter>
  );
}
