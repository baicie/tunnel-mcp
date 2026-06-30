import { existsSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";

const root = process.cwd();
const bundleDir = join(root, "src-tauri", "target", "release", "bundle");

function walk(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir).flatMap((entry) => {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) return walk(full);
    return full;
  });
}

const files = walk(bundleDir);
const hasMacArtifact = files.some((file) =>
  /\.dmg$|\.app\.tar\.gz$/.test(file),
);
const hasWinArtifact = files.some((file) => /\.msi$|\.exe$/.test(file));
const hasLatestJson = files.some((file) => file.endsWith("latest.json"));

if (!hasMacArtifact && !hasWinArtifact) {
  console.error(
    "No installer artifact found under src-tauri/target/release/bundle",
  );
  process.exit(1);
}

if (!hasLatestJson) {
  console.warn(
    "latest.json not found. This is acceptable before updater signing is enabled, but must be fixed before public auto-update.",
  );
}

console.log("release artifacts verified");
