import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "../lib/api";
import { ARTICLES_PAGE_SIZE } from "../lib/constants";

export function useArticles(feedId: number | null, unreadOnly = false, favoritesOnly = false) {
  return useQuery({
    queryKey: ["articles", feedId, unreadOnly, favoritesOnly],
    queryFn: () => api.getArticles(feedId, 0, ARTICLES_PAGE_SIZE, unreadOnly, favoritesOnly),
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
    mutationFn: (feedId: number | null) => api.markAllRead(feedId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["articles"] });
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
    },
  });
}

export function useMarkAllUnread() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (feedId: number | null) => api.markAllUnread(feedId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["articles"] });
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
    },
  });
}

export function useFavoritesCount() {
  return useQuery({
    queryKey: ["favorites-count"],
    queryFn: () => api.getFavoritesCount(),
  });
}

export function useToggleFavorite() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (articleId: number) => api.toggleFavorite(articleId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["articles"] });
      queryClient.invalidateQueries({ queryKey: ["favorites-count"] });
    },
  });
}
