import { useEffect, useState } from "react";
import { Button } from "../components/ui/button";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import {
  approveRequest,
  listApprovalRequests,
  rejectRequest,
} from "../lib/api/approvals";
import type { ApprovalRequest } from "../lib/approvals/types";

function statusClasses(status: ApprovalRequest["status"]): string {
  switch (status) {
    case "approved":
      return "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400";
    case "rejected":
      return "bg-red-500/15 text-red-600 dark:text-red-400";
    case "expired":
      return "bg-gray-500/15 text-gray-600 dark:text-gray-400";
    case "pending":
    default:
      return "bg-blue-500/15 text-blue-600 dark:text-blue-400";
  }
}

function formatTimestamp(ts: number): string {
  return new Date(ts).toLocaleString();
}

export function ApprovalsPage() {
  const [requests, setRequests] = useState<ApprovalRequest[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function refresh() {
    setError(null);
    try {
      setRequests(await listApprovalRequests());
    } catch (err) {
      setError(String(err));
    }
  }

  async function act(
    action: () => Promise<ApprovalRequest>,
    successLabel: string,
  ) {
    setBusy(true);
    setError(null);
    try {
      await action();
      await refresh();
    } catch (err) {
      setError(`${successLabel} failed: ${String(err)}`);
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    refresh();
  }, []);

  return (
    <Page
      title="Approvals"
      description="Pending write requests from MCP. Approve to let the write execute, reject to deny."
    >
      <Section title={`Requests (${requests.length})`}>
        <div className="flex items-center justify-between gap-2 pb-2 text-sm">
          <span className="text-xs text-muted-foreground">
            Approved records stay visible for audit. Pending records expire
            automatically if left untouched.
          </span>
          <Button
            variant="outline"
            size="sm"
            disabled={busy}
            onClick={() => refresh()}
          >
            Refresh
          </Button>
        </div>
        {error ? (
          <p role="alert" className="pb-2 text-sm text-red-500">
            {error}
          </p>
        ) : null}
        {requests.length ? (
          <ul className="grid gap-2 text-sm">
            {requests.map((request) => (
              <li key={request.id} className="rounded border border-border p-3">
                <header className="flex flex-wrap items-center gap-2">
                  <strong>{request.tool}</strong>
                  <code className="break-all">{request.targetPath}</code>
                  <span
                    className={`rounded px-2 py-0.5 text-xs ${statusClasses(
                      request.status,
                    )}`}
                  >
                    {request.status}
                  </span>
                </header>
                <p className="mt-2 text-xs text-muted-foreground">
                  {request.summary}
                </p>
                <p className="mt-1 text-xs text-muted-foreground">
                  Created {formatTimestamp(request.createdAt)} · Expires{" "}
                  {formatTimestamp(request.expiresAt)}
                </p>
                {request.diff ? (
                  <pre className="mt-2 max-h-48 overflow-auto rounded bg-muted p-2 text-xs">
                    {request.diff}
                  </pre>
                ) : null}
                {request.status === "pending" ? (
                  <div className="mt-3 flex gap-2">
                    <Button
                      size="sm"
                      variant="success"
                      disabled={busy}
                      onClick={() =>
                        act(() => approveRequest(request.id), "Approve")
                      }
                    >
                      Approve
                    </Button>
                    <Button
                      size="sm"
                      variant="destructive"
                      disabled={busy}
                      onClick={() =>
                        act(() => rejectRequest(request.id), "Reject")
                      }
                    >
                      Reject
                    </Button>
                  </div>
                ) : null}
              </li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-muted-foreground">
            No approval requests yet. When the MCP server wants to write a file,
            a request will appear here.
          </p>
        )}
      </Section>
    </Page>
  );
}
