import { useIsMutating, useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "../lib/api";

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

export function useImportOpml() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (fileOrPath: string | File) => api.importOpml(fileOrPath),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
      queryClient.invalidateQueries({ queryKey: ["articles"] });
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

/** Shared key so any component can observe a running "refresh all" via `useIsMutating`. */
export const REFRESH_ALL_MUTATION_KEY = ["refresh-all"] as const;

export function useRefreshAllFeeds() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationKey: REFRESH_ALL_MUTATION_KEY,
    mutationFn: () => api.refreshAllFeeds(),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
      queryClient.invalidateQueries({ queryKey: ["articles"] });
    },
  });
}

export function useIsRefreshingAll(): boolean {
  return useIsMutating({ mutationKey: REFRESH_ALL_MUTATION_KEY }) > 0;
}
