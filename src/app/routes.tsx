import type { ComponentType } from "react";
import { DashboardPage } from "../pages/DashboardPage";
import { SettingsPage } from "../pages/SettingsPage";
import { AboutPage } from "../pages/AboutPage";
import type { UseShellSettingsResult } from "../lib/settings/useShellSettings";

export type RouteId = "dashboard" | "settings" | "about";

export interface ShellPageProps {
  shellSettings: UseShellSettingsResult;
}

export interface ShellRoute {
  id: RouteId;
  label: string;
  title: string;
  component: ComponentType<ShellPageProps>;
}

export const routes: ShellRoute[] = [
  {
    id: "dashboard",
    label: "Dashboard",
    title: "Dashboard",
    component: DashboardPage,
  },
  {
    id: "settings",
    label: "Settings",
    title: "Settings",
    component: SettingsPage,
  },
  {
    id: "about",
    label: "About",
    title: "About",
    component: AboutPage,
  },
];
