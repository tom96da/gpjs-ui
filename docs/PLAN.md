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

- [x] `pnpm-workspace.yaml` + root `package.json`
- [x] `packages/vue` placeholder package (npm name `@gpjs-ui/vue`)
- [x] `packages/gpjs-ui` placeholder package (framework-agnostic; deferred — scaffold when Phase 2 implementation begins)
- [ ] `examples/hello-vue` placeholder package (deferred — needs `@gpjs-ui/vue` to have real content first) — lands as part of Phase 2 Unit iv

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

## Phase 2: JS core bridge (`gpjs-ui`) & Vue 3 custom renderer (`@gpjs-ui/vue`)

See [docs/ROADMAP.md#phase-2](./ROADMAP.md#phase-2-js-core-bridge-gpjs-ui--vue-3-custom-renderer-gpjs-uivue)
and [docs/FFI.md](./FFI.md) for the design this implements.

Scope grew beyond ROADMAP.md's four bullet points once planning dug into
the actual `@vue/runtime-core` `createRenderer` API surface: the native
bridge has two real gaps that block a working custom renderer, not just
missing polish — see Prerequisites.

### Prerequisites — native bridge gap fixes (Rust, `crates/gpjs-ui`)

`setAttribute` is currently the only JS-bound setter — there's no
JS-reachable way to touch `style_props`, even though rendering
(`element.rs`) reads style exclusively from there. And `appendChild` only
appends at the end (`Vec::push`) — `RendererOptions.insert(el, parent,
anchor)` needs to insert before a specific sibling for correct Vue list
(`v-for`) diffing.

- [x] Add `setStyle(nodeId, key, value)` to `__gpjsui_native__`
      (`src/js/bindings.rs`), mirroring `setAttribute`'s validation/
      error-mapping pattern, calling `VirtualTree::set_style`
- [x] Tests mirroring `setAttribute`'s existing ones (happy path, unknown
      node id, non-primitive value)
- [x] Add `VirtualTree::insert_before(parent_id, child_id, anchor_id:
      Option<NodeId>)` in `src/tree.rs` (`None` anchor = append at end,
      so `append_child` becomes a thin wrapper over it) — an `anchor_id`
      naming a real node that isn't currently a child falls back to
      appending at the end, only a wholly unknown id errors
- [x] Bind it as `insertBefore(parentId, childId, anchorId | null)` in
      `bindings.rs`, same error-mapping pattern as `appendChild`
- [x] Tests: insert at start/middle/end, unknown parent/child/anchor id
- [x] Update `docs/FFI.md`'s binding function table and prose to add
      `setStyle`/`insertBefore`, and correct the note that folds style
      into `setAttribute`

### Unit i — TS tooling scaffolding

- [ ] Root or per-package `tsconfig.json` (none exists yet) — shared base
      config extended by `packages/gpjs-ui` and `packages/vue`
- [ ] Vite library-mode build for both packages (ESM output,
      `types`/`exports` fields in each `package.json`)

### Unit ii — `gpjs-ui` core package (`packages/gpjs-ui`)

- [ ] Typed wrapper functions over `globalThis.__gpjsui_native__`:
      `createNode`, `appendChild`, `insertBefore` (new), `removeChild`,
      `setAttribute`, `setStyle` (new), `addEventListener`
- [ ] Callback registry owning the `globalThis.__gpjsui_callbacks__[id]`
      convention internally (id allocation/cleanup) — `@gpjs-ui/vue`
      should never touch that raw contract itself
- [ ] TS types for the `docs/FFI.md`-documented style-prop and tag
      vocabulary (compile-time safety on top of the native bridge's
      stringly-typed calls)
- [ ] Vitest unit tests: each wrapper call forwards correctly to a mocked
      `__gpjsui_native__`; callback registry id lifecycle

### Unit iii — `@gpjs-ui/vue` custom renderer (`packages/vue`)

`nodeOps` implements `@vue/runtime-core`'s `RendererOptions<HostNode,
HostElement>` — its 9 required methods (`createElement`, `createText`,
`createComment`, `insert`, `remove`, `setText`, `setElementText`,
`parentNode`, `nextSibling`); the 4 optional ones (`querySelector`,
`setScopeId`, `cloneNode`, `insertStaticContent`) are out of scope, not
needed without SSR/hydration. Built on `packages/gpjs-ui`, never on
`__gpjsui_native__` directly.

- [ ] `createElement`/`insert`/`remove` — the core node lifecycle, built
      on `packages/gpjs-ui`'s `createNode`/`insertBefore`/`removeChild`
- [ ] `createText`/`setText`/`setElementText` map to a `tag: "text"` node
      + `setAttribute(id, "value", text)`, matching `hello_world.rs`'s
      existing text-leaf convention
- [ ] `createComment` maps to a hidden node (`createNode("div")` +
      `setStyle(id, "display", "none")`) — `display: none` is already in
      the style vocab, so this needs no render/`element.rs` change
- [ ] JS-side parent/sibling bookkeeping map maintained as `nodeOps`
      processes `insert`/`remove`, answering `parentNode`/`nextSibling`
      and giving `remove(el)` the parent id `removeChild` needs (the
      native tree has no parent pointers, and `RendererOptions.remove`
      doesn't pass one)
- [ ] `patchProp`: `key === "style"` (an object, per Vue's
      `:style="{...}"` binding) → one `setStyle` call per entry, unknown
      keys silently ignored (matching `docs/FFI.md`'s existing
      "malformed/unrecognized → ignored" policy)
- [ ] `patchProp`: `isOn`-prefixed keys (`onClick`, ...) → register via
      the callback registry + `addEventListener` — only `"click"` is
      wired to real input today (`docs/FFI.md` v1), other `on*` props are
      inert for now
- [ ] `patchProp`: everything else → `setAttribute`
- [ ] `createGpjsuiApp(App)` convenience wrapper around
      `createRenderer(nodeOps).createApp` — mounts against a
      `HostElement`-typed root handle (see Unit iv for how the Rust host
      supplies the root node id)
- [ ] Vitest tests asserting the create/insert/patchProp/remove call
      sequence against a mocked `gpjs-ui` core (mirroring the pattern
      `@vue/runtime-test`'s reference renderer uses for its own tests)

### Unit iv — `examples/hello-vue` + one-shot Rust loader (manual GUI check)

The `.vue` SFC compiles via a minimal one-shot `@vue/compiler-sfc` script —
no Vite yet (that's Phase 3's job, but these compiler calls carry forward
unchanged when Phase 3 swaps in Vite's dev pipeline; only the thin "invoke
once" wrapper becomes obsolete then).

- [ ] `examples/hello-vue` package: a real `.vue` SFC recreating the
      existing `hello_world`/`click_counter` look (bordered box, text
      label, click-to-increment), written against `@gpjs-ui/vue`
- [ ] One-shot `@vue/compiler-sfc` build script: `compileScript({
      inlineTemplate: true })` for `<script setup>` SFCs, falling back to
      explicit `compileTemplate` for template-only files (no `<style>`
      handling needed, GPUI has no CSS concept)
- [ ] Bundle the compiled SFC together with `packages/gpjs-ui`/
      `packages/vue` into one JS file consumable by QuickJS
- [ ] New `crates/gpjs-ui/examples/gpui/hello_vue.rs`: Rust creates one
      root `VirtualNode` (matching `click_counter.rs`'s pattern),
      installs `__gpjsui_native__` bindings, reads the compiled bundle
      from disk, and `Engine::eval`s it with the root node id exposed to
      JS (same id-interpolation convention `click_counter.rs` already
      uses), rendering via `render_tree_with_events` each frame
- [ ] Manual: run the example and look at the window — see
      `docs/MANUAL_GUI_CHECK.md`; once confirmed, update that doc to list
      this as a verified 4th example alongside the existing three

### Docs

- [ ] Update `AGENTS.md`'s Status section once Phase 2 lands

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
