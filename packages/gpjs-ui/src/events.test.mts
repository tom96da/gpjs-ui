// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { beforeEach, describe, expect, it, vi } from "vitest";

import { addEventListener, disposeNode } from "./events.mts";

const native = {
  rootNodeId: vi.fn<() => number>(() => 0),
  createNode: vi.fn<(tag: string) => number>((_tag: string) => 1),
  appendChild: vi.fn<(parentId: number, childId: number) => void>(),
  insertBefore: vi.fn<(parentId: number, childId: number, anchorId: number | null) => void>(),
  removeChild: vi.fn<(parentId: number, childId: number) => void>(),
  setAttribute: vi.fn<(nodeId: number, key: string, value: unknown) => void>(),
  setStyle: vi.fn<(nodeId: number, key: string, value: unknown) => void>(),
  addEventListener: vi.fn<(nodeId: number, event: string, callbackId: number) => void>(),
};

beforeEach(() => {
  vi.clearAllMocks();
  globalThis.__gpjsui_native__ = native;
  delete (globalThis as { __gpjsui_callbacks__?: unknown }).__gpjsui_callbacks__;
});

describe("addEventListener's callback registry", () => {
  it("stores the listener at __gpjsui_callbacks__[id] and forwards that id natively", () => {
    const listener = vi.fn<() => void>();

    addEventListener(1, "click", listener);

    expect(native.addEventListener).toHaveBeenCalledTimes(1);
    const [nodeId, event, callbackId] = native.addEventListener.mock.calls[0]!;
    expect(nodeId).toBe(1);
    expect(event).toBe("click");
    expect(globalThis.__gpjsui_callbacks__[callbackId]).toBe(listener);
  });

  it("allocates a distinct id per registration", () => {
    addEventListener(1, "click", vi.fn());
    addEventListener(2, "click", vi.fn());

    const ids = Object.keys(globalThis.__gpjsui_callbacks__);
    expect(ids).toHaveLength(2);
  });

  it("frees the previous callback id when the same (node, event) re-registers", () => {
    addEventListener(1, "click", vi.fn());
    const firstId = native.addEventListener.mock.calls[0]![2];

    addEventListener(1, "click", vi.fn());
    const secondId = native.addEventListener.mock.calls[1]![2];

    expect(secondId).not.toBe(firstId);
    expect(globalThis.__gpjsui_callbacks__[firstId]).toBeUndefined();
    expect(Object.keys(globalThis.__gpjsui_callbacks__)).toHaveLength(1);
  });
});

describe("disposeNode", () => {
  it("frees every callback id registered for a node, across all event names", () => {
    addEventListener(1, "click", vi.fn());
    addEventListener(1, "hover", vi.fn());
    addEventListener(2, "click", vi.fn());

    disposeNode(1);

    expect(Object.keys(globalThis.__gpjsui_callbacks__)).toHaveLength(1);
  });

  it("re-registering (nodeId, event) after dispose allocates a fresh id, not a stale one", () => {
    addEventListener(1, "click", vi.fn());
    disposeNode(1);
    addEventListener(1, "click", vi.fn());

    const secondId = native.addEventListener.mock.calls[1]![2];
    expect(globalThis.__gpjsui_callbacks__[secondId]).toBeDefined();
    expect(Object.keys(globalThis.__gpjsui_callbacks__)).toHaveLength(1);
  });

  it("is a no-op for a node with no registered listeners", () => {
    expect(() => disposeNode(999)).not.toThrow();
  });
});
