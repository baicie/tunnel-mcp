import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { mkdtempSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  syncTemplateConfig,
  updatePackageJson,
  updateCargoToml,
  updateTauriConfig,
  validateTemplateConfig,
} from "./sync-template-config.mjs";

function createTempProject() {
  const root = mkdtempSync(join(tmpdir(), "desktop-shell-template-config-"));

  mkdirSync(join(root, "src/lib/brand"), { recursive: true });
  mkdirSync(join(root, "src/lib/settings"), { recursive: true });
  mkdirSync(join(root, "src-tauri/src/shell"), { recursive: true });

  writeFileSync(
    join(root, "package.json"),
    JSON.stringify(
      {
        name: "old-name",
        version: "0.1.0",
        scripts: {},
        dependencies: {},
      },
      null,
      2,
    ),
    "utf8",
  );

  writeFileSync(join(root, "README.md"), "# Old README\n", "utf8");

  writeFileSync(
    join(root, "src-tauri/Cargo.toml"),
    `[package]
name = "old-name"
version = "0.1.0"
description = "Old description"
edition = "2021"
`,
    "utf8",
  );

  writeFileSync(
    join(root, "src-tauri/tauri.conf.json"),
    JSON.stringify(
      {
        productName: "Old Product",
        identifier: "com.example.old",
        app: {
          windows: [
            {
              title: "Old Product",
              width: 1000,
              height: 700,
            },
          ],
        },
      },
      null,
      2,
    ),
    "utf8",
  );

  return root;
}

const config = {
  appName: "Acme Desk",
  packageName: "acme-desk",
  productName: "Acme Desk",
  identifier: "com.acme.desk",
  description: "Acme desktop application.",
  repositoryUrl: "https://github.com/acme/acme-desk",
  deepLinkScheme: "acme-desk",
  updaterEndpoint:
    "https://github.com/acme/acme-desk/releases/latest/download/latest.json",
};

describe("template config validation", () => {
  it("accepts valid config", () => {
    expect(validateTemplateConfig(config)).toEqual([]);
  });

  it("rejects invalid package name", () => {
    expect(
      validateTemplateConfig({
        ...config,
        packageName: "Acme Desk",
      }),
    ).toContain("packageName must be kebab-case, for example desktop-shell");
  });

  it("rejects invalid identifier", () => {
    expect(
      validateTemplateConfig({
        ...config,
        identifier: "invalid",
      }),
    ).toContain("identifier must look like com.example.desktop-shell");
  });
});

describe("template config sync helpers", () => {
  it("updates package.json", () => {
    const next = updatePackageJson(
      {
        name: "old-name",
        description: "old",
        scripts: {},
      },
      config,
    );

    expect(next.name).toBe("acme-desk");
    expect(next.description).toBe("Acme desktop application.");
    expect(next.repository).toEqual({
      type: "git",
      url: "https://github.com/acme/acme-desk",
    });
    expect(next.scripts["sync:template"]).toBe(
      "node scripts/sync-template-config.mjs",
    );
  });

  it("updates Cargo.toml package identity", () => {
    const next = updateCargoToml(
      `[package]
name = "old-name"
description = "old"
edition = "2021"
`,
      config,
    );

    expect(next).toContain('name = "acme-desk"');
    expect(next).toContain('description = "Acme desktop application."');
    expect(next).toContain("[lib]");
    expect(next).toContain('name = "acme_desk"');
  });

  it("updates tauri config", () => {
    const next = updateTauriConfig(
      {
        productName: "Old",
        identifier: "com.old.app",
        app: {
          windows: [{ title: "Old" }],
        },
      },
      config,
    );

    expect(next.productName).toBe("Acme Desk");
    expect(next.identifier).toBe("com.acme.desk");
    expect(next.app.windows[0].title).toBe("Acme Desk");
    expect(next.plugins.updater.endpoints).toEqual([
      "https://github.com/acme/acme-desk/releases/latest/download/latest.json",
    ]);
  });
});

describe("syncTemplateConfig", () => {
  it("writes generated files and updates project identity", async () => {
    const root = createTempProject();

    await expect(syncTemplateConfig(root, config)).resolves.toEqual([]);

    const packageJson = JSON.parse(
      readFileSync(join(root, "package.json"), "utf8"),
    );
    const cargoToml = readFileSync(join(root, "src-tauri/Cargo.toml"), "utf8");
    const tauriConfig = JSON.parse(
      readFileSync(join(root, "src-tauri/tauri.conf.json"), "utf8"),
    );
    const frontendBrand = readFileSync(
      join(root, "src/lib/brand/brand.ts"),
      "utf8",
    );
    const frontendTemplateConfig = readFileSync(
      join(root, "src/lib/brand/templateConfig.ts"),
      "utf8",
    );
    const frontendSettings = readFileSync(
      join(root, "src/lib/settings/settings.ts"),
      "utf8",
    );
    const rustBrand = readFileSync(
      join(root, "src-tauri/src/shell/brand.rs"),
      "utf8",
    );
    const readme = readFileSync(join(root, "README.md"), "utf8");

    expect(packageJson.name).toBe("acme-desk");
    expect(cargoToml).toContain('name = "acme-desk"');
    expect(tauriConfig.identifier).toBe("com.acme.desk");
    expect(frontendBrand).toContain("APP_BRAND");
    expect(frontendTemplateConfig).toContain("Acme Desk");
    expect(frontendSettings).toContain("SETTINGS_STORAGE_KEY");
    expect(rustBrand).toContain("com.acme.desk");
    expect(readme).toContain("<!-- TEMPLATE_IDENTITY_START -->");
  });

  it("detects drift in check mode", async () => {
    const root = createTempProject();

    const violations = await syncTemplateConfig(root, config, { check: true });

    expect(violations.length).toBeGreaterThan(0);
  });

  it("passes check mode after sync", async () => {
    const root = createTempProject();

    await syncTemplateConfig(root, config);

    await expect(
      syncTemplateConfig(root, config, { check: true }),
    ).resolves.toEqual([]);
  });
});
