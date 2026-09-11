// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import path from "node:path";

import { build as buildOnce } from "vite";

import { BUNDLE_FILE_NAME, resolveViteConfig } from "./config.mts";

/** Options for {@link build}. */
export interface BuildOptions {
  /** The app's own entry point — may import `.vue` files. */
  entry: string;
  /** Where the self-contained bundle is written — see `BuildResult.bundlePath` for the exact file. */
  outDir: string;
}

/** The result of a one-shot {@link build}. */
export interface BuildResult {
  /** Where the bundle was written — read this rather than assuming a name. */
  bundlePath: string;
}

/**
 * Builds `entry` into a minified, production bundle under `outDir` once, and
 * rejects on failure rather than reporting it through a callback. Never
 * starts or talks to `gpjs-ui-host` — that's `@gpjs-ui/cli`'s job.
 */
export async function build({ entry, outDir }: BuildOptions): Promise<BuildResult> {
  await buildOnce(resolveViteConfig({ entry, outDir, mode: "production", watch: false }));
  return { bundlePath: path.join(outDir, BUNDLE_FILE_NAME) };
}
