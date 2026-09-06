// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

// One-shot, ahead-of-time build: compiles src/hello_world.vue via
// @vue/compiler-sfc directly (no Vite dev server, no @vitejs/plugin-vue),
// then bundles the result together with @gpjs-ui/vue and @vue/runtime-core
// into one self-contained dist/bundle.js with zero unresolved imports,
// consumable by gpjs-ui-example-runner's Engine::eval_module.

import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { compileScript, parse } from "@vue/compiler-sfc";
import { build } from "vite";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const distDir = path.join(root, "dist");

const sfcPath = path.join(root, "src/hello_world.vue");
const source = await readFile(sfcPath, "utf8");
const { descriptor, errors } = parse(source, { filename: sfcPath });
if (errors.length > 0) {
  throw new AggregateError(errors, `failed to parse ${sfcPath}`);
}

// runtimeModuleName retargets the render function's auto-imported helpers
// (openBlock, createElementBlock, ...) from the default "vue" to
// "@vue/runtime-core" — the only Vue runtime package this repo depends on,
// see packages/vue/package.json.
const compiled = compileScript(descriptor, {
  id: "hello_world",
  inlineTemplate: true,
  templateOptions: {
    compilerOptions: { runtimeModuleName: "@vue/runtime-core" },
  },
});

const entrySource = `import { createGpjsuiApp } from "@gpjs-ui/vue";
import HelloWorld from "./_compiled.mjs";

createGpjsuiApp(HelloWorld).mount();
`;

await mkdir(distDir, { recursive: true });
await writeFile(path.join(distDir, "_compiled.mjs"), compiled.content);
await writeFile(path.join(distDir, "_entry.mjs"), entrySource);

await build({
  configFile: false,
  root,
  // @vue/runtime-core's dev-only warning branches check process.env.NODE_ENV
  // directly — QuickJS has no Node.js globals, so process is otherwise left
  // referencing nothing and throws. define replaces it with a literal at
  // build time, so no `process` reference survives into the bundle.
  define: { "process.env.NODE_ENV": JSON.stringify("production") },
  build: {
    lib: {
      entry: path.join(distDir, "_entry.mjs"),
      formats: ["es"],
      fileName: () => "bundle.js",
    },
    outDir: distDir,
    emptyOutDir: false,
    minify: false,
  },
});
