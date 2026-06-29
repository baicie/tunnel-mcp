import { describe, expect, it } from "vitest";
import {
  APP_TOP_BAR_HEIGHT,
  CUSTOM_CHROME_DRAG_BAR_HEIGHT,
  MAC_OVERLAY_DRAG_BAR_HEIGHT,
  getDragRegionAttrs,
  getDragRegionStyle,
  getNoDragStyle,
  getWindowChromePolicy,
} from "./windowChrome";

describe("getWindowChromePolicy", () => {
  it("uses macOS overlay chrome by default on macOS", () => {
    const policy = getWindowChromePolicy({ platform: "macos" });

    expect(policy.platform).toBe("macos");
    expect(policy.dragBarHeight).toBe(MAC_OVERLAY_DRAG_BAR_HEIGHT);
    expect(policy.headerHeight).toBe(APP_TOP_BAR_HEIGHT);
    expect(policy.contentTopOffset).toBe(
      MAC_OVERLAY_DRAG_BAR_HEIGHT + APP_TOP_BAR_HEIGHT,
    );
    expect(policy.showCustomWindowControls).toBe(false);
    expect(policy.enableDragRegion).toBe(true);
    expect(policy.useNativeDecorations).toBe(true);
  });

  it("keeps native decorations on Windows by default", () => {
    const policy = getWindowChromePolicy({ platform: "windows" });

    expect(policy.platform).toBe("windows");
    expect(policy.dragBarHeight).toBe(0);
    expect(policy.headerHeight).toBe(APP_TOP_BAR_HEIGHT);
    expect(policy.contentTopOffset).toBe(APP_TOP_BAR_HEIGHT);
    expect(policy.showCustomWindowControls).toBe(false);
    expect(policy.enableDragRegion).toBe(false);
    expect(policy.useNativeDecorations).toBe(true);
  });

  it("supports custom window controls on Windows", () => {
    const policy = getWindowChromePolicy({
      platform: "windows",
      windowsCustomControls: true,
    });

    expect(policy.platform).toBe("windows");
    expect(policy.dragBarHeight).toBe(CUSTOM_CHROME_DRAG_BAR_HEIGHT);
    expect(policy.contentTopOffset).toBe(
      CUSTOM_CHROME_DRAG_BAR_HEIGHT + APP_TOP_BAR_HEIGHT,
    );
    expect(policy.showCustomWindowControls).toBe(true);
    expect(policy.enableDragRegion).toBe(true);
    expect(policy.useNativeDecorations).toBe(false);
  });

  it("keeps native decorations on Linux by default", () => {
    const policy = getWindowChromePolicy({ platform: "linux" });

    expect(policy.platform).toBe("linux");
    expect(policy.dragBarHeight).toBe(0);
    expect(policy.contentTopOffset).toBe(APP_TOP_BAR_HEIGHT);
    expect(policy.showCustomWindowControls).toBe(false);
    expect(policy.enableDragRegion).toBe(false);
    expect(policy.useNativeDecorations).toBe(true);
  });

  it("supports custom window controls on Linux explicitly", () => {
    const policy = getWindowChromePolicy({
      platform: "linux",
      linuxCustomControls: true,
    });

    expect(policy.platform).toBe("linux");
    expect(policy.dragBarHeight).toBe(CUSTOM_CHROME_DRAG_BAR_HEIGHT);
    expect(policy.contentTopOffset).toBe(
      CUSTOM_CHROME_DRAG_BAR_HEIGHT + APP_TOP_BAR_HEIGHT,
    );
    expect(policy.showCustomWindowControls).toBe(true);
    expect(policy.enableDragRegion).toBe(true);
    expect(policy.useNativeDecorations).toBe(false);
  });

  it("falls back to native decorations for unknown platforms", () => {
    const policy = getWindowChromePolicy({ platform: "unknown" });

    expect(policy.platform).toBe("unknown");
    expect(policy.dragBarHeight).toBe(0);
    expect(policy.contentTopOffset).toBe(APP_TOP_BAR_HEIGHT);
    expect(policy.showCustomWindowControls).toBe(false);
    expect(policy.enableDragRegion).toBe(false);
    expect(policy.useNativeDecorations).toBe(true);
  });
});

describe("drag region helpers", () => {
  it("emits drag attrs when enabled", () => {
    expect(getDragRegionAttrs(true)).toEqual({
      "data-tauri-drag-region": true,
    });
  });

  it("omits drag attrs when disabled", () => {
    expect(getDragRegionAttrs(false)).toEqual({});
  });

  it("emits drag style when enabled", () => {
    expect(getDragRegionStyle(true)).toEqual({
      WebkitAppRegion: "drag",
    });
  });

  it("omits drag style when disabled", () => {
    expect(getDragRegionStyle(false)).toEqual({});
  });

  it("always emits no-drag style for interactive slots", () => {
    expect(getNoDragStyle()).toEqual({
      WebkitAppRegion: "no-drag",
    });
  });
});
