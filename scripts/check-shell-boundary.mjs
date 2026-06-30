import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = process.cwd();

// check-shell-boundary walks every file under `src/` and asserts that
// the only Tauri invoke call site is `src/lib/api/shell.ts` and that
// none of the legacy business markers appear in any non-allow-listed
// file.
//
// This scanner itself plus the runtime surface test are the only files
// legitimately allowed to mention these markers.
const forbiddenRuntimeMarkers = [
  "activeApp",
  "provider",
  "providers",
  "proxy",
  "mcp",
  "prompt",
  "prompts",
  "skills",
  "session",
  "sessions",
  "usage",
  "subscription",
  "workspace",
  "balance",
  "codex",
  "gemini",
  "claude",
  "openclaw",
  "opencode",
  "hermes",
  "webdav",
  "s3_sync",
];

// Allow-list intentionally excludes files that must reference legacy
// markers for legitimate reasons:
//   - the scanner itself (this file)
//   - the runtime surface test that asserts absence of these markers
//   - the route registry, which legitimately names product pages such as
//     McpServerPage after Phase 1
const allowedFiles = new Set([
  "scripts/check-shell-boundary.mjs",
  "scripts\\\\check-shell-boundary.mjs",
  "src/test/shellRuntimeSurface.test.ts",
  "src\\\\test\\\\shellRuntimeSurface.test.ts",
  "src/lib/brand/templateConfig.ts",
  "src\\\\lib\\\\brand\\\\templateConfig.ts",
  "src/lib/brand/brand.ts",
  "src\\\\lib\\\\brand\\\\brand.ts",
  "src/app/routes.tsx",
  "src\\\\app\\\\routes.tsx",
]);

// Product directories are allowed to mention tunnel / mcp / provider
// vocabulary. Markers are still forbidden anywhere else under `src/`.
const productDirectoryPrefixes = [
  "src/lib/tunnel/",
  "src/lib/mcp/",
  "src/lib/permissions/",
  "src/lib/approvals/",
  "src/lib/logs/",
  "src/lib/updater/",
  "src/lib/dashboard/",
  "src/pages/",
  "src/components/tunnel/",
  "src/components/mcp/",
  "src/components/permissions/",
  "src/components/approvals/",
  "src/components/logs/",
  "src/lib/api/tunnel.ts",
  "src/lib/api/mcp.ts",
  "src/lib/api/dashboard.ts",
];

function isProductPath(rel) {
  const normalized = toRepoPath(rel);
  return productDirectoryPrefixes.some((prefix) =>
    normalized.startsWith(prefix),
  );
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function toRepoPath(value) {
  return value.replace(/\\/g, "/");
}

function containsTauriInvokeCall(content) {
  return /\binvoke\s*(?:<[^;{}()]+>)?\s*\(/.test(content);
}

function isAllowListed(rel) {
  return allowedFiles.has(rel) || allowedFiles.has(toRepoPath(rel));
}

function walk(dir, files = []) {
  let entries = [];

  try {
    entries = readdirSync(dir);
  } catch {
    return files;
  }

  for (const entry of entries) {
    if (["node_modules", "dist", "target", ".git"].includes(entry)) {
      continue;
    }

    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);

    if (stat.isDirectory()) {
      walk(fullPath, files);
    } else {
      files.push(fullPath);
    }
  }

  return files;
}

const allowedInvokeFiles = [
  "src/lib/api/shell.ts",
  "src/lib/api/tunnel.ts",
  "src/lib/api/mcp.ts",
  "src/lib/api/permissions.ts",
  "src/lib/api/approvals.ts",
  "src/lib/api/dashboard.ts",
  "src/lib/api/logs.ts",
  "src/lib/api/updater.ts",
];

function allowedInvokeRel(rel) {
  const normalized = toRepoPath(rel);
  return allowedInvokeFiles.includes(normalized);
}

const files = walk(join(root, "src"));
const violations = [];

for (const file of files) {
  const rel = toRepoPath(relative(root, file));

  if (isAllowListed(rel)) {
    continue;
  }

  let content = "";

  try {
    content = readFileSync(file, "utf8");
  } catch {
    continue;
  }

  if (containsTauriInvokeCall(content) && !allowedInvokeRel(rel)) {
    violations.push(
      `${rel}: Tauri invoke must only appear in shell/tunnel/mcp api modules`,
    );
  }

  // Product paths are allowed to mention tunnel / mcp / provider
  // vocabulary; skip the marker scan entirely for them.
  if (isProductPath(rel)) {
    continue;
  }

  for (const marker of forbiddenRuntimeMarkers) {
    // Use word-boundary regex so substrings inside identifiers like
    // `QueryClientProvider` don't trip on `provider`.
    const pattern = new RegExp(`\\b${escapeRegExp(marker)}\\b`, "i");
    if (!pattern.test(content)) {
      continue;
    }

    // `<QueryClientProvider>` is part of @tanstack/react-query; the
    // shell template always wraps the app with it. Allow files that
    // only reference it via React provider composition.
    if (
      marker === "provider" &&
      (rel.endsWith("main.tsx") || rel.endsWith("ShellApp.test.tsx"))
    ) {
      continue;
    }

    violations.push(
      `${rel}: contains old frontend business marker "${marker}"`,
    );
  }
}

if (violations.length > 0) {
  console.error("Shell boundary check failed:");
  for (const violation of violations) {
    console.error(`- ${violation}`);
  }
  process.exit(1);
}

console.log("Shell boundary check passed.");
