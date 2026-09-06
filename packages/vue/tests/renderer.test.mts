// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { h, nextTick, reactive } from "@vue/runtime-core";
import { beforeEach, describe, expect, it } from "vitest";

import { createNode, rootNodeId } from "gpjs-ui";

import { createGpjsuiApp } from "../src/index.mts";

import type { NodeId } from "gpjs-ui";

import type { GpjsuiElement } from "../src/index.mts";

interface FakeNode {
  tag: string;
  attributes: Record<string, unknown>;
  style: Record<string, unknown>;
  children: NodeId[];
  listeners: Record<string, number>;
}

// A minimal, in-memory stand-in for the native host's retained tree — real
// enough to drive `gpjs-ui`'s actual wrapper functions and `@gpjs-ui/vue`'s
// actual `nodeOps`/`patchProp` end to end, without a real Rust process.
function installFakeNative(): Map<NodeId, FakeNode> {
  const nodes = new Map<NodeId, FakeNode>();
  let nextId = 0;

  function requireNode(id: NodeId): FakeNode {
    const node = nodes.get(id);
    if (!node) throw new Error(`unknown node id: ${id}`);
    return node;
  }

  function allocate(tag: string): NodeId {
    const id = nextId++;
    nodes.set(id, { tag, attributes: {}, style: {}, children: [], listeners: {} });
    return id;
  }

  // The host allocates its root along with the tree, before any JS runs.
  const rootId = allocate("div");

  globalThis.__gpjsui_native__ = {
    rootNodeId(): NodeId {
      return rootId;
    },
    createNode: allocate,
    appendChild(parentId: NodeId, childId: NodeId): void {
      globalThis.__gpjsui_native__.insertBefore(parentId, childId, null);
    },
    insertBefore(parentId: NodeId, childId: NodeId, anchorId: NodeId | null): void {
      const parent = requireNode(parentId);
      requireNode(childId);
      const anchorIndex = anchorId === null ? -1 : parent.children.indexOf(anchorId);
      if (anchorIndex === -1) {
        parent.children.push(childId);
      } else {
        parent.children.splice(anchorIndex, 0, childId);
      }
    },
    removeChild(parentId: NodeId, childId: NodeId): void {
      const parent = requireNode(parentId);
      const index = parent.children.indexOf(childId);
      if (index !== -1) parent.children.splice(index, 1);
    },
    setAttribute(nodeId: NodeId, key: string, value: unknown): void {
      requireNode(nodeId).attributes[key] = value;
    },
    setStyle(nodeId: NodeId, key: string, value: unknown): void {
      requireNode(nodeId).style[key] = value;
    },
    addEventListener(nodeId: NodeId, event: string, callbackId: number): void {
      requireNode(nodeId).listeners[event] = callbackId;
    },
  };

  return nodes;
}

function dispatch(node: FakeNode, event: string): void {
  const callbackId = node.listeners[event];
  if (callbackId === undefined) throw new Error(`no ${event} listener registered`);
  const callback = globalThis.__gpjsui_callbacks__[callbackId];
  if (callback === undefined) throw new Error(`no callback registered as ${callbackId}`);
  callback();
}

describe("@gpjs-ui/vue renderer, driven end to end through a real gpjs-ui core", () => {
  let nodes: Map<NodeId, FakeNode>;
  let root: GpjsuiElement;

  beforeEach(() => {
    nodes = installFakeNative();
    delete (globalThis as { __gpjsui_callbacks__?: unknown }).__gpjsui_callbacks__;
    root = { id: createNode("root"), kind: "element", parent: null, children: [] };
  });

  it("mounts styles, attributes, and text, then reacts to a click", async () => {
    const state = reactive({ count: 0 });
    const App = {
      setup() {
        return () =>
          h(
            "div",
            {
              style: { background: "#000" },
              class: "box",
              onClick: () => state.count++,
            },
            `count: ${state.count}`,
          );
      },
    };

    createGpjsuiApp(App).mount(root);
    await nextTick();

    const rootNode = nodes.get(root.id)!;
    expect(rootNode.children).toHaveLength(1);
    const div = nodes.get(rootNode.children[0]!)!;
    expect(div.style).toEqual({ background: "#000" });
    expect(div.attributes["class"]).toBe("box");
    expect(div.children).toHaveLength(1);
    const text = nodes.get(div.children[0]!)!;
    expect(text.attributes["value"]).toBe("count: 0");

    dispatch(div, "click");
    await nextTick();

    expect(nodes.get(div.children[0]!)!.attributes["value"]).toBe("count: 1");
  });

  it("reorders a keyed list via moves, keeping each item's node identity", async () => {
    const state = reactive({ items: [1, 2, 3] });
    const App = {
      setup() {
        return () =>
          h(
            "div",
            null,
            state.items.map((n) => h("div", { key: n, class: `item-${n}` })),
          );
      },
    };

    createGpjsuiApp(App).mount(root);
    await nextTick();

    const containerId = nodes.get(root.id)!.children[0]!;
    const originalOrder = nodes.get(containerId)!.children.slice();
    expect(originalOrder).toHaveLength(3);

    state.items = [3, 1, 2];
    await nextTick();

    const reordered = nodes.get(containerId)!.children;
    expect(reordered).toEqual([originalOrder[2], originalOrder[0], originalOrder[1]]);
  });

  it("mounts against the host's root container when mount gets no argument", async () => {
    const App = {
      setup() {
        return () => h("div", { class: "box" });
      },
    };

    createGpjsuiApp(App).mount();
    await nextTick();

    const hostRoot = nodes.get(rootNodeId())!;
    expect(hostRoot).not.toBe(nodes.get(root.id));
    expect(hostRoot.children).toHaveLength(1);
    expect(nodes.get(hostRoot.children[0]!)!.attributes["class"]).toBe("box");
  });
});
