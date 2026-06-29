import { useEffect, useState } from "react";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import { getMcpStatus } from "../lib/api/tunnel";
import type { McpServerStatus } from "../lib/tunnel/types";

export function McpServerPage() {
  const [status, setStatus] = useState<McpServerStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getMcpStatus()
      .then(setStatus)
      .catch((err) => setError(String(err)));
  }, []);

  return (
    <Page
      title="MCP Server"
      description="Embedded local MCP server placeholder. Phase 1 does not start a real MCP runtime."
    >
      <Section title="Local Server">
        {error ? <p role="alert">{error}</p> : null}
        <dl className="grid gap-2 text-sm">
          <div>
            <dt className="font-medium">Running</dt>
            <dd>{status?.running ? "Yes" : "No"}</dd>
          </div>
          <div>
            <dt className="font-medium">Port</dt>
            <dd>{status?.port ?? 17891}</dd>
          </div>
          <div>
            <dt className="font-medium">Tools</dt>
            <dd>
              {status?.tools.length
                ? status.tools.join(", ")
                : "No tools exposed"}
            </dd>
          </div>
          <div>
            <dt className="font-medium">Resources</dt>
            <dd>
              {status?.resources.length
                ? status.resources.join(", ")
                : "No resources authorized"}
            </dd>
          </div>
        </dl>
      </Section>
    </Page>
  );
}
