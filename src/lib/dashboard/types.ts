import type { ApprovalRequest } from "../approvals/types";
import type { PermissionScope } from "../permissions/types";
import type {
  McpServerStatus,
  PublicTunnelSettings,
  TunnelStatus,
} from "../tunnel/types";

export type ChecklistItem = {
  id: string;
  label: string;
  done: boolean;
  actionLabel?: string;
  actionPath?: string;
};

export type ProblemSeverity = "info" | "warning" | "error";

export type DashboardProblem = {
  id: string;
  severity: ProblemSeverity;
  title: string;
  message: string;
  actionLabel?: string;
  actionPath?: string;
};

export type DashboardSnapshot = {
  settings: PublicTunnelSettings;
  tunnel: TunnelStatus;
  mcp: McpServerStatus;
  permissions: PermissionScope[];
  approvals: ApprovalRequest[];
};
