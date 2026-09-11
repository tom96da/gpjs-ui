// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import path from "node:path";

import { build } from "vite";
import type { RolldownWatcher } from "rolldown";

import { BUNDLE_FILE_NAME, resolveViteConfig } from "./config.mts";

/** Options for {@link watch}. */
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

/** A running build watch — the value {@link watch} resolves to. */
export interface Watcher {
  /** Where the bundle is written — read this rather than assuming a name. */
  bundlePath: string;
  /** Stops watching and releases the underlying build process. */
  close(): Promise<void>;
}

/**
 * Builds `entry` into a bundle under `outDir` and rebuilds it on every
 * change. Never starts, reloads, or talks to `gpjs-ui-host` — that's
 * `@gpjs-ui/cli`'s job.
 */
export async function watch({ onBuild, onError, ...buildOptions }: WatchOptions): Promise<Watcher> {
  const bundlePath = path.join(buildOptions.outDir, BUNDLE_FILE_NAME);
  const result = await build(resolveViteConfig({ ...buildOptions, watch: true }));

  // build() types its return as the non-watch output too, since a single
  // call signature covers both — watch: {} in the resolved config means
  // it's always this.
  const watcher = result as RolldownWatcher;

  watcher.on("event", (event) => {
    if (event.code === "END") {
      onBuild(bundlePath);
    } else if (event.code === "ERROR") {
      onError({ message: event.error.message, stack: event.error.stack ?? null });
    }
  });

  return { bundlePath, close: () => watcher.close() };
}
