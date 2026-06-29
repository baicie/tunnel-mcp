import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { shellApi } from "../api/shell";
import { queryKeys } from "../query/queryKeys";
import { applyTheme, writeStoredTheme } from "./theme";
import { defaultShellSettings, type ShellSettings } from "./settings";

export interface UseShellSettingsResult {
  settings: ShellSettings;
  isLoading: boolean;
  isSaving: boolean;
  saveSettings: (settings: ShellSettings) => void;
}

export function useShellSettings(): UseShellSettingsResult {
  const queryClient = useQueryClient();

  const settingsQuery = useQuery({
    queryKey: queryKeys.settings,
    queryFn: shellApi.getSettings,
    placeholderData: defaultShellSettings,
  });

  const saveMutation = useMutation({
    mutationFn: shellApi.saveSettings,
    onSuccess: (settings) => {
      writeStoredTheme(settings.theme);
      applyTheme(settings.theme);
      queryClient.setQueryData(queryKeys.settings, settings);
    },
  });

  const settings = settingsQuery.data ?? defaultShellSettings;

  useEffect(() => {
    applyTheme(settings.theme);
  }, [settings.theme]);

  return {
    settings,
    isLoading: settingsQuery.isLoading,
    isSaving: saveMutation.isPending,
    saveSettings: (nextSettings) => saveMutation.mutate(nextSettings),
  };
}
