export type TunnelSettings = {
  openaiApiKey?: string;
  tunnelId?: string;
  tunnelClientPath?: string;
  autoStart: boolean;
  autoUpdateTunnelClient: boolean;
};

export type PublicTunnelSettings = Omit<TunnelSettings, "openaiApiKey"> & {
  openaiApiKeyMasked?: string;
  hasOpenaiApiKey: boolean;
};

export type TunnelStatus = {
  installed: boolean;
  running: boolean;
  version?: string;
  pid?: number;
  endpoint?: string;
  lastError?: string;
};

export type McpServerStatus = {
  running: boolean;
  port: number;
  tools: string[];
  resources: string[];
};

export type PermissionScope = {
  id: string;
  kind: "filesystem" | "command" | "app";
  pattern: string;
  access: "read" | "write" | "readwrite";
  requireApproval: boolean;
};
