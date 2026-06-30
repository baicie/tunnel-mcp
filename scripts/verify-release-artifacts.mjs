import { existsSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = process.cwd();

function argValue(name) {
  const index = process.argv.indexOf(`--${name}`);
  if (index === -1) return undefined;
  return process.argv[index + 1];
}

function hasFlag(name) {
  return process.argv.includes(`--${name}`);
}

const platform = argValue("platform") ?? process.platform;
const target = argValue("target");

const bundleDirs = [
  target
    ? join(root, "src-tauri", "target", target, "release", "bundle")
    : undefined,
  join(root, "src-tauri", "target", "release", "bundle"),
].filter(Boolean);

function walk(dir) {
  if (!existsSync(dir)) return [];

  return readdirSync(dir).flatMap((entry) => {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) return walk(full);
    return full;
  });
}

function assert(condition, message) {
  if (!condition) {
    console.error(message);
    process.exit(1);
  }
}

const files = bundleDirs.flatMap(walk);
const printableFiles = files.map((file) => relative(root, file));

assert(
  files.length > 0,
  `No release bundle files found. Checked: ${bundleDirs
    .map((dir) => relative(root, dir))
    .join(", ")}`,
);

const hasMacInstaller = files.some((file) =>
  /\.(dmg|app\.tar\.gz)$/.test(file),
);

const hasWindowsInstaller = files.some(
  (file) => /\.(msi)$/.test(file) || /setup.*\.exe$/i.test(file),
);

const hasLatestJson = files.some((file) => file.endsWith("latest.json"));

if (platform === "macos" || platform === "darwin") {
  assert(
    hasMacInstaller,
    `No macOS installer artifact found. Files:\n${printableFiles.join("\n")}`,
  );
} else if (platform === "windows" || platform === "win32") {
  assert(
    hasWindowsInstaller,
    `No Windows installer artifact found. Files:\n${printableFiles.join("\n")}`,
  );
} else {
  assert(
    hasMacInstaller || hasWindowsInstaller,
    `No installer artifact found. Files:\n${printableFiles.join("\n")}`,
  );
}

if (!hasLatestJson) {
  const message =
    "latest.json not found. This is acceptable before updater signing is enabled, but must be fixed before public auto-update.";

  if (hasFlag("require-updater-artifacts")) {
    console.error(message);
    process.exit(1);
  }

  console.warn(message);
}

console.log("release artifacts verified");
