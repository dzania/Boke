import { memo, useMemo, type ComponentPropsWithoutRef, type SyntheticEvent } from "react";
import { Fragment, jsx, jsxs } from "react/jsx-runtime";
import { toJsxRuntime, type Components } from "hast-util-to-jsx-runtime";
import type { Element } from "hast";
import CodeBlock from "./CodeBlock";
import { buildArticleTree } from "./articleHtml";
import { parseBaseUrl } from "../../lib/rehype/resolveUrls";
import { openUrl } from "../../lib/platform";

type WithNode<T extends keyof React.JSX.IntrinsicElements> = ComponentPropsWithoutRef<T> & {
  node?: Element;
};

function hideBrokenImage(event: SyntheticEvent<HTMLImageElement>) {
  event.currentTarget.classList.add("img-broken");
}

function ArticleImage({ node: _node, ...props }: WithNode<"img">) {
  return (
    <img
      {...props}
      loading="lazy"
      decoding="async"
      // Many image hosts reject hotlinks that carry a foreign Referer but allow an empty one.
      referrerPolicy="no-referrer"
      onError={hideBrokenImage}
    />
  );
}

function ArticleLink({ node: _node, href, children, ...props }: WithNode<"a">) {
  if (href && /^https?:\/\//i.test(href)) {
    return (
      <a
        {...props}
        href={href}
        onClick={(event) => {
          event.preventDefault();
          openUrl(href).catch((error: unknown) => console.error("Failed to open link", error));
        }}
      >
        {children}
      </a>
    );
  }
  return (
    <a {...props} href={href}>
      {children}
    </a>
  );
}

function LightCode(props: WithNode<"code">) {
  return <CodeBlock {...props} isDark={false} />;
}

function DarkCode(props: WithNode<"code">) {
  return <CodeBlock {...props} isDark />;
}

// Defined once at module scope so element types stay stable across renders. Inline component
// functions would make React unmount and re-highlight every code block on each render.
const LIGHT_COMPONENTS: Partial<Components> = {
  a: ArticleLink,
  code: LightCode,
  img: ArticleImage,
};
const DARK_COMPONENTS: Partial<Components> = { a: ArticleLink, code: DarkCode, img: ArticleImage };

interface ArticleContentProps {
  /** Article HTML as stored or fetched. */
  content: string;
  /** The article's own address, used to resolve relative URLs. */
  articleUrl: string | null;
  /** The feed's site address, the fallback base when the article has no link. */
  siteUrl: string | null;
  theme: "light" | "dark";
}

function ArticleContent({ content, articleUrl, siteUrl, theme }: ArticleContentProps) {
  const baseHref = parseBaseUrl([articleUrl, siteUrl])?.href ?? null;
  const tree = useMemo(
    () => buildArticleTree(content, baseHref === null ? null : new URL(baseHref)),
    [content, baseHref]
  );
  return useMemo(
    () =>
      toJsxRuntime(tree, {
        Fragment,
        jsx,
        jsxs,
        components: theme === "dark" ? DARK_COMPONENTS : LIGHT_COMPONENTS,
        passNode: true,
        // Feed CSS is often malformed; drop bad declarations instead of throwing.
        ignoreInvalidStyle: true,
      }),
    [tree, theme]
  );
}

export default memo(ArticleContent);
