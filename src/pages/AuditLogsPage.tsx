import { FormEvent, useEffect, useState } from "react";
import { Button } from "../components/ui/button";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import { exportDiagnostics, listLogs } from "../lib/api/logs";
import type { AuditLogEvent } from "../lib/logs/types";

export function AuditLogsPage() {
  const [logs, setLogs] = useState<AuditLogEvent[]>([]);
  const [type, setType] = useState("");
  const [requestId, setRequestId] = useState("");
  const [diagnosticsPath, setDiagnosticsPath] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function refresh() {
    setBusy(true);
    setError(null);
    try {
      setLogs(
        await listLogs({
          type: type || undefined,
          requestId: requestId || undefined,
          limit: 200,
        }),
      );
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    await refresh();
  }

  async function onExport() {
    setError(null);
    try {
      const path = await exportDiagnostics();
      setDiagnosticsPath(path);
    } catch (err) {
      setError(String(err));
    }
  }

  useEffect(() => {
    refresh();
  }, []);

  return (
    <Page
      title="Audit Logs"
      description="Tunnel, MCP, permission and approval events, exportable as a redacted diagnostics bundle."
    >
      <div className="flex flex-wrap items-center justify-between gap-2">
        <form
          onSubmit={submit}
          className="flex flex-wrap items-center gap-2 text-sm"
        >
          <input
            value={type}
            onChange={(event) => setType(event.target.value)}
            placeholder="type"
            className="rounded border border-border bg-background px-2 py-1 text-sm"
          />
          <input
            value={requestId}
            onChange={(event) => setRequestId(event.target.value)}
            placeholder="requestId"
            className="rounded border border-border bg-background px-2 py-1 text-sm"
          />
          <Button type="submit" variant="outline" size="sm" disabled={busy}>
            Filter
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={busy}
            onClick={refresh}
          >
            Refresh
          </Button>
        </form>

        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={onExport}
          disabled={busy}
        >
          Export diagnostics
        </Button>
      </div>

      {diagnosticsPath ? (
        <p className="text-sm text-muted-foreground">
          Exported: <code className="break-all">{diagnosticsPath}</code>
        </p>
      ) : null}

      {error ? (
        <p role="alert" className="text-sm text-red-500">
          {error}
        </p>
      ) : null}

      <Section title="Recent Events">
        {logs.length === 0 ? (
          <p className="text-sm text-muted-foreground">
            No matching events yet.
          </p>
        ) : (
          <ul className="grid gap-3 text-sm">
            {logs.map((log) => (
              <li
                key={log.id}
                className="rounded border border-border px-3 py-2"
              >
                <div className="flex flex-wrap items-center gap-2">
                  <time className="text-xs text-muted-foreground">
                    {new Date(log.createdAt).toLocaleString()}
                  </time>
                  <strong>
                    [{log.level}] {log.type}
                  </strong>
                  {log.requestId ? (
                    <code className="break-all text-xs">{log.requestId}</code>
                  ) : null}
                </div>
                <p className="mt-1">{log.message}</p>
                <pre className="mt-1 overflow-x-auto rounded bg-muted px-2 py-1 text-xs">
                  {JSON.stringify(log.metadata, null, 2)}
                </pre>
              </li>
            ))}
          </ul>
        )}
      </Section>
    </Page>
  );
}
