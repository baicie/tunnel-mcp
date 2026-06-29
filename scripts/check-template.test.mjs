import { describe, expect, it } from "vitest";
import {
  frontendAppSteps,
  frontendTemplateSteps,
  rustAppSteps,
  rustTemplateSteps,
  formatCommand,
  getTemplateSteps,
  parseArgs,
  validateOptions,
} from "./check-template.mjs";

describe("check-template", () => {
  it("parses frontend only flag", () => {
    expect(parseArgs(["--frontend-only"])).toEqual({
      frontendOnly: true,
      rustOnly: false,
      app: false,
      appFrontendOnly: false,
      appRustOnly: false,
      skipRust: false,
      list: false,
    });
  });

  it("parses rust only flag", () => {
    expect(parseArgs(["--rust-only"])).toEqual({
      frontendOnly: false,
      rustOnly: true,
      app: false,
      appFrontendOnly: false,
      appRustOnly: false,
      skipRust: false,
      list: false,
    });
  });

  it("parses skip rust flag", () => {
    expect(parseArgs(["--skip-rust"])).toEqual({
      frontendOnly: false,
      rustOnly: false,
      app: false,
      appFrontendOnly: false,
      appRustOnly: false,
      skipRust: true,
      list: false,
    });
  });

  it("parses app flags", () => {
    expect(parseArgs(["--app"])).toEqual({
      frontendOnly: false,
      rustOnly: false,
      app: true,
      appFrontendOnly: false,
      appRustOnly: false,
      skipRust: false,
      list: false,
    });

    expect(parseArgs(["--app-frontend-only"])).toMatchObject({
      appFrontendOnly: true,
    });

    expect(parseArgs(["--app-rust-only"])).toMatchObject({
      appRustOnly: true,
    });
  });

  it("parses list flag", () => {
    expect(parseArgs(["--list"])).toEqual({
      frontendOnly: false,
      rustOnly: false,
      app: false,
      appFrontendOnly: false,
      appRustOnly: false,
      skipRust: false,
      list: true,
    });
  });

  it("rejects conflicting mode flags", () => {
    expect(validateOptions(parseArgs(["--frontend-only", "--rust-only"]))).toBe(
      "Use only one of --frontend-only, --rust-only, --app, --app-frontend-only, --app-rust-only, or --skip-rust.",
    );
  });

  it("returns app steps for app modes", () => {
    expect(getTemplateSteps({ appFrontendOnly: true })).toEqual(
      frontendAppSteps,
    );
    expect(getTemplateSteps({ appRustOnly: true })).toEqual(rustAppSteps);
    expect(getTemplateSteps({ app: true })).toEqual([
      ...frontendAppSteps,
      ...rustAppSteps,
    ]);
  });

  it("returns frontend steps for frontend only mode", () => {
    expect(getTemplateSteps({ frontendOnly: true })).toEqual(
      frontendTemplateSteps,
    );
  });

  it("returns rust steps for rust only mode", () => {
    expect(getTemplateSteps({ rustOnly: true })).toEqual(rustTemplateSteps);
  });

  it("returns frontend steps when rust is skipped", () => {
    expect(getTemplateSteps({ skipRust: true })).toEqual(frontendTemplateSteps);
  });

  it("returns all steps by default", () => {
    expect(getTemplateSteps()).toEqual([
      ...frontendTemplateSteps,
      ...rustTemplateSteps,
    ]);
  });

  it("formats command", () => {
    expect(
      formatCommand({
        name: "TypeScript",
        command: "pnpm",
        args: ["typecheck"],
      }),
    ).toBe("pnpm typecheck");
  });

  it("keeps expected frontend step order", () => {
    expect(frontendTemplateSteps.map((step) => step.name)).toEqual([
      "Template config sync",
      "Brand boundary",
      "Shell boundary",
      "Frontend legacy files",
      "Template dependencies",
      "Docs",
      "Icons",
      "TypeScript",
      "Unit tests",
      "Renderer build",
    ]);
  });

  it("keeps expected rust step order", () => {
    expect(rustTemplateSteps.map((step) => step.name)).toEqual([
      "Renderer dist placeholder",
      "Rust formatting",
      "Rust clippy",
      "Cargo check",
      "Cargo tests",
    ]);
  });

  it("keeps expected app frontend step order", () => {
    expect(frontendAppSteps.map((step) => step.name)).toEqual([
      "Formatting",
      "Template config sync",
      "Icons",
      "TypeScript",
      "Unit tests",
      "Renderer build",
    ]);
  });

  it("keeps expected app rust step order", () => {
    expect(rustAppSteps.map((step) => step.name)).toEqual([
      "Renderer dist placeholder",
      "Rust formatting",
      "Rust clippy",
      "Cargo check",
      "Cargo lib tests",
    ]);
  });
});
