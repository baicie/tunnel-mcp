import type {
  ChecklistItem,
  DashboardProblem,
  DashboardSnapshot,
} from "./types";

export function buildChecklist(snapshot: DashboardSnapshot): ChecklistItem[] {
  return [
    {
      id: "openai-key",
      label: "OpenAI Key configured",
      done: snapshot.settings.hasOpenaiApiKey,
      actionLabel: "Open Settings",
      actionPath: "/settings",
    },
    {
      id: "tunnel-id",
      label: "Tunnel ID configured",
      done: Boolean(snapshot.settings.tunnelId),
      actionLabel: "Open Settings",
      actionPath: "/settings",
    },
    {
      id: "tunnel-installed",
      label: "tunnel-client installed",
      done: snapshot.tunnel.installed,
      actionLabel: "Install",
      actionPath: "/tunnel",
    },
    {
      id: "tunnel-running",
      label: "tunnel-client running",
      done: snapshot.tunnel.running,
      actionLabel: "Start",
      actionPath: "/tunnel",
    },
    {
      id: "mcp-running",
      label: "MCP Server running",
      done: snapshot.mcp.running,
      actionLabel: "Start MCP",
      actionPath: "/mcp",
    },
    {
      id: "permission-scope",
      label: "At least one resource directory authorized",
      done: snapshot.permissions.some((scope) => scope.kind === "filesystem"),
      actionLabel: "Add Permission",
      actionPath: "/permissions",
    },
  ];
}

export function buildProblems(snapshot: DashboardSnapshot): DashboardProblem[] {
  const problems: DashboardProblem[] = [];

  if (!snapshot.settings.hasOpenaiApiKey) {
    problems.push({
      id: "missing-openai-key",
      severity: "warning",
      title: "OpenAI Key is missing",
      message: "Configure a key before starting tunnel-client.",
      actionLabel: "Open Settings",
      actionPath: "/settings",
    });
  }

  if (!snapshot.settings.tunnelId) {
    problems.push({
      id: "missing-tunnel-id",
      severity: "warning",
      title: "Tunnel ID is missing",
      message: "Configure the tunnel id provided by the connector.",
      actionLabel: "Open Settings",
      actionPath: "/settings",
    });
  }

  if (snapshot.tunnel.lastError) {
    problems.push({
      id: "tunnel-error",
      severity: "error",
      title: "tunnel-client error",
      message: snapshot.tunnel.lastError,
      actionLabel: "Open Tunnel",
      actionPath: "/tunnel",
    });
  }

  if (!snapshot.mcp.running) {
    problems.push({
      id: "mcp-stopped",
      severity: "info",
      title: "MCP Server is not running",
      message:
        "Start the local MCP server before connecting through the tunnel.",
      actionLabel: "Open MCP",
      actionPath: "/mcp",
    });
  }

  if (snapshot.permissions.length === 0) {
    problems.push({
      id: "no-permissions",
      severity: "warning",
      title: "No local directory authorized",
      message:
        "Authorize at least one directory so GPT can read local resources.",
      actionLabel: "Open Permissions",
      actionPath: "/permissions",
    });
  }

  return problems;
}
