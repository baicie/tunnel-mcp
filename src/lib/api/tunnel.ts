import { invoke } from "@tauri-apps/api/core";
import type {
  McpServerStatus,
  PublicTunnelSettings,
  TunnelSettings,
  TunnelStatus,
} from "../tunnel/types";

export async function getTunnelSettings(): Promise<PublicTunnelSettings> {
  return invoke("get_tunnel_settings");
}

export async function saveTunnelSettings(
  settings: TunnelSettings,
): Promise<PublicTunnelSettings> {
  return invoke("save_tunnel_settings", { settings });
}

export async function getTunnelStatus(): Promise<TunnelStatus> {
  return invoke("get_tunnel_status");
}

export async function getMcpStatus(): Promise<McpServerStatus> {
  return invoke("get_mcp_status");
}
