import { useEffect, useState } from "react";
import {
  closeWindow,
  isWindowMaximized,
  minimizeWindow,
  onWindowResized,
  toggleMaximizeWindow,
} from "../../lib/platform/window";
import { getNoDragStyle } from "../../lib/platform/windowChrome";

/**
 * Custom window controls. Only mounted when the active chrome policy
 * tells us we're self-painting decorations (i.e. `setDecorations(false)`
 * was called). On macOS / native-decorated platforms this component
 * must never be rendered — the OS draws the traffic lights / min-max-
 * close cluster for us.
 */
export function WindowControls() {
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    let cleanup: (() => void) | undefined;

    const sync = async () => {
      setMaximized(await isWindowMaximized());
    };

    void sync();

    void onWindowResized(sync).then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      cleanup?.();
    };
  }, []);

  return (
    <div
      className="flex items-center gap-1"
      style={getNoDragStyle()}
      aria-label="Window controls"
    >
      <button
        type="button"
        className="flex h-8 w-8 items-center justify-center rounded-md text-sm text-muted-foreground hover:bg-muted hover:text-foreground"
        aria-label="Minimize window"
        title="Minimize"
        onClick={() => {
          void minimizeWindow();
        }}
      >
        —
      </button>

      <button
        type="button"
        className="flex h-8 w-8 items-center justify-center rounded-md text-sm text-muted-foreground hover:bg-muted hover:text-foreground"
        aria-label={maximized ? "Restore window" : "Maximize window"}
        title={maximized ? "Restore" : "Maximize"}
        onClick={async () => {
          await toggleMaximizeWindow();
          setMaximized(await isWindowMaximized());
        }}
      >
        {maximized ? "▣" : "□"}
      </button>

      <button
        type="button"
        className="flex h-8 w-8 items-center justify-center rounded-md text-sm text-muted-foreground hover:bg-red-500/15 hover:text-red-500"
        aria-label="Close window"
        title="Close"
        onClick={() => {
          void closeWindow();
        }}
      >
        ×
      </button>
    </div>
  );
}
