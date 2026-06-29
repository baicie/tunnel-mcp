import { useEffect, useState } from "react";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import { getTunnelStatus } from "../lib/api/tunnel";
import type { TunnelStatus } from "../lib/tunnel/types";

export function TunnelPage() {
  const [status, setStatus] = useState<TunnelStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getTunnelStatus()
      .then(setStatus)
      .catch((err) => setError(String(err)));
  }, []);

  return (
    <Page
      title="Tunnel"
      description="Tunnel-client lifecycle placeholder. Phase 1 only shows status; Phase 2 manages download, start, stop and health checks."
    >
      <Section title="Tunnel Status">
        {error ? <p role="alert">{error}</p> : null}
        <dl className="grid gap-2 text-sm">
          <div>
            <dt className="font-medium">Installed</dt>
            <dd>{status?.installed ? "Yes" : "No"}</dd>
          </div>
          <div>
            <dt className="font-medium">Running</dt>
            <dd>{status?.running ? "Yes" : "No"}</dd>
          </div>
          <div>
            <dt className="font-medium">Version</dt>
            <dd>{status?.version ?? "Not detected"}</dd>
          </div>
          <div>
            <dt className="font-medium">Endpoint</dt>
            <dd>{status?.endpoint ?? "Not connected"}</dd>
          </div>
          <div>
            <dt className="font-medium">Last Error</dt>
            <dd>{status?.lastError ?? "None"}</dd>
          </div>
        </dl>
      </Section>
    </Page>
  );
}
