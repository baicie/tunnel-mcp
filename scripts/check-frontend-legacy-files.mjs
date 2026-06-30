import { existsSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = process.cwd();

// check-frontend-legacy-files refuses to allow product-specific
// modules to linger in `src/` after the template is generated from
// this shell.
//
// The check has two layers:
//   1. exact path checks for known legacy module paths;
//   2. pattern checks for wildcard boundaries such as
//      `src/assets/legacy*` and `src/components/provider*`.
//
// Only a handful of `src/lib/api/`, `src/lib/query/`, and component
// sub-paths are part of the shell template. Everything else must not
// exist before product code starts filling the template in.
const forbiddenPaths = [
  "src/lib/api/auth.ts",
  "src/lib/api/copilot.ts",
  "src/lib/api/deeplink.ts",
  "src/lib/api/env.ts",
  "src/lib/api/failover.ts",
  "src/lib/api/globalProxy.ts",
  "src/lib/api/global-proxy.ts",
  "src/lib/api/importExport.ts",
  "src/lib/api/import-export.ts",
  "src/lib/api/model-fetch.ts",
  "src/lib/api/model-test.ts",
  "src/lib/api/models.ts",
  "src/lib/api/openclaw.ts",
  "src/lib/api/prompts.ts",
  "src/lib/api/providers.ts",
  "src/lib/api/proxy.ts",
  "src/lib/api/s3.ts",
  "src/lib/api/session.ts",
  "src/lib/api/sessions.ts",
  "src/lib/api/skills.ts",
  "src/lib/api/subscription.ts",
  "src/lib/api/sync.ts",
  "src/lib/api/usage.ts",
  "src/lib/api/balance.ts",
  "src/lib/api/vscode.ts",
  "src/lib/api/workspace.ts",
  "src/lib/api/hermes.ts",
  "src/lib/api/index.ts",
  "src/lib/authBinding.ts",
  "src/lib/clipboard.ts",
  "src/lib/requestOverrides.ts",
  "src/lib/usageRange.ts",
  "src/lib/userAgent.ts",
  "src/lib/version.ts",
  "src/lib/errors",
  "src/lib/schemas",
  "src/lib/utils",
  "src/lib/query/copilot.ts",
  "src/lib/query/failover.ts",
  "src/lib/query/proxy.ts",
  "src/lib/query/queries.ts",
  "src/lib/query/usage.ts",
  "src/lib/query/subscription.ts",
  "src/lib/query/mutations.ts",
  "src/lib/query/index.ts",
  "src/hooks",
  "src/i18n",
  "src/features",
  "src/views",
  "src/store",
  "src/types",
  "src/utils",
  "src/config",
  "src/contexts",
  "src/icons",
  "src/components/agents",
  "src/components/common",
  "src/components/deeplink",
  "src/components/env",
  "src/components/hermes",
  "src/components/icons",
  "src/components/mcp",
  "src/components/openclaw",
  "src/components/prompts",
  "src/components/providers",
  "src/components/proxy",
  "src/components/sessions",
  "src/components/skills",
  "src/components/subscription",
  "src/components/usage",
  "src/components/workspace",
  "src/components/BrandIcons.tsx",
  "src/components/ProviderIcon.tsx",
  "src/components/theme-provider.tsx",
  "src/components/UpdateBadge.tsx",
  "src/components/UsageFooter.tsx",
  "src/components/UsageScriptModal.tsx",
  "src/assets/icons",
  "src/types.ts",
];

const allowedExactPaths = new Set([
  "src/lib/api/shell.ts",
  "src/lib/api/shell.test.ts",
  "src/lib/api/tunnel.ts",
  "src/lib/api/mcp.ts",
  "src/lib/api/permissions.ts",
  "src/lib/query/queryClient.ts",
  "src/lib/query/queryKeys.ts",
]);

const forbiddenPatterns = [
  {
    name: "legacy asset wildcard",
    pattern: /^src\/assets\/legacy/i,
  },
  {
    name: "legacy api module",
    pattern: /^src\/lib\/api\/.+/i,
  },
  {
    name: "legacy query module",
    pattern: /^src\/lib\/query\/.+/i,
  },
  {
    name: "legacy top-level frontend directory",
    pattern:
      /^src\/(hooks|i18n|features|views|store|types|utils|config|contexts|icons)(\/|$)/i,
  },
  {
    name: "legacy business component wildcard",
    pattern:
      /^src\/components\/(agents|common|deeplink|env|hermes|mcp|openclaw|prompt|prompts|provider|providers|proxy|session|sessions|skills|subscription|usage|workspace)(\/|[-_.A-Z]|$)/i,
  },
];

function toRepoPath(value) {
  return value.replace(/\\/g, "/");
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

const violations = [];

for (const forbiddenPath of forbiddenPaths) {
  if (existsSync(join(root, forbiddenPath))) {
    violations.push(`${forbiddenPath}: exact legacy path still exists`);
  }
}

for (const file of walk(join(root, "src"))) {
  const rel = toRepoPath(relative(root, file));
  if (allowedExactPaths.has(rel)) {
    continue;
  }

  for (const rule of forbiddenPatterns) {
    if (rule.pattern.test(rel)) {
      violations.push(`${rel}: matches ${rule.name}`);
    }
  }
}

if (violations.length > 0) {
  console.error("Legacy frontend files still exist:");
  for (const violation of violations) {
    console.error(`- ${violation}`);
  }
  process.exit(1);
}

console.log("Legacy frontend file check passed.");
