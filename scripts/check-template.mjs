import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

export const frontendTemplateSteps = [
  {
    name: "Template config sync",
    command: "pnpm",
    args: ["check:template-config"],
  },
  {
    name: "Brand boundary",
    command: "pnpm",
    args: ["check:brand"],
  },
  {
    name: "Shell boundary",
    command: "pnpm",
    args: ["check:shell-boundary"],
  },
  {
    name: "Frontend legacy files",
    command: "pnpm",
    args: ["check:frontend-legacy"],
  },
  {
    name: "Template dependencies",
    command: "pnpm",
    args: ["check:template-deps"],
  },
  {
    name: "Docs",
    command: "pnpm",
    args: ["check:docs"],
  },
  {
    name: "Icons",
    command: "pnpm",
    args: ["icons:check"],
  },
  {
    name: "TypeScript",
    command: "pnpm",
    args: ["typecheck"],
  },
  {
    name: "Unit tests",
    command: "pnpm",
    args: ["test:unit"],
  },
  {
    name: "Renderer build",
    command: "pnpm",
    args: ["build:renderer"],
  },
];

export const rustTemplateSteps = [
  {
    name: "Renderer dist placeholder",
    command: "node",
    args: ["scripts/check-template-helpers.mjs", "ensure-dist"],
  },
  {
    name: "Rust formatting",
    command: "cargo",
    args: ["fmt", "--check", "--manifest-path", "src-tauri/Cargo.toml"],
  },
  {
    name: "Rust clippy",
    command: "cargo",
    args: [
      "clippy",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--",
      "-D",
      "warnings",
    ],
  },
  {
    name: "Cargo check",
    command: "pnpm",
    args: ["check:tauri"],
  },
  {
    name: "Cargo tests",
    command: "pnpm",
    args: ["test:tauri"],
  },
];

export const frontendAppSteps = [
  {
    name: "Formatting",
    command: "pnpm",
    args: ["format:check"],
  },
  {
    name: "Template config sync",
    command: "pnpm",
    args: ["check:template-config"],
  },
  {
    name: "Icons",
    command: "pnpm",
    args: ["icons:check"],
  },
  {
    name: "TypeScript",
    command: "pnpm",
    args: ["typecheck"],
  },
  {
    name: "Unit tests",
    command: "pnpm",
    args: ["test:unit"],
  },
  {
    name: "Renderer build",
    command: "pnpm",
    args: ["build:renderer"],
  },
];

export const rustAppSteps = [
  {
    name: "Renderer dist placeholder",
    command: "node",
    args: ["scripts/check-template-helpers.mjs", "ensure-dist"],
  },
  {
    name: "Rust formatting",
    command: "cargo",
    args: ["fmt", "--check", "--manifest-path", "src-tauri/Cargo.toml"],
  },
  {
    name: "Rust clippy",
    command: "cargo",
    args: [
      "clippy",
      "--manifest-path",
      "src-tauri/Cargo.toml",
      "--",
      "-D",
      "warnings",
    ],
  },
  {
    name: "Cargo check",
    command: "pnpm",
    args: ["check:tauri"],
  },
  {
    name: "Cargo lib tests",
    command: "cargo",
    args: ["test", "--manifest-path", "src-tauri/Cargo.toml", "--lib"],
  },
];

export function parseArgs(argv) {
  return {
    frontendOnly: argv.includes("--frontend-only"),
    rustOnly: argv.includes("--rust-only"),
    app: argv.includes("--app"),
    appFrontendOnly: argv.includes("--app-frontend-only"),
    appRustOnly: argv.includes("--app-rust-only"),
    skipRust: argv.includes("--skip-rust"),
    list: argv.includes("--list"),
  };
}

export function validateOptions(options) {
  const modeCount = [
    options.frontendOnly,
    options.rustOnly,
    options.app,
    options.appFrontendOnly,
    options.appRustOnly,
    options.skipRust,
  ].filter(Boolean).length;

  if (modeCount > 1) {
    return "Use only one of --frontend-only, --rust-only, --app, --app-frontend-only, --app-rust-only, or --skip-rust.";
  }

  return undefined;
}

export function getTemplateSteps(options = {}) {
  if (options.appFrontendOnly) {
    return frontendAppSteps;
  }

  if (options.appRustOnly) {
    return rustAppSteps;
  }

  if (options.app) {
    return [...frontendAppSteps, ...rustAppSteps];
  }

  if (options.frontendOnly) {
    return frontendTemplateSteps;
  }

  if (options.rustOnly) {
    return rustTemplateSteps;
  }

  if (options.skipRust) {
    return frontendTemplateSteps;
  }

  return [...frontendTemplateSteps, ...rustTemplateSteps];
}

export function formatCommand(step) {
  return [step.command, ...step.args].join(" ");
}

export function runStep(step, options = {}) {
  const result = spawnSync(step.command, step.args, {
    stdio: options.stdio ?? "inherit",
    shell: process.platform === "win32",
    cwd: options.cwd ?? process.cwd(),
    env: {
      ...process.env,
      ...(options.env ?? {}),
    },
  });

  return {
    name: step.name,
    command: formatCommand(step),
    status: result.status ?? 1,
    signal: result.signal,
    error: result.error,
  };
}

export function printStepHeader(index, total, step) {
  console.log("");
  console.log(`▶ [${index + 1}/${total}] ${step.name}`);
  console.log(`  ${formatCommand(step)}`);
}

export function printSummary(results) {
  console.log("");
  console.log("Template check summary:");
  console.log("");

  for (const result of results) {
    const icon = result.status === 0 ? "✓" : "✗";
    console.log(`${icon} ${result.name} - ${result.command}`);
  }

  console.log("");
}

export function runTemplateChecks(options = {}) {
  const validationError = validateOptions(options);

  if (validationError) {
    console.error(validationError);
    return {
      ok: false,
      results: [],
    };
  }

  const steps = getTemplateSteps(options);

  if (options.list) {
    for (const step of steps) {
      console.log(`${step.name}: ${formatCommand(step)}`);
    }

    return {
      ok: true,
      results: [],
    };
  }

  const results = [];

  for (const [index, step] of steps.entries()) {
    printStepHeader(index, steps.length, step);

    const result = runStep(step, options);

    results.push(result);

    if (result.error) {
      printSummary(results);
      console.error(`Template check failed while running ${step.name}:`);
      console.error(result.error);
      return {
        ok: false,
        results,
      };
    }

    if (result.status !== 0) {
      printSummary(results);
      console.error(`Template check failed at step: ${step.name}`);
      return {
        ok: false,
        results,
      };
    }
  }

  printSummary(results);
  console.log("Template check passed.");

  return {
    ok: true,
    results,
  };
}

export function runCli(argv = process.argv.slice(2)) {
  const options = parseArgs(argv);
  const result = runTemplateChecks(options);

  if (!result.ok) {
    process.exitCode = 1;
  }
}

const currentFile = fileURLToPath(import.meta.url);

if (process.argv[1] === currentFile) {
  runCli();
}
