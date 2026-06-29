import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

export const requiredDocs = [
  {
    path: "README.md",
    requiredSections: [
      "# Desktop Shell Template",
      "## What is this",
      "## Features",
      "## Quick Start",
      "## Rename Your App",
      "## Project Structure",
      "## Add a Page",
      "## Add a Shell Command",
      "## Add a Setting",
      "## Build",
      "## Release",
      "## Template Boundary",
      "## Checks",
    ],
  },
  {
    path: "docs/template-guide.md",
    requiredSections: [
      "# Template Guide",
      "## 1. Create your app",
      "## 2. Rename the app",
      "## 3. Add a page",
      "## 4. Add a setting",
      "## 5. Add a Tauri command",
      "## 6. Add UI components",
      "## 7. Add dependencies",
      "## 8. Build",
      "## 9. Update configuration",
      "## 10. Before committing",
    ],
    forbiddenSnippets: [
      "PHASE_ALLOWED_COMMANDS",
      "PHASE6_ALLOWED_COMMANDS",
      'label="Enable logs"',
      "onChange={(event)",
    ],
  },
  {
    path: "docs/architecture.md",
    requiredSections: [
      "# Architecture",
      "## Overview",
      "## Frontend structure",
      "## Frontend boundaries",
      "## Rust structure",
      "## Rust boundaries",
      "## Template identity",
      "## Command boundary",
      "## Forbidden legacy concepts",
      "## Dependency boundary",
      "## Testing strategy",
      "## Design rule",
    ],
    forbiddenSnippets: ["Select\nSwitch\nDialog\nTooltip", "Dialog\nTooltip"],
  },
  {
    path: "docs/checks.md",
    requiredSections: [
      "# Checks",
      "## App checks",
      "## Template checks",
      "## Frontend typecheck",
      "## Frontend unit tests",
      "## Template config check",
      "## Brand check",
      "## Shell boundary check",
      "## Frontend legacy file check",
      "## Template dependency check",
      "## Docs check",
      "## Rust tests",
      "## Manual smoke test",
      "## CI",
      "## When a check fails",
    ],
  },
];

export function checkDocs(root = process.cwd()) {
  const violations = [];

  for (const doc of requiredDocs) {
    const filePath = join(root, doc.path);

    if (!existsSync(filePath)) {
      violations.push(`${doc.path} is missing`);
      continue;
    }

    const content = readFileSync(filePath, "utf8");

    for (const section of doc.requiredSections) {
      if (!content.includes(section)) {
        violations.push(`${doc.path} is missing section: ${section}`);
      }
    }

    for (const snippet of doc.forbiddenSnippets ?? []) {
      if (content.includes(snippet)) {
        violations.push(
          `${doc.path} contains stale or misleading snippet: ${snippet}`,
        );
      }
    }
  }

  return violations;
}

export function runCli(root = process.cwd()) {
  const violations = checkDocs(root);

  if (violations.length > 0) {
    console.error("Docs check failed:");
    for (const violation of violations) {
      console.error(`- ${violation}`);
    }
    process.exitCode = 1;
    return;
  }

  console.log("Docs check passed.");
}

const currentFile = fileURLToPath(import.meta.url);

if (process.argv[1] === currentFile) {
  runCli();
}
