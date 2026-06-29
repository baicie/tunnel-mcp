# Checks

This document explains the verification commands used by Tunnel MCP after the Phase 0 fork.

## App checks

These are the default checks for apps created from this template.

Run:

```bash
pnpm check:app
```

It includes:

```txt
1. formatting
2. template config sync check
3. icon sync check
4. TypeScript check
5. frontend unit tests
6. renderer build
7. Rust formatting
8. Rust clippy
9. cargo check
10. cargo library tests
```

It intentionally does not include template-maintenance checks such as docs shape,
forbidden product dependencies, legacy frontend file guards, or product-neutral
shell vocabulary guards. Those are useful for maintaining the shell/template
layer, but they should not block a downstream product app created from the
template.

### Frontend only

```bash
pnpm check:app:frontend
```

### Rust only

```bash
pnpm check:app:rust
```

## Template checks

These are strict checks for maintaining the Phase 0 shell/template layer.

Run:

```bash
pnpm check:template
```

It includes:

```txt
1. template config sync check
2. brand check
3. shell boundary check
4. frontend legacy file check
5. template dependency check
6. docs check
7. icon sync check
8. TypeScript check
9. frontend unit tests
10. renderer build
11. Rust formatting
12. Rust clippy
13. cargo check
14. cargo test
```

### Frontend only

```bash
pnpm check:template:frontend
```

### Rust only

```bash
pnpm check:template:rust
```

### List check steps

```bash
node scripts/check-template.mjs --list
```

### All checks

```bash
pnpm check:all
```

`check:all` runs both `pnpm check:template` and `pnpm check:app` for this
repository. Use it before merging Phase 0 shell/template changes.

## Frontend typecheck

```bash
pnpm typecheck
```

Validates TypeScript.

## Frontend unit tests

```bash
pnpm test:unit
```

Runs Vitest.

## Template config check

```bash
pnpm check:template-config
```

Ensures generated identity files match:

```txt
template.config.ts
```

If it fails, run:

```bash
pnpm sync:template
```

## Brand check

```bash
pnpm check:brand
```

Ensures:

```txt
1. old brand markers do not exist
2. template identity is not hardcoded in business code
3. generated brand files are the only identity source
```

## Shell boundary check

```bash
pnpm check:shell-boundary
```

Ensures frontend shell code does not contain legacy concepts such as:

```txt
provider
proxy
mcp
prompt
usage
subscription
workspace
codex
gemini
claude
```

It also ensures:

```txt
invoke only appears in src/lib/api/shell.ts
```

## Frontend legacy file check

```bash
pnpm check:frontend-legacy
```

Ensures old frontend directories and files are removed:

```txt
src/hooks
src/i18n
src/features
src/views
src/store
src/types
old src/lib/api files
```

## Template dependency check

```bash
pnpm check:template-deps
```

Ensures minimal template dependencies.

Forbidden frontend dependencies include:

```txt
@codemirror/*
@dnd-kit/*
recharts
i18next
react-i18next
flexsearch
framer-motion
cmdk
```

Forbidden Rust direct dependencies include:

```txt
reqwest
axum
hyper
rusqlite
rquickjs
zip
brotli
zstd
```

## Docs check

```bash
pnpm check:docs
```

Ensures required docs exist and contain required Phase 0 sections.

## Icon sync check

```bash
pnpm icons:check
```

Ensures generated icon assets match:

```txt
scripts/assets/source-icon.svg
```

If it fails, run:

```bash
pnpm icons:generate
```

## Rust tests

```bash
pnpm test:tauri
```

Equivalent to:

```bash
cd src-tauri && cargo test
```

## Manual smoke test

Run:

```bash
pnpm dev
```

Check:

```txt
1. app starts
2. Dashboard opens by default
3. Settings page opens
4. About page opens
5. theme setting can be saved
6. external link can be opened
7. window controls do not crash
```

## CI

CI should run:

```bash
pnpm check:app:frontend
pnpm check:app:rust
```

Local full verification for this repository should use `pnpm check:all`.
Downstream product apps can use `pnpm check:app`.

This repository may also run the stricter template-maintenance commands:

```bash
pnpm check:template:frontend
pnpm check:template:rust
```

## When a check fails

### Template config drift

Run:

```bash
pnpm sync:template
```

### Brand hardcoding

Use:

```txt
src/lib/brand/brand.ts
```

instead of string literals.

### Shell boundary violation

Remove product-specific concepts from the shell layer or move them to a later
product phase with an explicit boundary update.

### Dependency violation

Remove the dependency or move the feature to an optional extension.
