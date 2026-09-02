<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Repository structure

A map of this repository for AI agents. Read [AGENTS.md](../AGENTS.md) first — this file is the detailed reference it points to.

```
gpjs-ui/
├── README.md                # public-facing project pitch (features, tech stack, license)
├── LICENSE-MIT
├── LICENSE-APACHE           # dual-licensed MIT OR Apache-2.0
├── AGENTS.md                # agent instructions entry point — read this first
├── CLAUDE.md                # Claude Code entry point; just `@AGENTS.md`
├── docs/
│   ├── STRUCTURE.md         # this file
│   ├── ARCHITECTURE.md      # target tech stack, system diagram, HMR delivery design
│   ├── ROADMAP.md           # planned phased implementation (Vue 3 first, React later)
│   ├── FFI.md               # JS↔Rust host bridge function surface (spec, not yet implemented)
│   ├── PLAN.md              # checkbox-tracked, per-phase task breakdown of ROADMAP.md
│   └── GIT.md               # commit message format and the commit-review policy
├── .devcontainer/
│   ├── devcontainer.json    # builds from ./Dockerfile, adds the `node` and `claude-code` features
│   ├── Dockerfile           # extends the Rust devcontainer image with gpui's native build deps
│   └── post-create.sh       # post-create ownership fixes (non-image-layer setup only)
├── .github/
│   └── dependabot.yml       # auto-updates the devcontainer image/features only, for now
├── Cargo.toml               # Rust workspace manifest
├── Cargo.lock               # locked Rust dependency graph, including git-pinned gpui
├── crates/
│   └── gpjs-ui/             # Rust host crate placeholder for Phase 1
└── third_party/             # pinned upstream sources, as git submodules — see below
    ├── zed/                 # zed-industries/zed @ v1.17.2
    └── rquickjs/            # DelSkayn/rquickjs @ v0.12.2
        └── sys/quickjs/     # nested submodule: quickjs-ng @ the commit rquickjs v0.12.2 pins
```

## Status

This repository is at the scaffold stage: the Rust workspace and `crates/gpjs-ui` exist, but the crate is a placeholder with no real implementation yet. The pnpm/frontend workspace has not landed yet.

- Rust: `cargo check -p gpjs-ui`, `cargo test -p gpjs-ui`.

Update this file and [AGENTS.md](../AGENTS.md) as real crates, packages, commands, and ownership boundaries land — do not let either go stale.

## Planned workspace layout

Later phases add more workspace members and packages (see
[docs/ROADMAP.md](./ROADMAP.md)):

```
gpjs-ui/
├── Cargo.toml
├── crates/
│   ├── gpjs-ui/             # GPUI host, retained tree, QuickJS host bridge (Phase 1)
│   ├── gpjs-ui-cli/         # `cargo gpjsui` dev/build CLI, manages the Vite process (Phase 3)
│   └── gpjs-ui-macros/      # host bridge binding helper macros
├── packages/
│   ├── gpjs-ui/             # `gpjs-ui` — framework-agnostic host bridge wrapper (Phase 2)
│   ├── vue/                 # `@gpjs-ui/vue` — Vue 3 custom renderer (Phase 2)
│   ├── vite-runtime/        # `@gpjs-ui/vite-runtime` — Vite Runtime API integration for HMR (Phase 3)
│   └── react/               # `@gpjs-ui/react` — React custom renderer, future (Phase 4)
└── examples/
    └── hello-vue/           # sample Vue 3 app
```

This is a plan, not current structure — update this section (or replace it
with the real layout) as each crate/package actually lands, per
[docs/ROADMAP.md](./ROADMAP.md).

## `third_party/` — pinned upstream sources

These are **git submodules pinned to a specific tagged release commit**, not moving branches. Their purpose right now is reference/study material during development (e.g. reading how `gpui` or `rquickjs` implement something) — whether any of this ends up wired into the actual build (git submodule vs. crates.io dependency) is still undecided.

| Path | Upstream | Pinned at | Why it's here |
|---|---|---|---|
| `third_party/zed` | [zed-industries/zed](https://github.com/zed-industries/zed) | `v1.17.2` | Source of the `gpui` crate this project builds its rendering on. |
| `third_party/rquickjs` | [DelSkayn/rquickjs](https://github.com/DelSkayn/rquickjs) | `v0.12.2` | Rust bindings to QuickJS this project uses as its JS runtime. |
| `third_party/rquickjs/sys/quickjs` | [quickjs-ng/quickjs](https://github.com/quickjs-ng/quickjs) | commit pinned by rquickjs `v0.12.2` | rquickjs's own nested submodule — the actual QuickJS engine (the actively-maintained `quickjs-ng` fork, not Bellard's original `bellard/quickjs`). Not the same tree as `bellard/quickjs`; add that separately if it's ever needed for comparison. |

All are registered **shallow** (`submodule.<name>.shallow = true` in the relevant `.gitmodules`) since full history is large and irrelevant here.

- Clone this repo with submodules: `git clone --recurse-submodules <url>` (respects shallow settings).
- Init/update after a plain clone: `git submodule update --init --depth 1 --recursive` — `--recursive` is required to reach `third_party/rquickjs/sys/quickjs`, since it's registered in rquickjs's own `.gitmodules`, not this repo's.
- `third_party/rquickjs/sys/quickjs/test262` (the ECMA-262 conformance test suite, very large) is intentionally left uninitialized — it's not needed here.
- To bump a pin: `cd` into the submodule, `git fetch --depth 1 origin <new-tag> && git checkout FETCH_HEAD`, then commit the updated gitlink from the superproject. Always pin to a tag/commit, never track a branch.
