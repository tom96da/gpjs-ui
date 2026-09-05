// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

export type NodeId = number;
export type CallbackId = number;

export type AttributeValue = string | number | boolean;

export type TagName = "text" | (string & {});

export interface StyleProps {
  display?: "flex" | "block" | "grid" | "none";
  flex_direction?: "row" | "column" | "row_reverse" | "column_reverse";
  justify_content?: "start" | "end" | "center" | "stretch";
  align_items?: "start" | "end" | "center" | "stretch";
  gap?: number;
  width?: number | "auto";
  height?: number | "auto";
  border_width?: number;
  background?: number | string;
  border_color?: number | string;
  text_color?: number | string;
  corner_radius?: number;
  text_size?: number;
}

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

export function createNode(tag: TagName): NodeId {
  return native().createNode(tag);
}

export function appendChild(parentId: NodeId, childId: NodeId): void {
  native().appendChild(parentId, childId);
}

export function insertBefore(parentId: NodeId, childId: NodeId, anchorId: NodeId | null): void {
  native().insertBefore(parentId, childId, anchorId);
}

export function removeChild(parentId: NodeId, childId: NodeId): void {
  native().removeChild(parentId, childId);
}

export function setAttribute(nodeId: NodeId, key: string, value: AttributeValue): void {
  native().setAttribute(nodeId, key, value);
}

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
