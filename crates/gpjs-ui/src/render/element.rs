// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Converts a retained [`VirtualTree`] into real `gpui` elements.
//!
//! Split into two layers, as tracked in `docs/PLAN.md`:
//! - a pure spec layer ([`ElementSpec`]/[`StyleSpec`]/[`build_spec`]) that
//!   has no `gpui` dependency and is exhaustively unit-testable on its own;
//! - a thin gpui layer ([`build_element`]/[`render_tree`]) that turns that
//!   spec into a real [`AnyElement`].
//!
//! The recognized `style_props`/`attributes` keys are documented in
//! `docs/FFI.md`. Unrecognized keys and malformed values are silently
//! ignored rather than erroring: this runs on the render path, not a JS call
//! boundary, so there's no channel to raise a catchable exception through.

use std::collections::HashMap;

use gpui::prelude::*;
use gpui::{AnyElement, Display, Fill, FlexDirection, Hsla, Length, StyleRefinement, div, px, rgb};

use crate::tree::{AttributeValue, NodeId, VirtualTree};

/// What kind of element a [`VirtualNode`](crate::tree::VirtualNode) maps to.
///
/// Only two kinds exist for now — there's no per-tag dispatch table, since
/// there's exactly one container builder to pick from until a real second
/// element kind is designed.
#[derive(Debug, Clone, PartialEq)]
pub enum ElementTag {
    /// Any tag name other than `"text"`: a generic styled box.
    Container,
    /// A `"text"` tag: a leaf whose content is its `"value"` attribute
    /// (missing or non-string → empty content, never a panic).
    Text(String),
}

/// A length in pixels, or `"auto"`. `gpui`-independent mirror of the subset
/// of [`gpui::Length`] this layer supports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthSpec {
    Px(f64),
    Auto,
}

/// `gpui`-independent mirror of [`gpui::Display`]'s supported variants.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplaySpec {
    Flex,
    Block,
    Grid,
    None,
}

/// `gpui`-independent mirror of [`gpui::FlexDirection`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirectionSpec {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

/// `gpui`-independent mirror of the common subset of `gpui`'s `AlignItems`
/// and `JustifyContent` (`= AlignContent`) enums — the variants both share
/// and that this layer supports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignSpec {
    Start,
    End,
    Center,
    Stretch,
}

/// A plain-data, `gpui`-independent style description, built from a
/// [`VirtualNode`](crate::tree::VirtualNode)'s `style_props`. See
/// `docs/FFI.md` for the recognized keys.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StyleSpec {
    pub display: Option<DisplaySpec>,
    pub flex_direction: Option<FlexDirectionSpec>,
    pub justify_content: Option<AlignSpec>,
    pub align_items: Option<AlignSpec>,
    /// Uniform gap (px) applied to both rows and columns.
    pub gap: Option<f64>,
    pub width: Option<LengthSpec>,
    pub height: Option<LengthSpec>,
    /// Uniform border width (px) applied to all four sides.
    pub border_width: Option<f64>,
    /// `0xRRGGBB`.
    pub background: Option<u32>,
    /// `0xRRGGBB`.
    pub border_color: Option<u32>,
    /// Uniform corner radius (px) applied to all four corners.
    pub corner_radius: Option<f64>,
    /// `0xRRGGBB`. Cascades to descendant text, like `gpui`'s own
    /// `.text_color()` — text leaves carry no style of their own.
    pub text_color: Option<u32>,
    pub text_size: Option<f64>,
}

/// A `gpui`-independent, recursive description of one
/// [`VirtualNode`](crate::tree::VirtualNode) and its subtree.
#[derive(Debug, Clone, PartialEq)]
pub struct ElementSpec {
    pub id: NodeId,
    pub tag: ElementTag,
    pub style: StyleSpec,
    pub children: Vec<ElementSpec>,
}

fn as_str(value: &AttributeValue) -> Option<&str> {
    match value {
        AttributeValue::String(s) => Some(s.as_str()),
        _ => None,
    }
}

fn as_number(value: &AttributeValue) -> Option<f64> {
    match value {
        AttributeValue::Number(n) => Some(*n),
        _ => None,
    }
}

fn length_spec_from(value: &AttributeValue) -> Option<LengthSpec> {
    match value {
        AttributeValue::Number(n) => Some(LengthSpec::Px(*n)),
        AttributeValue::String(s) if s == "auto" => Some(LengthSpec::Auto),
        _ => None,
    }
}

fn display_spec_from_str(s: &str) -> Option<DisplaySpec> {
    match s {
        "flex" => Some(DisplaySpec::Flex),
        "block" => Some(DisplaySpec::Block),
        "grid" => Some(DisplaySpec::Grid),
        "none" => Some(DisplaySpec::None),
        _ => None,
    }
}

fn flex_direction_spec_from_str(s: &str) -> Option<FlexDirectionSpec> {
    match s {
        "row" => Some(FlexDirectionSpec::Row),
        "column" => Some(FlexDirectionSpec::Column),
        "row_reverse" => Some(FlexDirectionSpec::RowReverse),
        "column_reverse" => Some(FlexDirectionSpec::ColumnReverse),
        _ => None,
    }
}

fn align_spec_from_str(s: &str) -> Option<AlignSpec> {
    match s {
        "start" => Some(AlignSpec::Start),
        "end" => Some(AlignSpec::End),
        "center" => Some(AlignSpec::Center),
        "stretch" => Some(AlignSpec::Stretch),
        _ => None,
    }
}

/// Builds a [`StyleSpec`] from a node's raw `style_props`, ignoring any key
/// or value this layer doesn't (yet) recognize.
fn style_spec_from_props(props: &HashMap<String, AttributeValue>) -> StyleSpec {
    let mut style = StyleSpec::default();
    for (key, value) in props {
        match key.as_str() {
            "display" => style.display = as_str(value).and_then(display_spec_from_str),
            "flex_direction" => {
                style.flex_direction = as_str(value).and_then(flex_direction_spec_from_str)
            }
            "justify_content" => style.justify_content = as_str(value).and_then(align_spec_from_str),
            "align_items" => style.align_items = as_str(value).and_then(align_spec_from_str),
            "gap" => style.gap = as_number(value),
            "width" => style.width = length_spec_from(value),
            "height" => style.height = length_spec_from(value),
            "border_width" => style.border_width = as_number(value),
            "background" => style.background = as_number(value).map(|n| n as u32),
            "border_color" => style.border_color = as_number(value).map(|n| n as u32),
            "corner_radius" => style.corner_radius = as_number(value),
            "text_color" => style.text_color = as_number(value).map(|n| n as u32),
            "text_size" => style.text_size = as_number(value),
            _ => {}
        }
    }
    style
}

/// Builds an [`ElementSpec`] for `root` and its whole subtree. `None` if
/// `root` doesn't resolve, matching [`VirtualTree::get`]'s convention. A
/// child id that doesn't resolve is skipped rather than panicking —
/// `VirtualTree`'s own API guarantees this can't actually happen (nodes are
/// never deallocated), but this is ultimately fed by JS-supplied data, so
/// the render path stays defensive anyway.
pub fn build_spec(tree: &VirtualTree, root: NodeId) -> Option<ElementSpec> {
    let node = tree.get(root)?;

    let tag = if node.tag_name() == "text" {
        let content = match node.attributes().get("value") {
            Some(AttributeValue::String(s)) => s.clone(),
            _ => String::new(),
        };
        ElementTag::Text(content)
    } else {
        ElementTag::Container
    };

    let style = style_spec_from_props(node.style_props());
    let children = node
        .children()
        .iter()
        .filter_map(|&child_id| build_spec(tree, child_id))
        .collect();

    Some(ElementSpec {
        id: root,
        tag,
        style,
        children,
    })
}

fn length_from_spec(spec: LengthSpec) -> Length {
    match spec {
        LengthSpec::Px(n) => px(n as f32).into(),
        LengthSpec::Auto => Length::Auto,
    }
}

/// Applies a [`StyleSpec`] onto a real `gpui` [`StyleRefinement`], by direct
/// field assignment rather than `gpui`'s named Tailwind-scale builder
/// methods (`.gap_3()`, `.size_8()`, ...) — those only cover fixed steps,
/// not the arbitrary numbers this spec carries.
fn apply_style(style: &mut StyleRefinement, spec: &StyleSpec) {
    if let Some(display) = spec.display {
        style.display = Some(match display {
            DisplaySpec::Flex => Display::Flex,
            DisplaySpec::Block => Display::Block,
            DisplaySpec::Grid => Display::Grid,
            DisplaySpec::None => Display::None,
        });
    }
    if let Some(direction) = spec.flex_direction {
        style.flex_direction = Some(match direction {
            FlexDirectionSpec::Row => FlexDirection::Row,
            FlexDirectionSpec::Column => FlexDirection::Column,
            FlexDirectionSpec::RowReverse => FlexDirection::RowReverse,
            FlexDirectionSpec::ColumnReverse => FlexDirection::ColumnReverse,
        });
    }
    if let Some(justify) = spec.justify_content {
        style.justify_content = Some(match justify {
            AlignSpec::Start => gpui::JustifyContent::Start,
            AlignSpec::End => gpui::JustifyContent::End,
            AlignSpec::Center => gpui::JustifyContent::Center,
            AlignSpec::Stretch => gpui::JustifyContent::Stretch,
        });
    }
    if let Some(align) = spec.align_items {
        style.align_items = Some(match align {
            AlignSpec::Start => gpui::AlignItems::Start,
            AlignSpec::End => gpui::AlignItems::End,
            AlignSpec::Center => gpui::AlignItems::Center,
            AlignSpec::Stretch => gpui::AlignItems::Stretch,
        });
    }
    if let Some(gap) = spec.gap {
        let gap = px(gap as f32);
        style.gap.width = Some(gap.into());
        style.gap.height = Some(gap.into());
    }
    if let Some(width) = spec.width {
        style.size.width = Some(length_from_spec(width));
    }
    if let Some(height) = spec.height {
        style.size.height = Some(length_from_spec(height));
    }
    if let Some(border_width) = spec.border_width {
        let width = px(border_width as f32);
        style.border_widths.top = Some(width.into());
        style.border_widths.right = Some(width.into());
        style.border_widths.bottom = Some(width.into());
        style.border_widths.left = Some(width.into());
    }
    if let Some(color) = spec.background {
        style.background = Some(Fill::from(Hsla::from(rgb(color))));
    }
    if let Some(color) = spec.border_color {
        style.border_color = Some(rgb(color).into());
    }
    if let Some(radius) = spec.corner_radius {
        let radius = px(radius as f32);
        style.corner_radii.top_left = Some(radius.into());
        style.corner_radii.top_right = Some(radius.into());
        style.corner_radii.bottom_right = Some(radius.into());
        style.corner_radii.bottom_left = Some(radius.into());
    }
    if let Some(color) = spec.text_color {
        style.text.color = Some(rgb(color).into());
    }
    if let Some(size) = spec.text_size {
        style.text.font_size = Some(px(size as f32).into());
    }
}

/// Recursively converts an [`ElementSpec`] into a real `gpui` [`AnyElement`].
///
/// Every container is tagged with a `.debug_selector("node-{id}")` — a
/// documented no-op outside test builds — so tests (including the
/// layout-parity test in `tests/layout_parity.rs`) can look its computed
/// bounds up by [`NodeId`].
pub fn build_element(spec: &ElementSpec) -> AnyElement {
    match &spec.tag {
        ElementTag::Text(content) => content.clone().into_any_element(),
        ElementTag::Container => {
            let id = spec.id;
            let mut element = div().debug_selector(move || format!("node-{id}"));
            apply_style(element.style(), &spec.style);
            for child in &spec.children {
                element = element.child(build_element(child));
            }
            element.into_any_element()
        }
    }
}

/// Composes [`build_spec`] and [`build_element`]: builds `root` and its
/// whole subtree from `tree` into a real `gpui` element. `None` if `root`
/// doesn't resolve.
pub fn render_tree(tree: &VirtualTree, root: NodeId) -> Option<AnyElement> {
    build_spec(tree, root).map(|spec| build_element(&spec))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod spec_layer {
        use super::*;

        #[test]
        fn unknown_root_returns_none() {
            let tree = VirtualTree::new();
            assert!(build_spec(&tree, 42).is_none());
        }

        #[test]
        fn non_text_tag_is_a_container() {
            let mut tree = VirtualTree::new();
            let id = tree.create_node("div");

            let spec = build_spec(&tree, id).unwrap();
            assert_eq!(spec.tag, ElementTag::Container);
        }

        #[test]
        fn text_tag_uses_its_value_attribute() {
            let mut tree = VirtualTree::new();
            let id = tree.create_node("text");
            tree.set_attribute(id, "value", "hello").unwrap();

            let spec = build_spec(&tree, id).unwrap();
            assert_eq!(spec.tag, ElementTag::Text("hello".into()));
        }

        #[test]
        fn text_tag_missing_value_is_empty_content() {
            let mut tree = VirtualTree::new();
            let id = tree.create_node("text");

            let spec = build_spec(&tree, id).unwrap();
            assert_eq!(spec.tag, ElementTag::Text(String::new()));
        }

        #[test]
        fn children_are_built_in_append_order() {
            let mut tree = VirtualTree::new();
            let parent = tree.create_node("div");
            let a = tree.create_node("div");
            let b = tree.create_node("div");
            tree.append_child(parent, a).unwrap();
            tree.append_child(parent, b).unwrap();

            let spec = build_spec(&tree, parent).unwrap();
            let child_ids: Vec<NodeId> = spec.children.iter().map(|c| c.id).collect();
            assert_eq!(child_ids, &[a, b]);
        }

        #[test]
        fn recognized_style_keys_are_mapped() {
            let mut tree = VirtualTree::new();
            let id = tree.create_node("div");
            tree.set_style(id, "display", "flex").unwrap();
            tree.set_style(id, "flex_direction", "column").unwrap();
            tree.set_style(id, "justify_content", "center").unwrap();
            tree.set_style(id, "align_items", "stretch").unwrap();
            tree.set_style(id, "gap", 8.0).unwrap();
            tree.set_style(id, "width", 120.0).unwrap();
            tree.set_style(id, "height", "auto").unwrap();
            tree.set_style(id, "border_width", 1.0).unwrap();
            tree.set_style(id, "background", 0x505050 as f64).unwrap();
            tree.set_style(id, "border_color", 0x0000ff as f64).unwrap();
            tree.set_style(id, "corner_radius", 4.0).unwrap();
            tree.set_style(id, "text_color", 0xffffff as f64).unwrap();
            tree.set_style(id, "text_size", 20.0).unwrap();

            let spec = build_spec(&tree, id).unwrap();
            assert_eq!(
                spec.style,
                StyleSpec {
                    display: Some(DisplaySpec::Flex),
                    flex_direction: Some(FlexDirectionSpec::Column),
                    justify_content: Some(AlignSpec::Center),
                    align_items: Some(AlignSpec::Stretch),
                    gap: Some(8.0),
                    width: Some(LengthSpec::Px(120.0)),
                    height: Some(LengthSpec::Auto),
                    border_width: Some(1.0),
                    background: Some(0x505050),
                    border_color: Some(0x0000ff),
                    corner_radius: Some(4.0),
                    text_color: Some(0xffffff),
                    text_size: Some(20.0),
                }
            );
        }

        #[test]
        fn unrecognized_style_key_is_ignored() {
            let mut tree = VirtualTree::new();
            let id = tree.create_node("div");
            tree.set_style(id, "not_a_real_prop", 1.0).unwrap();

            let spec = build_spec(&tree, id).unwrap();
            assert_eq!(spec.style, StyleSpec::default());
        }

        #[test]
        fn malformed_enum_value_is_ignored_not_a_panic() {
            let mut tree = VirtualTree::new();
            let id = tree.create_node("div");
            tree.set_style(id, "display", "not-a-real-display-value").unwrap();

            let spec = build_spec(&tree, id).unwrap();
            assert_eq!(spec.style.display, None);
        }
    }

    mod gpui_layer {
        use super::*;
        use gpui::{TestAppContext, point, size};

        #[gpui::test]
        fn container_with_fixed_size_lays_out_at_that_size(cx: &mut TestAppContext) {
            let mut tree = VirtualTree::new();
            let root = tree.create_node("div");
            tree.set_style(root, "width", 120.0).unwrap();
            tree.set_style(root, "height", 80.0).unwrap();

            let cx = cx.add_empty_window();
            cx.draw(point(px(0.), px(0.)), size(px(800.), px(600.)), |_, _| {
                render_tree(&tree, root).unwrap()
            });

            let bounds = cx
                .debug_bounds("node-0")
                .expect("root container should be tagged with a debug selector");
            assert_eq!(bounds.size.width, px(120.0));
            assert_eq!(bounds.size.height, px(80.0));
        }

        #[gpui::test]
        fn text_leaf_renders_without_panicking(cx: &mut TestAppContext) {
            let mut tree = VirtualTree::new();
            let root = tree.create_node("text");
            tree.set_attribute(root, "value", "hello").unwrap();

            let cx = cx.add_empty_window();
            cx.draw(point(px(0.), px(0.)), size(px(800.), px(600.)), |_, _| {
                render_tree(&tree, root).unwrap()
            });
        }
    }
}
