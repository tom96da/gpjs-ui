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

This repository is at the bootstrap stage: no Rust crate or frontend package exists yet. There are no build, lint, or test commands to run. [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) and [docs/ROADMAP.md](./docs/ROADMAP.md) describe the *planned* target design — not code that exists yet. When the workspace is scaffolded, update this file with the actual commands (`cargo build`, `cargo test`, package manager for the frontend, etc.) instead of leaving this section stale.

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
- **Dev container**: the project targets `mcr.microsoft.com/devcontainers/rust:2-1-trixie` with the `node` devcontainer feature. Use `pnpm` for any JS/frontend tooling — Vite's officially supported and tested runtime is Node.js, and this project's HMR bridge builds directly on Vite's less battle-tested Runtime API (`vite/module-runner`), so avoid introducing a second, less-proven runtime (e.g. Bun) there.
- **Commit messages**: this repo follows [Conventional Commits](https://www.conventionalcommits.org/):
  ```
  <type>[optional scope]: <description>

  [optional body]

  [optional footer(s)]
  ```
  - `type` is one of `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.
  - `description` is imperative, lower-case, no trailing period (e.g. `feat(runtime): add quickjs bridge`).
  - `body` is a concise bullet list of what was done and why — not prose.
  - A breaking change is marked either with `!` after the type/scope (`feat!: ...`) or a `BREAKING CHANGE:` footer — not both unless it aids clarity.
  - Scope is optional; use it for the affected area once the workspace has named crates/packages (e.g. `fix(gpui-shell): ...`).
  - Any commit Claude is involved in must include a `Co-Authored-By: Claude <noreply@anthropic.com>` trailer (adjust the model name if relevant, e.g. `Claude Sonnet 5`).
- Keep this file (not just README.md) up to date as real architecture, module boundaries, and commands land — this is the file agents read first.
