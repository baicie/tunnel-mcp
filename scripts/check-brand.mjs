import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = process.cwd();

// check-brand enforces two things:
//   1. No old brand markers leak from a prior product codebase.
//   2. The current template identity (`Desktop Shell` / `desktop-shell` /
//      `com.example.desktop-shell` / `github.com/example/desktop-shell`)
//      is not hard-coded outside of the configured allow-list.
const forbiddenOldBrandMarkers = [
  "CC Switch",
  "cc-switch",
  "cc_switch",
  "ccswitch",
];

const hardcodedTemplateMarkers = [
  "Desktop Shell",
  "desktop-shell",
  "com.example.desktop-shell",
  "github.com/example/desktop-shell",
];

// Allow-list: files that legitimately reference template identity.
const allowList = new Set([
  "template.config.ts",
  "scripts/sync-template-config.mjs",
  "scripts/sync-template-config.test.mjs",
  "scripts/check-template-deps.mjs",
  "scripts/check-template-deps.test.mjs",
  "scripts/check-brand.mjs",
  "scripts/check-docs.mjs",
  "scripts/check-docs.test.mjs",
  "scripts/check-frontend-legacy-files.mjs",
  "scripts/check-shell-boundary.mjs",
  "src/lib/brand/templateConfig.ts",
  "src/lib/brand/brand.ts",
  "src/lib/brand/brand.test.ts",
  "src/lib/settings/settings.ts",
  "src/lib/settings/settings.test.ts",
  "src-tauri/src/shell/brand.rs",
  "src-tauri/tests/brand_boundary_test.rs",
  "src-tauri/tests/template_config_sync_test.rs",
  "README.md",
  "package.json",
  "src-tauri/Cargo.toml",
  "src-tauri/tauri.conf.json",
]);

const includeRoots = ["src", "src-tauri/src", "src-tauri/tests", "scripts"];

const includeFiles = [
  "template.config.ts",
  "package.json",
  "README.md",
  "src-tauri/Cargo.toml",
  "src-tauri/tauri.conf.json",
];

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

function toRepoPath(value) {
  return value.replace(/\\/g, "/");
}

const files = [
  ...includeFiles.map((file) => join(root, file)),
  ...includeRoots.flatMap((dir) => walk(join(root, dir))),
];

const violations = [];

for (const file of files) {
  const rel = toRepoPath(relative(root, file));
  const allowListed = allowList.has(rel);

  let content = "";

  try {
    content = readFileSync(file, "utf8");
  } catch {
    continue;
  }

  if (!allowListed) {
    for (const marker of forbiddenOldBrandMarkers) {
      if (content.includes(marker)) {
        violations.push(`${rel}: contains old brand marker "${marker}"`);
      }
    }
  }

  if (allowListed) {
    continue;
  }

  for (const marker of hardcodedTemplateMarkers) {
    if (content.includes(marker)) {
      violations.push(`${rel}: contains hardcoded template marker "${marker}"`);
    }
  }
}

if (violations.length > 0) {
  console.error("Brand check failed:");
  for (const violation of violations) {
    console.error(`- ${violation}`);
  }
  process.exit(1);
}

console.log("Brand check passed.");
