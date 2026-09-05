# gpjs-ui

The Rust host engine for gpjs-ui: a [`gpui`](https://www.gpui.rs/)-based
window and render host embedding a QuickJS runtime (via
[`rquickjs`](https://github.com/DelSkayn/rquickjs)). It exposes a host
bridge (`__gpjsui_native__`) that JS-side renderers drive to build and
mutate a retained virtual tree, which this crate renders directly with
`gpui`.

The retained tree, QuickJS runtime bootstrap, host bindings, GPUI render
conversion, and click-event dispatch are implemented and tested.
