import { listApprovalRequests } from "./approvals";
import { getMcpStatus, getTunnelSettings, getTunnelStatus } from "./tunnel";
import { listPermissionScopes } from "./permissions";
import type { DashboardSnapshot } from "../dashboard/types";

export async function getDashboardSnapshot(): Promise<DashboardSnapshot> {
  const [settings, tunnel, mcp, permissions, approvals] = await Promise.all([
    getTunnelSettings(),
    getTunnelStatus(),
    getMcpStatus(),
    listPermissionScopes(),
    listApprovalRequests(),
  ]);

  return { settings, tunnel, mcp, permissions, approvals };
}
