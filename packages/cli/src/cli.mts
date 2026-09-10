// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { dev } from "./dev.mts";

const USAGE = "Usage: gpjsui dev";

/** Parses argv and runs the named subcommand — today, only `dev`. */
export async function run(argv: readonly string[] = process.argv): Promise<void> {
  if (argv[2] !== "dev") {
    process.stderr.write(`${USAGE}\n`);
    process.exitCode = 1;
    return;
  }

  const controller = new AbortController();
  const onSignal = (): void => {
    process.stderr.write("[gpjsui] shutting down\n");
    controller.abort();
  };
  // A wrapper script runner (e.g. `pnpm run`) commonly delivers SIGTERM to
  // its child directly on Ctrl-C, rather than relying on the terminal to
  // signal the whole process group — SIGINT alone left the host orphaned.
  process.on("SIGINT", onSignal);
  process.on("SIGTERM", onSignal);

  await dev({ signal: controller.signal });
}
