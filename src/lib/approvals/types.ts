export type ApprovalStatus = "pending" | "approved" | "rejected" | "expired";
export type ApprovalSource = "mcp";
export type ApprovalTool = "files.write" | "files.patch";

export type ApprovalRequest = {
  id: string;
  source: ApprovalSource;
  tool: ApprovalTool;
  targetPath: string;
  summary: string;
  diff?: string;
  contentSha256?: string;
  createdAt: number;
  expiresAt: number;
  status: ApprovalStatus;
};
