// Self-contained icon generator for the desktop shell template.
//
// Renders the brand mark SVG to a set of PNGs and wraps them into the
// Apple `.icns` and Windows `.ico` containers that `tauri.conf.json`
// references. Uses `sharp` to rasterise the SVG at each required size.
//
// Run:
//   node scripts/generate-icons.mjs
//   node scripts/generate-icons.mjs ./path/to/icon.svg
//   node scripts/generate-icons.mjs --check   # verify outputs match source
//
// Outputs (in src-tauri/icons/):
//   Desktop icons: 32x32.png, 64x64.png, 128x128.png, 128x128@2x.png,
//                 icon.png (1024x1024 master), icon.icns, icon.ico
//   Windows Store: Square30x30Logo.png, Square44x44Logo.png, ...,
//                 StoreLogo.png
//   macOS App:    AppIcon-20x20@1x/2x/3x.png, AppIcon-29x29@1x/2x/3x.png,
//                 AppIcon-40x40@1x/2x/3x.png, AppIcon-60x60@2x/3x.png,
//                 AppIcon-76x76@1x/2x.png, AppIcon-83.5x83.5@2x.png,
//                 AppIcon-512@2x.png
//   Android:      ic_launcher.png/ic_launcher_round.png/ic_launcher_foreground.png
//                 at mdpi/hdpi/xhdpi/xxhdpi/xxxhdpi densities
//   macOS Tray:    statusTemplate.png (18x18), statusTemplate@2x.png (36x36),
//                 statusbar_template_3x.png (54x54)
//   DMG BG:        dmg-background.png

import { writeFileSync, mkdirSync, readFileSync, existsSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import sharp from "sharp";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = join(__dirname, "..");
const DEFAULT_SVG_PATH = join(__dirname, "assets", "source-icon.svg");
const OUT_DIR = join(ROOT, "src-tauri", "icons");

const args = new Set(process.argv.slice(2));
const sourceArg = process.argv.slice(2).find((arg) => !arg.startsWith("--"));
const SVG_PATH = sourceArg
  ? resolve(process.cwd(), sourceArg)
  : DEFAULT_SVG_PATH;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async function renderSvg(size) {
  return sharp(readFileSync(SVG_PATH)).resize(size, size).png().toBuffer();
}

// ---------------------------------------------------------------------------
// .icns encoder
// ---------------------------------------------------------------------------

const ICNS_SIZES = [16, 32, 64, 128, 256, 512];

function icnsType(size) {
  return {
    16: "icp4",
    32: "icp5",
    64: "icp6",
    128: "ic07",
    256: "ic08",
    512: "ic09",
  }[size];
}

async function encodeIcns(svgBuffer, sizes) {
  const chunks = [];
  let totalPayload = 0;
  for (const size of sizes) {
    const type = icnsType(size);
    if (!type) continue;
    const png = await sharp(svgBuffer).resize(size, size).png().toBuffer();
    const length = Buffer.alloc(4);
    length.writeUInt32BE(png.length + 8, 0);
    chunks.push({ type, data: png, length });
    totalPayload += png.length + 8;
  }
  const header = Buffer.alloc(8);
  header.write("icns", 0, "ascii");
  header.writeUInt32BE(totalPayload + 8, 4);
  return Buffer.concat([
    header,
    ...chunks.flatMap(({ type, data, length }) => [
      Buffer.from(type, "ascii"),
      length,
      data,
    ]),
  ]);
}

// ---------------------------------------------------------------------------
// .ico encoder
// ---------------------------------------------------------------------------

async function encodeIco(svgBuffer, sizes) {
  const pngBuffers = await Promise.all(
    sizes.map((size) => sharp(svgBuffer).resize(size, size).png().toBuffer()),
  );

  const reserved = Buffer.from([0, 0]);
  const type = Buffer.from([1, 0]);
  const count = Buffer.alloc(2);
  count.writeUInt16LE(sizes.length, 0);

  const directoryEntries = [];
  for (let i = 0; i < sizes.length; i++) {
    const size = sizes[i];
    const png = pngBuffers[i];
    const width = size >= 256 ? 0 : size;
    const height = size >= 256 ? 0 : size;
    const entry = Buffer.alloc(16);
    entry[0] = width;
    entry[1] = height;
    entry[2] = 0;
    entry[3] = 0;
    entry.writeUInt16LE(1, 4);
    entry.writeUInt16LE(32, 6);
    entry.writeUInt32LE(png.length, 8);
    entry.writeUInt32LE(0, 12);
    directoryEntries.push(entry);
  }

  const headerSize = 6 + directoryEntries.length * 16;
  let cursor = headerSize;
  for (const entry of directoryEntries) {
    entry.writeUInt32LE(cursor, 12);
    cursor += entry.readUInt32LE(8);
  }

  return Buffer.concat([
    reserved,
    type,
    count,
    ...directoryEntries,
    ...pngBuffers,
  ]);
}

// ---------------------------------------------------------------------------
// Icon set definitions
// ---------------------------------------------------------------------------

const DESKTOP_OUTPUTS = [
  { name: "32x32.png", size: 32 },
  { name: "64x64.png", size: 64 },
  { name: "128x128.png", size: 128 },
  { name: "128x128@2x.png", size: 256 },
  { name: "icon.png", size: 1024 },
];

const WINDOWS_STORE_OUTPUTS = [
  { name: "Square30x30Logo.png", size: 30 },
  { name: "Square44x44Logo.png", size: 44 },
  { name: "Square71x71Logo.png", size: 71 },
  { name: "Square89x89Logo.png", size: 89 },
  { name: "Square107x107Logo.png", size: 107 },
  { name: "Square142x142Logo.png", size: 142 },
  { name: "Square150x150Logo.png", size: 150 },
  { name: "Square284x284Logo.png", size: 284 },
  { name: "Square310x310Logo.png", size: 310 },
  { name: "StoreLogo.png", size: 50 },
];

const IOS_OUTPUTS = [
  { name: "AppIcon-20x20@1x.png", size: 20 },
  { name: "AppIcon-20x20@2x.png", size: 40 },
  { name: "AppIcon-20x20@3x.png", size: 60 },
  { name: "AppIcon-29x29@1x.png", size: 29 },
  { name: "AppIcon-29x29@2x.png", size: 58 },
  { name: "AppIcon-29x29@3x.png", size: 87 },
  { name: "AppIcon-40x40@1x.png", size: 40 },
  { name: "AppIcon-40x40@2x.png", size: 80 },
  { name: "AppIcon-40x40@3x.png", size: 120 },
  { name: "AppIcon-60x60@2x.png", size: 120 },
  { name: "AppIcon-60x60@3x.png", size: 180 },
  { name: "AppIcon-76x76@1x.png", size: 76 },
  { name: "AppIcon-76x76@2x.png", size: 152 },
  { name: "AppIcon-83.5x83.5@2x.png", size: 167 },
  { name: "AppIcon-512@2x.png", size: 1024 },
];

const ANDROID_DENSITIES = {
  "mipmap-mdpi": 48,
  "mipmap-hdpi": 72,
  "mipmap-xhdpi": 96,
  "mipmap-xxhdpi": 144,
  "mipmap-xxxhdpi": 192,
};

const MACOS_TRAY_OUTPUTS = [
  { name: "statusTemplate.png", size: 18 },
  { name: "statusTemplate@2x.png", size: 36 },
  { name: "statusbar_template_3x.png", size: 54 },
];

const DMG_BACKGROUND = { name: "dmg-background.png", size: 840 };

// ---------------------------------------------------------------------------
// Generator
// ---------------------------------------------------------------------------

async function ensureOutputs() {
  mkdirSync(OUT_DIR, { recursive: true });

  const svgBuffer = readFileSync(SVG_PATH);
  const generated = [];

  const log = (path, bytes, extra = "") =>
    console.log(
      `  ${relative(process.cwd(), path)}${extra ? ` (${extra})` : ""} ${bytes} bytes`,
    );

  // Desktop icons
  for (const { name, size } of DESKTOP_OUTPUTS) {
    const path = join(OUT_DIR, name);
    const n = await renderSvg(size);
    writeFileSync(path, n);
    generated.push(path);
    log(path, n.length, `${size}x${size}`);
  }

  // Windows Store icons
  for (const { name, size } of WINDOWS_STORE_OUTPUTS) {
    const path = join(OUT_DIR, name);
    const n = await renderSvg(size);
    writeFileSync(path, n);
    generated.push(path);
    log(path, n.length, `${size}x${size}`);
  }

  // iOS icons
  mkdirSync(join(OUT_DIR, "ios"), { recursive: true });
  for (const { name, size } of IOS_OUTPUTS) {
    const path = join(OUT_DIR, "ios", name);
    const n = await renderSvg(size);
    writeFileSync(path, n);
    generated.push(path);
    log(path, n.length, `${size}x${size}`);
  }

  // Android icons
  for (const [folder, size] of Object.entries(ANDROID_DENSITIES)) {
    const dir = join(OUT_DIR, "android", folder);
    mkdirSync(dir, { recursive: true });
    for (const variant of ["", "_round", "_foreground"]) {
      const name = `ic_launcher${variant}.png`;
      const path = join(dir, name);
      const n = await renderSvg(size);
      writeFileSync(path, n);
      generated.push(path);
      log(path, n.length, `${size}x${size}`);
    }
  }

  // macOS tray icons
  mkdirSync(join(OUT_DIR, "tray", "macos"), { recursive: true });
  for (const { name, size } of MACOS_TRAY_OUTPUTS) {
    const path = join(OUT_DIR, "tray", "macos", name);
    const n = await renderSvg(size);
    writeFileSync(path, n);
    generated.push(path);
    log(path, n.length, `${size}x${size}`);
  }

  // DMG background
  const dmgPath = join(OUT_DIR, "dmg-background.png");
  const dmgBuf = await renderSvg(DMG_BACKGROUND.size);
  writeFileSync(dmgPath, dmgBuf);
  generated.push(dmgPath);
  log(dmgPath, dmgBuf.length, `${DMG_BACKGROUND.size}x${DMG_BACKGROUND.size}`);

  // icon.icns
  const icns = await encodeIcns(svgBuffer, ICNS_SIZES);
  const icnsPath = join(OUT_DIR, "icon.icns");
  writeFileSync(icnsPath, icns);
  generated.push(icnsPath);
  log(icnsPath, icns.length);

  // icon.ico
  const ico = await encodeIco(svgBuffer, [16, 32, 48, 64, 128, 256]);
  const icoPath = join(OUT_DIR, "icon.ico");
  writeFileSync(icoPath, ico);
  generated.push(icoPath);
  log(icoPath, ico.length);

  return generated;
}

async function checkOutputs() {
  const violations = [];
  const svgBuffer = readFileSync(SVG_PATH);

  const checkPng = async ({ output, size }) => {
    const path = join(OUT_DIR, output);
    if (!existsSync(path)) return `${output}: missing`;
    const actual = readFileSync(path);
    const expected = await sharp(svgBuffer).resize(size, size).png().toBuffer();
    if (!actual.equals(expected)) return `${output}: out of date`;
    return null;
  };

  for (const entry of [...DESKTOP_OUTPUTS, ...WINDOWS_STORE_OUTPUTS]) {
    const v = await checkPng({ output: entry.name, size: entry.size });
    if (v) violations.push(v);
  }

  for (const entry of IOS_OUTPUTS) {
    const v = await checkPng({
      output: join("ios", entry.name),
      size: entry.size,
    });
    if (v) violations.push(v);
  }

  for (const entry of MACOS_TRAY_OUTPUTS) {
    const v = await checkPng({
      output: join("tray", "macos", entry.name),
      size: entry.size,
    });
    if (v) violations.push(v);
  }

  {
    const v = await checkPng({
      output: DMG_BACKGROUND.name,
      size: DMG_BACKGROUND.size,
    });
    if (v) violations.push(v);
  }

  for (const [folder, size] of Object.entries(ANDROID_DENSITIES)) {
    for (const variant of ["", "_round", "_foreground"]) {
      const name = `ic_launcher${variant}.png`;
      const v = await checkPng({ output: join("android", folder, name), size });
      if (v) violations.push(v);
    }
  }

  const icnsPath = join(OUT_DIR, "icon.icns");
  if (existsSync(icnsPath)) {
    const expected = await encodeIcns(svgBuffer, ICNS_SIZES);
    if (!readFileSync(icnsPath).equals(expected))
      violations.push("icon.icns: out of date");
  } else {
    violations.push("icon.icns: missing");
  }

  const icoPath = join(OUT_DIR, "icon.ico");
  if (existsSync(icoPath)) {
    const expected = await encodeIco(svgBuffer, [16, 32, 48, 64, 128, 256]);
    if (!readFileSync(icoPath).equals(expected))
      violations.push("icon.ico: out of date");
  } else {
    violations.push("icon.ico: missing");
  }

  return violations;
}

if (args.has("--check")) {
  const violations = await checkOutputs();
  if (violations.length > 0) {
    console.error("Icons are out of date:");
    for (const v of violations) console.error(`- ${v}`);
    process.exit(1);
  }
  console.log(`Icons are in sync with ${relative(process.cwd(), SVG_PATH)}.`);
} else {
  console.log(`Generating icons from ${relative(process.cwd(), SVG_PATH)}...`);
  const n = (await ensureOutputs()).length;
  console.log(`\nGenerated ${n} icon files.`);
}
