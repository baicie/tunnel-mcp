import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { mkdtempSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { checkDocs, requiredDocs } from "./check-docs.mjs";

function createTempProject() {
  return mkdtempSync(join(tmpdir(), "tunnel-mcp-docs-"));
}

function writeRequiredDocs(root) {
  for (const doc of requiredDocs) {
    const filePath = join(root, doc.path);
    mkdirSync(join(filePath, ".."), { recursive: true });

    writeFileSync(filePath, doc.requiredSections.join("\n\n"), "utf8");
  }
}

describe("check-docs", () => {
  it("passes when all required docs and sections exist", () => {
    const root = createTempProject();

    writeRequiredDocs(root);

    expect(checkDocs(root)).toEqual([]);
  });

  it("reports missing docs", () => {
    const root = createTempProject();

    expect(checkDocs(root)).toContain("README.md is missing");
    expect(checkDocs(root)).toContain(
      "docs/agents/issue-tracker.md is missing",
    );
  });

  it("reports missing sections", () => {
    const root = createTempProject();

    mkdirSync(join(root, "docs", "agents"), { recursive: true });
    writeFileSync(join(root, "README.md"), "# Tunnel MCP\n", "utf8");
    writeFileSync(
      join(root, "docs/agents/issue-tracker.md"),
      "# Issue Tracker: GitHub\n",
      "utf8",
    );

    const violations = checkDocs(root);

    expect(violations).toContain(
      "README.md is missing section: ## What is this",
    );
    expect(violations).toContain(
      "docs/agents/issue-tracker.md is missing section: ## Conventions",
    );
  });

  it("reports stale or misleading snippets", () => {
    const root = createTempProject();

    writeRequiredDocs(root);

    writeFileSync(
      join(root, "README.md"),
      [
        "# Tunnel MCP",
        ...requiredDocs.find((doc) => doc.path === "README.md")
          .requiredSections,
        "# Desktop Shell Template",
      ].join("\n\n"),
      "utf8",
    );

    const violations = checkDocs(root);

    expect(violations).toContain(
      "README.md contains stale or misleading snippet: # Desktop Shell Template",
    );
  });
});
