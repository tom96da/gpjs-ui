<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Host bridge (FFI) reference

The planned function surface exposed to JS as `globalThis.__gpjsui_native__`,
bound into the QuickJS context by the Rust host via `rquickjs`. This is a
target spec — see [docs/ROADMAP.md](./ROADMAP.md#phase-1-rust-host--ffi-bridge-core-gpjs-ui)
for when it lands; update this file as the real bindings are implemented.

It's called "FFI" for the calling-convention style (JS calling into Rust
functions with typed arguments), not a real C ABI or cross-process boundary —
JS and Rust share one process.

## Retained virtual tree

The Rust host keeps an in-memory, arena-allocated node structure
(`VirtualNode`) that mirrors the custom renderer's output. Each node has:

| Field | Type | Purpose |
| --- | --- | --- |
| `id` | `u32` | Stable handle returned to JS, used in all subsequent calls. |
| `tag_name` | `String` | Element kind, used to pick a GPUI element builder. |
| `style_props` | map | Layout/paint style properties. |
| `attributes` | map | Non-style attributes/props. |
| `children_ids` | `Vec<u32>` | Ordered child node handles. |

## Binding functions

| Function | Signature | Purpose |
| --- | --- | --- |
| `createNode` | `(tag: string) => number` | Allocate a `VirtualNode`, return its id. |
| `appendChild` | `(parentId: number, childId: number) => void` | Attach a child node. |
| `removeChild` | `(parentId: number, childId: number) => void` | Detach a child node. |
| `setAttribute` | `(nodeId: number, key: string, value: any) => void` | Set a style/attribute prop. |
| `addEventListener` | `(nodeId: number, event: string, callbackId: number) => void` | Register a JS callback for a native input event. |

On each GPUI `render()` frame cycle, the host recursively converts the
`VirtualNode` tree into GPUI `AnyElement` instances.
