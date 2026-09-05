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
- `cargo test -p gpjs-ui`

## TypeScript (`packages/*`)

### Test placement

Mirrors the Rust split above, using Vitest:

- **Unit tests**: `*.test.mts` co-located next to the module it tests
  (e.g. `packages/gpjs-ui/src/index.test.mts` next to `src/index.mts`).
  These mock `globalThis.__gpjsui_native__`/`__gpjsui_callbacks__` rather
  than driving a real QuickJS engine — see `index.test.mts` for the
  pattern (install a mock native object, assert the wrapper forwards to
  it correctly).
- **Integration tests**: a `tests/` directory at the package root, for
  tests that exercise real cross-module wiring instead of a mock — e.g.
  `@gpjs-ui/vue`'s `createRenderer` driven end-to-end through `nodeOps`
  against a real `packages/gpjs-ui`, not a mocked one. Not needed yet;
  add it when Phase 2 Unit iii's renderer tests need it (see
  [docs/PLAN.md](./PLAN.md)).

### Required checks

Every package under `packages/*` defines the same four scripts, all of
which must pass:

- `lint` — `oxlint`
- `format` — `oxfmt --check .`
- `typecheck` — `oxlint -A all --type-aware --type-check`
- `test` — `vitest run`, gated by a `pretest` script
  (`oxlint --type-aware --type-check && oxfmt --check .`) that pnpm runs
  automatically as its own separate step before `test` — not chained
  into `test` itself. So `pnpm test`/`pnpm -r test` already cover lint,
  type-check, and format; the standalone scripts exist for running just
  one check directly.

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
  that hasn't landed its first test yet (e.g. `packages/vue`, until Unit
  iii's tests land) without treating "zero tests" as a failure.
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
  above, which aren't bundled into `cargo test` itself the way `pretest`
  bundles them on the TypeScript side.
