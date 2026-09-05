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
- [x] `examples/hello-vue` placeholder package (deferred — needs `@gpjs-ui/vue` to have real content first) — landed differently than this line originally named: two separate example apps, `examples/hello_world` and `examples/click_counter` (Phase 2 Unit iv)

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

- [x] Root `tsconfig.base.json`, extended by each package's own
      `tsconfig.json` (`.mts` sources, `module: "preserve"` — the
      Vite-recommended setting for library builds, whose implied
      `moduleResolution: "bundler"` still requires an `.mts` file's own
      relative imports to name a real extension — neither extensionless
      nor a `.js`/`.mjs` specifier resolves to a sibling `.mts` file
      under this mode, only the literal `.mts` extension does (see Unit
      ii for the `allowImportingTsExtensions` flag that permits it)
- [x] Vite library-mode build for both packages (`vite.config.mts`, ESM
      output, `types`/`exports` fields in each `package.json`) — verified
      end-to-end: `pnpm -r typecheck`/`build`/`test` all pass against a
      placeholder `src/index.mts`
- [x] TypeScript 7's native compiler (`typescript@^7.0.2`, no
      programmatic API until 7.1) is used for the fast `tsc --noEmit`
      typecheck; `typescript` itself is npm-aliased to
      `@typescript/typescript6@^6.0.2` (its `tsc6` binary + full API) so
      API-consuming tooling (`unplugin-dts`) keeps working — the real
      native package lives under the `@typescript/native` alias instead
- [x] Declaration output via `unplugin-dts/vite` (not `vite-plugin-dts`,
      which is now just a thin, deprecated wrapper around it — see
      https://github.com/qmhc/unplugin-dts/blob/main/docs/en/usage.md)

### Unit ii — `gpjs-ui` core package (`packages/gpjs-ui`)

- [x] Typed wrapper functions over `globalThis.__gpjsui_native__`:
      `createNode`, `appendChild`, `insertBefore` (new), `removeChild`,
      `setAttribute`, `setStyle` (new), `addEventListener`
- [x] Callback registry owning the `globalThis.__gpjsui_callbacks__[id]`
      convention internally (id allocation/cleanup) — `@gpjs-ui/vue`
      should never touch that raw contract itself — re-registering the
      same `(nodeId, event)` frees its previous callback id; a stale id
      left behind in Rust's `EventListeners` is harmless once its
      JS-side entry is gone (dispatch just finds nothing, per
      `docs/FFI.md`'s event dispatch section). Not yet handled: a
      removed node's own listeners are never freed (`NodeId`s are never
      reused — `tree.rs`'s `create_node` — and there's no native
      "node removed" hook to react to), so this only bounds growth for
      handlers that get replaced in place, not ones whose node goes
      away — revisit once Unit iii's `remove(el)` needs it
- [x] TS types for the `docs/FFI.md`-documented style-prop and tag
      vocabulary (compile-time safety on top of the native bridge's
      stringly-typed calls) — `setStyle` is overloaded so known
      `StyleProps` keys get a typed value while unrecognized keys still
      fall back to the raw `AttributeValue` signature
- [x] Vitest unit tests: each wrapper call forwards correctly to a mocked
      `__gpjsui_native__`; callback registry id lifecycle — landed as
      `packages/gpjs-ui/src/index.test.mts`
- [x] Root `tsconfig.base.json` needed `allowImportingTsExtensions` once
      a real intra-package `.mts` import showed up (the test file
      importing `./index.mts`) — confirmed by elimination: with the flag
      off, an extensionless specifier and a `.js`/`.mjs` one both fail
      with `TS2307` (`module: "preserve"`'s implied `moduleResolution:
      "bundler"` doesn't extend `NodeNext`'s `.js`→`.ts`/`.mjs`→`.mts`
      mapping to this mode), so the literal `.mts` extension plus this
      flag is the only working spelling; the flag's own precondition
      (`noEmit`/`emitDeclarationOnly`) already holds via `noEmit: true`
- [x] `unplugin-dts`'s `include: ["src"]` also picked up `*.test.mts`
      files, emitting a stray `dist/index.test.d.mts` — fixed by adding
      `exclude: ["src/**/*.test.mts"]` to both packages'
      `vite.config.mts`

### Unit iii — `@gpjs-ui/vue` custom renderer (`packages/vue`)

`nodeOps` implements `@vue/runtime-core`'s `RendererOptions<HostNode,
HostElement>` — its 9 required methods (`createElement`, `createText`,
`createComment`, `insert`, `remove`, `setText`, `setElementText`,
`parentNode`, `nextSibling`); the 4 optional ones (`querySelector`,
`setScopeId`, `cloneNode`, `insertStaticContent`) are out of scope, not
needed without SSR/hydration. Built on `packages/gpjs-ui`, never on
`__gpjsui_native__` directly.

- [x] `createElement`/`insert`/`remove` — the core node lifecycle, built
      on `packages/gpjs-ui`'s `createNode`/`insertBefore`/`removeChild` —
      landed as `packages/vue/src/nodeOps.mts`. `HostNode`/`HostElement`
      aren't the bare `NodeId`: each is a JS object (`GpjsuiElement`/
      `GpjsuiText`) pairing the native id with the parent/children
      bookkeeping the next item describes, matching the shape
      `@vue/runtime-test`'s reference renderer uses for its own host nodes
- [x] Add `disposeNode(nodeId)` to `packages/gpjs-ui` (closes the gap
      Unit ii's callback registry left open — see its notes above): purges
      every callback id registered for `nodeId` from
      `registeredCallbackIds`/`__gpjsui_callbacks__`, across all event
      names, not just the one currently patched. `nodeOps.remove(el)`
      calls it alongside `removeChild` — this is the first point in the
      stack that actually knows a node is gone for good
- [x] `createText`/`setText`/`setElementText` map to a `tag: "text"` node + `setAttribute(id, "value", text)`, matching `hello_world.rs`'s
      existing text-leaf convention
- [x] `createComment` maps to a hidden node (`createNode("div")` +
      `setStyle(id, "display", "none")`) — `display: none` is already in
      the style vocab, so this needs no render/`element.rs` change
- [x] JS-side parent/sibling bookkeeping map maintained as `nodeOps`
      processes `insert`/`remove`, answering `parentNode`/`nextSibling`
      and giving `remove(el)` the parent id `removeChild` needs (the
      native tree has no parent pointers, and `RendererOptions.remove`
      doesn't pass one) — `insert` also detaches a child from its old
      parent first (in both the native tree and this bookkeeping) before
      attaching it to the new one, so moving a node for a keyed-list
      reorder doesn't dispose it
- [x] `patchProp`: `key === "style"` (an object, per Vue's
      `:style="{...}"` binding) → one `setStyle` call per entry, unknown
      keys silently ignored (matching `docs/FFI.md`'s existing
      "malformed/unrecognized → ignored" policy) — a value shape
      `setStyle` can't take (non-primitive) is skipped the same way,
      since unlike a style prop's own render-path fallback, `setStyle` is
      a JS call boundary that would otherwise raise
- [x] `patchProp`: `isOn`-prefixed keys (`onClick`, ...) → register via
      the callback registry + `addEventListener` — only `"click"` is
      wired to real input today (`docs/FFI.md` v1), other `on*` props are
      inert for now
- [x] `patchProp`: everything else → `setAttribute`, skipping non-primitive
      values the same way `style` does (removing a prop entirely — a
      `null`/`undefined` next value — is also a no-op: there's no native
      "unset" call to route it to, a deferred v1 gap like the others above)
- [x] `createGpjsuiApp(App)` convenience wrapper around
      `createRenderer(nodeOps).createApp` — mounts against a
      `HostElement`-typed root handle (see Unit iv for how the Rust host
      supplies the root node id)
- [x] Vitest tests asserting the create/insert/patchProp/remove call
      sequence against a mocked `gpjs-ui` core (mirroring the pattern
      `@vue/runtime-test`'s reference renderer uses for its own tests) —
      `packages/vue/src/nodeOps.test.mts`/`patchProp.test.mts`. Also added
      (beyond this item's original scope): `packages/vue/tests/
      renderer.test.mts`, an integration test driving `createGpjsuiApp`
      end-to-end through a real (unmocked) `gpjs-ui` core against a small
      in-memory fake standing in for `globalThis.__gpjsui_native__`,
      confirming a mounted component's styles/attributes/text and a
      keyed-list reorder (moves, not recreation) — the scenario that
      originally motivated this phase's `insertBefore` prerequisite

### Unit iv — Vue example apps + `gpjs-ui-example-runner` (manual GUI check)

Landed differently than originally planned on this line (one shared
`examples/hello-vue` package + a Cargo example living inside `crates/gpjs-ui`)
— see the design notes below for what changed and why, arrived at over
several rounds of review.

Each `.vue` SFC compiles via a minimal one-shot `@vue/compiler-sfc` script —
no Vite yet (that's Phase 3's job); only the thin "invoke once" wrapper
becomes obsolete when Phase 3 swaps in Vite's dev pipeline, the
`compileScript`/bundling calls themselves carry forward unchanged.

- [x] Two independent, pure-Vue/TS example apps, each its own pnpm
      workspace package with no Rust inside — `examples/hello_world`
      (recreating `hello_world.rs`'s static tree: bordered box, text
      label, a `v-for` row of six colored squares) and
      `examples/click_counter` (recreating `click_counter.rs`'s clickable,
      counting box, via `ref`/`computed`)
- [x] Each app's `scripts/build.mjs`: one-shot `@vue/compiler-sfc`
      `compileScript({ id, inlineTemplate: true, templateOptions:
      { compilerOptions: { runtimeModuleName: "@vue/runtime-core" } } })`
      (retargeted off the default `"vue"` specifier — this repo has no
      `vue` meta-package dependency anywhere), then a programmatic `vite
      build()` (`define` stubs `process.env.NODE_ENV`, since
      `@vue/runtime-core`'s dev-only warning branches check it directly
      and QuickJS has no Node globals) bundling the compiled SFC together
      with `@gpjs-ui/vue` and `@vue/runtime-core` into one self-contained
      `dist/bundle.js` — no unresolved imports, no Vite dev server
- [x] `Engine::eval_module` (`src/js/engine.rs`): `rquickjs::Module::declare`
      + `.eval()` + `Promise::finish()` inside the existing `Context::with`
      closure — no `ModuleLoader`/resolver needed, since the bundle this
      evaluates is fully self-contained. Returns `EngineResult<()>`, not a
      module's exports: a module's own completion value is always
      `undefined` per spec, so unlike `eval` there's nothing meaningful to
      convert to a caller-chosen type; state comes back through
      `globalThis`, the same convention `click_counter.rs` already uses
      for plain scripts
- [x] `crates/gpjs-ui/tests/js_core_integration.rs`: install real
      `__gpjsui_native__` bindings via `bindings::install`, then — through
      `Engine::with` directly rather than `eval_module`, since this needs
      the module's export table — `Module::declare`/`.eval()`/
      `Promise::finish()` the bundled `packages/gpjs-ui` output and call
      its exports back via `Module::get`, asserting against the resulting
      `VirtualTree` state. A real bundler renames a module's internal
      top-level bindings (confirmed empirically), so exports are only
      reachable this way, not by assuming a hand-written-JS-style
      same-scope call works against real compiled output
- [x] New crate `crates/gpjs-ui-example-runner` (**not** `crates/gpjs-ui/examples/gpui/hello_vue.rs`
      as originally planned): a single generic loader, `gpjs-ui-example-runner
      <path-to-bundle.js>`, used to run *either* example app. Creates one
      root `VirtualNode`, installs bindings, reads the bundle, substitutes
      a `__GPJSUI_ROOT_ID__` placeholder token with the real root id via
      `str::replace` (not `format!` — the bundle is a large file full of
      literal `{`/`}`, unlike `click_counter.rs`'s short inline literal),
      `eval_module`s it, then opens a GPUI window sized off the mounted
      root's own `width`/`height` style (falls back to 800×600 if unset)
      and always renders via `render_tree_with_events` + `EventDispatcher`
      (never plain `render_tree` — one binary can't know ahead of time
      whether a given bundle registered any click handlers)
- [x] `EventDispatcher::dispatch` (`src/render/bridge.rs`) bugfix, found
      while manually checking `click_counter`'s real Vue port: a callback
      that only *schedules* its effect via a microtask (as
      `@vue/runtime-core`'s reactivity scheduler does, batched via
      `Promise.resolve().then(...)`) hadn't actually mutated the tree by
      the time `dispatch` returned — nothing ever drained the pending job
      that would run it. Fixed by draining pending jobs right after
      calling the registered callbacks; regression test added to
      `tests/event_dispatch.rs` using a deferred callback
- [x] `packages/vue/package.json`: `@vue/runtime-core` moved from
      `dependencies` to `peerDependencies` (+ `devDependencies`, for this
      package's own build/test) — otherwise a consuming app could resolve
      its own separate copy of `@vue/runtime-core`, producing a duplicate
      Vue instance with reactivity split across the two copies
- [x] Manual: run the example and look at the window — confirmed
      end-to-end on both macOS (native) and from the devcontainer via
      XQuartz forwarding, both examples, per
      `docs/MANUAL_GUI_CHECK.md`

### Docs

- [x] Update `AGENTS.md`'s Status section once Phase 2 lands

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
