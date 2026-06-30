import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const root = process.cwd();

function read(path: string) {
  return readFileSync(join(root, path), "utf8");
}

describe("release smoke static checks", () => {
  it("contains Tunnel MCP app identity", () => {
    const config = read("template.config.ts");

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
      "AboutPage",
    ]) {
      expect(existsSync(join(root, "src", "pages", `${page}.tsx`))).toBe(true);
    }
  });

  it("contains required rust product modules", () => {
    for (const file of [
      "src-tauri/src/product/tunnel/client_download.rs",
      "src-tauri/src/product/tunnel/client_process.rs",
      "src-tauri/src/product/mcp/server.rs",
      "src-tauri/src/product/mcp/extra_tools.rs",
      "src-tauri/src/product/permissions/policy.rs",
      "src-tauri/src/product/approvals/store.rs",
      "src-tauri/src/product/approvals/write_guard.rs",
      "src-tauri/src/product/commands/approval.rs",
      "src-tauri/src/product/integrations/app_open.rs",
      "src-tauri/src/product/workspace/profile.rs",
      "src-tauri/src/product/workspace/store.rs",
      "src-tauri/src/product/logs/store.rs",
      "src-tauri/src/product/logs/diagnostics.rs",
      "src-tauri/src/product/security/local_token.rs",
      "src-tauri/src/product/security/secret_store.rs",
      "src-tauri/src/product/security/path_guard.rs",
    ]) {
      expect(existsSync(join(root, file))).toBe(true);
    }
  });

  it("contains release checklist for MVP acceptance", () => {
    const checklist = read("docs/release/mvp-checklist.md");

    expect(checklist).toContain("macOS arm64 installer can launch");
    expect(checklist).toContain("Windows x64 installer can launch");
    expect(checklist).toContain("OpenAI Key can be saved to secure storage");
    expect(checklist).toContain("MCP requires local token");
    expect(checklist).toContain("files/write creates approval request");
    expect(checklist).toContain("no token/key appears in diagnostics");
  });

  it("keeps security-critical release invariants visible", () => {
    expect(read("src-tauri/src/product/mcp/server.rs")).toContain(
      "x-tunnel-mcp-token",
    );
    expect(read("src-tauri/src/product/tunnel/client_download.rs")).toContain(
      "verify_sha256",
    );
    expect(read("src-tauri/src/product/approvals/write_guard.rs")).toContain(
      "get_valid_approved",
    );
    expect(read("src-tauri/src/product/logs/diagnostics.rs")).toContain(
      "redact_value",
    );
  });
});