<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Roadmap

Planned phased implementation of the design in
[docs/ARCHITECTURE.md](./ARCHITECTURE.md). The repository is at the bootstrap
stage (see [AGENTS.md](../AGENTS.md#status)) — no phase has started yet.
Update the status of each phase as it lands instead of leaving this stale.

Vue 3 support is built first end-to-end (Phases 1–3); React support is an
additive package added later (Phase 4), not a parallel effort.

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
   [docs/FFI.md](./FFI.md#binding-functions)) — the raw host-bridge calls and
   their TypeScript types are defined exactly once here, shared by every
   framework adapter (Vue now, React later) instead of duplicated in each.
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

## Phase 4: React custom renderer (future)

Not started, and not begun until Vue 3 support (Phases 1–3) is stable. Adds
`@gpjs-ui/react` as an additional package alongside `@gpjs-ui/vue`, using
`react-reconciler` against the same `gpjs-ui` core package (not
`__gpjsui_native__` directly — see Phase 2), plus `@vitejs/plugin-react` for
JSX/TSX compilation and HMR.

## Implementation guidelines

See [AGENTS.md](../AGENTS.md) for the guiding principles (memory safety at the
FFI boundary, keeping the render loop zero-overhead, developer ergonomics)
that apply across every phase.
