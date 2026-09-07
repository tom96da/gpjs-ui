// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The host objects gpjs-ui installs into a `QuickJS` realm.
//!
//! `QuickJS` supplies the language — `Array`, `JSON`, `Promise` and the rest
//! of the ECMAScript intrinsics — and nothing around it. Everything a script
//! expects the *host* to provide starts here.
//!
//! Depends on `rquickjs` and nothing else, so what it installs can be
//! replaced without reaching the code that draws a UI.

pub mod console;

mod inspect;
