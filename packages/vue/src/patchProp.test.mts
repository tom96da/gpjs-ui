// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("gpjs-ui", () => ({
  addEventListener:
    vi.fn<(nodeId: number, event: string, listener: (...args: unknown[]) => void) => void>(),
  setAttribute: vi.fn<(nodeId: number, key: string, value: unknown) => void>(),
  setStyle: vi.fn<(nodeId: number, key: string, value: unknown) => void>(),
}));

import * as gpjsUi from "gpjs-ui";

import { patchProp } from "./patchProp.mts";

import type { GpjsuiElement } from "./nodeOps.mts";

const el: GpjsuiElement = { id: 1, kind: "element", parent: null, children: [] };

beforeEach(() => {
  vi.clearAllMocks();
});

describe("style", () => {
  it("calls setStyle once per primitive-valued entry", () => {
    patchProp(el, "style", null, { background: "#000", gap: 8 }, undefined, null);

    expect(gpjsUi.setStyle).toHaveBeenCalledWith(1, "background", "#000");
    expect(gpjsUi.setStyle).toHaveBeenCalledWith(1, "gap", 8);
    expect(gpjsUi.setStyle).toHaveBeenCalledTimes(2);
  });

  it("skips entries whose value isn't a primitive setStyle can take", () => {
    patchProp(el, "style", null, { border_width: [1, 2] }, undefined, null);

    expect(gpjsUi.setStyle).not.toHaveBeenCalled();
  });

  it("does nothing for a null style value", () => {
    patchProp(el, "style", { background: "#000" }, null, undefined, null);

    expect(gpjsUi.setStyle).not.toHaveBeenCalled();
  });
});

describe("on*", () => {
  it("registers a function listener under the lower-cased event name", () => {
    const listener = vi.fn<() => void>();
    patchProp(el, "onClick", null, listener, undefined, null);

    expect(gpjsUi.addEventListener).toHaveBeenCalledWith(1, "click", listener);
  });

  it("ignores a non-function value", () => {
    patchProp(el, "onClick", null, undefined, undefined, null);

    expect(gpjsUi.addEventListener).not.toHaveBeenCalled();
  });
});

describe("everything else", () => {
  it.each([
    ["label", "hello"],
    ["count", 3],
    ["disabled", true],
  ])("forwards %s=%p to setAttribute", (key, value) => {
    patchProp(el, key, null, value, undefined, null);

    expect(gpjsUi.setAttribute).toHaveBeenCalledWith(1, key, value);
  });

  it("skips a value setAttribute can't take", () => {
    patchProp(el, "data", null, { nested: true }, undefined, null);

    expect(gpjsUi.setAttribute).not.toHaveBeenCalled();
  });
});
