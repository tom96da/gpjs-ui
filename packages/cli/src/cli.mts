// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { build } from "./build.mts";
import { dev } from "./dev.mts";
import { printFault, toFault } from "./fault.mts";
import { packageApp } from "./package.mts";

const USAGE = "Usage: gpjsui <dev|build|package>";

/** Parses argv and runs the named subcommand: `dev`, `build`, or `package`. */
export async function run(argv: readonly string[] = process.argv): Promise<void> {
  const command = argv[2];

  if (command === "build") {
    try {
      const bundlePath = await build();
      process.stdout.write(`[gpjsui] built ${bundlePath}\n`);
    } catch (error) {
      printFault(process.stderr, "build failed", toFault(error));
      process.exitCode = 1;
    }
    return;
  }

  if (command === "package") {
    try {
      const { appPath } = await packageApp();
      process.stdout.write(`[gpjsui] packaged ${appPath}\n`);
    } catch (error) {
      printFault(process.stderr, "package failed", toFault(error));
      process.exitCode = 1;
    }
    return;
  }

  if (command !== "dev") {
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
