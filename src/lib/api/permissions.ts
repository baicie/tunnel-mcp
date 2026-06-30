import { invoke } from "@tauri-apps/api/core";
import type {
  PermissionAccess,
  PermissionDecision,
  PermissionScope,
} from "../permissions/types";

export type NewPermissionScope = Omit<PermissionScope, "id">;

export async function listPermissionScopes(): Promise<PermissionScope[]> {
  return invoke<PermissionScope[]>("list_permission_scopes");
}

export async function addPermissionScope(
  scope: NewPermissionScope,
): Promise<PermissionScope> {
  return invoke<PermissionScope>("add_permission_scope", { scope });
}

export async function removePermissionScope(
  id: string,
): Promise<PermissionScope[]> {
  return invoke<PermissionScope[]>("remove_permission_scope", { id });
}

export async function checkPermission(
  path: string,
  access: PermissionAccess,
): Promise<PermissionDecision> {
  return invoke<PermissionDecision>("check_permission", { path, access });
}
