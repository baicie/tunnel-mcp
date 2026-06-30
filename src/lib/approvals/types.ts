export type ApprovalStatus = "pending" | "approved" | "rejected" | "expired";
export type ApprovalTool = "files.write" | "files.patch";

export type ApprovalRequest = {
  id: string;
  source: string;
  tool: ApprovalTool;
  targetPath: string;
  summary: string;
  diff?: string;
  createdAt: number;
  expiresAt: number;
  status: ApprovalStatus;
};
