import { invoke } from "@tauri-apps/api/core";
import type { ApprovalRequest } from "../approvals/types";

export async function listApprovalRequests(): Promise<ApprovalRequest[]> {
  return invoke<ApprovalRequest[]>("list_approval_requests");
}

export async function approveRequest(id: string): Promise<ApprovalRequest> {
  return invoke<ApprovalRequest>("approve_request", { id });
}

export async function rejectRequest(id: string): Promise<ApprovalRequest> {
  return invoke<ApprovalRequest>("reject_request", { id });
}
