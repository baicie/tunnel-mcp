import path from "node:path";
import { readFileSync } from "node:fs";
import { Buffer } from "node:buffer";
import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import ts from "typescript";

interface TemplateConfig {
  appName: string;
  packageName: string;
  productName: string;
  identifier: string;
  description: string;
  repositoryUrl: string;
  deepLinkScheme: string;
  updaterEndpoint: string;
}

function assertTemplateConfig(value: unknown): asserts value is TemplateConfig {
  if (!value || typeof value !== "object") {
    throw new Error("template.config.ts must default-export an object");
  }

  const config = value as Record<string, unknown>;
  const requiredFields: Array<keyof TemplateConfig> = [
    "appName",
    "packageName",
    "productName",
    "identifier",
    "description",
    "repositoryUrl",
    "deepLinkScheme",
    "updaterEndpoint",
  ];

  const invalidFields = requiredFields.filter(
    (field) =>
      typeof config[field] !== "string" ||
      String(config[field]).trim() === "",
  );

  if (invalidFields.length > 0) {
    throw new Error(
      `template.config.ts has invalid fields: ${invalidFields.join(", ")}`,
    );
  }
}

async function loadTemplateConfig(): Promise<TemplateConfig> {
  const source = readFileSync(
    path.resolve(__dirname, "template.config.ts"),
    "utf8",
  );

  const transpiled = ts.transpileModule(source, {
    compilerOptions: {
      module: ts.ModuleKind.ES2022,
      target: ts.ScriptTarget.ES2022,
      esModuleInterop: true,
      moduleResolution: ts.ModuleResolutionKind.Bundler,
    },
  }).outputText;

  const dataUrl = `data:text/javascript;base64,${Buffer.from(
    transpiled,
  ).toString("base64")}`;
  const module = (await import(dataUrl)) as { default?: unknown };
  const config = module.default;

  assertTemplateConfig(config);

  return config;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function templateConfigPlugin(templateConfig: TemplateConfig): Plugin {
  return {
    name: "template-config-html-substitute",
    transformIndexHtml: {
      order: "pre",
      handler(html) {
        return html
          .replaceAll(
            "%TEMPLATE_PRODUCT_NAME%",
            escapeHtml(templateConfig.productName),
          )
          .replaceAll(
            "%TEMPLATE_DESCRIPTION%",
            escapeHtml(templateConfig.description),
          );
      },
    },
  };
}

export default defineConfig(async () => {
  const templateConfig = await loadTemplateConfig();

  return {
    root: "src",
    plugins: [templateConfigPlugin(templateConfig), react(), tailwindcss()],
    base: "./",
    build: {
      outDir: "../dist",
      emptyOutDir: true,
    },
    server: {
      port: 3000,
      strictPort: true,
    },
    resolve: {
      alias: {
        "@": path.resolve(__dirname, "./src"),
      },
    },
    clearScreen: false,
    envPrefix: ["VITE_", "TAURI_"],
  };
});
