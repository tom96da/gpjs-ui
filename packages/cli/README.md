# @gpjs-ui/cli

The `gpjsui` command: the only package that knows both `@gpjs-ui/vite` and
`@gpjs-ui/host-client` exist, wiring one's bundler watch to the other's
dev protocol client.

`gpjsui dev` watches an app's entry point, starts `gpjs-ui-host` once the
first bundle lands, and reloads it on every rebuild. `gpjsui build` runs
the same pipeline once, with the watcher removed and production settings
on, and never starts a host. `gpjsui package` builds the same way, then
pairs the bundle with a prebuilt `gpjs-ui-host` into a distributable
application. `@gpjs-ui/vite` is the bundler wired in by default for all
three — swapping it for another `Bundler` (the contract this package
exports as a type) is a dependency change here, not an edit anywhere else.

## App entry

No entry point is required. Drop a `src/App.vue` and that's a whole app —
`gpjsui` wraps it in `createGpjsuiApp(App).mount()` itself. Commit a
`src/main.mts` instead for full control over bootstrapping; it wins
outright when both exist.

## `gpjsui package`

Emits a platform-native application into `dist/`: a `.app` on macOS, a
plain directory on Linux — other platforms aren't supported yet. Either
way, the layout puts the host binary and `bundle.js` beside each other, so
the app launches with no arguments and no terminal.

It needs a release build of the host to bundle: either install this
workspace's per-platform `@gpjs-ui/host-*` package, or, from a source
checkout, `cargo build -p gpjs-ui-host --release` first.

App metadata comes from the app's own `package.json`, with an optional
`"gpjsui"` key overriding what's derived from it:

```jsonc
{
  "name": "click_counter",
  "version": "1.0.0",
  "gpjsui": {
    "productName": "Click Counter", // defaults to "name", scope stripped
    "identifier": "com.example.click-counter", // defaults to a generated org.gpjsui.<slug>
    "icon": "assets/icon.icns" // resolved relative to the app's own directory
  }
}
```

An `identifier` should be world-unique, so `gpjsui package` prints a note
when it falls back to the generated one rather than using it silently.
