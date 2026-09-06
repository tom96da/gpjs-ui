# gpjs-ui-host

The runtime binary behind a gpjs-ui app: given the path to a prebuilt,
self-contained JS bundle, it installs `__gpjsui_native__` bindings,
evaluates the bundle as an ES module, and opens a GPUI window rendering
whatever tree the bundle mounted.

```sh
pnpm --filter hello_world build
cargo run -p gpjs-ui-host -- examples/hello_world/dist/bundle.js
```

It loads one fixed, already-bundled file and nothing else: no knowledge of
Vite, dev servers, or HMR, and never watches for changes. Rebuild the
bundle and re-run it to pick up an edit.
