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
  "src-tauri/src/product/tunnel/client_download.rs",
  "src-tauri/src/product/mcp/server.rs",
  "src-tauri/src/product/permissions/policy.rs",
  "src-tauri/src/product/approvals/store.rs",
  "src-tauri/src/product/logs/store.rs",
];

function assert(condition, message) {
  if (!condition) {
    console.error(message);
    process.exit(1);
  }
}

for (const file of requiredFiles) {
  assert(existsSync(join(root, file)), `Missing required file: ${file}`);
}

const config = readFileSync(join(root, "template.config.ts"), "utf8");
assert(
  config.includes("Tunnel MCP"),
  "template.config.ts should contain product name",
);
assert(
  config.includes("com.baicie.tunnel-mcp"),
  "template.config.ts should contain bundle identifier",
);

const commands = [
  ["pnpm", ["check:all"]],
  ["pnpm", ["test:unit"]],
];

for (const [command, args] of commands) {
  const result = spawnSync(command, args, {
    stdio: "inherit",
    shell: process.platform === "win32",
  });
  assert(result.status === 0, `${command} ${args.join(" ")} failed`);
}

const cargo = spawnSync("cargo", ["test"], {
  cwd: join(root, "src-tauri"),
  stdio: "inherit",
  shell: process.platform === "win32",
});
assert(cargo.status === 0, "cargo test failed");

console.log("release smoke passed");
