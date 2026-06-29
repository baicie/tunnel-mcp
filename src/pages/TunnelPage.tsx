import { useEffect, useState } from "react";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import {
  getTunnelStatus,
  installTunnelClient,
  restartTunnelClient,
  startTunnelClient,
  stopTunnelClient,
} from "../lib/api/tunnel";
import type { TunnelStatus } from "../lib/tunnel/types";

const DEFAULT_MANIFEST_URL =
  "https://github.com/baicie/tunnel-client/releases/latest/download/manifest.json";

export function TunnelPage() {
  const [status, setStatus] = useState<TunnelStatus | null>(null);
  const [manifestUrl, setManifestUrl] = useState(DEFAULT_MANIFEST_URL);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    const next = await getTunnelStatus();
    setStatus(next);
  }

  async function run(action: () => Promise<TunnelStatus>) {
    setBusy(true);
    setError(null);
    try {
      const next = await action();
      setStatus(next);
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }

  useEffect(() => {
    refresh().catch((err) => setError(String(err)));
  }, []);

  return (
    <Page
      title="Tunnel"
      description="Install, start, stop and restart the local tunnel-client. Phase 2 only manages the binary lifecycle."
    >
      <Section title="Tunnel Status">
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
            <dt className="font-medium">PID</dt>
            <dd>{status?.pid ?? "-"}</dd>
          </div>
          {status?.endpoint ? (
            <div>
              <dt className="font-medium">Endpoint</dt>
              <dd>{status.endpoint}</dd>
            </div>
          ) : null}
          {status?.lastError ? (
            <div>
              <dt className="font-medium">Last Error</dt>
              <dd className="text-red-500">{status.lastError}</dd>
            </div>
          ) : null}
        </dl>
      </Section>

      <Section title="Lifecycle">
        <label className="flex flex-col gap-1 text-sm">
          <span className="font-medium">Manifest URL</span>
          <Input
            value={manifestUrl}
            onChange={(event) => setManifestUrl(event.target.value)}
          />
        </label>

        <div className="flex flex-wrap gap-2 pt-2">
          <Button
            variant="default"
            disabled={busy}
            onClick={() => run(() => installTunnelClient({ manifestUrl }))}
          >
            Install
          </Button>
          <Button
            variant="outline"
            disabled={busy || !status?.installed}
            onClick={() => run(startTunnelClient)}
          >
            Start
          </Button>
          <Button
            variant="outline"
            disabled={busy || !status?.running}
            onClick={() => run(stopTunnelClient)}
          >
            Stop
          </Button>
          <Button
            variant="outline"
            disabled={busy || !status?.installed}
            onClick={() => run(restartTunnelClient)}
          >
            Restart
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
