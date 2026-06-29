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

const allowedInvokeFile = "src/lib/api/shell.ts";

// Allow-list intentionally excludes files that must reference legacy
// markers for legitimate reasons:
//   - the scanner itself (this file)
//   - the runtime surface test that asserts absence of these markers
const allowedFiles = new Set([
  "scripts/check-shell-boundary.mjs",
  "scripts\\check-shell-boundary.mjs",
  "src/test/shellRuntimeSurface.test.ts",
  "src\\test\\shellRuntimeSurface.test.ts",
]);

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

  if (containsTauriInvokeCall(content) && rel !== allowedInvokeFile) {
    violations.push(
      `${rel}: Tauri invoke must only appear in ${allowedInvokeFile}`,
    );
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
