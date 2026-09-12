# click_counter

A Vue port of [`crates/gpjs-ui/examples/click_counter.rs`](../../crates/gpjs-ui/examples/click_counter.rs):
a bordered box whose label counts up when clicked, built as a real `.vue`
SFC (`src/App.vue`) with `ref`/`computed` reactivity instead of Rust
`VirtualTree` calls and a hand-written click handler.

```sh
cargo build -p gpjs-ui-host
pnpm --filter click_counter dev
```

`gpjsui dev` (from [`@gpjs-ui/cli`](../../packages/cli/README.md)) builds
`src/App.vue` and every rebuild after, starts
[`gpjs-ui-host`](../../crates/gpjs-ui-host/README.md) once the first build
lands, and reloads it on every following one.

`pnpm --filter click_counter build` runs the same build once, minified and
without starting a host, writing `dist/bundle.js`. Launch the built bundle
directly, without the dev protocol, with:

```sh
cargo run -p gpjs-ui-host -- examples/click_counter/dist/bundle.js
```
