import { FormEvent, useEffect, useState } from "react";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import { Button } from "../components/ui/button";
import { getTunnelSettings, saveTunnelSettings } from "../lib/api/tunnel";
import type { PublicTunnelSettings, TunnelSettings } from "../lib/tunnel/types";

const EMPTY_SETTINGS: TunnelSettings = {
  openaiApiKey: "",
  tunnelId: "",
  tunnelClientPath: "",
  tunnelClientVersion: undefined,
  resourceRoot: "",
  mcpServerPort: 17891,
  logLevel: "info",
  autoStart: false,
  autoUpdateTunnelClient: true,
};

export function SettingsPage() {
  const [settings, setSettings] = useState<TunnelSettings>(EMPTY_SETTINGS);
  const [publicSettings, setPublicSettings] =
    useState<PublicTunnelSettings | null>(null);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getTunnelSettings()
      .then((value) => {
        setPublicSettings(value);
        setSettings({
          openaiApiKey: "",
          tunnelId: value.tunnelId ?? "",
          tunnelClientPath: value.tunnelClientPath ?? "",
          tunnelClientVersion: value.tunnelClientVersion,
          resourceRoot: value.resourceRoot ?? "",
          mcpServerPort: value.mcpServerPort,
          logLevel: value.logLevel,
          autoStart: value.autoStart,
          autoUpdateTunnelClient: value.autoUpdateTunnelClient,
        });
      })
      .catch((err) => setError(String(err)));
  }, []);

  async function submit(event: FormEvent) {
    event.preventDefault();
    setSaving(true);
    setError(null);
    try {
      const saved = await saveTunnelSettings(settings);
      setPublicSettings(saved);
      setSettings((prev) => ({
        ...prev,
        openaiApiKey: "",
        tunnelClientVersion: saved.tunnelClientVersion,
      }));
    } catch (err) {
      setError(String(err));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Page
      title="Settings"
      description="Configure startup behavior, MCP port, logging, update policy, resource root and tunnel-client behavior."
    >
      <Section>
        <form onSubmit={submit} className="flex flex-col gap-5">
          <label className="flex flex-col gap-1 text-sm">
            <span className="font-medium">OpenAI Key</span>
            <input
              type="password"
              className="rounded-md border border-border-default bg-background px-3 py-2 text-sm"
              value={settings.openaiApiKey ?? ""}
              placeholder={publicSettings?.openaiApiKeyMasked ?? "sk-..."}
              onChange={(event) =>
                setSettings((prev) => ({
                  ...prev,
                  openaiApiKey: event.target.value,
                }))
              }
            />
            {publicSettings?.hasOpenaiApiKey ? (
              <span className="text-xs text-muted-foreground">
                Stored. Leave blank to keep the existing key.
              </span>
            ) : null}
          </label>

          <label className="flex flex-col gap-1 text-sm">
            <span className="font-medium">Tunnel ID</span>
            <input
              className="rounded-md border border-border-default bg-background px-3 py-2 text-sm"
              value={settings.tunnelId ?? ""}
              onChange={(event) =>
                setSettings((prev) => ({
                  ...prev,
                  tunnelId: event.target.value,
                }))
              }
            />
          </label>

          <label className="flex flex-col gap-1 text-sm">
            <span className="font-medium">Tunnel Client Path</span>
            <input
              className="rounded-md border border-border-default bg-background px-3 py-2 text-sm"
              value={settings.tunnelClientPath ?? ""}
              onChange={(event) =>
                setSettings((prev) => ({
                  ...prev,
                  tunnelClientPath: event.target.value,
                }))
              }
            />
          </label>

          <label className="flex flex-col gap-1 text-sm">
            <span className="font-medium">MCP Server Port</span>
            <input
              type="number"
              min={1}
              max={65535}
              className="rounded-md border border-border-default bg-background px-3 py-2 text-sm"
              value={settings.mcpServerPort}
              onChange={(event) =>
                setSettings((prev) => ({
                  ...prev,
                  mcpServerPort: Number(event.target.value) || 17891,
                }))
              }
            />
          </label>

          <label className="flex flex-col gap-1 text-sm">
            <span className="font-medium">Log Level</span>
            <select
              className="rounded-md border border-border-default bg-background px-3 py-2 text-sm"
              value={settings.logLevel}
              onChange={(event) =>
                setSettings((prev) => ({
                  ...prev,
                  logLevel: event.target.value as TunnelSettings["logLevel"],
                }))
              }
            >
              <option value="error">error</option>
              <option value="warn">warn</option>
              <option value="info">info</option>
              <option value="debug">debug</option>
              <option value="trace">trace</option>
            </select>
          </label>

          <label className="flex flex-col gap-1 text-sm">
            <span className="font-medium">Resource Root</span>
            <input
              className="rounded-md border border-border-default bg-background px-3 py-2 text-sm"
              value={settings.resourceRoot ?? ""}
              placeholder="/path/to/authorized/root"
              onChange={(event) =>
                setSettings((prev) => ({
                  ...prev,
                  resourceRoot: event.target.value,
                }))
              }
            />
            <span className="text-xs text-muted-foreground">
              Phase 1 stores the intended root only. Later phases must still
              require explicit resource authorization.
            </span>
          </label>

          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={settings.autoStart}
              onChange={(event) =>
                setSettings((prev) => ({
                  ...prev,
                  autoStart: event.target.checked,
                }))
              }
            />
            <span>Auto start tunnel-client</span>
          </label>

          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={settings.autoUpdateTunnelClient}
              onChange={(event) =>
                setSettings((prev) => ({
                  ...prev,
                  autoUpdateTunnelClient: event.target.checked,
                }))
              }
            />
            <span>Auto update tunnel-client</span>
          </label>

          {error ? (
            <p role="alert" className="text-sm text-red-500">
              {error}
            </p>
          ) : null}

          <div>
            <Button type="submit" variant="default" disabled={saving}>
              {saving ? "Saving..." : "Save"}
            </Button>
          </div>
        </form>
      </Section>
    </Page>
  );
}
