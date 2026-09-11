# @gpjs-ui/vite

The Vite half of [gpjs-ui](https://github.com/tom96da/gpjs-ui)'s build —
the only package here that imports `vite` — bundling an app's entry point
into one self-contained bundle that `gpjs-ui-host` can evaluate: no
unresolved imports, no dependency on QuickJS having Node.js globals.

`watch(options)` builds `options.entry` into a bundle under
`options.outDir` — the returned `Watcher`'s `bundlePath` names the exact
file — rebuilding it on every change and reporting each result through
`options.onBuild`/`options.onError`. `build(options)` runs the same
pipeline once, minified and with `outDir` cleared first, resolving with
the bundle's path or rejecting on failure rather than reporting it
through a callback. This package never starts, reloads, or talks to
`gpjs-ui-host` itself — driving that from a build is `@gpjs-ui/cli`'s
job, which is also the only place either package name is written down,
so swapping bundlers is a dependency change there rather than an edit
here.

The template compiler is retargeted at `@vue/runtime-core` — the only Vue
runtime package this repo's own apps depend on — instead of the default
`vue` import. `@vitejs/plugin-vue` itself still imports `vue` directly for
its own internals, which is why it's a peer dependency here: this
package's own build-time need for `vue` never reaches an app's dependency
tree or the bundle it produces.
