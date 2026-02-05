import { useQuery } from "@tanstack/react-query";
import * as api from "../lib/tauri";
import { SEARCH_MIN_QUERY_LENGTH } from "../lib/constants";

export function useSearchArticles(query: string, limit = 50) {
  return useQuery({
    queryKey: ["search", query],
    queryFn: () => api.searchArticles(query, limit),
    enabled: query.length >= SEARCH_MIN_QUERY_LENGTH,
  });
}
