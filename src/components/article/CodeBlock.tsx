import { type ReactNode } from "react";
import { ShikiHighlighter, isInlineCode, type Element } from "react-shiki";

function extractText(node: ReactNode): string {
  if (node == null || typeof node === "boolean") return "";
  if (typeof node === "string" || typeof node === "number") return String(node);
  if (Array.isArray(node)) return node.map(extractText).join("");
  if (typeof node === "object" && "props" in node) {
    const props = (node as { props: { children?: ReactNode } }).props;
    return extractText(props.children);
  }
  return "";
}

// Common programming language names for detection
const KNOWN_LANGS = new Set([
  "abap", "actionscript", "ada", "apache", "apex", "apl", "applescript", "asm",
  "bash", "bat", "c", "clojure", "cmake", "cobol", "coffeescript", "cpp", "csharp",
  "css", "csv", "d", "dart", "diff", "docker", "dockerfile", "elixir", "elm",
  "erb", "erlang", "fish", "fortran", "fsharp", "gherkin", "git", "glsl", "go",
  "graphql", "groovy", "hack", "haml", "handlebars", "haskell", "hcl", "hlsl",
  "html", "http", "ini", "java", "javascript", "jinja", "json", "jsonc", "jsonnet",
  "jsx", "julia", "kotlin", "latex", "less", "liquid", "lisp", "log", "lua",
  "make", "makefile", "markdown", "matlab", "mermaid", "nasm", "nginx", "nim",
  "nix", "objc", "objective-c", "ocaml", "pascal", "perl", "php", "plaintext",
  "plsql", "postcss", "powershell", "prisma", "prolog", "proto", "protobuf",
  "pug", "puppet", "python", "r", "razor", "regex", "rest", "ruby", "rust",
  "sass", "scala", "scheme", "scss", "sh", "shell", "shellscript", "smalltalk",
  "solidity", "sparql", "sql", "ssh", "stylus", "svelte", "swift", "tcl",
  "terraform", "tex", "toml", "ts", "tsx", "turtle", "twig", "typescript",
  "vb", "verilog", "vhdl", "vim", "vue", "wasm", "xml", "xsl", "yaml", "zig",
  "zsh",
]);

/**
 * Try to extract a programming language from a className string.
 * Handles: "language-rust", "rust", "sourceCode rust", "highlight-source-rust", "brush: rust", etc.
 */
function detectLanguage(className: string | undefined): string | undefined {
  if (!className) return undefined;

  // Try "language-xxx" first
  const langMatch = /language-(\w+)/.exec(className);
  if (langMatch) return langMatch[1];

  // Split on spaces and check each token
  const tokens = className.split(/[\s:]+/).filter(Boolean);
  for (const token of tokens) {
    const lower = token.toLowerCase();
    if (KNOWN_LANGS.has(lower)) return lower;
  }

  return undefined;
}

interface CodeBlockProps extends React.ComponentPropsWithoutRef<"code"> {
  node?: Element;
  isDark?: boolean;
}

export default function CodeBlock({ className, children, node, isDark, ...props }: CodeBlockProps) {
  if (node && isInlineCode(node)) {
    return (
      <code className={className} {...props}>
        {children}
      </code>
    );
  }

  const lang = detectLanguage(className);
  const code = extractText(children).replace(/\n$/, "");

  if (!lang) {
    return <code {...props}>{code}</code>;
  }

  return (
    <ShikiHighlighter
      key={isDark ? "dark" : "light"}
      language={lang}
      theme={isDark ? "github-dark-dimmed" : "github-light"}
      showLanguage={true}
      addDefaultStyles={false}
    >
      {code}
    </ShikiHighlighter>
  );
}
