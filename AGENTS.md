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
- **Frontend**: Vue 3 (first-class support) via a custom renderer, built with Vite. React and other frameworks are a future goal, not yet implemented.

See [README.md](./README.md) for the full pitch. `gpjs-ui` is currently a development code name.

## Status

This repository is at the bootstrap stage: no Rust crate or frontend package exists yet. There are no build, lint, or test commands to run. When the workspace is scaffolded, update this file with the actual commands (`cargo build`, `cargo test`, package manager for the frontend, etc.) instead of leaving this section stale.

## Repository structure

See [docs/STRUCTURE.md](./docs/STRUCTURE.md) for the full directory map, including the pinned `third_party/` submodules (zed, rquickjs, quickjs-ng) and how to init/update them.

## Conventions

- **License**: dual-licensed MIT OR Apache-2.0 (see [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-APACHE](./LICENSE-APACHE)). Prefix new source files with the SPDX header used at the top of this file (AGENTS.md):
  ```
  Copyright (c) <year> tom96da
  SPDX-License-Identifier: MIT OR Apache-2.0
  ```
- **Commit messages**: this repo follows [Conventional Commits](https://www.conventionalcommits.org/):
  ```
  <type>[optional scope]: <description>

  [optional body]

  [optional footer(s)]
  ```
  - `type` is one of `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`.
  - `description` is imperative, lower-case, no trailing period (e.g. `feat(runtime): add quickjs bridge`).
  - A breaking change is marked either with `!` after the type/scope (`feat!: ...`) or a `BREAKING CHANGE:` footer — not both unless it aids clarity.
  - Scope is optional; use it for the affected area once the workspace has named crates/packages (e.g. `fix(gpui-shell): ...`).
- Keep this file (not just README.md) up to date as real architecture, module boundaries, and commands land — this is the file agents read first.
