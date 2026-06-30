import type { PermissionScope } from "../permissions/types";

export type WorkspaceProfile = {
  id: string;
  name: string;
  rootPath: string;
  permissionScopes: PermissionScope[];
  createdAt: number;
  updatedAt: number;
};

export type SaveWorkspaceProfileInput = {
  id?: string;
  name: string;
  rootPath: string;
  permissionScopes: PermissionScope[];
};
