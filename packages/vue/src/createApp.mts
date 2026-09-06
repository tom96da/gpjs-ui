// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { createRenderer } from "@vue/runtime-core";

import { rootNodeId } from "gpjs-ui";

import { nodeOps } from "./nodeOps.mts";
import { patchProp } from "./patchProp.mts";

import type { App, Component, ComponentPublicInstance } from "@vue/runtime-core";

import type { GpjsuiElement, GpjsuiNode } from "./nodeOps.mts";

const renderer = createRenderer<GpjsuiNode, GpjsuiElement>({ ...nodeOps, patchProp });

/**
 * `@vue/runtime-core`'s `App`, with `mount` additionally callable with no
 * argument to target the host's own root container. Every other method is
 * `App`'s unchanged, including the `this`-returning ones — `use`, `mixin`,
 * `component` and `directive` all keep this wider `mount` when chained.
 */
export type GpjsuiApp = App<GpjsuiElement> & {
  mount: (rootContainer?: GpjsuiElement) => ComponentPublicInstance;
};

function hostRootElement(): GpjsuiElement {
  return { id: rootNodeId(), kind: "element", parent: null, children: [] };
}

/**
 * Creates a Vue app whose root mounts against a {@link GpjsuiElement} host
 * handle instead of a DOM element. `app.mount()` with no argument targets
 * the host's root container; `app.unmount`, `app.use`, etc. all behave
 * exactly as `@vue/runtime-core` itself documents them.
 * @param rootComponent - the component to mount as the app's root
 * @param rootProps - props to pass to that root component
 * @returns the created app
 */
export function createGpjsuiApp(
  rootComponent: Component,
  rootProps?: Record<string, unknown> | null,
): GpjsuiApp {
  const app = renderer.createApp(rootComponent, rootProps);
  // Captured before the override, or the replacement would call itself.
  const mountAt = app.mount.bind(app);
  return Object.assign(app, {
    mount: (rootContainer?: GpjsuiElement) => mountAt(rootContainer ?? hostRootElement()),
  });
}
