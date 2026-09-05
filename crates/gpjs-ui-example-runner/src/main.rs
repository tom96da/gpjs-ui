// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Generic loader for gpjs-ui's Vue example apps (`examples/hello_world`,
//! `examples/click_counter`, ...): given the path to one of their prebuilt,
//! self-contained JS bundles, this installs `__gpjsui_native__` bindings,
//! `eval_module`s the bundle, and opens a GPUI window rendering whatever
//! tree the bundle mounted.
//!
//! Not `gpjs-ui-cli` (a separate, future crate that will manage a *live*
//! Vite/HMR dev process for arbitrary apps) — this only ever loads one
//! fixed, already-bundled file, with no knowledge of Vite, dev servers, or
//! HMR.
//!
//! Always renders via `render_tree_with_events` + `EventDispatcher`, never
//! plain `render_tree`: since one binary has to handle any example
//! generically, it can't know ahead of time whether the loaded bundle
//! registered any click handlers. The event-aware path is a no-op for a
//! bundle that registers none.

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
/// `root` is the empty container this runner creates and hands to the
/// bundle to `mount()` against — the mounted app becomes `root`'s first
/// (and only) child, never `root` itself, so `width`/`height` are read from
/// that child's style, not `root`'s.
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

/// Substituted with the real root [`NodeId`] before `eval_module`. A bundle
/// read from disk is full of literal `{`/`}` characters, so — unlike
/// `click_counter.rs`, which interpolates ids straight into a short JS
/// literal via `format!` — a plain `str::replace` on this token is used
/// instead; `format!` on a whole bundle's text would panic on those braces.
const ROOT_ID_PLACEHOLDER: &str = "__GPJSUI_ROOT_ID__";

struct ExampleApp {
    host: Rc<RefCell<Host>>,
    root: NodeId,
    dispatcher: EventDispatcher,
}

impl Render for ExampleApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let host = self.host.borrow();
        render_tree_with_events(&host.tree, self.root, &self.dispatcher).unwrap()
    }
}

fn run_example(bundle_path: &str) {
    let bundle = fs::read_to_string(bundle_path)
        .unwrap_or_else(|err| panic!("failed to read {bundle_path}: {err}"));

    application().run(move |cx: &mut App| {
        let host = Rc::new(RefCell::new(Host::default()));
        let root = host.borrow_mut().tree.create_node("div");

        let engine = Engine::new().unwrap();
        engine.with(|ctx| install(&ctx, &host)).unwrap();

        let source = bundle.replace(ROOT_ID_PLACEHOLDER, &root.to_string());
        engine.eval_module("bundle.mjs", &source).unwrap();

        let dispatcher = EventDispatcher::new(Rc::new(engine), Rc::clone(&host));

        let (width, height) = content_window_size(&host.borrow(), root);
        let bounds = Bounds::centered(None, size(px(width), px(height)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| ExampleApp {
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
        eprintln!("usage: gpjs-ui-example-runner <path-to-bundle.js>");
        return ExitCode::FAILURE;
    };

    run_example(&bundle_path);
    ExitCode::SUCCESS
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn content_window_size_reads_the_mounted_root_childs_style() {
        let mut host = Host::default();
        let root = host.tree.create_node("div");
        let content = host.tree.create_node("div");
        host.tree.set_style(content, "width", 300.0).unwrap();
        host.tree.set_style(content, "height", 150.0).unwrap();
        host.tree.append_child(root, content).unwrap();

        assert_eq!(content_window_size(&host, root), (300.0, 150.0));
    }

    #[test]
    fn content_window_size_falls_back_when_unset() {
        let mut host = Host::default();
        let root = host.tree.create_node("div");
        let content = host.tree.create_node("div");
        host.tree.append_child(root, content).unwrap();

        assert_eq!(content_window_size(&host, root), DEFAULT_WINDOW_SIZE);
    }

    #[test]
    fn content_window_size_falls_back_when_nothing_mounted() {
        let mut host = Host::default();
        let root = host.tree.create_node("div");

        assert_eq!(content_window_size(&host, root), DEFAULT_WINDOW_SIZE);
    }
}
