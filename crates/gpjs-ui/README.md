# gpjs-ui

The Rust host engine for gpjs-ui: a [`gpui`](https://www.gpui.rs/)-based
window and render host embedding a QuickJS runtime (via
[`rquickjs`](https://github.com/DelSkayn/rquickjs)). It exposes a host
bridge (`__gpjsui_native__`) that JS-side renderers drive to build and
mutate a retained virtual tree, which this crate renders directly with
`gpui`.

The tree is single-parent and freed explicitly: attaching a node detaches
it from where it was, and `destroyNode` is what releases one and everything
below it. Input reaches JS through callbacks registered per `(node, event)`,
and a callback that throws is reported rather than swallowed.
