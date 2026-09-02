<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Architecture

Target architecture for gpjs-ui. This describes the design agents should build
toward — the repository is still at the bootstrap stage (see
[AGENTS.md](../AGENTS.md#status)), so treat this as a plan, not a description
of code that exists yet. Update it as each piece actually lands; don't let it
drift from reality.

The phased build-out of this design is tracked in [docs/ROADMAP.md](./ROADMAP.md).
The JS↔Rust binding surface is specced in [docs/FFI.md](./FFI.md).

## Tech stack

| Layer | Technology | Primary role |
| --- | --- | --- |
| **Native core** | Rust + [`gpui`](https://www.gpui.rs/) (wgpu) | Window management, event loop, retained virtual tree, direct GPU rendering. |
| **JS engine** | QuickJS via [`rquickjs`](https://github.com/DelSkayn/rquickjs) | Embedded, lightweight JS runtime executing UI logic and reactivity. |
| **Core JS package** | `gpjs-ui` (framework-agnostic) | Thin, typed JS wrapper around the host bridge (`__gpjsui_native__`), shared by every framework adapter instead of duplicated in each. |
| **Frontend framework** | `@gpjs-ui/vue` (first-class, current) / `@gpjs-ui/react` (future, see [Roadmap](./ROADMAP.md#phase-4-react-custom-renderer-future)) | Custom renderer mapping virtual component trees to `gpjs-ui` calls. |
| **Bundler & dev tooling** | Vite, used in library/build mode (no browser dev server) | Compiles `.vue`/`.tsx` via the official `@vitejs/plugin-vue` (and later `@vitejs/plugin-react`); HMR is delivered through Vite's Runtime API instead of Vite's browser client — see [HMR delivery](#hmr-delivery). |
| **Host bridge** | In-process Rust functions bound into the QuickJS context via `rquickjs` | Transfers mutation operations (`createNode`, `setAttribute`, `appendChild`, ...) from JS to the Rust host. Not a real C ABI or IPC boundary — everything runs in one process. |

Why Vite instead of a bare bundler (e.g. raw Rolldown): Vite owns the official,
maintained Vue SFC (and future React JSX) compiler integration
(`@vitejs/plugin-vue`). Reimplementing SFC compilation (template compile, CSS
extraction, source maps) on top of a bare bundler would be significant extra
work for no runtime benefit, since gpjs-ui never uses Vite's browser dev
server anyway — only its library/build API and its Runtime API (see below).
Modern Vite is also moving its own internals onto Rolldown, so the
Rust-bundler speed benefit isn't lost by choosing Vite.

## System architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                      [ Dev Environment ]                         │
│  [ .vue / .tsx ] ──▶ [ Vite (library mode, watch) ]              │
│                             │ Vite Runtime API                   │
│                             │ (ModuleRunner + custom Transport)  │
└─────────────────────────────┼────────────────────────────────────┘
                              ▼
┌──────────────────────────────────────────────────────────────────┐
│                    [ gpjs-ui Native Runtime ]                    │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ JS Runtime (QuickJS via `rquickjs`)                        │  │
│  │   - Vue 3 application (React: future, see Roadmap)         │  │
│  │   - Custom renderer (`createRenderer` / `react-reconciler`)│  │
│  │   - `gpjs-ui` core (typed `__gpjsui_native__` wrapper)     │  │
│  │   - Custom ModuleEvaluator (runs Vite-transformed modules  │  │
│  │     inside QuickJS instead of Node's `vm`)                 │  │
│  └──────────────────────────┬─────────────────────────────────┘  │
│                             │ Host bridge call                   │
│  ┌──────────────────────────▼─────────────────────────────────┐  │
│  │ Rust Host Core (GPUI engine)                               │  │
│  │   - Retained virtual tree (state & layout)                 │  │
│  │   - Host bridge dispatcher (`createNode`, `setAttribute`)  │  │
│  │   - GPUI element builder ──▶ [ Native GPU (wgpu/Metal) ]   │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

## HMR delivery

Vite ships a **Runtime API** (`vite/module-runner`, the same mechanism
`vite-node`/Vitest/Nuxt dev SSR use to run Vite-transformed modules outside a
browser) built specifically for "run Vite modules with HMR in a non-browser
environment." gpjs-ui uses it instead of Vite's browser client:

- A custom **`ModuleRunnerTransport`** bridges WebSocket messages between the
  Vite dev server (a Node process) and the Rust host.
- A custom **module evaluator** executes the transformed module source inside
  QuickJS, in place of the default Node `vm`-based evaluator.

This gets real HMR (module graph invalidation, accept/dispose boundaries)
without reimplementing Vite's HMR protocol from scratch — the only new code is
the transport and the evaluator.

## Host bridge (FFI)

See [docs/FFI.md](./FFI.md) for the exact function surface and the retained
virtual tree's node structure.
