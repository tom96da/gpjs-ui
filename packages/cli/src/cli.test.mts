// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("./dev.mts", () => ({
  dev: vi.fn<(options: DevOptions) => Promise<void>>(() => new Promise(() => {})),
}));

import { run } from "./cli.mts";
import { dev } from "./dev.mts";

import type { DevOptions } from "./dev.mts";

const mockedDev = vi.mocked(dev);

afterEach(() => {
  mockedDev.mockClear();
  process.removeAllListeners("SIGINT");
  process.removeAllListeners("SIGTERM");
  process.exitCode = undefined;
});

describe("run", () => {
  it("prints usage and sets a non-zero exit code for anything but dev", async () => {
    await run(["node", "gpjsui"]);

    expect(process.exitCode).toBe(1);
    expect(mockedDev).not.toHaveBeenCalled();
  });

  it.each(["SIGINT", "SIGTERM"] as const)(
    "aborts dev()'s signal on %s — a script runner may deliver either on Ctrl-C",
    (signal) => {
      void run(["node", "gpjsui", "dev"]);

      const options = mockedDev.mock.calls[0]?.[0];
      expect(options?.signal.aborted).toBe(false);

      process.emit(signal);

      expect(options?.signal.aborted).toBe(true);
    },
  );
});
