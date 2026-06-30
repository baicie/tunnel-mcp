import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const root = process.cwd();

describe("release smoke static checks", () => {
  it("contains Tunnel MCP app identity", () => {
    const config = readFileSync(join(root, "template.config.ts"), "utf8");
    expect(config).toContain("Tunnel MCP");
    expect(config).toContain("com.baicie.tunnel-mcp");
  });

  it("contains required product pages", () => {
    for (const page of [
      "DashboardPage",
      "TunnelPage",
      "McpServerPage",
      "ResourcesPage",
      "PermissionsPage",
      "ApprovalsPage",
      "AuditLogsPage",
    ]) {
      expect(existsSync(join(root, "src", "pages", `${page}.tsx`))).toBe(true);
    }
  });

  it("contains required rust product modules", () => {
    for (const file of [
      "src-tauri/src/product/tunnel/client_download.rs",
      "src-tauri/src/product/mcp/server.rs",
      "src-tauri/src/product/permissions/policy.rs",
      "src-tauri/src/product/approvals/store.rs",
      "src-tauri/src/product/logs/store.rs",
    ]) {
      expect(existsSync(join(root, file))).toBe(true);
    }
  });
});
