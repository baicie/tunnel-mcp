import { invoke } from "@tauri-apps/api/core";
import type { McpServerStatus } from "../tunnel/types";

export async function startMcpServer(): Promise<McpServerStatus> {
  return invoke("start_mcp_server");
}

export async function stopMcpServer(): Promise<McpServerStatus> {
  return invoke("stop_mcp_server");
}

export async function getMcpStatus(): Promise<McpServerStatus> {
  return invoke("get_mcp_status");
}
