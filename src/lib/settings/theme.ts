import { APP_BRAND } from "../brand/brand";
import type { ThemeMode } from "./settings";

const THEME_STORAGE_KEY = `${APP_BRAND.packageName}.theme`;

export function readStoredTheme(): ThemeMode {
  if (typeof window === "undefined") {
    return "system";
  }

  const saved = window.localStorage.getItem(THEME_STORAGE_KEY);

  if (saved === "system" || saved === "light" || saved === "dark") {
    return saved;
  }

  return "system";
}

export function writeStoredTheme(theme: ThemeMode): void {
  if (typeof window === "undefined") {
    return;
  }

  window.localStorage.setItem(THEME_STORAGE_KEY, theme);
}

export function resolveSystemTheme(): "light" | "dark" {
  if (typeof window === "undefined") {
    return "light";
  }

  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

export function applyTheme(theme: ThemeMode): void {
  if (typeof document === "undefined") {
    return;
  }

  const resolvedTheme = theme === "system" ? resolveSystemTheme() : theme;
  document.documentElement.classList.toggle("dark", resolvedTheme === "dark");
  document.documentElement.dataset.theme = theme;
}
