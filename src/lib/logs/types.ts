export type LogLevel = "info" | "warn" | "error";

export type AuditLogEvent = {
  id: string;
  requestId?: string;
  type: string;
  level: LogLevel;
  message: string;
  metadata: Record<string, unknown>;
  createdAt: number;
};
