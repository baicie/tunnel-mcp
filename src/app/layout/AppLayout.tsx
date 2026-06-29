import { useEffect, useMemo, type PropsWithChildren } from "react";
import type { RouteId, ShellRoute } from "../routes";
import type { ShellSettings } from "../../lib/settings/settings";
import { AppContent } from "./AppContent";
import { AppTopBar } from "./AppTopBar";
import { WindowDragBar } from "../../components/titlebar/WindowDragBar";
import { setWindowDecorations } from "../../lib/platform/window";
import { getWindowChromePolicy } from "../../lib/platform/windowChrome";

interface AppLayoutProps extends PropsWithChildren {
  routes: ShellRoute[];
  activeRoute: RouteId;
  onRouteChange: (route: RouteId) => void;
  settings: ShellSettings;
}

/**
 * Tolerant view of `ShellSettings`. The generated settings file ships
 * with `theme` / `startMinimized`; if a future caller opts into a
 * `windowChrome` flag the layout threads it through without forcing
 * the auto-generated `ShellSettings` interface to change.
 */
interface ShellSettingsWithChrome {
  theme: ShellSettings["theme"];
  startMinimized?: boolean;
  windowChrome?: {
    linuxCustomControls?: boolean;
    windowsCustomControls?: boolean;
  };
}

export function AppLayout(props: AppLayoutProps) {
  const settings = props.settings as ShellSettingsWithChrome;

  const chrome = useMemo(
    () =>
      getWindowChromePolicy({
        linuxCustomControls:
          settings.windowChrome?.linuxCustomControls ?? false,
        windowsCustomControls:
          settings.windowChrome?.windowsCustomControls ?? false,
      }),
    [
      settings.windowChrome?.linuxCustomControls,
      settings.windowChrome?.windowsCustomControls,
    ],
  );

  useEffect(() => {
    void setWindowDecorations(chrome.useNativeDecorations);
  }, [chrome.useNativeDecorations]);

  return (
    <div
      className="flex h-screen flex-col overflow-hidden bg-background text-foreground selection:bg-primary/30"
      style={{ paddingTop: chrome.contentTopOffset }}
    >
      <WindowDragBar chrome={chrome} />

      <AppTopBar
        routes={props.routes}
        activeRoute={props.activeRoute}
        onRouteChange={props.onRouteChange}
        chrome={chrome}
      />

      <main className="flex min-h-0 flex-1 flex-col overflow-y-auto">
        <AppContent>{props.children}</AppContent>
      </main>
    </div>
  );
}
