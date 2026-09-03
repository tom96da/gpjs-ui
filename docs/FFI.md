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

### Tag vocabulary (v1)

Implemented by `crates/gpjs-ui/src/render/element.rs` (Unit v). Only two
kinds exist so far — there's no per-tag dispatch table yet, since there's
exactly one container builder to pick from until a real second element kind
is designed:

| `tag_name` | Maps to |
| --- | --- |
| `"text"` | A leaf. Content comes from the `"value"` string attribute (missing/non-string → empty content, never a panic). |
| anything else | A generic styled container (a GPUI `div()`). |

### Style prop vocabulary (v1)

Also implemented by `render/element.rs`. This is a deliberately small,
initial set — exactly what's needed to express `examples/gpui/hello_world.rs`'s
flex-box shapes and solid fills, not a full CSS surface. Unrecognized keys
and malformed enum-string values are silently ignored (forward-compatible,
never a panic) — this is a rendering path, not a JS call boundary, so
there's no channel to raise a catchable exception through.

| Key | Value | Maps to |
| --- | --- | --- |
| `display` | `"flex"`\|`"block"`\|`"grid"`\|`"none"` | `Style::display` |
| `flex_direction` | `"row"`\|`"column"`\|`"row_reverse"`\|`"column_reverse"` | `flex_direction` |
| `justify_content` / `align_items` | `"start"`\|`"end"`\|`"center"`\|`"stretch"` | resp. fields |
| `gap` | number (px) | `gap.width` & `gap.height` (uniform) |
| `width` / `height` | number (px) or `"auto"` | `size.width` / `size.height` |
| `border_width` | number (px) | all four `border_widths.*` (uniform) |
| `background` / `border_color` / `text_color` | number (hex `0xRRGGBB`) or string (`"#rrggbb"`/`"#rgb"`) | `Fill`/`Hsla` |
| `corner_radius` | number (px) | all four `corner_radii.*` (uniform) |
| `text_size` | number (px) | `text.font_size` |

`text_color`/`text_size` only apply to containers — text leaves cascade
their style from an ancestor container, exactly like GPUI's own
`.text_color()`/`.text_size()`; there's no separate per-leaf text styling.

Deliberately deferred, not yet implemented: percentage lengths, min/max
size, margin/padding, flex-grow/shrink/basis, per-side border/corner
values, box-shadow, `border_style` (solid vs. dashed — GPUI's own
`Style::border_style`, distinct from `border_width`/`border_color`; unset
always renders solid, GPUI's default), and additional align/justify
variants beyond the four above.

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

`addEventListener`'s dispatch (Unit vi) is expected to need no thread-safe/
cross-thread callback machinery: gpjs-ui's embedded QuickJS and the GPUI
event loop already share one process and are driven synchronously (see
`crates/gpjs-ui/src/js/engine.rs`'s `Context::with`), unlike an architecture
where JS runs in a separate runtime that loads a native addon (JS and the
native UI layer on different threads/processes). A GPUI event handler
closure should be able to call directly into the registered
`rquickjs::Function` in place.
