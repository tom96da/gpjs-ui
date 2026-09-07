// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Runtime binary behind a gpjs-ui app: loads one prebuilt, self-contained
//! JS bundle and opens a GPUI window on whatever tree it mounts.
//!
//! One binary serves any app, so it cannot know whether a bundle registers
//! input handlers; it always renders through `EventDispatcher`, which wires
//! only the nodes something listens to.
//!
//! `--dev` additionally answers messages on stdin and writes them on stdout
//! — see `protocol.rs`. stdout is then the message channel and nothing else
//! may go there, so every diagnostic, and the app's own `console`, go to
//! stderr.
//!
//! Nothing here panics on a failure a user can cause.

mod protocol;

use std::cell::{Cell, RefCell};
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::process::ExitCode;
use std::rc::Rc;
use std::thread;

use gpui::{
    App, Bounds, Context, Window, WindowBounds, WindowHandle, WindowOptions, div, prelude::*, px,
    size,
};
use gpui_platform::application;

use serde_json::Value;

use gpjs_ui::js::bindings::install;
use gpjs_ui::{
    AttributeValue, Engine, EngineError, ErrorReporter, EventDispatcher, Host, NodeId,
    drain_jobs_and_refresh, render_tree_with_events,
};
use gpjs_ui_jsenv::console;

use crate::protocol::{ErrorCode, Incoming, Method, Outgoing};

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

/// One loaded bundle: the engine running its JS, the tree that JS built, and
/// the dispatcher wiring events back. A reload replaces all of it at once,
/// so it travels together and a half-swapped state cannot exist.
struct Session {
    engine: Rc<Engine>,
    host: Rc<RefCell<Host>>,
    root: NodeId,
    dispatcher: EventDispatcher,
}

impl Session {
    /// Starts an engine, gives it everything a bundle expects to find, and
    /// evaluates the bundle into a fresh tree.
    ///
    /// `console` goes in before the bundle runs, so a bundle that logs while
    /// evaluating is heard rather than met with a `ReferenceError`.
    ///
    /// Dropping a previous `Session` takes its whole `QuickJS` runtime with
    /// it, so no node, listener or callback survives a reload.
    ///
    /// # Errors
    ///
    /// Returns the thrown value if the engine fails to start, the bindings
    /// or `console` fail to install, or `bundle` throws while evaluating.
    fn load(bundle: &str, reporter: ErrorReporter) -> Result<Self, EngineError> {
        let host = Rc::new(RefCell::new(Host::default()));
        let root = host.borrow().root;

        let engine = Engine::new()?;
        engine.with(|ctx| {
            console::install(&ctx, &console::to_stderr())
                .and_then(|()| install(&ctx, &host))
                .map_err(|err| EngineError::capture(&ctx, &err))
        })?;
        engine.eval_module("bundle.mjs", bundle)?;

        let engine = Rc::new(engine);
        let dispatcher =
            EventDispatcher::new(Rc::clone(&engine), Rc::clone(&host)).with_reporter(reporter);
        Ok(Self {
            engine,
            host,
            root,
            dispatcher,
        })
    }
}

struct HostedApp {
    session: Session,
}

impl Render for HostedApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let session = &self.session;
        let host = session.host.borrow();
        render_tree_with_events(&host.tree, session.root, &session.dispatcher)
            .unwrap_or_else(|| div().into_any_element())
    }
}

/// Where an app's own failures go. In dev the client is listening, and a
/// fault it can show beats a line it has to notice in a log.
fn reporter_for(dev: bool) -> ErrorReporter {
    if dev {
        Rc::new(|err: &EngineError| send(&Outgoing::app_error(err)))
    } else {
        gpjs_ui::stderr_reporter()
    }
}

/// Why a request could not be answered with a result.
#[derive(Debug)]
enum Failure {
    /// Something outside JS: an unreadable file, a window that went away.
    Message(ErrorCode, String),
    /// A value the app threw, which carries its own frames.
    Thrown(EngineError),
}

/// Writes one protocol line to stdout. A failure here means the parent is
/// gone, which the stdin reader notices on its own.
fn send(message: &Outgoing) {
    match protocol::encode(message) {
        Ok(line) => {
            let mut stdout = io::stdout().lock();
            if let Err(err) = writeln!(stdout, "{line}").and_then(|()| stdout.flush()) {
                log::warn!("failed to write to stdout: {err}");
            }
        }
        Err(err) => log::error!("failed to encode {message:?}: {err}"),
    }
}

/// Reads stdin on its own thread, because `gpui`'s `AsyncApp` isn't `Send`
/// and a blocking read must not sit on the main thread. The channel closing
/// means end of input: the parent went away.
fn stdin_lines() -> async_channel::Receiver<String> {
    let (sender, receiver) = async_channel::unbounded();
    thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            match line {
                Ok(line) => {
                    if sender.send_blocking(line).is_err() {
                        break;
                    }
                }
                Err(err) => {
                    log::error!("failed to read stdin: {err}");
                    break;
                }
            }
        }
    });
    receiver
}

/// Answers the request `id` came from. A notification carries no id and
/// takes no reply.
fn respond(id: Option<&Value>, outcome: Result<(), Failure>) {
    let Some(id) = id else { return };
    send(&match outcome {
        Ok(()) => Outgoing::result(id.clone()),
        Err(Failure::Message(code, message)) => Outgoing::error(id.clone(), code, message),
        Err(Failure::Thrown(err)) => Outgoing::thrown(id.clone(), ErrorCode::BundleFailed, &err),
    });
}

/// Re-reads the bundle and evaluates it into a fresh [`Session`], swapping
/// the window over only once that succeeds — a bundle that fails to load
/// leaves the last working one on screen.
fn reload(
    window: &WindowHandle<HostedApp>,
    cx: &mut gpui::AsyncApp,
    bundle_path: &str,
    id: Option<&Value>,
) {
    let outcome = fs::read_to_string(bundle_path)
        .map_err(|err| {
            Failure::Message(
                ErrorCode::BundleFailed,
                format!("failed to read {bundle_path}: {err}"),
            )
        })
        .and_then(|bundle| Session::load(&bundle, reporter_for(true)).map_err(Failure::Thrown))
        .and_then(|session| {
            window
                .update(cx, |app, window, _| {
                    app.session = session;
                    drain_jobs_and_refresh(&app.session.engine, window);
                })
                .map_err(|err| Failure::Message(ErrorCode::BundleFailed, err.to_string()))
        });

    respond(id, outcome);
}

/// Answers protocol messages until `shutdown`, or until the parent closes
/// stdin, then quits the app.
fn serve_dev_protocol(cx: &mut App, window: WindowHandle<HostedApp>, bundle_path: String) {
    let lines = stdin_lines();
    cx.spawn(async move |cx: &mut gpui::AsyncApp| {
        while let Ok(line) = lines.recv().await {
            let (id, method) = match protocol::decode(&line) {
                Ok(Incoming::Call { id, method }) => (id, method),
                Ok(Incoming::Empty) => continue,
                Err((code, message)) => {
                    send(&Outgoing::error(Value::Null, code, message));
                    continue;
                }
            };

            match method {
                Method::Reload => reload(&window, cx, &bundle_path, id.as_ref()),
                Method::Shutdown => {
                    respond(id.as_ref(), Ok(()));
                    break;
                }
                Method::Unknown => respond(
                    id.as_ref(),
                    Err(Failure::Message(
                        ErrorCode::MethodNotFound,
                        "unknown method".to_owned(),
                    )),
                ),
            }
        }
        cx.update(|cx| cx.quit());
    })
    .detach();
}

/// Brings up the engine, the tree and the window.
///
/// # Errors
///
/// Returns the window that failed to open, or the value the bundle threw.
fn start(cx: &mut App, bundle: &str, dev: bool) -> Result<WindowHandle<HostedApp>, Failure> {
    let session = Session::load(bundle, reporter_for(dev)).map_err(Failure::Thrown)?;
    let (width, height) = content_window_size(&session.host.borrow(), session.root);

    let bounds = Bounds::centered(None, size(px(width), px(height)), cx);
    let window = cx
        .open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| HostedApp { session }),
        )
        .map_err(|err| Failure::Message(ErrorCode::BundleFailed, err.to_string()))?;
    cx.activate(true);

    // A reactivity scheduler batches its first effects into a microtask, so
    // mounting leaves work queued that nothing else would come back for
    // until the first input event — or never, in an app that takes none.
    window
        .update(cx, |app, window, _| {
            drain_jobs_and_refresh(&app.session.engine, window);
        })
        .map_err(|err| Failure::Message(ErrorCode::BundleFailed, err.to_string()))?;

    Ok(window)
}

/// Reports a startup failure wherever anyone is listening. There is no
/// window to keep, so this is the last thing the process says.
fn report_startup_failure(failure: &Failure, dev: bool) {
    match (failure, dev) {
        (Failure::Thrown(err), true) => {
            send(&Outgoing::thrown(Value::Null, ErrorCode::BundleFailed, err));
        }
        (Failure::Thrown(err), false) => eprintln!("{err}"),
        (Failure::Message(code, message), true) => {
            send(&Outgoing::error(Value::Null, *code, message.clone()));
        }
        (Failure::Message(_, message), false) => eprintln!("{message}"),
    }
}

fn run_bundle(bundle_path: &str, dev: bool) -> ExitCode {
    let bundle = match fs::read_to_string(bundle_path) {
        Ok(bundle) => bundle,
        Err(err) => {
            eprintln!("failed to read {bundle_path}: {err}");
            return ExitCode::FAILURE;
        }
    };
    let bundle_path = bundle_path.to_owned();

    // `run` blocks until the app quits, so the outcome comes back out
    // through a cell rather than a return value.
    let failed = Rc::new(Cell::new(false));
    let reported = Rc::clone(&failed);
    application().run(move |cx: &mut App| match start(cx, &bundle, dev) {
        Ok(window) => {
            if dev {
                send(&Outgoing::ready());
                serve_dev_protocol(cx, window, bundle_path.clone());
            }
        }
        Err(failure) => {
            report_startup_failure(&failure, dev);
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

    let args: Vec<String> = env::args().skip(1).collect();
    let (dev, bundle_path) = match args.as_slice() {
        [path] => (false, path),
        [flag, path] if flag == "--dev" => (true, path),
        _ => {
            eprintln!("usage: gpjs-ui-host [--dev] <path-to-bundle.js>");
            return ExitCode::FAILURE;
        }
    };

    run_bundle(bundle_path, dev)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    fn load(bundle: &str) -> Result<Session, EngineError> {
        Session::load(bundle, reporter_for(false))
    }

    #[test]
    fn a_bundle_can_log_while_it_evaluates() {
        assert!(
            load("console.log('mounting', { ready: true });").is_ok(),
            "console has to exist before the bundle runs, not after"
        );
    }

    #[test]
    fn a_bundle_that_throws_yields_nothing() {
        let err = load("throw new Error('boom');").err().unwrap();

        assert_eq!(err.message(), "Error: boom");
        assert!(
            err.stack().is_some(),
            "which the client reports as data.stack"
        );
    }

    const DEFERS_ITS_MOUNT: &str = r"
        globalThis.mounted = false;
        Promise.resolve().then(() => { globalThis.mounted = true; });
    ";

    #[test]
    fn evaluating_a_bundle_leaves_a_queued_microtask_pending() {
        let session = load(DEFERS_ITS_MOUNT).unwrap();

        assert!(
            !session.engine.eval::<bool>("globalThis.mounted;").unwrap(),
            "which is what start has to drain once the window is up"
        );
    }

    #[gpui::test]
    fn bringing_the_window_up_runs_what_mounting_only_queued(cx: &mut TestAppContext) {
        cx.update(|cx| start(cx, DEFERS_ITS_MOUNT, false).unwrap());
        cx.run_until_parked();

        let ran: bool = cx.update(|cx| {
            cx.windows()
                .first()
                .and_then(|window| {
                    window
                        .downcast::<HostedApp>()?
                        .read_with(cx, |app, _| {
                            app.session.engine.eval::<bool>("globalThis.mounted;").ok()
                        })
                        .ok()
                        .flatten()
                })
                .unwrap_or(false)
        });

        assert!(ran, "an onMounted-style effect must not wait for an event");
    }

    // Attaches one node under the root and registers a listener on it, so a
    // reload has both tree and registry state that could leak.
    const MOUNTING_BUNDLE: &str = r"
        const node = __gpjsui_native__.createNode('div');
        __gpjsui_native__.appendChild(__gpjsui_native__.rootNodeId(), node);
        __gpjsui_native__.addEventListener(node, 'click', 0);
    ";

    #[test]
    fn a_reload_leaves_no_stale_nodes_listeners_or_callbacks() {
        let first = load(MOUNTING_BUNDLE).unwrap();
        let first_child = first.host.borrow().tree.get(first.root).unwrap().children()[0];

        let second = load(MOUNTING_BUNDLE).unwrap();
        drop(first);

        let host = second.host.borrow();
        let children = host.tree.get(second.root).unwrap().children();
        assert_eq!(children.len(), 1, "the reloaded tree must not accumulate");
        assert_eq!(
            children[0], first_child,
            "ids restart, so the tree is new rather than appended to"
        );
        assert_eq!(host.listeners.callbacks_for(children[0], "click"), &[0]);
        assert!(
            !second
                .engine
                .eval::<bool>("typeof globalThis.__gpjsui_callbacks__ !== 'undefined';")
                .unwrap(),
            "the fresh engine must not carry the previous callback registry"
        );
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
