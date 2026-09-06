# @gpjs-ui/vue

Vue 3 custom renderer for [gpjs-ui](https://github.com/tom96da/gpjs-ui),
an ultra-lightweight, Webview-free desktop application framework powered by
GPUI and QuickJS. It maps Vue's `createRenderer` lifecycle methods
(`nodeOps`, `patchProp`) onto the `gpjs-ui` package's typed wrapper
functions, never talking to the native host bridge directly, and exposes
`createGpjsuiApp` — `@vue/runtime-core`'s `createApp`, fixed to mount
against this renderer's host-node handles instead of a DOM element.
`app.mount()` with no argument targets the host's own root container.

The renderer's node lifecycle (`createElement`/`insert`/`remove`, text and
comment nodes, prop patching for style/attributes/events) is implemented
and tested.
