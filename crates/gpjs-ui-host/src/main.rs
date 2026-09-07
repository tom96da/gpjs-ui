// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Runtime binary behind a gpjs-ui app: loads one prebuilt, self-contained
//! JS bundle and opens a GPUI window on whatever tree it mounts.
//!
//! One binary serves any app, so it cannot know whether a bundle registers
//! input handlers; it always renders through `EventDispatcher`, which wires
//! only the nodes something listens to.
//!
//! Nothing here panics on a failure a user can cause.

use std::cell::{Cell, RefCell};
use std::env;
use std::fs;
use std::process::ExitCode;
use std::rc::Rc;

use gpui::{App, Bounds, Context, Window, WindowBounds, WindowOptions, div, prelude::*, px, size};
use gpui_platform::application;

use gpjs_ui::js::bindings::install;
use gpjs_ui::{
    AttributeValue, Engine, EngineError, EventDispatcher, Host, NodeId, render_tree_with_events,
};
use gpjs_ui_jsenv::console;

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
        render_tree_with_events(&host.tree, self.root, &self.dispatcher)
            .unwrap_or_else(|| div().into_any_element())
    }
}

/// Starts an engine, gives it everything a bundle expects to find, and
/// evaluates the bundle into a fresh tree.
///
/// `console` goes in before the bundle runs, so a bundle that logs while
/// evaluating is heard rather than met with a `ReferenceError`.
///
/// # Errors
///
/// Returns the message to report if any of that fails.
fn load(bundle: &str) -> Result<(Engine, Rc<RefCell<Host>>), String> {
    let host = Rc::new(RefCell::new(Host::default()));

    let engine = Engine::new().map_err(|err| err.to_string())?;
    engine
        .with(|ctx| {
            console::install(&ctx, &console::to_stderr())
                .and_then(|()| install(&ctx, &host))
                .map_err(|err| EngineError::capture(&ctx, &err))
        })
        .map_err(|err| err.to_string())?;
    engine
        .eval_module("bundle.mjs", bundle)
        .map_err(|err| err.to_string())?;

    Ok((engine, host))
}

/// Brings up the engine, the tree and the window.
///
/// # Errors
///
/// Returns the message to report if any of that fails.
fn start(cx: &mut App, bundle: &str) -> Result<(), String> {
    let (engine, host) = load(bundle)?;
    let root = host.borrow().root;
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
    .map_err(|err| err.to_string())?;
    cx.activate(true);
    Ok(())
}

fn run_bundle(bundle_path: &str) -> ExitCode {
    let bundle = match fs::read_to_string(bundle_path) {
        Ok(bundle) => bundle,
        Err(err) => {
            eprintln!("failed to read {bundle_path}: {err}");
            return ExitCode::FAILURE;
        }
    };

    // `run` blocks until the app quits, so the outcome comes back out
    // through a cell rather than a return value.
    let failed = Rc::new(Cell::new(false));
    let reported = Rc::clone(&failed);
    application().run(move |cx: &mut App| {
        if let Err(message) = start(cx, &bundle) {
            eprintln!("{message}");
            reported.set(true);
            cx.quit();
        }
    });

    if failed.get() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn main() -> ExitCode {
    env_logger::init();

    let Some(bundle_path) = env::args().nth(1) else {
        eprintln!("usage: gpjs-ui-host <path-to-bundle.js>");
        return ExitCode::FAILURE;
    };

    run_bundle(&bundle_path)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn a_bundle_can_log_while_it_evaluates() {
        assert!(
            load("console.log('mounting', { ready: true });").is_ok(),
            "console has to exist before the bundle runs, not after"
        );
    }

    #[test]
    fn a_bundle_that_throws_yields_nothing() {
        assert!(load("throw new Error('boom');").is_err());
    }

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
