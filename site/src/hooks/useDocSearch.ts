import { useCallback } from "react";
import { useNavigate } from "@tanstack/react-router";

export type SearchParams = { q?: string };

export function validateSearch(search: Record<string, unknown>): SearchParams {
  return {
    q: typeof search.q === "string" ? search.q : undefined,
  };
}

export function useDocSearchChange(from: string): (value: string) => void {
  const navigate = useNavigate({ from });

  return useCallback(
    function handleSearchChange(value: string): void {
      navigate({
        search: value ? { q: value } : {},
        replace: true,
      });
    },
    [navigate],
  );
}
