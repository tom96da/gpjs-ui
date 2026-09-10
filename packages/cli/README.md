# @gpjs-ui/cli

The `gpjsui` command: the only package that knows both `@gpjs-ui/vite` and
`@gpjs-ui/host-client` exist, wiring one's bundler watch to the other's
dev protocol client.

`gpjsui dev` watches an app's entry point, starts `gpjs-ui-host` once the
first bundle lands, and reloads it on every rebuild. `@gpjs-ui/vite` is
the bundler wired in by default — swapping it for another `Bundler`
(the contract this package exports as a type) is a dependency change
here, not an edit anywhere else.

## App entry

No entry point is required. Drop a `src/App.vue` and that's a whole app —
`gpjsui` wraps it in `createGpjsuiApp(App).mount()` itself. Commit a
`src/main.mts` instead for full control over bootstrapping; it wins
outright when both exist.
