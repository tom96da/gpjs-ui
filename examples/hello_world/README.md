# hello_world

A Vue port of [`crates/gpjs-ui/examples/hello_world.rs`](../../crates/gpjs-ui/examples/hello_world.rs):
a static tree — a bordered box, a text label, and a row of six colored
squares — built as a real `.vue` SFC (`src/App.vue`) instead of Rust
`VirtualTree` calls.

```sh
cargo build -p gpjs-ui-host
pnpm --filter hello_world dev
```

`gpjsui dev` (from [`@gpjs-ui/cli`](../../packages/cli/README.md)) builds
`src/App.vue` and every rebuild after, starts
[`gpjs-ui-host`](../../crates/gpjs-ui-host/README.md) once the first build
lands, and reloads it on every following one.
