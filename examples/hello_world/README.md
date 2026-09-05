# hello_world

A Vue port of [`crates/gpjs-ui/examples/hello_world.rs`](../../crates/gpjs-ui/examples/hello_world.rs):
a static tree — a bordered box, a text label, and a row of six colored
squares — built as a real `.vue` SFC (`src/hello_world.vue`) instead of
Rust `VirtualTree` calls.

```sh
pnpm --filter hello_world build
cargo run -p gpjs-ui-example-runner -- examples/hello_world/dist/bundle.js
```

`scripts/build.mjs` is a one-shot, ahead-of-time build (no Vite dev server —
that's Phase 3's job): it compiles the SFC via `@vue/compiler-sfc` directly,
then bundles the result together with `@gpjs-ui/vue` and `@vue/runtime-core`
into one self-contained `dist/bundle.js`, consumable by
[`gpjs-ui-example-runner`](../../crates/gpjs-ui-example-runner/README.md).
