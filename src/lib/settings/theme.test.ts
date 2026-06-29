import { beforeEach, describe, expect, it } from "vitest";
import { applyTheme, readStoredTheme, writeStoredTheme } from "./theme";

describe("theme", () => {
  beforeEach(() => {
    if (typeof window !== "undefined") {
      window.localStorage.clear();
    }

    if (typeof document !== "undefined") {
      document.documentElement.className = "";
      delete document.documentElement.dataset.theme;
    }
  });

  it("returns system by default", () => {
    expect(readStoredTheme()).toBe("system");
  });

  it("stores and reads theme", () => {
    writeStoredTheme("dark");
    expect(readStoredTheme()).toBe("dark");
  });

  it("applies dark theme", () => {
    applyTheme("dark");
    expect(document.documentElement.classList.contains("dark")).toBe(true);
    expect(document.documentElement.dataset.theme).toBe("dark");
  });
});
