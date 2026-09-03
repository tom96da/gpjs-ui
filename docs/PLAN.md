<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Implementation Plan

Granular, checkbox-tracked task breakdown that complements
[docs/ROADMAP.md](./ROADMAP.md)'s phase-level design intent with per-task
tracking. Sections are **append-only**: a finished phase's section is never
rewritten or deleted, only its checkboxes are ticked. New scope discovered
mid-implementation is appended as a new item near the relevant task, not
folded into an existing one's wording. This lets progress be read straight
off the checkboxes, with no need to keep prose in sync with reality.

## Phase 1: Rust host & FFI bridge core (`gpjs-ui`)

See [docs/ROADMAP.md#phase-1](./ROADMAP.md#phase-1-rust-host--ffi-bridge-core-gpjs-ui)
and [docs/FFI.md](./FFI.md) for the design this implements.

### Prerequisites

- [x] Add native build dependencies for `gpui` to the devcontainer

### Rust workspace

- [x] Root `Cargo.toml` workspace scaffold (`gpui` as a git dependency)
- [x] `crates/gpjs-ui/Cargo.toml`

### Phase 2 landing spot (scaffolding only, no renderer logic yet)

- [ ] `pnpm-workspace.yaml` + root `package.json`
- [ ] `packages/vue` placeholder package (npm name `@gpjs-ui/vue`)
- [ ] `packages/gpjs-ui` placeholder package (framework-agnostic; deferred — scaffold when Phase 2 implementation begins)
- [ ] `examples/hello-vue` placeholder package (deferred — needs `@gpjs-ui/vue` to have real content first)

### Unit i — VirtualNode arena

- [x] Implement `crates/gpjs-ui/src/tree.rs`
- [x] Unit tests: id uniqueness, append order, detach-keeps-node-alive,
      remove-of-absent-child is a no-op, set-attribute overwrites,
      unknown-id lookup returns `None` rather than panicking

### Unit ii — QuickJS runtime bootstrap

- [x] Implement `crates/gpjs-ui/src/js/engine.rs`
- [x] Tests: `ctx.eval` smoke test, syntax-error propagation,
      two engines don't share globals

### Unit iii — `__gpjsui_native__` bindings

- [x] Implement `crates/gpjs-ui/src/js/bindings.rs`
- [x] Tests driven through `ctx.eval` (happy path, type round-trips,
      bad id/type raises a catchable JS exception, not a Rust panic)

### Unit iv — minimal gpui window

- [x] `crates/gpjs-ui/examples/gpui/hello_world.rs`
- [ ] `#[gpui::test]` headless smoke test — tried, then removed: its
      `TestPlatform` doesn't exercise real rendering or fonts, so it can't
      catch the class of bug this unit cares about. `cargo check -p
      gpjs-ui --all-targets` compile-checks the example instead.
- [ ] Manual: run the example and look at the window — confirmed on macOS
      only so far; see [docs/MANUAL_GUI_CHECK.md](./MANUAL_GUI_CHECK.md).

### Unit v — VirtualNode → AnyElement conversion

- [x] `crates/gpjs-ui/src/render/element.rs` (pure spec layer + thin gpui layer)
- [x] Tests for both layers
- [x] Test: for each of gpui's own examples (starting with hello_world),
      compare computed layout (not pixels) between the upstream version
      and the same layout built via gpjs-ui's `VirtualTree` through JS —
      should work headlessly, since layout computation alone doesn't need
      real rendering/fonts (unlike Unit iv's removed test) — landed as
      `crates/gpjs-ui/tests/layout_parity.rs`, reproducing hello_world's
      shape (not a literal copy: that's a binary example, not importable);
      only hello_world's shape is covered so far, not gpui's other examples
- [x] Manual: a new example rendering a real `VirtualTree` through gpjs-ui —
      `crates/gpjs-ui/examples/hello_world.rs`; confirmed on macOS by
      comparing side-by-side against `examples/gpui/hello_world.rs`. Layout,
      colors, and text all matched; the six squares' dashed borders didn't —
      GPUI's `border_style` (solid vs. dashed) isn't in the v1 style
      vocabulary yet, so it always renders solid
- [ ] The v1 style vocabulary (`docs/FFI.md`) is deliberately incomplete:
      percentage lengths, min/max size, margin/padding,
      flex-grow/shrink/basis, per-side border/corner values, box-shadow, and
      align/justify variants beyond start/end/center/stretch are not yet
      implemented — add as real usage needs them

### Unit vi — event-driven JS invocation (zero-overhead render loop)

- [x] `crates/gpjs-ui/src/render/bridge.rs` — `EventDispatcher`, dispatching
      `"click"` only so far (see docs/FFI.md's "Event dispatch" section);
      other event names aren't wired to real GPUI input yet
- [x] Give every container a real GPUI `ElementId` (e.g.
      `ElementId::Integer(node_id as u64)`, reusing our existing stable
      `NodeId`) before wiring up any interactivity. `build_element`
      currently assigns none — harmless today (Unit v has no interactive
      state), but a container with no id loses GPUI's
      `InteractiveElementState` (hover/active/focus/pointer-capture) across
      re-renders, since GPUI keys that state off the element's
      `GlobalElementId`. (Cross-checked against a competing GPUI-based
      framework that hit exactly this bug.) Confirmed while implementing:
      `on_click` itself doesn't exist without `.id(...)` first — this
      wasn't just about state persistence, `on_click` is only reachable via
      `StatefulInteractiveElement`.
- [x] Test: one simulated input event triggers exactly one JS call — landed
      as `crates/gpjs-ui/tests/event_dispatch.rs`'s
      `click_dispatches_to_js_exactly_once`. Dropped the "and one
      `cx.notify()`" half of this item: confirmed empirically that
      `window.refresh()` (not `cx.notify()`/`Context<V>` — no entity
      plumbing needed through `render_tree`/`build_element` at all) is what
      `dispatch()` calls, but also confirmed a click on *any* interactive
      element already makes GPUI redraw for its own hover/active
      bookkeeping regardless — so a redraw-count assertion can't actually
      isolate this call's effect, and isn't included in the test.
- [x] Test: repeated renders with no new events touch the JS engine zero
      times — `event_dispatch.rs`'s
      `repeated_builds_with_no_event_never_touch_the_js_engine` (a plain
      `#[test]`: building an `AnyElement` needs no `gpui` App/Window at
      all, confirmed while implementing)
- [x] Manual: clickable element mutates state via JS and re-renders — landed
      as a new example, `examples/click_counter.rs` (not folded into
      `examples/hello_world.rs`: that one's whole point is being a faithful,
      non-interactive recreation of gpui's own static example, so bolting
      interactivity onto it would undermine what its name/doc comment
      promise); confirmed on macOS — clicking the box counts up

### Docs

- [x] Update `docs/STRUCTURE.md` and `AGENTS.md`'s Status section to match
      what actually landed

## Evergreen checklists

Re-apply these on every relevant future PR — they are not phase-scoped and
never get "checked off" permanently.

### FFI safety review (any PR touching `crates/gpjs-ui/src/js/*`)

- [ ] No `rquickjs::Value`/`Ctx<'js>`/`Persistent<T>` is stored in
      `VirtualTree`, the event-listener registry, or any other long-lived
      struct
- [ ] Every JS argument is converted to an owned plain Rust type before
      touching shared state
- [ ] Non-primitive values at typed boundaries (e.g. `setAttribute`) raise a
      catchable JS exception, not a panic or silent coercion
- [ ] Unknown/stale ids from JS are handled as `None`/a JS exception, never
      a Rust panic
