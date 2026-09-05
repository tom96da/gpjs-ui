// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

/** Stable handle to a node in the native retained tree, returned by {@link createNode} and used in every later call that touches that node. */
export type NodeId = number;

/** Id assigned to one registered event listener. Managed internally by {@link addEventListener}/{@link disposeNode} — callers never see or pass one directly. */
export type CallbackId = number;

/** A value {@link setAttribute}/{@link setStyle} can take. Anything else raises a catchable exception rather than being silently coerced. */
export type AttributeValue = string | number | boolean;

/**
 * Element kind passed to {@link createNode}. `"text"` is the only tag with
 * dedicated rendering behavior — a leaf that renders its `"value"`
 * attribute's string content. Any other string is a generic styled
 * container; the `string & {}` half of this type keeps `"text"`'s
 * autocomplete while still accepting an arbitrary tag name.
 */
export type TagName = "text" | (string & {});

/**
 * Style properties recognized by {@link setStyle}'s typed overload. A
 * deliberately incomplete, v1 layout/paint vocabulary — an unrecognized key
 * or a value shape that doesn't parse (e.g. a malformed enum string) is
 * ignored by the renderer rather than applied or thrown.
 */
export interface StyleProps {
  /** Layout mode for this node's children. `"none"` skips both layout and rendering for the whole subtree. */
  display?: "flex" | "block" | "grid" | "none";
  /** Main-axis direction for a `"flex"` container. */
  flex_direction?: "row" | "column" | "row_reverse" | "column_reverse";
  /** Main-axis alignment of children within this container. */
  justify_content?: "start" | "end" | "center" | "stretch";
  /** Cross-axis alignment of children within this container. */
  align_items?: "start" | "end" | "center" | "stretch";
  /** Uniform spacing, in px, between children — applied on both axes. */
  gap?: number;
  /** Fixed width in px, or `"auto"` to size to content. */
  width?: number | "auto";
  /** Fixed height in px, or `"auto"` to size to content. */
  height?: number | "auto";
  /** Uniform border width in px, applied to all four sides. */
  border_width?: number;
  /** Fill color, as a hex number (`0xRRGGBB`) or a CSS-style string (`"#rrggbb"`/`"#rgb"`). */
  background?: number | string;
  /** Border color. Accepts the same formats as {@link StyleProps.background}. */
  border_color?: number | string;
  /** Color for this container's own text. Cascades to descendant text leaves, same as {@link StyleProps.text_size}; there's no separate per-leaf text styling. */
  text_color?: number | string;
  /** Uniform corner radius in px, applied to all four corners. */
  corner_radius?: number;
  /** Font size in px for this container's own text. Cascades to descendant text leaves, same as {@link StyleProps.text_color}. */
  text_size?: number;
}

/**
 * Signature of a callback registered via {@link addEventListener}. The
 * native host currently always calls it with exactly one argument, the
 * {@link NodeId} the event fired on — the type stays variadic so a future
 * event kind can add a richer payload without a breaking signature change.
 */
export type EventListener = (...args: unknown[]) => void;

interface GpjsuiNative {
  createNode(tag: string): NodeId;
  appendChild(parentId: NodeId, childId: NodeId): void;
  insertBefore(parentId: NodeId, childId: NodeId, anchorId: NodeId | null): void;
  removeChild(parentId: NodeId, childId: NodeId): void;
  setAttribute(nodeId: NodeId, key: string, value: AttributeValue): void;
  setStyle(nodeId: NodeId, key: string, value: AttributeValue): void;
  addEventListener(nodeId: NodeId, event: string, callbackId: CallbackId): void;
}

declare global {
  // eslint-disable-next-line no-var
  var __gpjsui_native__: GpjsuiNative;
  // eslint-disable-next-line no-var
  var __gpjsui_callbacks__: Record<CallbackId, EventListener>;
}

function native(): GpjsuiNative {
  return globalThis.__gpjsui_native__;
}

/**
 * Allocates a new node of kind `tag` in the native tree. The node starts out
 * detached — attach it with {@link appendChild}/{@link insertBefore}.
 * @param tag - the node's element kind
 * @returns the new node's id
 */
export function createNode(tag: TagName): NodeId {
  return native().createNode(tag);
}

/**
 * Attaches `childId` as the last child of `parentId`. A thin wrapper over
 * {@link insertBefore} with no anchor.
 * @param parentId - the container to attach to
 * @param childId - the node being attached
 */
export function appendChild(parentId: NodeId, childId: NodeId): void {
  native().appendChild(parentId, childId);
}

/**
 * Attaches `childId` as a child of `parentId`, positioned before `anchorId`
 * (or at the end, if `anchorId` is `null`). If `anchorId` names a real node
 * that just isn't currently a child of `parentId`, this falls back to
 * appending at the end instead of throwing — only a wholly unknown
 * `anchorId` raises.
 * @param parentId - the container to attach to
 * @param childId - the node being attached
 * @param anchorId - the sibling to insert before, or `null` to append at the end
 */
export function insertBefore(parentId: NodeId, childId: NodeId, anchorId: NodeId | null): void {
  native().insertBefore(parentId, childId, anchorId);
}

/**
 * Detaches `childId` from `parentId`. The detached node stays alive (and
 * keeps its own children) — it isn't freed, so it can be re-attached
 * elsewhere.
 * @param parentId - the current container
 * @param childId - the node being detached
 */
export function removeChild(parentId: NodeId, childId: NodeId): void {
  native().removeChild(parentId, childId);
}

/**
 * Sets a non-style attribute/prop on `nodeId`. See {@link setStyle} for
 * layout/paint properties, which live in a separate map.
 * @param nodeId - the node to update
 * @param key - the attribute name
 * @param value - the attribute value
 */
export function setAttribute(nodeId: NodeId, key: string, value: AttributeValue): void {
  native().setAttribute(nodeId, key, value);
}

/**
 * Sets a style property on `nodeId` — the only way to reach {@link setAttribute}'s
 * counterpart style map. A key from {@link StyleProps} gets compile-time
 * checking on its value's shape; any other string key still forwards as a
 * raw {@link AttributeValue}, for style properties this package's types
 * haven't caught up with yet.
 * @param nodeId - the node to update
 * @param key - the style property name
 * @param value - the style property value
 */
export function setStyle<K extends keyof StyleProps>(
  nodeId: NodeId,
  key: K,
  value: NonNullable<StyleProps[K]>,
): void;
export function setStyle(nodeId: NodeId, key: string, value: AttributeValue): void;
export function setStyle(nodeId: NodeId, key: string, value: AttributeValue): void {
  native().setStyle(nodeId, key, value);
}

// `addEventListener` owns the `__gpjsui_callbacks__[id]` convention so
// callers never touch that raw contract themselves. Re-registering the same
// (nodeId, event) frees its previous callback id rather than leaking it — the
// native side has no `removeEventListener`, so a stale id left in
// `EventListeners` is harmless once its JS-side entry is gone (dispatch just
// finds nothing and silently skips it).
let nextCallbackId = 0;
const registeredCallbackIds = new Map<string, CallbackId>();

function callbackRegistry(): Record<CallbackId, EventListener> {
  return (globalThis.__gpjsui_callbacks__ ??= {});
}

/**
 * Registers `listener` to run whenever `event` fires on `nodeId`.
 * Re-registering the same `(nodeId, event)` pair replaces the previous
 * listener rather than adding a second one. Only `"click"` is wired to a
 * real native input event today; other event names are accepted but never
 * fire.
 * @param nodeId - the node to listen on
 * @param event - the event name (e.g. `"click"`)
 * @param listener - called when the event fires
 */
export function addEventListener(nodeId: NodeId, event: string, listener: EventListener): void {
  const key = `${nodeId}:${event}`;
  const previousId = registeredCallbackIds.get(key);
  if (previousId !== undefined) {
    delete callbackRegistry()[previousId];
  }

  const callbackId = nextCallbackId++;
  callbackRegistry()[callbackId] = listener;
  registeredCallbackIds.set(key, callbackId);
  native().addEventListener(nodeId, event, callbackId);
}

/**
 * Frees every callback id registered for `nodeId`, across all event names —
 * the counterpart to a node going away for good. There is no native "node
 * removed" hook, so callers must invoke this themselves once they know a
 * node won't come back (a framework adapter's element-removal path, for
 * example); it is otherwise never called automatically.
 * @param nodeId - the node that's gone for good
 */
export function disposeNode(nodeId: NodeId): void {
  const prefix = `${nodeId}:`;
  for (const key of registeredCallbackIds.keys()) {
    if (!key.startsWith(prefix)) continue;
    const callbackId = registeredCallbackIds.get(key);
    if (callbackId !== undefined) {
      delete callbackRegistry()[callbackId];
    }
    registeredCallbackIds.delete(key);
  }
}
