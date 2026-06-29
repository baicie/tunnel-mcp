# Agent Guide

This repo is a Tauri 2 + React desktop shell template. Keep it small, generic,
and reusable.

## Project Shape

- Frontend code lives in `src/`.
- Tauri/Rust code lives in `src-tauri/`.
- Template identity starts in `template.config.ts`; generated identity files
  should be updated with `pnpm sync:template`.
- Project docs live in `README.md` and `docs/`.
- Icon source lives in `scripts/assets/source-icon.svg`; generated icon assets
  live in `src-tauri/icons/`.

## Agent Skills

### Issue Tracker

Issues and PRDs are tracked in GitHub Issues for
`baicie/tauri-template`. See `docs/agents/issue-tracker.md`.

### Triage Labels

Use the default triage vocabulary: `needs-triage`, `needs-info`,
`ready-for-agent`, `ready-for-human`, and `wontfix`. See
`docs/agents/triage-labels.md`.

### Domain Docs

This is a single-context repo. Read the root docs first, then ADRs if they are
added later. See `docs/agents/domain.md`.

### Recommended Skills

- Use `build` or `incremental-implementation` for multi-file changes.
- Use `test` or `test-driven-development` when changing behavior.
- Use `diagnose` for failing tests, broken builds, and runtime bugs.
- Use `frontend-ui-engineering` and `shadcn` when changing React UI.
- Use `ci-cd-and-automation` when editing `.github/workflows/`.
- Use `review` before merging larger changes.

### Language

- 除非用户明确要求其它语言,Agent 在本仓库中的回复一律使用中文(简体)。
  代码、commit message、PR 标题/描述以及文件内容继续沿用英文等原有约定,
  不要因为中文回复而改动代码或文档语言。

## Coding Rules

- Prefer existing structure over new abstractions.
- Keep shell/template code product-neutral. Do not add product-specific modules
  such as accounts, provider management, prompt editors, analytics dashboards,
  or cloud backends.
- Frontend Tauri `invoke` calls belong only in `src/lib/api/shell.ts`.
- Rust command adapters belong in `src-tauri/src/commands/`; reusable shell
  logic belongs in `src-tauri/src/shell/`.
- Do not manually edit generated brand files. Change `template.config.ts`, then
  run `pnpm sync:template`.
- Do not manually edit generated icon outputs unless debugging the generator.
  Change `scripts/assets/source-icon.svg`, then run `pnpm icons:generate`.

## Validation

Use the app gate locally and in the default CI:

```bash
pnpm check:all
```

`check:all` must stay suitable for apps created from this template. Do not add
template-maintenance checks to it if they would block product-specific code,
dependencies, or documentation.

Useful narrower checks:

```bash
pnpm check:app:frontend
pnpm check:app:rust
pnpm check:template:frontend
pnpm check:template:rust
pnpm icons:check
pnpm typecheck
pnpm test:unit
pnpm test:tauri
```

Before changing CI, make sure the command exists in `package.json` and can run
locally.

Use `pnpm check:template` only when maintaining this template repository itself.
