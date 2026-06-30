export type PermissionKind = "filesystem" | "command" | "app";
export type PermissionAccess = "read" | "write" | "readwrite";

export type PermissionScope = {
  id: string;
  kind: PermissionKind;
  pattern: string;
  access: PermissionAccess;
  requireApproval: boolean;
};

export type PermissionDecision = {
  allowed: boolean;
  requireApproval: boolean;
  reason: string;
};
