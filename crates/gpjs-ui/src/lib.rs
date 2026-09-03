// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod js;
pub mod render;
pub mod tree;

pub use js::bindings::{EventListeners, Host};
pub use js::engine::{Engine, EngineError, EngineResult};
pub use render::element::{
    AlignSpec, DisplaySpec, ElementSpec, ElementTag, FlexDirectionSpec, LengthSpec, StyleSpec,
    build_element, build_spec, render_tree,
};
pub use tree::{AttributeValue, NodeId, TreeError, VirtualNode, VirtualTree};
