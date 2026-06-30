import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

const root = process.cwd();

const requiredFiles = [
  "template.config.ts",
  "src/pages/DashboardPage.tsx",
  "src/pages/TunnelPage.tsx",
  "src/pages/McpServerPage.tsx",
  "src/pages/ResourcesPage.tsx",
  "src/pages/PermissionsPage.tsx",
  "src/pages/ApprovalsPage.tsx",
  "src/pages/AuditLogsPage.tsx",
  "src/pages/AboutPage.tsx",
  "docs/release/mvp-checklist.md",
  "src-tauri/src/product/tunnel/client_download.rs",
  "src-tauri/src/product/tunnel/client_process.rs",
  "src-tauri/src/product/mcp/server.rs",
  "src-tauri/src/product/permissions/policy.rs",
  "src-tauri/src/product/approvals/store.rs",
  "src-tauri/src/product/approvals/write_guard.rs",
  "src-tauri/src/product/logs/store.rs",
  "src-tauri/src/product/logs/diagnostics.rs",
  "src-tauri/src/product/security/local_token.rs",
  "src-tauri/src/product/security/secret_store.rs",
  "src-tauri/src/product/security/path_guard.rs",
];

function assert(condition, message) {
  if (!condition) {
    console.error(message);
    process.exit(1);
  }
}

function read(file) {
  return readFileSync(join(root, file), "utf8");
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: process.platform === "win32",
    ...options,
  });

  assert(result.status === 0, `${command} ${args.join(" ")} failed`);
}

for (const file of requiredFiles) {
  assert(existsSync(join(root, file)), `Missing required file: ${file}`);
}

const config = read("template.config.ts");
assert(
  config.includes("Tunnel MCP"),
  "template.config.ts should contain product name",
);
assert(
  config.includes("com.baicie.tunnel-mcp"),
  "template.config.ts should contain bundle identifier",
);

const checklist = read("docs/release/mvp-checklist.md");
for (const requiredItem of [
  "macOS arm64 installer can launch",
  "Windows x64 installer can launch",
  "OpenAI Key can be saved to secure storage",
  "MCP requires local token",
  "files/write creates approval request",
  "no token/key appears in diagnostics",
]) {
  assert(
    checklist.includes(requiredItem),
    `MVP checklist missing required item: ${requiredItem}`,
  );
}

const mcpServer = read("src-tauri/src/product/mcp/server.rs");
assert(
  mcpServer.includes("[127, 0, 0, 1]"),
  "MCP server must bind to 127.0.0.1",
);
assert(
  mcpServer.includes("x-tunnel-mcp-token"),
  "MCP server must require x-tunnel-mcp-token",
);
assert(
  mcpServer.includes("LocalTokenStore"),
  "MCP server must verify LocalTokenStore token",
);

const clientDownload = read("src-tauri/src/product/tunnel/client_download.rs");
assert(
  clientDownload.includes("verify_sha256"),
  "tunnel-client download must verify sha256",
);

const clientProcess = read("src-tauri/src/product/tunnel/client_process.rs");
assert(
  clientProcess.includes("--local-mcp-token"),
  "tunnel-client start must pass local MCP token",
);
assert(
  clientProcess.includes("--openai-key-env"),
  "tunnel-client start must pass OpenAI key through env indirection",
);

const settings = read("src-tauri/src/product/settings.rs");
assert(
  settings.includes("skip_serializing") ||
    !settings.includes("openai_api_key: Some"),
  "OpenAI key must not be persisted in settings.json",
);

const secretStore = read("src-tauri/src/product/security/secret_store.rs");
assert(
  secretStore.includes("keyring::Entry"),
  "OpenAI key must use keyring-backed secret store",
);

const writeTools = read("src-tauri/src/product/mcp/write_tools.rs");
const writeGuard = read("src-tauri/src/product/approvals/write_guard.rs");
assert(
  writeTools.includes("approvalId") && writeTools.includes("ApprovalRequired"),
  "files.write must require approval before write",
);
assert(
  writeGuard.includes("get_valid_approved"),
  "write guard must verify approved approval request before writing",
);

const diagnostics = read("src-tauri/src/product/logs/diagnostics.rs");
assert(
  diagnostics.includes("redact_value"),
  "diagnostics export must redact payload",
);

run("pnpm", ["check:all"]);
run("pnpm", ["test:unit"]);
run("cargo", ["test"], { cwd: join(root, "src-tauri") });

console.log("release smoke passed");
