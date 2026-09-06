// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Runtime binary behind a gpjs-ui app: given the path to a prebuilt,
//! self-contained JS bundle, this installs `__gpjsui_native__` bindings,
//! `eval_module`s the bundle, and opens a GPUI window rendering whatever
//! tree the bundle mounted.
//!
//! Loads one fixed, already-bundled file and nothing else: no knowledge of
//! Vite, dev servers, or HMR, and no change watching.
//!
//! Always renders via `render_tree_with_events` + `EventDispatcher`, never
//! plain `render_tree`: since one binary has to handle any app generically,
//! it can't know ahead of time whether the loaded bundle registered any
//! click handlers. The event-aware path is a no-op for a bundle that
//! registers none.

#![allow(clippy::unwrap_used)]

use std::cell::RefCell;
use std::env;
use std::fs;
use std::process::ExitCode;
use std::rc::Rc;

use gpui::{App, Bounds, Context, Window, WindowBounds, WindowOptions, prelude::*, px, size};
use gpui_platform::application;

use gpjs_ui::js::bindings::install;
use gpjs_ui::{AttributeValue, Engine, EventDispatcher, Host, NodeId, render_tree_with_events};

/// Window size to fall back to when the mounted app's root element doesn't
/// declare an explicit `width`/`height` style (e.g. a fully fluid layout).
const DEFAULT_WINDOW_SIZE: (f32, f32) = (800.0, 600.0);

/// Reads the window size straight from the app the bundle mounted, so the
/// window fits its content instead of leaving a black margin around a
/// smaller (or clipping a larger) fixed-size app.
///
/// `root` is the empty container the [`Host`] allocates for the bundle to
/// `mount()` against — the mounted app becomes `root`'s first (and only)
/// child, never `root` itself, so `width`/`height` are read from that
/// child's style, not `root`'s.
fn content_window_size(host: &Host, root: NodeId) -> (f32, f32) {
    let style = host
        .tree
        .get(root)
        .and_then(|node| node.children().first())
        .and_then(|&content_id| host.tree.get(content_id))
        .map(gpjs_ui::VirtualNode::style_props);

    let dimension = |key: &str| {
        style
            .and_then(|props| props.get(key))
            .and_then(|value| match value {
                // A window dimension in px is always far within f32's
                // precision range — no meaningful truncation risk here.
                #[allow(clippy::cast_possible_truncation)]
                AttributeValue::Number(n) => Some(*n as f32),
                _ => None,
            })
    };

    (
        dimension("width").unwrap_or(DEFAULT_WINDOW_SIZE.0),
        dimension("height").unwrap_or(DEFAULT_WINDOW_SIZE.1),
    )
}

struct HostedApp {
    host: Rc<RefCell<Host>>,
    root: NodeId,
    dispatcher: EventDispatcher,
}

impl Render for HostedApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let host = self.host.borrow();
        render_tree_with_events(&host.tree, self.root, &self.dispatcher).unwrap()
    }
}

fn run_bundle(bundle_path: &str) {
    let bundle = fs::read_to_string(bundle_path)
        .unwrap_or_else(|err| panic!("failed to read {bundle_path}: {err}"));

    application().run(move |cx: &mut App| {
        let host = Rc::new(RefCell::new(Host::default()));
        let root = host.borrow().root;

        let engine = Engine::new().unwrap();
        engine.with(|ctx| install(&ctx, &host)).unwrap();

        engine.eval_module("bundle.mjs", &bundle).unwrap();

        let dispatcher = EventDispatcher::new(Rc::new(engine), Rc::clone(&host));

        let (width, height) = content_window_size(&host.borrow(), root);
        let bounds = Bounds::centered(None, size(px(width), px(height)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HostedApp {
                    host,
                    root,
                    dispatcher,
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}

fn main() -> ExitCode {
    env_logger::init();

    let Some(bundle_path) = env::args().nth(1) else {
        eprintln!("usage: gpjs-ui-host <path-to-bundle.js>");
        return ExitCode::FAILURE;
    };

    run_bundle(&bundle_path);
    ExitCode::SUCCESS
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn content_window_size_reads_the_mounted_root_childs_style() {
        let mut host = Host::default();
        let root = host.root;
        let content = host.tree.create_node("div");
        host.tree.set_style(content, "width", 300.0).unwrap();
        host.tree.set_style(content, "height", 150.0).unwrap();
        host.tree.append_child(root, content).unwrap();

        assert_eq!(content_window_size(&host, root), (300.0, 150.0));
    }

    #[test]
    fn content_window_size_falls_back_when_unset() {
        let mut host = Host::default();
        let root = host.root;
        let content = host.tree.create_node("div");
        host.tree.append_child(root, content).unwrap();

        assert_eq!(content_window_size(&host, root), DEFAULT_WINDOW_SIZE);
    }

    #[test]
    fn content_window_size_falls_back_when_nothing_mounted() {
        let host = Host::default();

        assert_eq!(content_window_size(&host, host.root), DEFAULT_WINDOW_SIZE);
    }
}
