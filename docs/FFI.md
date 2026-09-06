<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Host bridge (FFI) reference

The function surface exposed to JS as `globalThis.__gpjsui_native__`, bound
into the QuickJS context by the Rust host via `rquickjs`. Phase 1's core
bindings and Phase 2's `setStyle`/`insertBefore` additions are implemented
and tested (see [AGENTS.md](../AGENTS.md#status) and
[docs/ROADMAP.md](./ROADMAP.md#phase-1-rust-host--ffi-bridge-core-gpjs-ui));
the tag/style vocabulary below is deliberately incomplete by design and grows
as real usage needs more of it — update this file whenever a binding, tag, or
style prop actually lands.

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
| `rootNodeId` | `() => number` | Return the host-allocated root container's id — the node a mounting app attaches itself under. Allocated with the tree, so it always resolves. |
| `createNode` | `(tag: string) => number` | Allocate a `VirtualNode`, return its id. |
| `appendChild` | `(parentId: number, childId: number) => void` | Attach a child node at the end of `parentId`'s children. Thin wrapper over `insertBefore` with no anchor. |
| `insertBefore` | `(parentId: number, childId: number, anchorId: number \| null) => void` | Attach a child node before `anchorId` (or at the end if `null`). If `anchorId` names a real node that isn't currently a child of `parentId`, falls back to appending at the end; only a wholly unknown `anchorId` throws. |
| `removeChild` | `(parentId: number, childId: number) => void` | Detach a child node. |
| `setAttribute` | `(nodeId: number, key: string, value: any) => void` | Set a non-style attribute prop. |
| `setStyle` | `(nodeId: number, key: string, value: any) => void` | Set a style prop — the only JS-reachable way to touch `style_props`; `setAttribute` writes to the separate `attributes` map instead. |
| `addEventListener` | `(nodeId: number, event: string, callbackId: number) => void` | Register a JS callback for a native input event. |

On each GPUI `render()` frame cycle, the host recursively converts the
`VirtualNode` tree into GPUI `AnyElement` instances.

### Event dispatch (v1: `"click"` only)

Implemented by `crates/gpjs-ui/src/render/bridge.rs` (Unit vi):
`render_tree_with_events`/`build_element_with_events`
(`crates/gpjs-ui/src/render/element.rs`) wire every container's click to an
`EventDispatcher`, which looks up and calls the JS callbacks registered
for `(nodeId, "click")` via `addEventListener`, then requests a redraw.
Other event names aren't wired to any real GPUI input yet — extend as
needed, same "deliberately incomplete" framing as the style vocabulary.

`addEventListener` itself is unchanged and needs no thread-safe/cross-thread
callback machinery: gpjs-ui's embedded QuickJS and the GPUI event loop
already share one process and are driven synchronously (see
`crates/gpjs-ui/src/js/engine.rs`'s `Context::with`), unlike an architecture
where JS runs in a separate runtime that loads a native addon (JS and the
native UI layer on different threads/processes) — confirmed, not just
assumed, by `crates/gpjs-ui/tests/event_dispatch.rs`.

`EventListeners` only ever stores the plain `u32` `callbackId` it's given —
never an `rquickjs::Value`/`Function`/`Persistent<T>`, per the FFI safety
checklist below. The real JS function has to live somewhere, so the
convention is: **the caller stores it itself**, at
`globalThis.__gpjsui_callbacks__[callbackId]`, before calling
`addEventListener` with that id. `EventDispatcher::dispatch` looks the real
function up fresh inside one `Engine::with` call and drops it before that
call returns — it never crosses into a long-lived Rust struct. A missing
`__gpjsui_callbacks__`/callback entry, a non-function entry, or an
exception thrown by the callback are all silently skipped rather than
propagated: a bad listener must not take down the host.
