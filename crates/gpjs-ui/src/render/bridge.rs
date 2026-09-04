// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Dispatches a native input event into the JS callbacks registered for it
//! via `__gpjsui_native__.addEventListener` (`crate::js::bindings`), then
//! requests a redraw.
//!
//! Zero-overhead by construction: [`EventDispatcher::dispatch`] is the only
//! thing that ever touches the JS engine or calls
//! [`Window::refresh`](gpui::Window::refresh) on this path — attaching a
//! handler to an element (`render/element.rs`'s `build_element_with_events`)
//! only clones a couple of `Rc`s, so a re-render with no new input never
//! runs JS.
//!
//! ## Where the real JS function lives
//!
//! `EventListeners` (`crate::js::bindings`) only ever stores a plain `u32`
//! callback id — never an `rquickjs::Value`/`Function`/`Persistent<T>`, per
//! this project's FFI safety rule against storing those in any long-lived
//! struct. The actual function has to live somewhere, so the convention is:
//! the JS caller stores it itself, at
//! `globalThis.__gpjsui_callbacks__[callbackId]`, before calling
//! `addEventListener` with that id. [`EventDispatcher::dispatch`] looks the
//! real function up fresh inside one `Engine::with` call and drops it
//! before that call returns — it never crosses into Rust-held state.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::Window;
use rquickjs::{Function, Object};

use crate::js::bindings::Host;
use crate::js::engine::Engine;
use crate::tree::NodeId;

/// Everything needed to dispatch a native event into JS: the engine to call
/// into, and the registry of which JS callback ids are listening for which
/// `(node id, event name)`.
#[derive(Clone)]
pub struct EventDispatcher {
    engine: Rc<Engine>,
    host: Rc<RefCell<Host>>,
}

impl EventDispatcher {
    pub fn new(engine: Rc<Engine>, host: Rc<RefCell<Host>>) -> Self {
        Self { engine, host }
    }

    /// Calls every JS callback registered for `(node_id, event)` (via
    /// `__gpjsui_native__.addEventListener`), passing `node_id`, then
    /// requests a redraw. A no-op — touching neither the JS engine nor
    /// `window` — if nothing is registered for `(node_id, event)`. Never
    /// panics: a missing `__gpjsui_callbacks__` registry, a missing entry
    /// in it, a non-function entry, or an exception thrown by the callback
    /// itself are all silently skipped rather than propagated — a bad
    /// listener must not take down the host.
    pub fn dispatch(&self, node_id: NodeId, event: &str, window: &mut Window) {
        let callback_ids = self
            .host
            .borrow()
            .listeners
            .callbacks_for(node_id, event)
            .to_vec();
        if callback_ids.is_empty() {
            return;
        }

        self.engine.with(|ctx| {
            let Ok(callbacks) = ctx.globals().get::<_, Object>("__gpjsui_callbacks__") else {
                return;
            };
            for callback_id in callback_ids {
                if let Ok(callback) = callbacks.get::<_, Function>(callback_id) {
                    let _ = callback.call::<_, ()>((node_id,));
                }
            }
        });

        window.refresh();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::js::bindings::install;
    use gpui::TestAppContext;

    fn dispatcher_with_engine() -> (EventDispatcher, Rc<RefCell<Host>>) {
        let engine = Rc::new(Engine::new().unwrap());
        let host = Rc::new(RefCell::new(Host::default()));
        engine.with(|ctx| install(&ctx, &host)).unwrap();
        (EventDispatcher::new(engine, Rc::clone(&host)), host)
    }

    #[gpui::test]
    fn no_listener_registered_is_a_no_op(cx: &mut TestAppContext) {
        let (dispatcher, host) = dispatcher_with_engine();
        let node_id = host.borrow_mut().tree.create_node("div");

        // Would panic (missing global) if dispatch tried to call into JS.
        let cx = cx.add_empty_window();
        cx.update(|window, _| dispatcher.dispatch(node_id, "click", window));
    }

    #[gpui::test]
    fn missing_callbacks_registry_does_not_panic(cx: &mut TestAppContext) {
        let (dispatcher, host) = dispatcher_with_engine();
        let node_id = host.borrow_mut().tree.create_node("div");
        host.borrow_mut().listeners.register(node_id, "click", 0);

        // No `__gpjsui_callbacks__` global defined at all.
        let cx = cx.add_empty_window();
        cx.update(|window, _| dispatcher.dispatch(node_id, "click", window));
    }

    #[gpui::test]
    fn throwing_callback_does_not_panic(cx: &mut TestAppContext) {
        let (dispatcher, host) = dispatcher_with_engine();
        let node_id = host.borrow_mut().tree.create_node("div");
        host.borrow_mut().listeners.register(node_id, "click", 0);
        dispatcher
            .engine
            .eval::<()>(
                "globalThis.__gpjsui_callbacks__ = { 0: () => { throw new Error('boom'); } };",
            )
            .unwrap();

        let cx = cx.add_empty_window();
        cx.update(|window, _| dispatcher.dispatch(node_id, "click", window));
    }
}
