// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

pub mod js;
pub mod render;
pub mod tree;

pub use js::bindings::{EventListeners, Host};
pub use js::engine::{Engine, EngineError, EngineResult};
pub use render::bridge::EventDispatcher;
pub use render::element::{
    AlignSpec, DisplaySpec, ElementSpec, ElementTag, FlexDirectionSpec, LengthSpec, StyleSpec,
    build_element, build_element_with_events, build_spec, render_tree, render_tree_with_events,
};
pub use tree::{AttributeValue, NodeId, TreeError, VirtualNode, VirtualTree};
