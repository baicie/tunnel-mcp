import { useMemo, useState } from "react";
import { AppLayout } from "./layout/AppLayout";
import { routes, type RouteId } from "./routes";
import { useShellSettings } from "../lib/settings/useShellSettings";

export function ShellApp() {
  const [activeRoute, setActiveRoute] = useState<RouteId>("dashboard");
  const shellSettings = useShellSettings();

  const activeRouteConfig = useMemo(
    () => routes.find((route) => route.id === activeRoute) ?? routes[0],
    [activeRoute],
  );

  const Page = activeRouteConfig.component;

  return (
    <AppLayout
      routes={routes}
      activeRoute={activeRoute}
      onRouteChange={setActiveRoute}
      settings={shellSettings.settings}
    >
      <Page shellSettings={shellSettings} />
    </AppLayout>
  );
}
