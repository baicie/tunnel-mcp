import { invoke } from "@tauri-apps/api/core";
import type {
  McpServerStatus,
  PublicTunnelSettings,
  TunnelClientLogLine,
  TunnelSettings,
  TunnelStatus,
} from "../tunnel/types";

export type InstallTunnelClientInput = {
  manifestUrl: string;
  version?: string;
};

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

export async function installTunnelClient(
  input: InstallTunnelClientInput,
): Promise<TunnelStatus> {
  return invoke("install_tunnel_client", { input });
}

export async function startTunnelClient(): Promise<TunnelStatus> {
  return invoke("start_tunnel_client");
}

export async function stopTunnelClient(): Promise<TunnelStatus> {
  return invoke("stop_tunnel_client");
}

export async function restartTunnelClient(): Promise<TunnelStatus> {
  return invoke("restart_tunnel_client");
}

export async function getTunnelClientLogs(): Promise<TunnelClientLogLine[]> {
  return invoke("get_tunnel_client_logs");
}
