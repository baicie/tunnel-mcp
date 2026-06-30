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
      tunnelClientVersion: undefined,
      resourceRoot: undefined,
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
          tunnelClientVersion: "0.1.0",
          resourceRoot: undefined,
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
          tools: ["files/read", "files/write"],
          resources: ["filesystem:/tmp"],
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

  it("does not treat write-only filesystem scope as readable resource authorization", () => {
    const items = buildChecklist(
      snapshot({
        permissions: [
          {
            id: "1",
            kind: "filesystem",
            pattern: "/tmp/**",
            access: "write",
            requireApproval: true,
          },
        ],
      }),
    );

    expect(items.find((item) => item.id === "permission-scope")?.done).toBe(
      false,
    );
  });
});

describe("buildProblems", () => {
  it("returns actionable setup problems", () => {
    const problems = buildProblems(snapshot());
    const ids = problems.map((item) => item.id);

    expect(ids).toContain("missing-openai-key");
    expect(ids).toContain("missing-tunnel-id");
    expect(ids).toContain("tunnel-not-installed");
    expect(ids).toContain("mcp-stopped");
    expect(ids).toContain("no-readable-permissions");
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

  it("classifies token related tunnel errors", () => {
    const problems = buildProblems(
      snapshot({
        tunnel: {
          installed: true,
          running: false,
          health: "unhealthy",
          localMcpPortOpen: false,
          lastError: "401 invalid token",
        },
      }),
    );

    expect(problems.find((item) => item.id === "token-invalid")).toBeTruthy();
  });

  it("classifies MCP port bind errors", () => {
    const problems = buildProblems(
      snapshot({
        mcp: {
          running: false,
          port: 17891,
          tools: [],
          resources: [],
          authorizedRoots: [],
          lastError: "failed to bind MCP server: address already in use",
        },
      }),
    );

    expect(
      problems.find((item) => item.id === "mcp-port-occupied"),
    ).toBeTruthy();
  });

  it("surfaces pending approvals", () => {
    const problems = buildProblems(
      snapshot({
        approvals: [
          {
            id: "1",
            source: "mcp",
            tool: "files.write",
            targetPath: "/tmp/a.txt",
            summary: "write",
            createdAt: 1,
            expiresAt: 2,
            status: "pending",
          },
        ],
      }),
    );

    expect(
      problems.find((item) => item.id === "pending-approvals"),
    ).toBeTruthy();
  });
});
