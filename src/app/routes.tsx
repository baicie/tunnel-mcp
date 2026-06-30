import type { ComponentType } from "react";
import { AboutPage } from "../pages/AboutPage";
import { ApprovalsPage } from "../pages/ApprovalsPage";
import { AuditLogsPage } from "../pages/AuditLogsPage";
import { DashboardPage } from "../pages/DashboardPage";
import { McpServerPage } from "../pages/McpServerPage";
import { PermissionsPage } from "../pages/PermissionsPage";
import { ResourcesPage } from "../pages/ResourcesPage";
import { SettingsPage } from "../pages/SettingsPage";
import { TunnelPage } from "../pages/TunnelPage";
import type { UseShellSettingsResult } from "../lib/settings/useShellSettings";

export type RouteId =
  | "dashboard"
  | "tunnel"
  | "mcp"
  | "resources"
  | "permissions"
  | "approvals"
  | "audit"
  | "settings"
  | "about";

export interface ShellPageProps {
  shellSettings: UseShellSettingsResult;
  onNavigate: (route: RouteId) => void;
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
    id: "tunnel",
    label: "Tunnel",
    title: "Tunnel",
    component: TunnelPage,
  },
  {
    id: "mcp",
    label: "MCP Server",
    title: "MCP Server",
    component: McpServerPage,
  },
  {
    id: "resources",
    label: "Resources",
    title: "Resources",
    component: ResourcesPage,
  },
  {
    id: "permissions",
    label: "Permissions",
    title: "Permissions",
    component: PermissionsPage,
  },
  {
    id: "approvals",
    label: "Approvals",
    title: "Approvals",
    component: ApprovalsPage,
  },
  {
    id: "audit",
    label: "Audit Logs",
    title: "Audit Logs",
    component: AuditLogsPage,
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
