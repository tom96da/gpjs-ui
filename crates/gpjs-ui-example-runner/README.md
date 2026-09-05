# gpjs-ui-example-runner

A one-shot loader for the Vue example apps under `examples/`
(`examples/hello_world`, `examples/click_counter`): given the path to one
of their prebuilt, self-contained JS bundles, it installs
`__gpjsui_native__` bindings, evaluates the bundle as an ES module, and
opens a GPUI window rendering whatever tree the bundle mounted.

```sh
pnpm --filter hello_world build
cargo run -p gpjs-ui-example-runner -- examples/hello_world/dist/bundle.js
```

This is **not** `gpjs-ui-cli` — a separate, future crate that will manage a
*live* Vite/HMR dev process for arbitrary apps (see
[docs/ROADMAP.md](../../docs/ROADMAP.md)'s Phase 3). This crate only ever
loads one fixed, already-bundled file; it has no knowledge of Vite, dev
servers, or HMR, and isn't meant to grow into that.
