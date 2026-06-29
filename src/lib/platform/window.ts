import { getCurrentWindow } from "@tauri-apps/api/window";

async function safeRun(action: () => Promise<void>): Promise<void> {
  try {
    await action();
  } catch {
    // Browser test environment or non-Tauri runtime.
  }
}

async function safeGet<T>(action: () => Promise<T>, fallback: T): Promise<T> {
  try {
    return await action();
  } catch {
    return fallback;
  }
}

export function minimizeWindow(): Promise<void> {
  return safeRun(() => getCurrentWindow().minimize());
}

export function toggleMaximizeWindow(): Promise<void> {
  return safeRun(() => getCurrentWindow().toggleMaximize());
}

export function closeWindow(): Promise<void> {
  return safeRun(() => getCurrentWindow().close());
}

export function setWindowDecorations(decorations: boolean): Promise<void> {
  return safeRun(() => getCurrentWindow().setDecorations(decorations));
}

export function isWindowMaximized(): Promise<boolean> {
  return safeGet(() => getCurrentWindow().isMaximized(), false);
}

export async function onWindowResized(
  handler: () => void | Promise<void>,
): Promise<() => void> {
  try {
    return await getCurrentWindow().onResized(() => {
      void handler();
    });
  } catch {
    return () => {};
  }
}
