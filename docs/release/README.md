# Release Guide

## Local smoke

```bash
pnpm release:smoke
```

## Local release check

```bash
pnpm release:check
```

## Platform artifact verification

macOS arm64:

```bash
pnpm tauri build --target aarch64-apple-darwin
pnpm release:verify-artifacts -- --platform macos --target aarch64-apple-darwin
```

Windows x64:

```bash
pnpm tauri build
pnpm release:verify-artifacts -- --platform windows
```

## Manual MVP acceptance

Before publishing a public release, complete:

```txt
docs/release/mvp-checklist.md
```