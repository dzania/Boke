import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "../lib/tauri";
import { ARTICLES_PAGE_SIZE } from "../lib/constants";

export function useArticles(feedId: number | null, unreadOnly = false) {
  return useQuery({
    queryKey: ["articles", feedId, unreadOnly],
    queryFn: () => api.getArticles(feedId, 0, ARTICLES_PAGE_SIZE, unreadOnly),
  });
}

export function useArticle(articleId: number | null) {
  return useQuery({
    queryKey: ["article", articleId],
    queryFn: () => {
      if (articleId === null) {
        throw new Error("articleId is required");
      }
      return api.getArticle(articleId);
    },
    enabled: articleId !== null,
  });
}

export function useToggleRead() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (articleId: number) => api.toggleRead(articleId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["articles"] });
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
    },
  });
}

export function useMarkAllRead() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (feedId: number) => api.markAllRead(feedId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["articles"] });
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
    },
  });
}

export function useToggleFavorite() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (articleId: number) => api.toggleFavorite(articleId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["articles"] });
    },
  });
}
