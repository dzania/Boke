import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "../lib/tauri";

export function useTags() {
  return useQuery({
    queryKey: ["tags"],
    queryFn: api.getTags,
  });
}

export function useCreateTag() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => api.createTag(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["tags"] });
    },
  });
}

export function useTagFeed() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ feedId, tagId }: { feedId: number; tagId: number }) =>
      api.tagFeed(feedId, tagId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
      queryClient.invalidateQueries({ queryKey: ["tags"] });
    },
  });
}

export function useUntagFeed() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ feedId, tagId }: { feedId: number; tagId: number }) =>
      api.untagFeed(feedId, tagId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
      queryClient.invalidateQueries({ queryKey: ["tags"] });
    },
  });
}

export function useDeleteTag() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (tagId: number) => api.deleteTag(tagId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["tags"] });
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
    },
  });
}
