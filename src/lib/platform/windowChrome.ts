import type { CSSProperties } from "react";
import { detectPlatform, type DesktopPlatform } from "./platform";

export const APP_TOP_BAR_HEIGHT = 64;
export const MAC_OVERLAY_DRAG_BAR_HEIGHT = 28;
export const CUSTOM_CHROME_DRAG_BAR_HEIGHT = 32;

export interface WindowChromeOptions {
  platform?: DesktopPlatform;
  linuxCustomControls?: boolean;
  windowsCustomControls?: boolean;
}

export interface WindowChromePolicy {
  platform: DesktopPlatform;
  dragBarHeight: number;
  headerHeight: number;
  contentTopOffset: number;
  showCustomWindowControls: boolean;
  enableDragRegion: boolean;
  useNativeDecorations: boolean;
}

/**
 * The shell picks one of three chrome strategies based on the host OS:
 *
 * - macOS: an `Overlay` title bar reserved by the OS so we paint a
 *   28px transparent drag strip and rely on macOS traffic-light
 *   controls.
 * - Windows / Linux: keep native decorations by default so users get
 *   the familiar title bar, and only opt into self-painted controls
 *   once the user explicitly enables `*CustomControls`. Linux avoids
 *   enabling the drag region by default to side-step GTK / Wayland
 *   quirks.
 */
export function getWindowChromePolicy(
  options: WindowChromeOptions = {},
): WindowChromePolicy {
  const platform = options.platform ?? detectPlatform();
  const headerHeight = APP_TOP_BAR_HEIGHT;

  if (platform === "macos") {
    const dragBarHeight = MAC_OVERLAY_DRAG_BAR_HEIGHT;

    return {
      platform,
      dragBarHeight,
      headerHeight,
      contentTopOffset: dragBarHeight + headerHeight,
      showCustomWindowControls: false,
      enableDragRegion: true,
      useNativeDecorations: true,
    };
  }

  if (platform === "windows") {
    const useCustomControls = options.windowsCustomControls ?? false;
    const dragBarHeight = useCustomControls ? CUSTOM_CHROME_DRAG_BAR_HEIGHT : 0;

    return {
      platform,
      dragBarHeight,
      headerHeight,
      contentTopOffset: dragBarHeight + headerHeight,
      showCustomWindowControls: useCustomControls,
      enableDragRegion: useCustomControls,
      useNativeDecorations: !useCustomControls,
    };
  }

  if (platform === "linux") {
    const useCustomControls = options.linuxCustomControls ?? false;
    const dragBarHeight = useCustomControls ? CUSTOM_CHROME_DRAG_BAR_HEIGHT : 0;

    return {
      platform,
      dragBarHeight,
      headerHeight,
      contentTopOffset: dragBarHeight + headerHeight,
      showCustomWindowControls: useCustomControls,
      enableDragRegion: useCustomControls,
      useNativeDecorations: !useCustomControls,
    };
  }

  return {
    platform,
    dragBarHeight: 0,
    headerHeight,
    contentTopOffset: headerHeight,
    showCustomWindowControls: false,
    enableDragRegion: false,
    useNativeDecorations: true,
  };
}

export function getDragRegionAttrs(enabled: boolean): Record<string, unknown> {
  if (!enabled) {
    return {};
  }

  return {
    "data-tauri-drag-region": true,
  };
}

export function getDragRegionStyle(enabled: boolean): CSSProperties {
  if (!enabled) {
    return {};
  }

  return {
    WebkitAppRegion: "drag",
  } as CSSProperties;
}

export function getNoDragStyle(): CSSProperties {
  return {
    WebkitAppRegion: "no-drag",
  } as CSSProperties;
}
