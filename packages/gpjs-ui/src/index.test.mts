// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { beforeEach, describe, expect, it, vi } from "vitest";

import {
  addEventListener,
  appendChild,
  createNode,
  insertBefore,
  removeChild,
  setAttribute,
  setStyle,
} from "./index.mts";

function installMockNative() {
  const native = {
    createNode: vi.fn((_tag: string) => 1),
    appendChild: vi.fn(),
    insertBefore: vi.fn(),
    removeChild: vi.fn(),
    setAttribute: vi.fn(),
    setStyle: vi.fn(),
    addEventListener: vi.fn(),
  };
  globalThis.__gpjsui_native__ = native;
  return native;
}

beforeEach(() => {
  delete (globalThis as { __gpjsui_callbacks__?: unknown }).__gpjsui_callbacks__;
});

describe("wrapper functions forward to __gpjsui_native__", () => {
  it("createNode", () => {
    const native = installMockNative();
    expect(createNode("div")).toBe(1);
    expect(native.createNode).toHaveBeenCalledWith("div");
  });

  it("appendChild", () => {
    const native = installMockNative();
    appendChild(1, 2);
    expect(native.appendChild).toHaveBeenCalledWith(1, 2);
  });

  it("insertBefore", () => {
    const native = installMockNative();
    insertBefore(1, 2, 3);
    expect(native.insertBefore).toHaveBeenCalledWith(1, 2, 3);
  });

  it("insertBefore with a null anchor", () => {
    const native = installMockNative();
    insertBefore(1, 2, null);
    expect(native.insertBefore).toHaveBeenCalledWith(1, 2, null);
  });

  it("removeChild", () => {
    const native = installMockNative();
    removeChild(1, 2);
    expect(native.removeChild).toHaveBeenCalledWith(1, 2);
  });

  it("setAttribute", () => {
    const native = installMockNative();
    setAttribute(1, "label", "hello");
    expect(native.setAttribute).toHaveBeenCalledWith(1, "label", "hello");
  });

  it("setStyle", () => {
    const native = installMockNative();
    setStyle(1, "display", "flex");
    expect(native.setStyle).toHaveBeenCalledWith(1, "display", "flex");
  });
});

describe("addEventListener's callback registry", () => {
  it("stores the listener at __gpjsui_callbacks__[id] and forwards that id natively", () => {
    const native = installMockNative();
    const listener = vi.fn();

    addEventListener(1, "click", listener);

    expect(native.addEventListener).toHaveBeenCalledTimes(1);
    const [nodeId, event, callbackId] = native.addEventListener.mock.calls[0]!;
    expect(nodeId).toBe(1);
    expect(event).toBe("click");
    expect(globalThis.__gpjsui_callbacks__[callbackId]).toBe(listener);
  });

  it("allocates a distinct id per registration", () => {
    installMockNative();

    addEventListener(1, "click", vi.fn());
    addEventListener(2, "click", vi.fn());

    const ids = Object.keys(globalThis.__gpjsui_callbacks__);
    expect(ids).toHaveLength(2);
  });

  it("frees the previous callback id when the same (node, event) re-registers", () => {
    const native = installMockNative();

    addEventListener(1, "click", vi.fn());
    const firstId = native.addEventListener.mock.calls[0]![2];

    addEventListener(1, "click", vi.fn());
    const secondId = native.addEventListener.mock.calls[1]![2];

    expect(secondId).not.toBe(firstId);
    expect(globalThis.__gpjsui_callbacks__[firstId]).toBeUndefined();
    expect(Object.keys(globalThis.__gpjsui_callbacks__)).toHaveLength(1);
  });
});
