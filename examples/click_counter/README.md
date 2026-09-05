# click_counter

A Vue port of [`crates/gpjs-ui/examples/click_counter.rs`](../../crates/gpjs-ui/examples/click_counter.rs):
a bordered box whose label counts up when clicked, built as a real `.vue`
SFC (`src/click_counter.vue`) with `ref`/`computed` reactivity instead of
Rust `VirtualTree` calls and a hand-written click handler.

```sh
pnpm --filter click_counter build
cargo run -p gpjs-ui-example-runner -- examples/click_counter/dist/bundle.js
```

`scripts/build.mjs` is a one-shot, ahead-of-time build (no Vite dev server):
it compiles the SFC via `@vue/compiler-sfc` directly, then bundles the result
together with `@gpjs-ui/vue` and `@vue/runtime-core` into one self-contained
`dist/bundle.js`, consumable by
[`gpjs-ui-example-runner`](../../crates/gpjs-ui-example-runner/README.md).
