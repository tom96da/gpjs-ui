<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Roadmap

Planned phased implementation of the design in
[docs/ARCHITECTURE.md](./ARCHITECTURE.md). This file describes phase-level
design intent only — it doesn't track progress itself. See
[AGENTS.md](../AGENTS.md#status) for which phases have landed so far and
[docs/PLAN.md](./PLAN.md) for the checkbox-tracked, per-task breakdown.

Vue 3 support is built first end-to-end (Phases 1–3), followed by majority
style/Tailwind coverage (Phase 4); React support is an additive package
added later (Phase 5), not a parallel effort. Full style/Tailwind parity
(Phase 7) is the final styling milestone, after cross-platform support
(Phase 6); app-owned Rust extensions (Phase 8) come after that, and are the
only phase where an app author needs a Rust toolchain at all.

## Phase 1: Rust host & FFI bridge core (`gpjs-ui`)

1. **QuickJS context setup**: use `rquickjs` to spin up a managed QuickJS
   runtime inside the GPUI event loop.
2. **Retained virtual tree**: an in-memory, arena-allocated `VirtualNode`
   structure — see [docs/FFI.md](./FFI.md#retained-virtual-tree).
3. **Binding functions** exposed to JS as `globalThis.__gpjsui_native__` — see
   [docs/FFI.md](./FFI.md#binding-functions).
4. **GPUI rendering pipeline**: recursively convert the `VirtualNode` tree into
   GPUI `AnyElement` instances during GPUI's `render()` frame cycle.

## Phase 2: JS core bridge (`gpjs-ui`) & Vue 3 custom renderer (`@gpjs-ui/vue`)

1. **`gpjs-ui`** (`packages/gpjs-ui`): a framework-agnostic, typed JS wrapper
   around `globalThis.__gpjsui_native__` (see
   [docs/FFI.md](./FFI.md#binding-functions)) — see
   [docs/ARCHITECTURE.md](./ARCHITECTURE.md#tech-stack) for why this is a
   separate, shared package rather than logic duplicated into each framework
   adapter.
2. **`@gpjs-ui/vue`** (`packages/vue`): a custom Vue 3 runtime adapter
   using `@vue/runtime-core`'s `createRenderer`, built on `gpjs-ui` rather
   than calling `__gpjsui_native__` directly.
3. Map Vue node lifecycle methods (`createElement`, `insert`, `remove`,
   `patchProp`) to `gpjs-ui`'s calls.
4. A unified mount API, e.g. `createGpjsuiApp(App).mount('#root')`.

## Phase 3: Developer tooling & HMR integration

The `gpjsui` CLI's process orchestration is owned by the **JS/TS side**:
`@gpjs-ui/cli` (`packages/cli`) is the parent process, holding Vite
in-process and spawning the Rust host (`crates/gpjs-ui-host`) as a child,
with dev-server messages bridged over the child's stdio. The alternatives
considered — a Rust-primary `crates/gpjs-ui-cli` owning everything, and
Rust-primary logic behind a thin npm `bin` wrapper — were rejected because:

- Node already owns every orchestration primitive this needs (Vite's own
  server/watch/restart API, `child_process`, terminal logging), where Rust
  would grow equivalent process/IPC/file-watching plumbing from scratch.
- The host binary ships through npm either way, since app authors aren't
  expected to have a Rust toolchain (Phase 8 is the one exception). A Rust
  CLI would add a second binary to distribute for no gain.
- It keeps `crates/gpjs-ui` and the host free of process/IPC concerns.

Phase 3 lands in four numbered stages, with real HMR deliberately **last**:
a full-reload dev loop already needs the whole spawn/teardown/remount
skeleton HMR builds on, and this ordering makes `dev`, `build`, and
packaging usable end-to-end before the hardest piece starts.

A GitHub Actions **CI** workflow running
[docs/TESTING.md](./TESTING.md)'s required checks lands before 3.1, so the
first multi-package phase isn't built without one. Its **CD** counterpart
lands after 3.3, when there is something to release.

### Phase 3.1: `gpjsui dev` (full reload)

1. **`@gpjs-ui/cli`** (`packages/cli`): `gpjsui dev` runs Vite in
   library/watch mode (not its browser dev server), using
   `@vitejs/plugin-vue` to compile `.vue` SFCs, then spawns the host and
   sends it a reload message on every rebuild.
2. **`crates/gpjs-ui-host`**: the runtime binary, grown out of Phase 2's
   `gpjs-ui-example-runner` — opens the GPUI window and evaluates a bundle
   in QuickJS, and in dev mode reads newline-delimited JSON messages on
   stdin, re-evaluating the bundle in a fresh engine against a reset tree on
   each reload. Its stdout is the protocol channel; logs go to stderr.
3. **Native root handle**: a binding replacing Phase 2's
   `__GPJSUI_ROOT_ID__` source substitution, so an app's entry point is
   plain code (`createGpjsuiApp(App).mount()`) with no host-injected token
   in it.

Component state is *not* preserved across a reload — that's exactly what
Phase 3.4 adds.

### Phase 3.2: `gpjsui build`

The one-shot production counterpart of 3.1's pipeline, emitting a
self-contained bundle. Subsumes the per-example `scripts/build.mjs` files
Phase 2 Unit iv hand-rolled.

### Phase 3.3: Application packaging

Pairs a built bundle with a prebuilt host binary into a distributable
application (`.app`/`.exe`), plus the per-platform npm distribution of those
prebuilt hosts. **Design constraint**: keep the host binary swappable, so
Phase 8 can substitute an app-compiled one.

This is the **first release milestone**: once packaging works, the framework
is published as `v0.0.1`, to npm only (`gpjs-ui`, `@gpjs-ui/vue`,
`@gpjs-ui/cli`, and the per-platform host packages). The Rust crates stay
`publish = false` — nothing outside this repo depends on them until Phase 8.

### Phase 3.4: HMR (`@gpjs-ui/vite-runtime`)

**HMR bridge** (`@gpjs-ui/vite-runtime`, at `packages/vite-runtime`): a
custom `ModuleRunnerTransport` and module evaluator against Vite's Runtime
API (`vite/module-runner`), so updated modules are evaluated inside QuickJS
and trigger a GPUI redraw while component state survives. The runner itself
runs inside QuickJS, not on the Node side — see
[docs/ARCHITECTURE.md](./ARCHITECTURE.md#hmr-delivery) for why, and for why
this is preferred over a hand-rolled HMR protocol.

## Phase 4: Majority style & Tailwind coverage (future)

Not started, and not begun until Phase 3.1's Vite integration lands —
Tailwind's own JIT compiler runs as a build-time step, so it needs a real
Vite pipeline to plug into. Full CSS/Tailwind parity is not the goal here
(see Phase 7); this phase targets the "structural" utility categories that
cover the large majority of real-world usage and map cleanly onto GPUI's
native styling model:

1. **Native style vocabulary expansion** (`crates/gpjs-ui`): close the gaps
   flagged as "deliberately incomplete" since Phase 1 (see
   [docs/FFI.md](./FFI.md)) — margin/padding, percentage lengths, min/max
   size, flex-grow/shrink/basis, per-side border width/radius, basic
   box-shadow, font-weight/family, line-height/letter-spacing.
2. **Tailwind class resolver**: gpjs-ui has no real CSS engine, so Tailwind
   utility classes can't generate actual CSS — a Vite plugin (building on
   Phase 3's pipeline) scans `class="..."` usage and maps each recognized
   utility directly to a `setStyle` call, rather than through a stylesheet.
3. **Scope target: roughly 70–75% of Tailwind's utility classes** —
   layout/flexbox/grid, spacing, sizing, typography basics, solid
   background/text/border colors, borders/radius, basic shadow. Explicitly
   deferred to Phase 7: responsive breakpoint variants (`sm:`/`md:`/...),
   state variants (`hover:`/`focus:`/`group-*`), dark mode,
   animations/transitions, transforms, filters/backdrop-filters, and
   arbitrary bracket values (`w-[137px]`) — these need real design work
   (e.g. mapping `hover:` onto GPUI's own interactive element states)
   rather than a straightforward style-prop translation.

## Phase 5: React custom renderer (future)

Not started, and not begun until Vue 3 support (Phases 1–3) is stable. Adds
`@gpjs-ui/react` as an additional package alongside `@gpjs-ui/vue`, using
`react-reconciler` against the same `gpjs-ui` core package (not
`__gpjsui_native__` directly — see Phase 2), plus `@vitejs/plugin-react` for
JSX/TSX compilation and HMR.

## Phase 6: Cross-platform support (future)

Not started, and not begun until the core Rust host design (Phases 1–2)
is stable — same reasoning as Phase 5. macOS is the primary development
target until then. This phase properly supports Linux (resolving the
devcontainer's unconfirmed rendering — see docs/MANUAL_GUI_CHECK.md) and
adds the `gpui_windows` platform backend for Windows.

## Phase 7: 100% style & Tailwind parity (future)

Not started, and not begun until cross-platform support (Phase 6) is
stable. Closes exactly the gap Phase 4 deferred: full CSS-property parity
in the native style vocabulary/render pipeline (animations/transitions,
transforms, filters, gradients, arbitrary values), state variants mapped
onto GPUI's own interactive element states (hover/focus/active),
responsive breakpoints (no browser viewport concept exists here, so this
needs its own window-size-aware style-resolution design), and dark mode.
The final styling milestone: every Tailwind utility class Vue (and later
React) authors reach for should resolve to a correct native rendering, not
just the common ones Phase 4 covers.

## Phase 8: App-owned Rust extensions (future)

Not started, and not begun until Phase 3.3's packaging and Phase 7's
styling are stable. Every phase before this one assumes app authors write
only JS/TS and consume a prebuilt host binary; this phase adds the opt-in
case where an app moves its own heavy work (compute, native I/O) into Rust
and still ships as a single application:

1. **App-owned host build**: an app that carries its own Rust crate gets a
   host compiled from source with that crate linked in, in place of the
   prebuilt binary — the swappability Phase 3.3 is required to preserve.
   Apps without one keep needing no Rust toolchain.
2. **Extension binding surface**: a stable way for app-owned Rust code to
   register its own functions alongside `__gpjsui_native__` (see
   [docs/FFI.md](./FFI.md#binding-functions)), rather than patching the
   host's own bindings. This is the likely driver for
   `crates/gpjs-ui-macros` (see [docs/STRUCTURE.md](./STRUCTURE.md)).
3. **MSRV verification**: `rust-version` is held equal to the pinned
   toolchain while these crates have no consumers outside this repo. Once
   app crates compile against them it drops to a real floor, checked by its
   own job — see
   [docs/TESTING.md](./TESTING.md#toolchain-pinning-and-msrv).

## Implementation guidelines

See [AGENTS.md](../AGENTS.md) for the guiding principles (memory safety at the
FFI boundary, keeping the render loop zero-overhead, developer ergonomics)
that apply across every phase.
