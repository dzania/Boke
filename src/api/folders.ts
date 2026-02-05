import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import * as api from "../lib/tauri";

export function useFolders() {
  return useQuery({
    queryKey: ["folders"],
    queryFn: api.getFolders,
  });
}

export function useCreateFolder() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => api.createFolder(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["folders"] });
    },
  });
}

export function useRenameFolder() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ folderId, name }: { folderId: number; name: string }) =>
      api.renameFolder(folderId, name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["folders"] });
    },
  });
}

export function useDeleteFolder() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (folderId: number) => api.deleteFolder(folderId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["folders"] });
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
    },
  });
}

export function useMoveFeedToFolder() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ feedId, folderId }: { feedId: number; folderId: number | null }) =>
      api.moveFeedToFolder(feedId, folderId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["feeds"] });
      queryClient.invalidateQueries({ queryKey: ["folders"] });
    },
  });
}
