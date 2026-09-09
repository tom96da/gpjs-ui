// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import path from "node:path";

import { afterAll, afterEach, beforeAll, describe, expect, it } from "vitest";

import { watch } from "../src/index.mts";

import type { Watcher } from "../src/index.mts";

// Scratch apps live under tests/tmp/ (gitignored) rather than a real OS
// tmpdir: this mirrors how Vite resolves a real app's @vue/runtime-core
// import, by walking up to this package's own node_modules.
const scratchRoot = path.join(import.meta.dirname, "tmp");

let watchers: Watcher[] = [];

beforeAll(() => mkdir(scratchRoot, { recursive: true }));
afterAll(() => rm(scratchRoot, { recursive: true, force: true }));
afterEach(async () => {
  await Promise.all(watchers.map((watcher) => watcher.close()));
  watchers = [];
});

async function makeApp(
  vueSource: string,
): Promise<{ entry: string; outDir: string; vuePath: string }> {
  const appDir = await mkdtemp(path.join(scratchRoot, "app-"));
  const vuePath = path.join(appDir, "App.vue");
  const entry = path.join(appDir, "entry.mts");
  await writeFile(vuePath, vueSource);
  await writeFile(entry, `import App from "./App.vue";\nexport default App;\n`);
  return { entry, outDir: path.join(appDir, "dist"), vuePath };
}

describe("watch", () => {
  it("compiles a .vue file into a self-contained bundle", async () => {
    const { entry, outDir } = await makeApp(
      `<script setup>\nconst msg = "hello";\n</script>\n<template><div>{{ msg }}</div></template>\n`,
    );

    let watcher!: Watcher;
    const bundlePath = await new Promise<string>((resolve, reject) => {
      watch({
        entry,
        outDir,
        mode: "development",
        onBuild: resolve,
        onError: (error) => reject(new Error(error.message)),
      })
        .then((w) => {
          watcher = w;
          watchers.push(w);
        })
        .catch(reject);
    });

    expect(watcher.bundlePath).toBe(bundlePath);
    const bundle = await readFile(watcher.bundlePath, "utf8");
    expect(bundle).toContain("@vue/runtime-core");
    expect(bundle).not.toMatch(/from\s+["']vue["']/);
  }, 20000);

  it("rebuilds when the .vue file changes", async () => {
    const { entry, outDir, vuePath } = await makeApp(
      `<script setup>\nconst msg = "first";\n</script>\n<template><div>{{ msg }}</div></template>\n`,
    );

    let builds = 0;
    let resolveBuild!: (bundlePath: string) => void;
    let nextBuild = new Promise<string>((resolve) => {
      resolveBuild = resolve;
    });

    const watcher = await watch({
      entry,
      outDir,
      mode: "development",
      onBuild: (bundlePath) => {
        builds += 1;
        resolveBuild(bundlePath);
      },
      onError: (error) => {
        throw new Error(error.message);
      },
    });
    watchers.push(watcher);

    await nextBuild;
    nextBuild = new Promise((resolve) => {
      resolveBuild = resolve;
    });

    await writeFile(
      vuePath,
      `<script setup>\nconst msg = "second";\n</script>\n<template><div>{{ msg }}</div></template>\n`,
    );
    const bundlePath = await nextBuild;

    expect(builds).toBe(2);
    const bundle = await readFile(bundlePath, "utf8");
    expect(bundle).toContain("second");
  }, 20000);

  it("reports a syntax error without throwing", async () => {
    const { entry, outDir } = await makeApp(
      `<script setup>\nconst broken = ;\n</script>\n<template><div/></template>\n`,
    );

    const error = await new Promise<{ message: string; stack: string | null }>(
      (resolve, reject) => {
        watch({
          entry,
          outDir,
          mode: "development",
          onBuild: () => reject(new Error("expected a build error, got a successful build")),
          onError: resolve,
        })
          .then((watcher) => watchers.push(watcher))
          .catch(reject);
      },
    );

    expect(error.message).toBeTruthy();
  }, 20000);
});
