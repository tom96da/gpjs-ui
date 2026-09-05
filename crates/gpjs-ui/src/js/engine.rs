// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

//! `QuickJS` runtime bootstrap.
//!
//! An [`Engine`] pairs one `QuickJS` runtime with one execution context. Each
//! `Engine` is fully independent: creating a global on one has no effect on
//! any other `Engine`, since they don't share a runtime or a heap.

use rquickjs::{Context, Ctx, FromJs, Module, Runtime};

pub use rquickjs::Error as EngineError;

pub type EngineResult<T> = Result<T, EngineError>;

pub struct Engine {
    // Kept alive for the lifetime of `context`, which internally holds a
    // reference-counted handle back to it; QuickJS ties runtime-wide state
    // (the heap, GC) to this handle rather than to the context.
    _runtime: Runtime,
    context: Context,
}

impl Engine {
    /// Creates a new engine with a fresh `QuickJS` runtime and context, with
    /// the standard set of built-in JS intrinsics (`Array`, `JSON`, ...)
    /// enabled.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying `QuickJS` runtime or context fails
    /// to initialize.
    pub fn new() -> EngineResult<Self> {
        let runtime = Runtime::new()?;
        let context = Context::full(&runtime)?;
        Ok(Self {
            _runtime: runtime,
            context,
        })
    }

    /// Evaluates `source` as a JS script and converts its completion value
    /// to `V`. A syntax error, a thrown exception, or a value that can't
    /// convert to `V` are all returned as an `Err`, never a panic.
    ///
    /// # Errors
    ///
    /// Returns an error for a JS syntax error, an uncaught thrown exception,
    /// or a completion value that doesn't convert to `V`.
    pub fn eval<V>(&self, source: &str) -> EngineResult<V>
    where
        V: for<'js> FromJs<'js>,
    {
        self.context.with(|ctx| ctx.eval(source))
    }

    /// Runs `f` with access to this engine's [`Ctx`], for operations `eval`
    /// doesn't cover — such as registering native functions on the global
    /// object.
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Ctx<'_>) -> R,
    {
        self.context.with(f)
    }

    /// Declares and evaluates `source` as an ES module named `name`, driving
    /// its top-level evaluation to completion before returning.
    ///
    /// This engine has no `ModuleLoader` installed, so `source` must be
    /// fully self-contained — no unresolved `import`s for anything to
    /// resolve them against. A module's own completion value is always
    /// `undefined` per spec, so unlike [`eval`](Self::eval) there's nothing
    /// meaningful to convert to a caller-chosen type; state comes back the
    /// same way `examples/click_counter.rs` already does it for plain
    /// scripts — the module's top-level code writes to `globalThis`, and a
    /// separate `eval` call reads it back afterward.
    ///
    /// # Errors
    ///
    /// Returns an error for a syntax/link error, an uncaught exception
    /// thrown during evaluation, or an unsettled promise if the module
    /// awaits something with no pending job left to drive it (not expected
    /// for a self-contained module with no top-level `await`).
    pub fn eval_module(&self, name: &str, source: &str) -> EngineResult<()> {
        self.context.with(|ctx| {
            let (_module, promise) = Module::declare(ctx, name, source)?.eval()?;
            promise.finish()
        })
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn eval_smoke_test() {
        let engine = Engine::new().unwrap();
        let result: i32 = engine.eval("1 + 2").unwrap();
        assert_eq!(result, 3);
    }

    #[test]
    fn syntax_error_propagates_as_err() {
        let engine = Engine::new().unwrap();
        let result: EngineResult<i32> = engine.eval("1 +");
        assert!(result.is_err());
    }

    #[test]
    fn two_engines_do_not_share_globals() {
        let a = Engine::new().unwrap();
        let b = Engine::new().unwrap();

        a.eval::<()>("globalThis.probe = 42;").unwrap();

        let seen_by_a: i32 = a.eval("probe").unwrap();
        assert_eq!(seen_by_a, 42);

        let seen_by_b: String = b.eval("typeof probe").unwrap();
        assert_eq!(seen_by_b, "undefined");
    }

    #[test]
    fn eval_module_runs_top_level_code_to_completion() {
        let engine = Engine::new().unwrap();
        engine
            .eval_module(
                "probe.mjs",
                "export const answer = 41; globalThis.seen = answer + 1;",
            )
            .unwrap();

        let seen: i32 = engine.eval("globalThis.seen").unwrap();
        assert_eq!(seen, 42);
    }

    #[test]
    fn eval_module_uncaught_exception_propagates_as_err() {
        let engine = Engine::new().unwrap();
        let result = engine.eval_module("throws.mjs", "throw new Error('boom');");
        assert!(result.is_err());
    }
}
