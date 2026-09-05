// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

import { createRenderer } from "@vue/runtime-core";

import { nodeOps } from "./nodeOps.mts";
import { patchProp } from "./patchProp.mts";

import type { CreateAppFunction } from "@vue/runtime-core";

import type { GpjsuiElement, GpjsuiNode } from "./nodeOps.mts";

const renderer = createRenderer<GpjsuiNode, GpjsuiElement>({ ...nodeOps, patchProp });

/**
 * Creates a Vue app whose root mounts against a {@link GpjsuiElement} host
 * handle instead of a DOM element. A thin, type-fixed convenience wrapper
 * around `@vue/runtime-core`'s `createRenderer(nodeOps).createApp` —
 * `app.mount`, `app.unmount`, `app.use`, etc. all behave exactly as
 * `@vue/runtime-core` itself documents them.
 */
export const createGpjsuiApp: CreateAppFunction<GpjsuiElement> = renderer.createApp;
