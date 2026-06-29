import { useEffect, useState } from "react";
import { Button } from "../components/ui/button";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import { getMcpStatus, startMcpServer, stopMcpServer } from "../lib/api/mcp";
import type { McpServerStatus } from "../lib/tunnel/types";

export function McpServerPage() {
  const [status, setStatus] = useState<McpServerStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function run(action: () => Promise<McpServerStatus>) {
    setBusy(true);
    setError(null);
    try {
      setStatus(await action());
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    run(getMcpStatus);
  }, []);

  return (
    <Page
      title="MCP Server"
      description="Embedded local MCP server. Phase 3 exposes safe read-only resources over JSON-RPC at 127.0.0.1."
    >
      <Section title="Status">
        <dl className="grid gap-2 text-sm">
          <div>
            <dt className="font-medium">Running</dt>
            <dd>{status?.running ? "Yes" : "No"}</dd>
          </div>
          <div>
            <dt className="font-medium">Listen</dt>
            <dd>127.0.0.1:{status?.port ?? 17891}</dd>
          </div>
        </dl>
      </Section>

      <Section title="Tools">
        {status?.tools.length ? (
          <ul className="list-disc pl-5 text-sm">
            {status.tools.map((tool) => (
              <li key={tool}>{tool}</li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-muted-foreground">No tools exposed.</p>
        )}
      </Section>

      <Section title="Resources">
        {status?.resources.length ? (
          <ul className="list-disc pl-5 text-sm">
            {status.resources.map((resource) => (
              <li key={resource}>{resource}</li>
            ))}
          </ul>
        ) : (
          <p className="text-sm text-muted-foreground">
            No resources authorized.
          </p>
        )}
      </Section>

      <Section title="Lifecycle">
        <div className="flex flex-wrap gap-2">
          <Button
            variant="default"
            disabled={busy || status?.running}
            onClick={() => run(startMcpServer)}
          >
            Start
          </Button>
          <Button
            variant="outline"
            disabled={busy || !status?.running}
            onClick={() => run(stopMcpServer)}
          >
            Stop
          </Button>
        </div>
        {error ? (
          <p role="alert" className="pt-2 text-sm text-red-500">
            {error}
          </p>
        ) : null}
      </Section>
    </Page>
  );
}
