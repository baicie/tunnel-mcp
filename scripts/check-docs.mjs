import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

export const requiredDocs = [
  {
    path: "README.md",
    requiredSections: [
      "# Tunnel MCP",
      "## What is this",
      "## Current phase",
      "## Features",
      "## Quick Start",
      "## Identity",
      "## Project Structure",
      "## Product layer",
      "## Shell Boundary",
      "## Checks",
      "## Documentation",
    ],
    forbiddenSnippets: [
      "# Desktop Shell Template",
      "Phase 0 ships a clean shell",
      "Product modules will live under product-layer paths and be added in later phases.",
    ],
  },
  {
    path: "docs/agents/issue-tracker.md",
    requiredSections: [
      "# Issue Tracker: GitHub",
      "## Conventions",
      '## When A Skill Says "Publish To The Issue Tracker"',
      '## When A Skill Says "Fetch The Relevant Ticket"',
    ],
    forbiddenSnippets: ["baicie/tauri-template"],
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
