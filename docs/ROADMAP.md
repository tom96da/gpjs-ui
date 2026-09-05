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
(Phase 7) is the final milestone, after cross-platform support (Phase 6).

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

## Phase 3: Developer tooling & HMR integration (`@gpjs-ui/vite-runtime`)

1. **Vite process management**: in debug builds, the Rust host spawns Vite in
   library/watch mode (not its browser dev server) via `std::process::Command`,
   using `@vitejs/plugin-vue` to compile `.vue` SFCs.
2. **HMR bridge** (`@gpjs-ui/vite-runtime`, at `packages/vite-runtime`):
   implement a custom `ModuleRunnerTransport` and module evaluator against
   Vite's Runtime API (`vite/module-runner`) so updated modules are evaluated
   inside QuickJS and trigger a GPUI view refresh (`cx.notify()`) — see
   [docs/ARCHITECTURE.md](./ARCHITECTURE.md#hmr-delivery) for why this is
   preferred over a hand-rolled HMR protocol.

## Phase 4: Majority style & Tailwind coverage (future)

Not started, and not begun until Phase 3's Vite integration lands —
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

## Implementation guidelines

See [AGENTS.md](../AGENTS.md) for the guiding principles (memory safety at the
FFI boundary, keeping the render loop zero-overhead, developer ergonomics)
that apply across every phase.
