// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import {
  appendChild,
  createNode,
  disposeNode,
  insertBefore,
  removeChild,
  setAttribute,
  setStyle,
} from "gpjs-ui";

import type { RendererOptions } from "@vue/runtime-core";

import type { NodeId, TagName } from "gpjs-ui";

/**
 * A container host node: either a real element or the hidden stand-in
 * {@link nodeOps.createComment} uses. Also `@vue/runtime-core`'s
 * `HostElement` — the only host node type that can act as a parent.
 *
 * `gpjs-ui`'s native tree has no parent pointers and no way to list a
 * node's children, so `parent`/`children` are maintained here as a
 * JS-side shadow of the tree, kept in sync by {@link nodeOps.insert}/
 * {@link nodeOps.remove}.
 */
export interface GpjsuiElement {
  /** The underlying native node's id — the only part of this object the native tree itself knows about. */
  readonly id: NodeId;
  /** `"comment"` for the hidden stand-in {@link nodeOps.createComment} produces; `"element"` for every other container. Purely informational — both render identically once created. */
  readonly kind: "element" | "comment";
  /** The current parent, or `null` if this node isn't attached to the tree. */
  parent: GpjsuiElement | null;
  /** This node's children, in render order. */
  children: GpjsuiNode[];
}

/** A text leaf host node — `@vue/runtime-core`'s `HostNode` for text. */
export interface GpjsuiText {
  /** The underlying native node's id. */
  readonly id: NodeId;
  /** Discriminates this node within the {@link GpjsuiNode} union. */
  readonly kind: "text";
  /** The current parent, or `null` if this node isn't attached to the tree. */
  parent: GpjsuiElement | null;
  /** The text content, mirroring what's been pushed to the native node via {@link setAttribute}. */
  text: string;
}

/** Any host node {@link nodeOps} can produce: an element, comment, or text leaf. */
export type GpjsuiNode = GpjsuiElement | GpjsuiText;

function createTextNode(text: string): GpjsuiText {
  const id = createNode("text");
  setAttribute(id, "value", text);
  return { id, kind: "text", parent: null, text };
}

// Detaches `child` from its current parent, in both the native tree and this
// module's parent/children bookkeeping, without disposing its event
// listeners — used by `insert` (a move keeps the node alive) as opposed to
// `remove` (a permanent removal, which does dispose them).
function detach(child: GpjsuiNode): void {
  const parent = child.parent;
  if (!parent) return;

  removeChild(parent.id, child.id);
  const index = parent.children.indexOf(child);
  if (index !== -1) parent.children.splice(index, 1);
  child.parent = null;
}

/**
 * `@vue/runtime-core`'s {@link RendererOptions}`<GpjsuiNode, GpjsuiElement>`,
 * minus `patchProp` (see `./patchProp.mts`) — the host-node lifecycle half
 * of `@gpjs-ui/vue`'s custom renderer, built entirely on the `gpjs-ui` core
 * package's typed wrapper functions, never on the native bridge directly.
 */
export const nodeOps: Omit<RendererOptions<GpjsuiNode, GpjsuiElement>, "patchProp"> = {
  /**
   * Allocates a new element node for `tag`. Vue's other `createElement`
   * parameters (namespace, `is`, initial props) don't apply here — GPUI
   * has no SVG/MathML/custom-element concept — and are ignored.
   * @param tag - the node's element kind
   * @returns the new, parentless, childless element
   */
  createElement(tag: TagName): GpjsuiElement {
    return { id: createNode(tag), kind: "element", parent: null, children: [] };
  },

  /**
   * Creates a text leaf.
   * @param text - the initial text content
   * @returns the new text leaf
   */
  createText: createTextNode,

  // gpjs-ui has no comment concept — a hidden, childless container stands
  // in for one. The comment's own text carries no rendered meaning here and
  // is discarded, same as the hidden node itself.
  /**
   * Creates the hidden container that stands in for a Vue comment node
   * (e.g. a `v-if`/`v-for` placeholder).
   * @param _text - the comment's text; accepted for interface compatibility, but discarded since nothing renders it
   * @returns the new hidden element
   */
  createComment(_text: string): GpjsuiElement {
    const id = createNode("div");
    setStyle(id, "display", "none");
    return { id, kind: "comment", parent: null, children: [] };
  },

  /**
   * Updates a text leaf's content in place.
   * @param node - the text leaf to update
   * @param text - the new text content
   */
  setText(node: GpjsuiText, text: string): void {
    setAttribute(node.id, "value", text);
    node.text = text;
  },

  /**
   * Replaces all of `el`'s children with, at most, a single text child
   * holding `text` (or none, for an empty string) — the fast path Vue takes
   * for an element whose only dynamic content is its own text.
   * @param el - the container whose children are replaced
   * @param text - the new text content
   */
  setElementText(el: GpjsuiElement, text: string): void {
    for (const child of el.children.splice(0)) {
      removeChild(el.id, child.id);
      disposeNode(child.id);
      child.parent = null;
    }
    if (!text) return;

    const textNode = createTextNode(text);
    appendChild(el.id, textNode.id);
    textNode.parent = el;
    el.children.push(textNode);
  },

  /**
   * Attaches `child` to `parent`, before `anchor` (or at the end, if
   * `anchor` is omitted/`null`). If `child` is already attached elsewhere,
   * it's detached first — this is how Vue moves a node during a keyed-list
   * reorder, so the node is never disposed or recreated for a move.
   * @param child - the node being attached
   * @param parent - the new container
   * @param anchor - the sibling to insert before, or omitted/`null` to append at the end
   */
  insert(child: GpjsuiNode, parent: GpjsuiElement, anchor?: GpjsuiNode | null): void {
    detach(child);

    const anchorIndex = anchor ? parent.children.indexOf(anchor) : -1;
    insertBefore(parent.id, child.id, anchor && anchorIndex !== -1 ? anchor.id : null);
    if (anchorIndex === -1) {
      parent.children.push(child);
    } else {
      parent.children.splice(anchorIndex, 0, child);
    }
    child.parent = parent;
  },

  /**
   * Detaches `child` from its parent for good and frees its registered
   * event listeners. A no-op if `child` has no parent (already removed, or
   * never attached).
   * @param child - the node being removed
   */
  remove(child: GpjsuiNode): void {
    if (!child.parent) return;

    detach(child);
    disposeNode(child.id);
  },

  /**
   * Looks up the current parent.
   * @param node - the node to query
   * @returns the current parent, or `null` if it isn't attached
   */
  parentNode(node: GpjsuiNode): GpjsuiElement | null {
    return node.parent;
  },

  /**
   * Looks up the next sibling.
   * @param node - the node to query
   * @returns the sibling immediately after `node` in its parent's children, or `null` if there is none (or `node` isn't attached)
   */
  nextSibling(node: GpjsuiNode): GpjsuiNode | null {
    const parent = node.parent;
    if (!parent) return null;

    const index = parent.children.indexOf(node);
    return parent.children[index + 1] ?? null;
  },
};
