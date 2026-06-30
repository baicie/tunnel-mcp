import { invoke } from "@tauri-apps/api/core";
import type {
  TunnelClientVersionStatus,
  UpdateCheckResult,
} from "../updater/types";

export async function checkAppUpdate(): Promise<UpdateCheckResult> {
  return invoke("check_app_update");
}

export async function checkTunnelClientUpdate(
  manifestUrl: string,
): Promise<TunnelClientVersionStatus> {
  return invoke("check_tunnel_client_update", { manifestUrl });
}

export async function updateTunnelClient(
  manifestUrl: string,
): Promise<TunnelClientVersionStatus> {
  return invoke("update_tunnel_client", { manifestUrl });
}

export async function rollbackTunnelClient(): Promise<TunnelClientVersionStatus> {
  return invoke("rollback_tunnel_client");
}
