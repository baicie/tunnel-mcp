import { useEffect, useState } from "react";
import { Page } from "../components/layout/Page";
import { Section } from "../components/layout/Section";
import {
  getMcpStatus,
  getTunnelSettings,
  getTunnelStatus,
} from "../lib/api/tunnel";
import type {
  McpServerStatus,
  PublicTunnelSettings,
  TunnelStatus,
} from "../lib/tunnel/types";

export function DashboardPage() {
  const [settings, setSettings] = useState<PublicTunnelSettings | null>(null);
  const [tunnel, setTunnel] = useState<TunnelStatus | null>(null);
  const [mcp, setMcp] = useState<McpServerStatus | null>(null);

  useEffect(() => {
    Promise.all([getTunnelSettings(), getTunnelStatus(), getMcpStatus()]).then(
      ([settingsValue, tunnelValue, mcpValue]) => {
        setSettings(settingsValue);
        setTunnel(tunnelValue);
        setMcp(mcpValue);
      },
    );
  }, []);

  const checklist = [
    ["OpenAI Key configured", Boolean(settings?.hasOpenaiApiKey)],
    ["Tunnel ID configured", Boolean(settings?.tunnelId)],
    ["tunnel-client installed", Boolean(tunnel?.installed)],
    ["tunnel-client running", Boolean(tunnel?.running)],
    ["MCP Server running", Boolean(mcp?.running)],
  ] as const;

  return (
    <Page
      title="Dashboard"
      description="Setup checklist, connection status, recent activity and problems will live here."
    >
      <Section title="Setup Checklist">
        <ul className="space-y-1 text-sm">
          {checklist.map(([label, done]) => (
            <li key={label} className="flex items-center gap-2">
              <span aria-hidden>{done ? "✅" : "⬜"}</span>
              <span>{label}</span>
            </li>
          ))}
        </ul>
      </Section>
    </Page>
  );
}
