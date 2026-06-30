import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

const root = process.cwd();

// Each forbidden marker is composed from its character codes so the
// scanner script can still scan this test file (it lives in its own
// allow-list). At runtime the array is rebuilt into the same words
// the scanner itself checks for.
const forbiddenMarkers: string[] = [
  "ac" + "tiveApp",
  "pro" + "vider",
  "pro" + "viders",
  "pro" + "xy",
  "mc" + "p",
  "pro" + "mpt",
  "pro" + "mpts",
  "skil" + "ls",
  "sess" + "ion",
  "sess" + "ions",
  "usa" + "ge",
  "subsc" + "ription",
  "work" + "space",
  "bal" + "ance",
  "cod" + "ex",
  "gem" + "ini",
  "clau" + "de",
  "open" + "claw",
  "open" + "code",
  "her" + "mes",
  "web" + "dav",
  "s3" + "_sync",
];

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function toRepoPath(value: string): string {
  return value.replace(/\\/g, "/");
}

function containsTauriInvokeCall(content: string): boolean {
  return /\binvoke\s*(?:<[^;{}()]+>)?\s*\(/.test(content);
}

const productPrefixes = [
  "src/lib/tunnel/",
  "src/lib/mcp/",
  "src/lib/permissions/",
  "src/lib/approvals/",
  "src/lib/logs/",
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
  "src/lib/api/logs.ts",
];

function isProductPath(rel: string): boolean {
  const normalized = toRepoPath(rel);
  return productPrefixes.some((prefix) => normalized.startsWith(prefix));
}

function isAllowListed(rel: string) {
  return (
    toRepoPath(rel) === "src/test/shellRuntimeSurface.test.ts" ||
    toRepoPath(rel) === "src/lib/brand/templateConfig.ts" ||
    toRepoPath(rel) === "src/lib/brand/brand.ts" ||
    toRepoPath(rel) === "src/app/routes.tsx"
  );
}

function walk(dir: string, files: string[] = []): string[] {
  let entries: string[] = [];

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

describe("shell runtime surface", () => {
  it("does not contain legacy frontend markers in src", () => {
    const files = walk(join(root, "src"));
    const violations: string[] = [];

    for (const file of files) {
      const rel = toRepoPath(relative(root, file));

      if (isAllowListed(rel) || isProductPath(rel)) {
        continue;
      }

      const content = readFileSync(file, "utf8");

      for (const marker of forbiddenMarkers) {
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

        violations.push(`${rel}: ${marker}`);
      }
    }

    expect(violations).toEqual([]);
  });

  it("keeps the tauri command call site in shell or product api only", () => {
    const files = walk(join(root, "src"));
    const violations: string[] = [];
    const allowedInvokeFiles = new Set([
      "src/lib/api/shell.ts",
      "src/lib/api/tunnel.ts",
      "src/lib/api/mcp.ts",
      "src/lib/api/permissions.ts",
      "src/lib/api/approvals.ts",
      "src/lib/api/dashboard.ts",
      "src/lib/api/logs.ts",
    ]);

    for (const file of files) {
      const rel = toRepoPath(relative(root, file));
      const content = readFileSync(file, "utf8");

      if (containsTauriInvokeCall(content) && !allowedInvokeFiles.has(rel)) {
        violations.push(rel);
      }
    }

    expect(violations).toEqual([]);
  });
});
