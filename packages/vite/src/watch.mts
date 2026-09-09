// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import path from "node:path";

import vue from "@vitejs/plugin-vue";
import { build } from "vite";

import type { RolldownWatcher } from "rolldown";

const BUNDLE_FILE_NAME = "bundle.js";

export interface WatchOptions {
  /** The app's own entry point — may import `.vue` files. */
  entry: string;
  /** Where the self-contained bundle is written — see `Watcher.bundlePath` for the exact file. */
  outDir: string;
  /** `"development"` keeps `@vue/runtime-core`'s own warnings; `"production"` strips them. */
  mode: "development" | "production";
  /** Called after each successful (re)build, with the path to the freshly written bundle. */
  onBuild: (bundlePath: string) => void;
  /** Called instead of `onBuild` when a (re)build fails. */
  onError: (error: { message: string; stack: string | null }) => void;
}

export interface Watcher {
  /** Where the bundle is written — read this rather than assuming a name. */
  bundlePath: string;
  /** Stops watching and releases the underlying build process. */
  close(): Promise<void>;
}

type BuildOptions = Omit<WatchOptions, "onBuild" | "onError">;

/**
 * Starts the underlying Vite build in watch mode, compiling `.vue` files
 * via `@vitejs/plugin-vue` targeted at `@vue/runtime-core` rather than the
 * `vue` meta-package it requires to run.
 */
async function createBuildWatcher({ entry, outDir, mode }: BuildOptions): Promise<RolldownWatcher> {
  const result = await build({
    configFile: false,
    root: path.dirname(entry),
    mode,
    clearScreen: false,
    logLevel: "silent",
    define: {
      "process.env.NODE_ENV": JSON.stringify(mode),
    },
    plugins: [
      vue({
        template: {
          compilerOptions: { runtimeModuleName: "@vue/runtime-core" },
        },
      }),
    ],
    build: {
      lib: {
        entry,
        formats: ["es"],
        fileName: () => BUNDLE_FILE_NAME,
      },
      outDir,
      minify: mode === "production",
      watch: {},
    },
  });

  // build() types its return as the non-watch output too, since a single
  // call signature covers both — watch: {} above means it's always this.
  return result as RolldownWatcher;
}

/**
 * Builds `entry` into a bundle under `outDir` and rebuilds it on every
 * change. Never starts, reloads, or talks to `gpjs-ui-host` — that's
 * `@gpjs-ui/cli`'s job.
 */
export async function watch({ onBuild, onError, ...buildOptions }: WatchOptions): Promise<Watcher> {
  const bundlePath = path.join(buildOptions.outDir, BUNDLE_FILE_NAME);
  const watcher = await createBuildWatcher(buildOptions);

  watcher.on("event", (event) => {
    if (event.code === "END") {
      onBuild(bundlePath);
    } else if (event.code === "ERROR") {
      onError({ message: event.error.message, stack: event.error.stack ?? null });
    }
  });

  return { bundlePath, close: () => watcher.close() };
}
