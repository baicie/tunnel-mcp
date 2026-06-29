import { APP_BRAND } from "../../lib/brand/brand";
import type { RouteId, ShellRoute } from "../routes";
import { WindowControls } from "../../components/titlebar/WindowControls";
import { Badge, Button } from "../../components/ui";
import {
  getDragRegionAttrs,
  getDragRegionStyle,
  getNoDragStyle,
  type WindowChromePolicy,
} from "../../lib/platform/windowChrome";

interface AppTopBarProps {
  routes: ShellRoute[];
  activeRoute: RouteId;
  onRouteChange: (route: RouteId) => void;
  chrome: WindowChromePolicy;
}

/**
 * The shell's fixed top row. Follows the same three-zone layout on
 * every platform:
 *
 *   left: brand + status pill (clicking the brand jumps home)
 *   centre: route tabs
 *   right: quick action placeholders + (optional) custom close cluster
 *
 * The whole header is registered as a drag region when the active
 * chrome policy opts into it. Buttons stay clickable because each
 * non-drag slot is wrapped with `WebkitAppRegion: no-drag`.
 */
function getRouteLabel(route: ShellRoute): string {
  const candidate = route as ShellRoute & {
    label?: string;
    title?: string;
    name?: string;
  };

  return (
    candidate.label ?? candidate.title ?? candidate.name ?? String(route.id)
  );
}

export function AppTopBar(props: AppTopBarProps) {
  const activeRoute = props.routes.find(
    (route) => route.id === props.activeRoute,
  );
  const activeRouteLabel = activeRoute ? getRouteLabel(activeRoute) : "";

  return (
    <header
      className="fixed left-0 right-0 z-50 w-full border-b bg-background/80 backdrop-blur-md"
      {...getDragRegionAttrs(props.chrome.enableDragRegion)}
      style={{
        top: props.chrome.dragBarHeight,
        height: props.chrome.headerHeight,
        ...getDragRegionStyle(props.chrome.enableDragRegion),
      }}
    >
      <div className="flex h-full items-center justify-between gap-3 px-6">
        <div
          className="flex min-w-0 shrink-0 items-center gap-2"
          style={getNoDragStyle()}
        >
          <Button
            variant="link"
            className="h-auto truncate px-0 text-xl font-semibold no-underline hover:no-underline"
            onClick={() => {
              const dashboard = props.routes.find(
                (route) => String(route.id) === "dashboard",
              );

              if (dashboard) {
                props.onRouteChange(dashboard.id);
              }
            }}
          >
            {APP_BRAND.windowTitle}
          </Button>

          <Badge variant="success" className="hidden sm:inline-flex">
            Ready
          </Badge>
        </div>

        <nav
          className="hidden min-w-0 flex-1 justify-center md:flex"
          aria-label="Primary navigation"
          style={getNoDragStyle()}
        >
          <div className="flex max-w-full items-center gap-1 overflow-hidden rounded-2xl bg-muted p-1">
            {props.routes.map((route) => {
              const active = route.id === props.activeRoute;

              return (
                <button
                  key={String(route.id)}
                  type="button"
                  className={[
                    "truncate rounded-xl px-4 py-2 text-sm font-medium transition-colors",
                    active
                      ? "bg-background text-foreground shadow-sm"
                      : "text-muted-foreground hover:bg-background/60 hover:text-foreground",
                  ].join(" ")}
                  onClick={() => props.onRouteChange(route.id)}
                >
                  {getRouteLabel(route)}
                </button>
              );
            })}
          </div>
        </nav>

        <div
          className="flex min-w-0 shrink-0 items-center gap-2"
          style={getNoDragStyle()}
        >
          <span className="max-w-[160px] truncate text-sm text-muted-foreground md:hidden">
            {activeRouteLabel}
          </span>

          <Button variant="ghost" size="sm" title="Tunnel">
            Tunnel
          </Button>

          <Button variant="success" size="sm" title="Bridge">
            Bridge
          </Button>

          <Button
            variant="ghost"
            size="sm"
            title="Logs"
            className="hidden lg:inline-flex"
          >
            Logs
          </Button>

          <Button
            size="icon"
            className="ml-1 rounded-full bg-orange-500 text-white shadow-lg shadow-orange-500/30 hover:bg-orange-600 hover:text-white"
            title="Add"
            aria-label="Add"
          >
            +
          </Button>

          {props.chrome.showCustomWindowControls && <WindowControls />}
        </div>
      </div>
    </header>
  );
}
