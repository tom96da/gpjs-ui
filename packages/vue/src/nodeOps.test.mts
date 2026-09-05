// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("gpjs-ui", () => ({
  createNode: vi.fn<(tag: string) => number>(),
  appendChild: vi.fn<(parentId: number, childId: number) => void>(),
  insertBefore: vi.fn<(parentId: number, childId: number, anchorId: number | null) => void>(),
  removeChild: vi.fn<(parentId: number, childId: number) => void>(),
  setAttribute: vi.fn<(nodeId: number, key: string, value: unknown) => void>(),
  setStyle: vi.fn<(nodeId: number, key: string, value: unknown) => void>(),
  disposeNode: vi.fn<(nodeId: number) => void>(),
}));

import * as gpjsUi from "gpjs-ui";

import { nodeOps } from "./nodeOps.mts";

import type { GpjsuiElement, GpjsuiText } from "./nodeOps.mts";

let nextId = 1;

beforeEach(() => {
  vi.clearAllMocks();
  nextId = 1;
  vi.mocked(gpjsUi.createNode).mockImplementation(() => nextId++);
});

function element(): GpjsuiElement {
  return { id: nextId++, kind: "element", parent: null, children: [] };
}

describe("createElement", () => {
  it("allocates a native node for the tag and returns a parentless, childless element", () => {
    const el = nodeOps.createElement("div");
    expect(gpjsUi.createNode).toHaveBeenCalledWith("div");
    expect(el).toEqual({ id: 1, kind: "element", parent: null, children: [] });
  });
});

describe("createText", () => {
  it("creates a text node and forwards the initial value", () => {
    const node = nodeOps.createText("hello");
    expect(gpjsUi.createNode).toHaveBeenCalledWith("text");
    expect(gpjsUi.setAttribute).toHaveBeenCalledWith(1, "value", "hello");
    expect(node).toEqual({ id: 1, kind: "text", parent: null, text: "hello" });
  });
});

describe("createComment", () => {
  it("creates a hidden container in place of a comment", () => {
    const node = nodeOps.createComment("v-if");
    expect(gpjsUi.createNode).toHaveBeenCalledWith("div");
    expect(gpjsUi.setStyle).toHaveBeenCalledWith(1, "display", "none");
    expect(node).toEqual({ id: 1, kind: "comment", parent: null, children: [] });
  });
});

describe("setText", () => {
  it("forwards the new value and updates the node", () => {
    const node = nodeOps.createText("old") as GpjsuiText;
    nodeOps.setText(node, "new");
    expect(gpjsUi.setAttribute).toHaveBeenCalledWith(1, "value", "new");
    expect(node.text).toBe("new");
  });
});

describe("setElementText", () => {
  it("replaces existing children with one text child", () => {
    const el = element();
    const oldChild = element();
    el.children.push(oldChild);
    oldChild.parent = el;

    nodeOps.setElementText(el, "hello");

    expect(gpjsUi.removeChild).toHaveBeenCalledWith(el.id, oldChild.id);
    expect(gpjsUi.disposeNode).toHaveBeenCalledWith(oldChild.id);
    expect(oldChild.parent).toBeNull();
    expect(el.children).toHaveLength(1);
    expect(el.children[0]).toMatchObject({ kind: "text", text: "hello" });
  });

  it("leaves no children behind for an empty string", () => {
    const el = element();
    const oldChild = element();
    el.children.push(oldChild);
    oldChild.parent = el;

    nodeOps.setElementText(el, "");

    expect(el.children).toHaveLength(0);
  });
});

describe("insert", () => {
  it("appends when no anchor is given", () => {
    const parent = element();
    const child = element();

    nodeOps.insert(child, parent);

    expect(gpjsUi.insertBefore).toHaveBeenCalledWith(parent.id, child.id, null);
    expect(parent.children).toEqual([child]);
    expect(child.parent).toBe(parent);
  });

  it("inserts before an existing anchor child", () => {
    const parent = element();
    const first = element();
    const anchor = element();
    nodeOps.insert(first, parent);
    nodeOps.insert(anchor, parent);
    vi.mocked(gpjsUi.insertBefore).mockClear();

    const inserted = element();
    nodeOps.insert(inserted, parent, anchor);

    expect(gpjsUi.insertBefore).toHaveBeenCalledWith(parent.id, inserted.id, anchor.id);
    expect(parent.children).toEqual([first, inserted, anchor]);
  });

  it("falls back to appending when the anchor isn't currently a child", () => {
    const parent = element();
    const strayAnchor = element();
    const child = element();

    nodeOps.insert(child, parent, strayAnchor);

    expect(gpjsUi.insertBefore).toHaveBeenCalledWith(parent.id, child.id, null);
    expect(parent.children).toEqual([child]);
  });

  it("moves a child already attached elsewhere, detaching it from its old parent first", () => {
    const oldParent = element();
    const newParent = element();
    const child = element();
    nodeOps.insert(child, oldParent);

    nodeOps.insert(child, newParent);

    expect(gpjsUi.removeChild).toHaveBeenCalledWith(oldParent.id, child.id);
    expect(gpjsUi.disposeNode).not.toHaveBeenCalled();
    expect(oldParent.children).toHaveLength(0);
    expect(newParent.children).toEqual([child]);
    expect(child.parent).toBe(newParent);
  });
});

describe("remove", () => {
  it("detaches from the parent and disposes the node's listeners", () => {
    const parent = element();
    const child = element();
    nodeOps.insert(child, parent);

    nodeOps.remove(child);

    expect(gpjsUi.removeChild).toHaveBeenCalledWith(parent.id, child.id);
    expect(gpjsUi.disposeNode).toHaveBeenCalledWith(child.id);
    expect(parent.children).toHaveLength(0);
    expect(child.parent).toBeNull();
  });

  it("is a no-op for a node with no parent", () => {
    const child = element();

    nodeOps.remove(child);

    expect(gpjsUi.removeChild).not.toHaveBeenCalled();
    expect(gpjsUi.disposeNode).not.toHaveBeenCalled();
  });
});

describe("parentNode / nextSibling", () => {
  it("read back what insert recorded", () => {
    const parent = element();
    const first = element();
    const second = element();
    nodeOps.insert(first, parent);
    nodeOps.insert(second, parent);

    expect(nodeOps.parentNode(first)).toBe(parent);
    expect(nodeOps.nextSibling(first)).toBe(second);
    expect(nodeOps.nextSibling(second)).toBeNull();
  });

  it("return null for a node with no parent", () => {
    const detached = element();

    expect(nodeOps.parentNode(detached)).toBeNull();
    expect(nodeOps.nextSibling(detached)).toBeNull();
  });
});
