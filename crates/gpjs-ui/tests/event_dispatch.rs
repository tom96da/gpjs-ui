// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration tests for `gpjs_ui::render::bridge`'s event dispatch: a real
//! `VirtualTree`, wired to a real QuickJS callback via the
//! `__gpjsui_callbacks__` convention (see `EventDispatcher`'s doc comment),
//! rendered through `render_tree_with_events`.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{Context, Modifiers, Render, TestAppContext, VisualTestContext, Window, point, px, prelude::*};

use gpjs_ui::{Engine, EventDispatcher, Host, NodeId, render_tree_with_events};

fn build_clickable_tree() -> (Rc<RefCell<Host>>, NodeId) {
    let host = Rc::new(RefCell::new(Host::default()));
    let mut host_mut = host.borrow_mut();
    let node = host_mut.tree.create_node("div");
    host_mut.tree.set_style(node, "width", 100.0).unwrap();
    host_mut.tree.set_style(node, "height", 100.0).unwrap();
    host_mut.listeners.register(node, "click", 0);
    drop(host_mut);
    (host, node)
}

fn install_click_counter(engine: &Engine) {
    engine
        .eval::<()>(
            "globalThis.__gpjsui_callbacks__ = { \
                0: () => { globalThis.clicks = (globalThis.clicks || 0) + 1; } \
            };",
        )
        .unwrap();
}

fn clicks(engine: &Engine) -> f64 {
    engine.eval::<f64>("globalThis.clicks || 0").unwrap()
}

/// Building the element tree (what happens on every re-render) must never
/// touch the JS engine by itself — only an actual dispatched event may.
/// Plain `#[test]`: building an `AnyElement` needs no `gpui` App/Window.
#[test]
fn repeated_builds_with_no_event_never_touch_the_js_engine() {
    let (host, node) = build_clickable_tree();
    let engine = Rc::new(Engine::new().unwrap());
    install_click_counter(&engine);
    let dispatcher = EventDispatcher::new(Rc::clone(&engine), Rc::clone(&host));

    for _ in 0..5 {
        let host = host.borrow();
        let _ = render_tree_with_events(&host.tree, node, &dispatcher).unwrap();
    }

    assert_eq!(
        clicks(&engine),
        0.0,
        "building the element tree must never call into JS on its own"
    );
}

struct ClickableRoot {
    host: Rc<RefCell<Host>>,
    node: NodeId,
    dispatcher: EventDispatcher,
}

impl Render for ClickableRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let host = self.host.borrow();
        render_tree_with_events(&host.tree, self.node, &self.dispatcher).unwrap()
    }
}

// Note: this doesn't assert on `ClickableRoot`'s render count. A click on
// any interactive element already makes GPUI redraw for its own hover/
// active bookkeeping, regardless of whether `dispatch()` also calls
// `window.refresh()` — confirmed empirically, not just assumed — so a
// render-count assertion here couldn't actually isolate `dispatch()`'s own
// effect. The JS-side call count below is the real, unambiguous signal.
#[gpui::test]
fn click_dispatches_to_js_exactly_once(cx: &mut TestAppContext) {
    let (host, node) = build_clickable_tree();
    let engine = Rc::new(Engine::new().unwrap());
    install_click_counter(&engine);
    let dispatcher = EventDispatcher::new(Rc::clone(&engine), Rc::clone(&host));

    let window = cx.add_window(|_, _| ClickableRoot {
        host: Rc::clone(&host),
        node,
        dispatcher,
    });

    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    assert_eq!(clicks(&engine), 0.0, "mounting must never touch the JS engine");

    let mut cx = VisualTestContext::from_window(window.into(), cx);
    cx.simulate_click(point(px(10.0), px(10.0)), Modifiers::none());
    cx.run_until_parked();

    assert_eq!(
        clicks(&engine),
        1.0,
        "exactly one click must dispatch exactly one JS call"
    );
}
