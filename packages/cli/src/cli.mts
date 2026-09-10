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
  process.on("SIGINT", () => {
    controller.abort();
  });

  await dev({ signal: controller.signal });
}
