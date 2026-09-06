<!--
Copyright (c) 2026 tom96da
SPDX-License-Identifier: MIT OR Apache-2.0
-->

# Testing conventions

Where tests live, what kind of test goes where, and what must pass before a
change is done — for both languages in this repo, kept as a matched pair.
Complements [docs/GIT.md](./GIT.md)'s repo-wide rules the same way
[docs/FFI.md](./FFI.md) complements [docs/ARCHITECTURE.md](./ARCHITECTURE.md).
Update this file as real conventions land, same as the other docs here.

## Rust (`crates/gpjs-ui`)

### Test placement

- **Unit tests**: `#[cfg(test)] mod tests` inline in the module they test
  (e.g. `src/tree.rs`, `src/js/bindings.rs`, `src/js/engine.rs`).
- **Integration tests**: `tests/*.rs`, one file per cross-module concern
  (`tests/layout_parity.rs`, `tests/event_dispatch.rs`) — Cargo compiles
  each as its own crate against `gpjs-ui`'s public API only, the same
  boundary a real external caller would see.
- **Manual/GUI checks**: tracked in
  [docs/MANUAL_GUI_CHECK.md](./MANUAL_GUI_CHECK.md) instead of automated —
  see that doc for why the devcontainer can't do this alone and how to
  actually run the check.

### Required checks

All of the following must pass, not just `cargo test`:

- `cargo check -p gpjs-ui --all-targets` — the `--all-targets` also
  compile-checks `examples/`, which has no automated test of its own (see
  `crates/gpjs-ui/examples/hello_world.rs`'s doc comment)
- `cargo clippy -p gpjs-ui --all-targets`
- `cargo fmt --all -- --check`
- `cargo test -p gpjs-ui` — `tests/js_core_integration.rs` reads
  `packages/gpjs-ui/dist/index.js` off disk, so run
  `pnpm --filter gpjs-ui build` first, or this one test fails with a
  message saying so
- `cargo check -p gpjs-ui-example-runner --all-targets` and
  `cargo clippy -p gpjs-ui-example-runner --all-targets` — no `cargo test`
  for this crate, it's a manual-check binary like `crates/gpjs-ui`'s own
  examples, not a library with automated tests

### Toolchain pinning and MSRV

`rust-toolchain.toml` pins the toolchain. rustup honours it for every
`cargo`/`rustup` call in this workspace, locally and on CI alike, so one
`rustc` builds everything. GitHub-hosted runner images ship a different
Rust version per OS and update on their own schedule, so the
`ubuntu-latest` and `macos-latest` legs disagree without it.

The pin declares `components` as well, so clippy and rustfmt arrive with
the toolchain. CI still runs `rustup component add` early, to download the
pinned toolchain before `Swatinem/rust-cache` derives its key from
`rustc -vV`. That download is expected on CI: the runner images don't carry
this exact version.

`Cargo.toml`'s `rust-version` is a separate declaration — the oldest
`rustc` this workspace supports — and is held equal to the pin. Bump both
together. It also feeds dependency resolution: `resolver = "3"` is
MSRV-aware and won't pick a dependency version needing a newer `rustc`.

The two can stay equal only while nothing outside this repo compiles
against these crates, which both `publish = false` settings currently
guarantee. Phase 8 ends that (see
[ROADMAP.md](./ROADMAP.md#phase-8-app-owned-rust-extensions-future)):
`rust-version` then drops below the pin and needs its own check, which
installs the floor toolchain and runs `cargo +<msrv> check`, the `+<msrv>`
overriding `rust-toolchain.toml`. That is a second toolchain, so a second
full gpui build under its own `Swatinem/rust-cache` key.

## TypeScript (`packages/*`)

### Test placement

Mirrors the Rust split above, using Vitest:

- **Unit tests**: `*.test.mts` co-located next to the module it tests
  (e.g. `packages/gpjs-ui/src/tree.test.mts` next to `src/tree.mts`).
  These mock `globalThis.__gpjsui_native__`/`__gpjsui_callbacks__` rather
  than driving a real QuickJS engine — see `tree.test.mts` for the
  pattern (install a mock native object, assert the wrapper forwards to
  it correctly). A barrel (`src/index.mts`) re-exports only; its modules
  carry the tests.
- **Integration tests**: a `tests/` directory at the package root, for
  tests that exercise real cross-module wiring against a real (unmocked)
  `packages/gpjs-ui` instead of a mock — e.g.
  `packages/vue/tests/renderer.test.mts`, which drives `createGpjsuiApp`
  end-to-end against a small in-memory fake standing in for
  `globalThis.__gpjsui_native__`, asserting on that fake's resulting tree
  state rather than on individual host-node-lifecycle calls the way the
  co-located unit tests do.

### Required checks

Every package under `packages/*` defines the same four scripts, all of
which must pass:

- `lint` — `oxlint`
- `format` — `oxfmt --check .`
- `typecheck` — `oxlint -A all --type-aware --type-check`
- `test` — `vitest run`

The repo root defines a `pretest`
(`oxlint --type-aware --type-check && oxfmt --check .`) that pnpm runs
automatically as its own separate step before the root's `test` — not
chained into `test` itself. So `pnpm test` from the root already covers
lint, type-check, and format; the standalone scripts exist for running
just one check directly. `pnpm -r test` does not fire it — `-r` skips the
root project — which is what lets CI run those checks as their own steps.

#### Agent-friendly lint/format output

When running these tools directly (not through `pnpm test`), pass
`oxlint`'s `-f`/`--format=agent` (e.g. `oxlint --format=agent`,
`oxlint -A all --type-aware --type-check --format=agent`) for output
meant to be parsed rather than read in a terminal — plain lines, no
decoration. `oxfmt` has no equivalent flag (`--check`'s own output is
already a few plain text lines); its `--write`/`--check`/`--list-different`
modes are the only output-shaping options it has.

### Build-tooling gotchas this split runs into

- Each package's `vite.config.mts` must exclude test files from
  `unplugin-dts`'s declaration scan:
  `dts({ include: ["src"], exclude: ["src/**/*.test.mts"] })`.
  `include` controls what the plugin emits declarations _for_, separate
  from the build's `lib.entry` — without the `exclude`, a co-located test
  file leaks a stray `dist/*.test.d.mts` into the published package.
  Apply the same pattern to any new package's `vite.config.mts`.
- `vitest.config`'s `test.passWithNoTests: true` (already set in every
  package's `vite.config.mts`) lets `pnpm -r test` succeed for a package
  that hasn't landed its first test yet (e.g. a newly scaffolded package,
  before its first test lands) without treating "zero tests" as a
  failure.
- Each package's `exports` carries a `"source"` condition pointing at
  `src/index.mts`, and `tsconfig.base.json` sets
  `customConditions: ["source"]`, so type-checking resolves workspace
  imports from source instead of from a built `dist/`, so the `lint` job
  needs no build step. Runtime resolution is untouched — Vite and
  vitest don't know the condition and fall through to `import` — so
  `pnpm -r test` still needs `pnpm -r build` first. `publishConfig.exports`
  drops the condition again when packing, since `files: ["dist"]` doesn't
  ship `src/`.
- A package's `tsconfig.json` `include` has to list every directory whose
  files are checked, `tests/` included. A file outside it still gets
  linted, but under default compiler options rather than
  `tsconfig.base.json`'s — so `strict` and `customConditions` silently
  don't apply to it.
- A co-located `*.test.mts` importing its sibling module needs the
  literal `.mts` extension (`from "./index.mts"`, not extensionless or
  `.js`/`.mjs`) — see `docs/PLAN.md`'s Phase 2 Unit i/ii notes for why
  `module: "preserve"`'s implied `moduleResolution: "bundler"` requires
  this, and why `tsconfig.base.json` sets
  `allowImportingTsExtensions: true` to allow it.

## Running tests

- Single package: `pnpm --filter <pkg> test` / `typecheck` / `build`
- Whole workspace: `pnpm -r test` / `typecheck` / `build` from the repo
  root
- Rust: `cargo test -p gpjs-ui` (see [AGENTS.md](../AGENTS.md#status)) —
  plus `cargo clippy`/`cargo fmt --check` from the Required checks list
  above, which aren't bundled into `cargo test` itself the way the root
  `pretest` bundles them on the TypeScript side.
