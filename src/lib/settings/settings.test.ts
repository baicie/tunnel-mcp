import { describe, expect, it } from "vitest";
import { APP_BRAND } from "../brand/brand";
import {
  defaultShellSettings,
  SETTINGS_STORAGE_KEY,
  THEME_STORAGE_KEY,
} from "./settings";

describe("settings identity", () => {
  it("uses brand package name for storage keys", () => {
    expect(SETTINGS_STORAGE_KEY).toBe(`${APP_BRAND.packageName}.settings`);
    expect(THEME_STORAGE_KEY).toBe(`${APP_BRAND.packageName}.theme`);
  });

  it("keeps default shell settings", () => {
    expect(defaultShellSettings).toEqual({
      theme: "system",
      startMinimized: false,
    });
  });
});
