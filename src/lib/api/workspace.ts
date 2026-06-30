import { invoke } from "@tauri-apps/api/core";
import type {
  SaveWorkspaceProfileInput,
  WorkspaceProfile,
} from "../workspace/types";

export async function listWorkspaceProfiles(): Promise<WorkspaceProfile[]> {
  return invoke("list_workspace_profiles");
}

export async function saveWorkspaceProfile(
  profile: SaveWorkspaceProfileInput,
): Promise<WorkspaceProfile> {
  return invoke("save_workspace_profile", { profile });
}

export async function removeWorkspaceProfile(
  id: string,
): Promise<WorkspaceProfile[]> {
  return invoke("remove_workspace_profile", { id });
}
