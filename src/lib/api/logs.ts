import { invoke } from "@tauri-apps/api/core";
import type { AuditLogEvent } from "../logs/types";

export type ListLogsInput = {
  type?: string;
  requestId?: string;
  limit?: number;
};

export async function listLogs(
  input: ListLogsInput = {},
): Promise<AuditLogEvent[]> {
  return invoke("list_logs", { input });
}

export async function exportDiagnostics(): Promise<string> {
  return invoke("export_diagnostics");
}
