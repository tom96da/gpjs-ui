// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { native } from "./native.mts";

import type { CallbackId, EventListener, NodeId } from "./types.mts";

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
