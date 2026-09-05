// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { addEventListener, setAttribute, setStyle } from "gpjs-ui";

import type { RendererOptions } from "@vue/runtime-core";

import type { EventListener } from "gpjs-ui";

import type { GpjsuiElement } from "./nodeOps.mts";

const isOn = (key: string): boolean => /^on[A-Z]/.test(key);

function patchStyle(el: GpjsuiElement, nextValue: unknown): void {
  if (typeof nextValue !== "object" || nextValue === null) return;

  for (const [key, value] of Object.entries(nextValue)) {
    // `setStyle` raises on a non-primitive value, the same way
    // `setAttribute` does — so a value shape it can't take (e.g. an array,
    // for DOM's multi-value CSS properties) is skipped rather than thrown.
    if (typeof value === "string" || typeof value === "number") {
      setStyle(el.id, key, value);
    }
  }
}

function patchEvent(el: GpjsuiElement, rawKey: string, nextValue: unknown): void {
  if (typeof nextValue !== "function") return;
  addEventListener(el.id, rawKey.slice(2).toLowerCase(), nextValue as EventListener);
}

/**
 * {@link RendererOptions.patchProp} — applies one changed `v-bind`/
 * attribute/event prop to a host element:
 *
 * - `style` (an object, per `:style="{...}"`) fans out to one
 *   {@link setStyle} call per entry; an entry whose value isn't a
 *   string/number is skipped.
 * - An `onXxx` key registers `nextValue` as an event listener for `xxx`, if
 *   it's a function — only `"click"` is wired to real input by the native
 *   host today, other event names are accepted but never fire.
 * - Everything else falls through to {@link setAttribute}, again skipping
 *   a non-string/number/boolean value rather than passing it through.
 *
 * There is no native "unset" call, so a prop that's removed entirely (a
 * `null`/`undefined` `nextValue`) is left as-is rather than cleared.
 */
export const patchProp: RendererOptions<unknown, GpjsuiElement>["patchProp"] = (
  el: GpjsuiElement,
  key: string,
  _prevValue: unknown,
  nextValue: unknown,
): void => {
  if (key === "style") {
    patchStyle(el, nextValue);
  } else if (isOn(key)) {
    patchEvent(el, key, nextValue);
  } else if (
    typeof nextValue === "string" ||
    typeof nextValue === "number" ||
    typeof nextValue === "boolean"
  ) {
    setAttribute(el.id, key, nextValue);
  }
};
