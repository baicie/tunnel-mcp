import { describe, expect, it } from "vitest";
import { APP_BRAND, formatAppTitle } from "./brand";
import { TEMPLATE_CONFIG } from "./templateConfig";

describe("APP_BRAND", () => {
  it("uses generated template config", () => {
    expect(APP_BRAND.displayName).toBe(TEMPLATE_CONFIG.appName);
    expect(APP_BRAND.packageName).toBe(TEMPLATE_CONFIG.packageName);
    expect(APP_BRAND.productName).toBe(TEMPLATE_CONFIG.productName);
    expect(APP_BRAND.identifier).toBe(TEMPLATE_CONFIG.identifier);
    expect(APP_BRAND.repositoryUrl).toBe(TEMPLATE_CONFIG.repositoryUrl);
    expect(APP_BRAND.updaterEndpoint).toBe(TEMPLATE_CONFIG.updaterEndpoint);
  });

  it("does not keep legacy brand shape", () => {
    const serialized = JSON.stringify(APP_BRAND).toLowerCase();
    const legacyProductNamePattern = /[^a-z0-9](c c|c\.c\.)[^a-z0-9]/;

    expect(serialized).not.toMatch(legacyProductNamePattern);
    expect(APP_BRAND.packageName).not.toContain(" ");
  });

  it("formats app title", () => {
    expect(formatAppTitle()).toBe(APP_BRAND.windowTitle);
    expect(formatAppTitle("Settings")).toBe(
      `Settings - ${APP_BRAND.windowTitle}`,
    );
  });
});
