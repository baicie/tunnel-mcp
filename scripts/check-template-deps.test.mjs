import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { mkdtempSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  checkTemplateDeps,
  findForbiddenCargoDependencies,
  findForbiddenFrontendDependencies,
  parseCargoDependencyNames,
} from "./check-template-deps.mjs";

function createTempProject() {
  return mkdtempSync(join(tmpdir(), "desktop-shell-template-deps-"));
}

function writeProjectFiles(root, packageJson, cargoToml) {
  mkdirSync(join(root, "src-tauri"), { recursive: true });

  writeFileSync(
    join(root, "package.json"),
    JSON.stringify(packageJson, null, 2),
    "utf8",
  );

  writeFileSync(join(root, "src-tauri", "Cargo.toml"), cargoToml, "utf8");
}

describe("check-template-deps", () => {
  it("accepts minimal template dependencies", () => {
    const root = createTempProject();

    writeProjectFiles(
      root,
      {
        dependencies: {
          "@tanstack/react-query": "^5.0.0",
          "@tauri-apps/api": "^2.0.0",
          react: "^18.3.1",
          "react-dom": "^18.3.1",
          clsx: "^2.0.0",
          zod: "^4.0.0",
        },
        devDependencies: {
          vite: "^6.0.0",
          vitest: "^2.1.0",
          typescript: "^5.6.0",
        },
      },
      `
[package]
name = "desktop-shell"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tauri = { version = "2.8.2", features = ["tray-icon"] }
tauri-plugin-opener = "2"
url = "2"
`,
    );

    expect(checkTemplateDeps(root)).toEqual([]);
  });

  it("detects forbidden frontend dependencies", () => {
    const packageJson = {
      dependencies: {
        "@lobehub/icons-static-svg": "^1.0.0",
        "@tauri-apps/plugin-updater": "^2.0.0",
        "framer-motion": "^11.0.0",
        i18next: "^23.0.0",
        "react-i18next": "^14.0.0",
        recharts: "^2.0.0",
        react: "^18.3.1",
      },
    };

    expect(findForbiddenFrontendDependencies(packageJson)).toEqual([
      "@lobehub/icons-static-svg",
      "@tauri-apps/plugin-updater",
      "framer-motion",
      "i18next",
      "react-i18next",
      "recharts",
    ]);
  });

  it("parses cargo dependency names, renamed packages, and dependency tables", () => {
    const names = parseCargoDependencyNames(`
[dependencies]
serde = { version = "1.0", features = ["derive"] }
tauri = "2"
reqwest = { version = "0.12", features = ["json"] }

[dev-dependencies]
tempfile = "3"

[dependencies.http-client]
package = "reqwest"
version = "0.12"

[target.'cfg(target_os = "linux")'.dependencies.legacy-db]
package = "rusqlite"
version = "0.31"
`);

    expect([...names].sort()).toEqual([
      "http-client",
      "legacy-db",
      "reqwest",
      "rusqlite",
      "serde",
      "tauri",
      "tempfile",
    ]);
  });

  it("detects forbidden cargo dependencies", () => {
    const cargoToml = `
[dependencies]
serde = "1"
http-client = { package = "reqwest", version = "0.12" }
axum = "0.7"
tauri = "2"

[dependencies.legacy-db]
package = "rusqlite"
version = "0.31"
`;

    expect(findForbiddenCargoDependencies(cargoToml)).toEqual([
      "axum",
      "reqwest",
      "rusqlite",
    ]);
  });

  it("returns violations for forbidden frontend and cargo deps", () => {
    const root = createTempProject();

    writeProjectFiles(
      root,
      {
        dependencies: {
          react: "^18.3.1",
          i18next: "^23.0.0",
          recharts: "^2.0.0",
        },
      },
      `
[dependencies]
serde = "1"
hyper = "1"
zip = "2"
`,
    );

    expect(checkTemplateDeps(root)).toEqual([
      "package.json contains forbidden frontend dependency: i18next",
      "package.json contains forbidden frontend dependency: recharts",
      "src-tauri/Cargo.toml contains forbidden cargo dependency: hyper",
      "src-tauri/Cargo.toml contains forbidden cargo dependency: zip",
    ]);
  });
});
