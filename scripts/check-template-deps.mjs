import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

// Dependencies that suggest product-specific feature creep (rich
// editors, icon packs, proxy stacks, …). Keep this list short and
// intentional; the shell template should only require the bare
// minimum to render and exercise the runtime boundary.
export const forbiddenFrontendDeps = [
  "@lobehub/icons-static-svg",
  "@tauri-apps/plugin-dialog",
  "@tauri-apps/plugin-process",
  "@tauri-apps/plugin-store",
  "@tauri-apps/plugin-updater",
  "framer-motion",
  "i18next",
  "react-i18next",
  "recharts",
];

export const forbiddenCargoDeps = [
  "anyhow",
  "chrono",
  "log",
  "once_cell",
  "serde_yaml",
  "tauri-plugin-deep-link",
  "tauri-plugin-single-instance",
  "thiserror",
  "tokio",
  "toml",
  "toml_edit",
  "webkit2gtk",
  "winreg",
  "windows-sys",
  "objc2",
  "objc2-app-kit",
  "reqwest",
  "axum",
  "tower",
  "tower-http",
  "hyper",
  "hyper-util",
  "hyper-rustls",
  "http",
  "http-body",
  "http-body-util",
  "httparse",
  "tokio-rustls",
  "rustls",
  "webpki-roots",
  "rustls-native-certs",
  "regex",
  "rquickjs",
  "zip",
  "flate2",
  "brotli",
  "zstd",
  "rusqlite",
  "indexmap",
  "rust_decimal",
  "uuid",
  "sha2",
  "hmac",
  "bytes",
  "async-stream",
  "futures",
];

export function readJsonFile(filePath) {
  return JSON.parse(readFileSync(filePath, "utf8"));
}

export function collectPackageDependencyNames(packageJson) {
  return new Set([
    ...Object.keys(packageJson.dependencies ?? {}),
    ...Object.keys(packageJson.devDependencies ?? {}),
    ...Object.keys(packageJson.optionalDependencies ?? {}),
    ...Object.keys(packageJson.peerDependencies ?? {}),
  ]);
}

export function isForbiddenFrontendDependency(name) {
  return forbiddenFrontendDeps.some((forbidden) => {
    if (forbidden.endsWith("/")) {
      return name.startsWith(forbidden);
    }

    return name === forbidden;
  });
}

export function findForbiddenFrontendDependencies(packageJson) {
  const names = collectPackageDependencyNames(packageJson);

  return [...names].filter(isForbiddenFrontendDependency).sort();
}

function stripInlineComment(line) {
  let inSingle = false;
  let inDouble = false;

  for (let index = 0; index < line.length; index += 1) {
    const char = line[index];
    const prev = line[index - 1];

    if (char === "'" && !inDouble) {
      inSingle = !inSingle;
    }

    if (char === '"' && !inSingle && prev !== "\\") {
      inDouble = !inDouble;
    }

    if (char === "#" && !inSingle && !inDouble) {
      return line.slice(0, index).trim();
    }
  }

  return line.trim();
}

function normalizeCargoDependencyName(name) {
  return name.trim().replace(/^"|"$/g, "").replace(/^'|'$/g, "");
}

function parseCargoPackageValue(line) {
  const match = line.match(/\bpackage\s*=\s*["']([^"']+)["']/);
  return match?.[1];
}

function parseDependencyTableName(section) {
  const normalized = section.trim();

  for (const key of [
    "dependencies",
    "dev-dependencies",
    "build-dependencies",
  ]) {
    if (normalized.startsWith(`${key}.`)) {
      return normalizeCargoDependencyName(normalized.slice(key.length + 1));
    }

    const targetMarker = `.${key}.`;
    const targetIndex = normalized.indexOf(targetMarker);
    if (targetIndex >= 0) {
      return normalizeCargoDependencyName(
        normalized.slice(targetIndex + targetMarker.length),
      );
    }
  }

  return undefined;
}

function isDependencyListSection(section) {
  const normalized = section.trim();

  if (
    normalized === "dependencies" ||
    normalized === "dev-dependencies" ||
    normalized === "build-dependencies"
  ) {
    return true;
  }

  return (
    normalized.endsWith(".dependencies") ||
    normalized.endsWith(".dev-dependencies") ||
    normalized.endsWith(".build-dependencies")
  );
}

export function parseCargoDependencyNames(cargoTomlContent) {
  const names = new Set();
  const lines = cargoTomlContent.split(/\r?\n/);
  let currentSection = "";
  let currentDependencyTableName;

  for (const rawLine of lines) {
    const line = stripInlineComment(rawLine);

    if (!line || line.startsWith("#")) {
      continue;
    }

    if (line.startsWith("[") && line.endsWith("]")) {
      currentSection = line.slice(1, -1).trim();
      currentDependencyTableName = parseDependencyTableName(currentSection);

      if (currentDependencyTableName) {
        names.add(currentDependencyTableName);
      }

      continue;
    }

    const packageName = parseCargoPackageValue(line);
    if (packageName) {
      names.add(packageName);
    }

    if (!isDependencyListSection(currentSection)) {
      continue;
    }

    const equalIndex = line.indexOf("=");

    if (equalIndex <= 0) {
      continue;
    }

    const name = normalizeCargoDependencyName(line.slice(0, equalIndex));

    if (name && name !== "package") {
      names.add(name);
    }
  }

  return names;
}

export function findForbiddenCargoDependencies(cargoTomlContent) {
  const names = parseCargoDependencyNames(cargoTomlContent);

  return [...names].filter((name) => forbiddenCargoDeps.includes(name)).sort();
}

export function checkTemplateDeps(root = process.cwd()) {
  const violations = [];

  const packageJsonPath = join(root, "package.json");
  const cargoTomlPath = join(root, "src-tauri", "Cargo.toml");

  if (!existsSync(packageJsonPath)) {
    violations.push("package.json is missing");
  } else {
    const packageJson = readJsonFile(packageJsonPath);
    const forbiddenPackageDeps = findForbiddenFrontendDependencies(packageJson);

    for (const dep of forbiddenPackageDeps) {
      violations.push(
        `package.json contains forbidden frontend dependency: ${dep}`,
      );
    }
  }

  if (!existsSync(cargoTomlPath)) {
    violations.push("src-tauri/Cargo.toml is missing");
  } else {
    const cargoToml = readFileSync(cargoTomlPath, "utf8");
    const forbiddenCargo = findForbiddenCargoDependencies(cargoToml);

    for (const dep of forbiddenCargo) {
      violations.push(
        `src-tauri/Cargo.toml contains forbidden cargo dependency: ${dep}`,
      );
    }
  }

  return violations;
}

export function runCli(root = process.cwd()) {
  const violations = checkTemplateDeps(root);

  if (violations.length > 0) {
    console.error("Template dependency check failed:");
    for (const violation of violations) {
      console.error(`- ${violation}`);
    }
    process.exitCode = 1;
    return;
  }

  console.log("Template dependency check passed.");
}

const currentFile = fileURLToPath(import.meta.url);

if (process.argv[1] === currentFile) {
  runCli();
}
