import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { mkdtempSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { checkDocs, requiredDocs } from "./check-docs.mjs";

function createTempProject() {
  return mkdtempSync(join(tmpdir(), "desktop-shell-docs-"));
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
  });

  it("reports missing sections", () => {
    const root = createTempProject();

    mkdirSync(join(root, "docs"), { recursive: true });
    writeFileSync(
      join(root, "README.md"),
      "# Desktop Shell Template\n",
      "utf8",
    );
    writeFileSync(
      join(root, "docs/template-guide.md"),
      "# Template Guide\n",
      "utf8",
    );
    writeFileSync(
      join(root, "docs/architecture.md"),
      "# Architecture\n",
      "utf8",
    );
    writeFileSync(join(root, "docs/checks.md"), "# Checks\n", "utf8");

    const violations = checkDocs(root);

    expect(violations).toContain(
      "README.md is missing section: ## What is this",
    );
    expect(violations).toContain(
      "docs/template-guide.md is missing section: ## 1. Create your app",
    );
    expect(violations).toContain(
      "docs/architecture.md is missing section: ## Overview",
    );
    expect(violations).toContain(
      "docs/checks.md is missing section: ## App checks",
    );
  });

  it("reports stale or misleading snippets", () => {
    const root = createTempProject();

    writeRequiredDocs(root);

    writeFileSync(
      join(root, "docs/template-guide.md"),
      [
        "# Template Guide",
        ...requiredDocs.find((doc) => doc.path === "docs/template-guide.md")
          .requiredSections,
        "PHASE_ALLOWED_COMMANDS",
        "PHASE6_ALLOWED_COMMANDS",
      ].join("\n\n"),
      "utf8",
    );

    const violations = checkDocs(root);

    expect(violations).toContain(
      "docs/template-guide.md contains stale or misleading snippet: PHASE_ALLOWED_COMMANDS",
    );
    expect(violations).toContain(
      "docs/template-guide.md contains stale or misleading snippet: PHASE6_ALLOWED_COMMANDS",
    );
  });
});
