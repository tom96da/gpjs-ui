<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# AGENTS.md

Instructions for AI coding agents working in this repository.

## Project

**gpjs-ui** is an ultra-lightweight, Webview-free desktop application framework:

- **Engine / Core**: Rust, built on [`gpui`](https://www.gpui.rs/) for direct GPU rendering (no Chromium/DOM).
- **JS Runtime**: QuickJS via [`rquickjs`](https://github.com/DelSkayn/rquickjs), for a micro-sized, sub-second-startup runtime.
- **Frontend**: Vue 3 (first-class support, built first) via a custom renderer. React and other frameworks are a future, additive goal — not yet implemented, and not started until Vue 3 support is stable.
- **Bundler / dev tooling**: Vite, used in library/build mode (not as a browser dev server) — see [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md).

See [README.md](./README.md) for the full pitch. `gpjs-ui` is currently a development code name.

## Status

Phase 1 (Rust host & FFI bridge core, `crates/gpjs-ui`) is functionally
complete per [docs/PLAN.md](./docs/PLAN.md): the retained `VirtualNode`
arena (`src/tree.rs`), the QuickJS runtime bootstrap (`src/js/engine.rs`),
the `__gpjsui_native__` bindings (`src/js/bindings.rs`), `VirtualNode` →
GPUI `AnyElement` conversion (`src/render/element.rs`), and click-event
dispatch back into JS (`src/render/bridge.rs`) are all implemented and
tested — see [docs/FFI.md](./docs/FFI.md) for the exact tag/style/event
vocabulary landed so far (all deliberately incomplete subsets, extended as
real usage needs them). The one item nobody in this container can do
unassisted is the manual visual check — see
[docs/MANUAL_GUI_CHECK.md](./docs/MANUAL_GUI_CHECK.md) for why and how to
actually run it.
[docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) and
[docs/ROADMAP.md](./docs/ROADMAP.md)'s Phase 2 is partway landed per
[docs/PLAN.md](./docs/PLAN.md): the pnpm workspace and `packages/gpjs-ui`'s
typed wrapper around `__gpjsui_native__` (Units i–ii) are done and tested.
`packages/vue`'s custom renderer (Unit iii onward), the Vite/HMR bridge
(Phase 3), and everything after are still *planned* target design, not
landed code.

Keep this section's status prose accurate as real logic lands — don't let it go stale.

## Architecture

- [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) — tech stack, system diagram, and how HMR is delivered into the embedded QuickJS runtime.
- [docs/ROADMAP.md](./docs/ROADMAP.md) — the phased build-out plan (Vue 3 first, React later as an additive package).
- [docs/FFI.md](./docs/FFI.md) — the JS↔Rust host bridge function surface.

### Guiding principles

- **Safety first**: Rust↔QuickJS bindings must handle pointer conversions and reference counts carefully — this boundary is the most likely source of memory leaks or segfaults.
- **Zero-overhead render loop**: don't run JS on every frame. JS executes only on reactivity updates, pushing snapshot mutations to Rust; Rust owns the retained tree and does layout/drawing natively.
- **Developer ergonomics**: frontend code stays strictly standard — `.vue`/`.tsx` code should feel identical to ordinary web development, not like it's targeting an embedded runtime.

## Repository structure

See [docs/STRUCTURE.md](./docs/STRUCTURE.md) for the full directory map, including the pinned `third_party/` submodules (zed, rquickjs, quickjs-ng) and how to init/update them.

## Conventions

- **License**: dual-licensed MIT OR Apache-2.0 (see [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-APACHE](./LICENSE-APACHE)). Prefix new source files with the SPDX header used at the top of this file (AGENTS.md):
  ```
  Copyright (c) <year> tom96da
  SPDX-License-Identifier: MIT OR Apache-2.0
  ```
- **Dev container**: `.devcontainer/Dockerfile` builds on `mcr.microsoft.com/devcontainers/rust:2-1-trixie`, adding the native build/runtime dependencies `gpui` needs (windowing, Vulkan, fontconfig — see the Dockerfile's comment), plus the `node` devcontainer feature. Use `pnpm` for any JS/frontend tooling — Vite's officially supported and tested runtime is Node.js, and this project's HMR bridge builds directly on Vite's less battle-tested Runtime API (`vite/module-runner`), so avoid introducing a second, less-proven runtime (e.g. Bun) there.
- **Git & commits**: see [docs/GIT.md](./docs/GIT.md) for the commit message format and, most importantly, the review policy — never run `git commit`/`git commit --amend` without first showing the exact diff and message for explicit approval.
- **Testing & tooling**: see [docs/TESTING.md](./docs/TESTING.md) for where tests live and the full set of checks (lint, format, type-check, tests) that must pass, for both Rust and TypeScript.
- Keep this file (not just README.md) up to date as real architecture, module boundaries, and commands land — this is the file agents read first.
