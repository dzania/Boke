import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "../lib/tauri";

export function useFeeds() {
  return useQuery({
    queryKey: ["feeds"],
    queryFn: api.getFeeds,
  });
}

export function useAddFeed() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (url: string) => api.addFeed(url),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
    },
  });
}

export function useRemoveFeed() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (feedId: number) => api.removeFeed(feedId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
      queryClient.invalidateQueries({ queryKey: ["articles"] });
    },
  });
}

export function useRefreshFeed() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (feedId: number) => api.refreshFeed(feedId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
      queryClient.invalidateQueries({ queryKey: ["articles"] });
    },
  });
}

export function useRefreshAllFeeds() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.refreshAllFeeds(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
      queryClient.invalidateQueries({ queryKey: ["articles"] });
    },
  });
}
