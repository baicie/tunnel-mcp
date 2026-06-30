import { useEffect, useMemo, useState } from "react";
import { Button } from "../components/ui/button";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import type { RouteId, ShellPageProps } from "../app/routes";
import { getDashboardSnapshot } from "../lib/api/dashboard";
import {
  buildChecklist,
  buildProblems,
} from "../lib/dashboard/build-dashboard";
import type {
  DashboardProblem,
  DashboardSnapshot,
} from "../lib/dashboard/types";
import type { ApprovalRequest } from "../lib/approvals/types";

function routeFromPath(path?: string): RouteId | null {
  switch (path) {
    case "/dashboard":
      return "dashboard";
    case "/tunnel":
      return "tunnel";
    case "/mcp":
      return "mcp";
    case "/resources":
      return "resources";
    case "/permissions":
      return "permissions";
    case "/approvals":
      return "approvals";
    case "/audit":
      return "audit";
    case "/settings":
      return "settings";
    case "/about":
      return "about";
    default:
      return null;
  }
}

function severityClasses(problem: DashboardProblem): string {
  switch (problem.severity) {
    case "error":
      return "border-red-500/40 bg-red-500/10 text-red-700 dark:text-red-300";
    case "warning":
      return "border-amber-500/40 bg-amber-500/10 text-amber-700 dark:text-amber-300";
    case "info":
    default:
      return "border-blue-500/40 bg-blue-500/10 text-blue-700 dark:text-blue-300";
  }
}

function approvalTimeLabel(approval: ApprovalRequest): string {
  const created = new Date(approval.createdAt).toLocaleString();
  const expires = new Date(approval.expiresAt).toLocaleString();

  return `Created ${created} · Expires ${expires}`;
}

export function DashboardPage(props: ShellPageProps) {
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function refresh() {
    setBusy(true);
    setError(null);

    try {
      setSnapshot(await getDashboardSnapshot());
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  function navigate(actionPath?: string) {
    const route = routeFromPath(actionPath);
    if (route) {
      props.onNavigate(route);
    }
  }

  useEffect(() => {
    refresh();
  }, []);

  const checklist = useMemo(
    () => (snapshot ? buildChecklist(snapshot) : []),
    [snapshot],
  );

  const problems = useMemo(
    () => (snapshot ? buildProblems(snapshot) : []),
    [snapshot],
  );

  const recentApprovals = useMemo(
    () =>
      snapshot
        ? [...snapshot.approvals]
            .sort((a, b) => b.createdAt - a.createdAt)
            .slice(0, 10)
        : [],
    [snapshot],
  );

  const pendingApprovals = recentApprovals.filter(
    (approval) => approval.status === "pending",
  );

  return (
    <Page
      title="Dashboard"
      description="Setup checklist, connection status, recent activity and actionable problems."
    >
      <div className="flex justify-end">
        <Button variant="outline" size="sm" disabled={busy} onClick={refresh}>
          Refresh
        </Button>
      </div>

      {error ? (
        <Section title="Load Error">
          <p role="alert" className="text-sm text-red-500">
            {error}
          </p>
        </Section>
      ) : null}

      <Section title="Setup Checklist">
        {checklist.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            Loading setup state...
          </p>
        ) : (
          <ul className="grid gap-2 text-sm">
            {checklist.map((item) => (
              <li
                key={item.id}
                className="flex flex-wrap items-center justify-between gap-2 rounded border border-border px-3 py-2"
              >
                <span className="flex items-center gap-2">
                  <span aria-hidden>{item.done ? "✅" : "⬜"}</span>
                  <span>{item.label}</span>
                </span>

                {!item.done && item.actionPath ? (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => navigate(item.actionPath)}
                  >
                    {item.actionLabel ?? "Open"}
                  </Button>
                ) : null}
              </li>
            ))}
          </ul>
        )}
      </Section>

      <Section title="Connection Status">
        <dl className="grid gap-2 text-sm">
          <div>
            <dt className="font-medium">Tunnel</dt>
            <dd>
              {snapshot?.tunnel.running
                ? "Connected / running"
                : "Disconnected"}
            </dd>
          </div>

          <div>
            <dt className="font-medium">Tunnel Health</dt>
            <dd>{snapshot?.tunnel.health ?? "unknown"}</dd>
          </div>

          <div>
            <dt className="font-medium">MCP</dt>
            <dd>
              {snapshot?.mcp.running
                ? `Ready on 127.0.0.1:${snapshot.mcp.port}`
                : "Unavailable"}
            </dd>
          </div>

          <div>
            <dt className="font-medium">Readable Roots</dt>
            <dd>
              {snapshot?.mcp.authorizedRoots?.length
                ? snapshot.mcp.authorizedRoots.join(", ")
                : "No authorized root exposed"}
            </dd>
          </div>
        </dl>
      </Section>

      <Section title="Problems">
        {!snapshot ? (
          <p className="text-sm text-muted-foreground">Loading problems...</p>
        ) : problems.length === 0 ? (
          <p className="text-sm text-muted-foreground">No known problems.</p>
        ) : (
          <ul className="grid gap-2 text-sm">
            {problems.map((problem) => (
              <li
                key={problem.id}
                className={`rounded border px-3 py-2 ${severityClasses(problem)}`}
              >
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <strong>{problem.title}</strong>
                  {problem.actionPath ? (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => navigate(problem.actionPath)}
                    >
                      {problem.actionLabel ?? "Open"}
                    </Button>
                  ) : null}
                </div>
                <p className="mt-1 text-xs">{problem.message}</p>
              </li>
            ))}
          </ul>
        )}
      </Section>

      <Section title="Recent Activity">
        <div className="grid gap-4 text-sm">
          <div>
            <h3 className="mb-2 text-sm font-medium">
              Recent Approvals
              {pendingApprovals.length
                ? ` (${pendingApprovals.length} pending)`
                : ""}
            </h3>

            {recentApprovals.length === 0 ? (
              <p className="text-sm text-muted-foreground">
                No approval request yet.
              </p>
            ) : (
              <ul className="grid gap-2">
                {recentApprovals.map((approval) => (
                  <li
                    key={approval.id}
                    className="rounded border border-border px-3 py-2"
                  >
                    <div className="flex flex-wrap items-center justify-between gap-2">
                      <span>
                        <strong>{approval.tool}</strong>{" "}
                        <code className="break-all">{approval.targetPath}</code>
                      </span>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => props.onNavigate("approvals")}
                      >
                        Open
                      </Button>
                    </div>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {approval.status} · {approval.summary}
                    </p>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {approvalTimeLabel(approval)}
                    </p>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div>
            <h3 className="mb-2 text-sm font-medium">Recent MCP Calls</h3>
            <p className="text-sm text-muted-foreground">
              MCP call history is not exposed by the current Phase 1–5 API
              surface yet. Approval activity is shown above; full call history
              should be wired when audit persistence lands.
            </p>
          </div>
        </div>
      </Section>
    </Page>
  );
}
