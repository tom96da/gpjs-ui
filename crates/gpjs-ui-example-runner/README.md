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

This crate loads one fixed, already-bundled file and nothing else: it has no
knowledge of Vite, dev servers, or HMR, and never watches for changes.
Rebuild the bundle and re-run it to pick up an edit.
