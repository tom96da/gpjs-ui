// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import path from "node:path";

import { describe, expect, it } from "vitest";

import { HostClient, HostError } from "../src/index.mts";

const mockHost = path.join(import.meta.dirname, "fixtures/mock-host.mts");
const wedgedMockHost = path.join(import.meta.dirname, "fixtures/mock-host-wedged.mts");

describe("HostClient", () => {
  it("spawns the host, calls it, and correlates the response by id", async () => {
    const notifications: { method: string; params: unknown }[] = [];
    const client = new HostClient({
      hostBin: mockHost,
      bundlePath: "bundle.js",
      onNotification: (method, params) => notifications.push({ method, params }),
      onStderr: () => {},
    });

    await client.start();
    await expect(client.call("reload")).resolves.toBeNull();
    await client.stop();

    expect(notifications).toEqual([{ method: "ready", params: { protocol: 0 } }]);
  });

  it("relays the host's real stderr and a stray stdout line, distinguishably", async () => {
    const lines: string[] = [];
    const client = new HostClient({
      hostBin: mockHost,
      bundlePath: "bundle.js",
      onStderr: (line) => lines.push(line),
    });

    await client.start();
    await client.call("reload");
    await client.stop();

    expect(lines.some((line) => line.includes("mock host: booted"))).toBe(true);
    expect(lines.some((line) => line.startsWith("[stray stdout] "))).toBe(true);
  });

  it("rejects a call the host answers with a JSON-RPC error", async () => {
    const client = new HostClient({
      hostBin: mockHost,
      bundlePath: "bundle.js",
      onStderr: () => {},
    });
    await client.start();

    await expect(client.call("bogus")).rejects.toThrow(HostError);

    await client.stop();
  });

  it("kills the child once the shutdown deadline passes without it exiting", async () => {
    const client = new HostClient({ hostBin: wedgedMockHost, bundlePath: "bundle.js" });
    await client.start();

    // A wedged app that ignores `shutdown` would otherwise hang this
    // `await` forever — resolving at all is the kill fallback working.
    await expect(client.stop(50)).resolves.toBeUndefined();
  }, 5000);
});
