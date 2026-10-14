import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
  type InfiniteData,
  type QueryClient,
  type QueryKey,
} from "@tanstack/react-query";
import * as api from "../lib/api";
import { ARTICLES_PAGE_SIZE } from "../lib/constants";
import type { ArticleFilter, ArticleListItem, ArticleScope, FeedWithMeta } from "../types";

type ArticlePages = InfiniteData<ArticleListItem[], number>;

export function articlesQueryKey(feedId: number | null, filter: ArticleFilter): QueryKey {
  return ["articles", feedId, filter];
}

/** Paged article list. Each page holds up to `ARTICLES_PAGE_SIZE` rows, fetched by offset. */
export function useArticles(feedId: number | null, filter: ArticleFilter) {
  return useInfiniteQuery({
    queryKey: articlesQueryKey(feedId, filter),
    queryFn: ({ pageParam }) =>
      api.getArticles(
        feedId,
        pageParam,
        ARTICLES_PAGE_SIZE,
        filter === "unread",
        filter === "favourites"
      ),
    initialPageParam: 0,
    getNextPageParam: (lastPage, allPages) =>
      lastPage.length < ARTICLES_PAGE_SIZE ? undefined : allPages.length * ARTICLES_PAGE_SIZE,
  });
}

/** Best available HTML body for an article. The backend fetches the full page for excerpts. */
export function useArticleContent(articleId: number | null) {
  return useQuery({
    queryKey: ["article-content", articleId],
    queryFn: () => {
      if (articleId === null) {
        throw new Error("articleId is required");
      }
      return api.getArticleContent(articleId);
    },
    enabled: articleId !== null,
    staleTime: Infinity,
  });
}

export function useFavoritesCount() {
  return useQuery({
    queryKey: ["favorites-count"],
    queryFn: () => api.getFavoritesCount(),
  });
}

type ArticlePatch = Partial<Pick<ArticleListItem, "is_read" | "is_favorite">>;

interface CacheSnapshot {
  entries: Array<[QueryKey, unknown]>;
}

/**
 * Applies `patch` to every cached copy of article `id`, in list pages and search results.
 *
 * Only pages that contain the article are replaced, and only the article's own object changes,
 * so memoized rows elsewhere keep their props and skip re-rendering.
 */
function patchArticleInCaches(
  queryClient: QueryClient,
  id: number,
  patch: ArticlePatch
): CacheSnapshot {
  const entries: Array<[QueryKey, unknown]> = [
    ...queryClient.getQueriesData({ queryKey: ["articles"] }),
    ...queryClient.getQueriesData({ queryKey: ["search"] }),
  ];
  const patchPage = (page: ArticleListItem[]) =>
    page.some((article) => article.id === id)
      ? page.map((article) => (article.id === id ? { ...article, ...patch } : article))
      : page;

  queryClient.setQueriesData<ArticlePages>({ queryKey: ["articles"] }, (data) =>
    data ? { ...data, pages: data.pages.map(patchPage) } : data
  );
  queryClient.setQueriesData<ArticleListItem[]>({ queryKey: ["search"] }, (data) =>
    data ? patchPage(data) : data
  );
  return { entries };
}

function restoreCaches(queryClient: QueryClient, snapshot: CacheSnapshot | undefined) {
  for (const [key, data] of snapshot?.entries ?? []) {
    queryClient.setQueryData(key, data);
  }
}

interface SetReadVariables {
  article: ArticleListItem;
  isRead: boolean;
}

/**
 * Sets an article's read flag with an optimistic cache update.
 *
 * The row stays in place even under the Unread filter; it only drops out when the list is next
 * refetched. This keeps the keyboard cursor stable while reading.
 */
export function useSetRead() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ article, isRead }: SetReadVariables) => api.setRead(article.id, isRead),
    onMutate: async ({ article, isRead }) => {
      await queryClient.cancelQueries({ queryKey: ["feeds"] });
      const feedsBefore = queryClient.getQueryData<FeedWithMeta[]>(["feeds"]);
      const snapshot = patchArticleInCaches(queryClient, article.id, { is_read: isRead });
      if (article.is_read !== isRead) {
        const delta = isRead ? -1 : 1;
        queryClient.setQueryData<FeedWithMeta[]>(["feeds"], (feeds) =>
          feeds?.map((feed) =>
            feed.id === article.feed_id
              ? { ...feed, unread_count: Math.max(0, feed.unread_count + delta) }
              : feed
          )
        );
      }
      return { snapshot, feedsBefore };
    },
    onError: (_error, _variables, context) => {
      restoreCaches(queryClient, context?.snapshot);
      queryClient.setQueryData(["feeds"], context?.feedsBefore);
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey: ["feeds"] }),
  });
}

interface SetFavoriteVariables {
  article: ArticleListItem;
  isFavorite: boolean;
}

/** Sets an article's favourite flag with an optimistic cache update. */
export function useSetFavorite() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ article, isFavorite }: SetFavoriteVariables) =>
      api.setFavorite(article.id, isFavorite),
    onMutate: ({ article, isFavorite }) => ({
      snapshot: patchArticleInCaches(queryClient, article.id, { is_favorite: isFavorite }),
    }),
    onError: (_error, _variables, context) => restoreCaches(queryClient, context?.snapshot),
    onSettled: () => queryClient.invalidateQueries({ queryKey: ["favorites-count"] }),
  });
}

function useBulkReadMutation(mutationFn: (scope: ArticleScope) => Promise<void>) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["articles"] });
      queryClient.invalidateQueries({ queryKey: ["search"] });
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
    },
  });
}

export function useMarkAllRead() {
  return useBulkReadMutation(api.markAllRead);
}

export function useMarkAllUnread() {
  return useBulkReadMutation(api.markAllUnread);
}
