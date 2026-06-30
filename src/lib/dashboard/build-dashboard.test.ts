import { describe, expect, it } from "vitest";
import { buildChecklist, buildProblems } from "./build-dashboard";
import type { DashboardSnapshot } from "./types";

function snapshot(
  overrides: Partial<DashboardSnapshot> = {},
): DashboardSnapshot {
  return {
    settings: {
      hasOpenaiApiKey: false,
      tunnelId: undefined,
      tunnelClientPath: undefined,
      autoStart: false,
      autoUpdateTunnelClient: true,
      mcpServerPort: 17891,
      logLevel: "info",
      openaiApiKeyMasked: undefined,
    },
    tunnel: {
      installed: false,
      running: false,
      health: "unknown",
      localMcpPortOpen: false,
    },
    mcp: {
      running: false,
      port: 17891,
      tools: [],
      resources: [],
      authorizedRoots: [],
    },
    permissions: [],
    approvals: [],
    ...overrides,
  };
}

describe("buildChecklist", () => {
  it("marks missing setup as not done", () => {
    const items = buildChecklist(snapshot());
    expect(items.every((item) => !item.done)).toBe(true);
  });

  it("marks ready setup as done", () => {
    const items = buildChecklist(
      snapshot({
        settings: {
          hasOpenaiApiKey: true,
          tunnelId: "tun_1",
          tunnelClientPath: "/bin/tunnel-client",
          autoStart: false,
          autoUpdateTunnelClient: true,
          mcpServerPort: 17891,
          logLevel: "info",
        },
        tunnel: {
          installed: true,
          running: true,
          health: "healthy",
          localMcpPortOpen: true,
        },
        mcp: {
          running: true,
          port: 17891,
          tools: ["files/read"],
          resources: ["filesystem"],
          authorizedRoots: ["/tmp"],
        },
        permissions: [
          {
            id: "1",
            kind: "filesystem",
            pattern: "/tmp/**",
            access: "read",
            requireApproval: false,
          },
        ],
      }),
    );
    expect(items.every((item) => item.done)).toBe(true);
  });
});

describe("buildProblems", () => {
  it("returns actionable problems", () => {
    const problems = buildProblems(snapshot());
    expect(problems.map((item) => item.id)).toContain("missing-openai-key");
    expect(problems.map((item) => item.id)).toContain("missing-tunnel-id");
    expect(problems.map((item) => item.id)).toContain("no-permissions");
  });

  it("includes tunnel last error", () => {
    const problems = buildProblems(
      snapshot({
        tunnel: {
          installed: true,
          running: false,
          health: "unhealthy",
          localMcpPortOpen: false,
          lastError: "crashed",
        },
      }),
    );
    expect(problems.find((item) => item.id === "tunnel-error")?.message).toBe(
      "crashed",
    );
  });
});
