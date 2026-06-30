import type {
  ChecklistItem,
  DashboardProblem,
  DashboardSnapshot,
} from "./types";
import type { PermissionScope } from "../permissions/types";

function hasReadableFilesystemScope(scopes: PermissionScope[]): boolean {
  return scopes.some(
    (scope) =>
      scope.kind === "filesystem" &&
      (scope.access === "read" || scope.access === "readwrite") &&
      !scope.requireApproval,
  );
}

function includesAny(value: string | undefined, keywords: string[]): boolean {
  const text = value?.toLowerCase() ?? "";
  return keywords.some((keyword) => text.includes(keyword));
}

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
      label: "At least one readable resource directory authorized",
      done: hasReadableFilesystemScope(snapshot.permissions),
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

  if (!snapshot.tunnel.installed) {
    problems.push({
      id: "tunnel-not-installed",
      severity: "warning",
      title: "tunnel-client is not installed",
      message: "Install tunnel-client before starting the remote tunnel.",
      actionLabel: "Open Tunnel",
      actionPath: "/tunnel",
    });
  }

  if (snapshot.tunnel.installed && !snapshot.tunnel.running) {
    problems.push({
      id: "tunnel-not-running",
      severity: "info",
      title: "tunnel-client is not running",
      message: "Start tunnel-client after Settings and MCP Server are ready.",
      actionLabel: "Open Tunnel",
      actionPath: "/tunnel",
    });
  }

  if (snapshot.tunnel.running && !snapshot.tunnel.localMcpPortOpen) {
    problems.push({
      id: "local-mcp-unreachable",
      severity: "warning",
      title: "Local MCP endpoint is not reachable",
      message:
        "tunnel-client is running, but the configured local MCP port is not reachable.",
      actionLabel: "Open MCP",
      actionPath: "/mcp",
    });
  }

  if (snapshot.tunnel.lastError) {
    const isTokenError = includesAny(snapshot.tunnel.lastError, [
      "token",
      "unauthorized",
      "401",
      "403",
      "invalid key",
      "invalid token",
    ]);

    const isTunnelIdError = includesAny(snapshot.tunnel.lastError, [
      "tunnel id",
      "tunnel-id",
      "invalid tunnel",
    ]);

    problems.push({
      id: isTokenError
        ? "token-invalid"
        : isTunnelIdError
          ? "tunnel-id-invalid"
          : "tunnel-error",
      severity: "error",
      title: isTokenError
        ? "Token or key is invalid"
        : isTunnelIdError
          ? "Tunnel ID is invalid"
          : "tunnel-client error",
      message: snapshot.tunnel.lastError,
      actionLabel:
        isTokenError || isTunnelIdError ? "Open Settings" : "Open Tunnel",
      actionPath: isTokenError || isTunnelIdError ? "/settings" : "/tunnel",
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

  if (snapshot.mcp.lastError) {
    const isPortError = includesAny(snapshot.mcp.lastError, [
      "address already in use",
      "already in use",
      "bind",
      "port",
    ]);

    problems.push({
      id: isPortError ? "mcp-port-occupied" : "mcp-error",
      severity: "error",
      title: isPortError ? "MCP Server port is occupied" : "MCP Server error",
      message: snapshot.mcp.lastError,
      actionLabel: "Open MCP",
      actionPath: "/mcp",
    });
  }

  if (!hasReadableFilesystemScope(snapshot.permissions)) {
    problems.push({
      id: "no-readable-permissions",
      severity: "warning",
      title: "No readable local directory authorized",
      message:
        "Authorize at least one readable filesystem scope so GPT can read local resources.",
      actionLabel: "Open Permissions",
      actionPath: "/permissions",
    });
  }

  const pendingApprovals = snapshot.approvals.filter(
    (approval) => approval.status === "pending",
  );

  if (pendingApprovals.length > 0) {
    problems.push({
      id: "pending-approvals",
      severity: "info",
      title: `${pendingApprovals.length} approval request(s) pending`,
      message: "Review pending write requests before they expire.",
      actionLabel: "Open Approvals",
      actionPath: "/approvals",
    });
  }

  return problems;
}
