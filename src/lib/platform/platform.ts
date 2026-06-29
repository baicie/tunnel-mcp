// Minimal platform detection for the renderer. The web view exposes
// the user agent and `navigator.platform`; running inside a Tauri
// webview reports the host desktop OS, which is enough to pick the
// window chrome policy below.

export type DesktopPlatform = "macos" | "windows" | "linux" | "unknown";

export function detectPlatform(): DesktopPlatform {
  try {
    if (typeof navigator === "undefined") {
      return "unknown";
    }

    const userAgent = navigator.userAgent ?? "";
    const platform = (navigator.platform ?? "").toLowerCase();

    if (/mac/i.test(userAgent) || platform.includes("mac")) {
      return "macos";
    }

    if (/windows|win32|win64/i.test(userAgent)) {
      return "windows";
    }

    if (/linux|x11/i.test(userAgent) && !/android/i.test(userAgent)) {
      return "linux";
    }

    return "unknown";
  } catch {
    return "unknown";
  }
}

export function isMac(): boolean {
  return detectPlatform() === "macos";
}

export function isWindows(): boolean {
  return detectPlatform() === "windows";
}

export function isLinux(): boolean {
  return detectPlatform() === "linux";
}
