// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import path from "node:path";

/** Overrides host binary resolution — the only way tests point this at a stand-in. */
const HOST_BIN_ENV_VAR = "GPJS_UI_HOST_BIN";

/**
 * Where `gpjs-ui-host` itself lives. `GPJS_UI_HOST_BIN` wins outright;
 * otherwise this resolves to this workspace's own Cargo build output, the
 * only place a binary exists before a per-platform npm package ships one.
 */
export function resolveHostBin(): string {
  const fromEnv = process.env[HOST_BIN_ENV_VAR];
  if (fromEnv) return fromEnv;

  const suffix = process.platform === "win32" ? ".exe" : "";
  return path.resolve(import.meta.dirname, "../../../target/debug/gpjs-ui-host" + suffix);
}
