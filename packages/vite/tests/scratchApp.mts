// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import path from "node:path";

// Scratch apps live under tests/tmp/ (gitignored) rather than a real OS
// tmpdir: this mirrors how Vite resolves a real app's @vue/runtime-core
// import, by walking up to this package's own node_modules.
const scratchRoot = path.join(import.meta.dirname, "tmp");

export const setUpScratchRoot = (): Promise<string | undefined> =>
  mkdir(scratchRoot, { recursive: true });
export const tearDownScratchRoot = (): Promise<void> =>
  rm(scratchRoot, { recursive: true, force: true });

export interface ScratchApp {
  entry: string;
  outDir: string;
  vuePath: string;
}

export async function makeApp(vueSource: string): Promise<ScratchApp> {
  const appDir = await mkdtemp(path.join(scratchRoot, "app-"));
  const vuePath = path.join(appDir, "App.vue");
  const entry = path.join(appDir, "entry.mts");
  await writeFile(vuePath, vueSource);
  await writeFile(entry, `import App from "./App.vue";\nexport default App;\n`);
  return { entry, outDir: path.join(appDir, "dist"), vuePath };
}
