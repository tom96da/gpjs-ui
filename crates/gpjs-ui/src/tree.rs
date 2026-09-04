// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Retained, arena-allocated virtual tree.
//!
//! `VirtualTree` owns every `VirtualNode` for the lifetime of the tree; there
//! is no destroy/free operation yet, so detaching a node from its parent
//! (`remove_child`) never deallocates it — the node stays reachable by id
//! for later re-attachment.

use std::collections::HashMap;
use std::fmt;

/// Stable handle returned to JS, used in every subsequent host-bridge call.
pub type NodeId = u32;

/// An owned, primitive JS value for a style/attribute prop.
///
/// Restricted to primitives on purpose: a non-primitive value (an object or
/// array) can't be represented here, so a caller converting a JS value into
/// one of these is forced to reject it explicitly instead of coercing or
/// storing it silently.
#[derive(Debug, Clone, PartialEq)]
pub enum AttributeValue {
    String(String),
    Number(f64),
    Bool(bool),
}

impl From<&str> for AttributeValue {
    fn from(value: &str) -> Self {
        AttributeValue::String(value.to_owned())
    }
}

impl From<String> for AttributeValue {
    fn from(value: String) -> Self {
        AttributeValue::String(value)
    }
}

impl From<f64> for AttributeValue {
    fn from(value: f64) -> Self {
        AttributeValue::Number(value)
    }
}

impl From<bool> for AttributeValue {
    fn from(value: bool) -> Self {
        AttributeValue::Bool(value)
    }
}

/// A single retained node. Always accessed through a [`VirtualTree`] — there
/// is no way to construct one standalone, since its [`id`](VirtualNode::id)
/// is only meaningful within the tree that allocated it.
#[derive(Debug, Clone)]
pub struct VirtualNode {
    id: NodeId,
    tag_name: String,
    style_props: HashMap<String, AttributeValue>,
    attributes: HashMap<String, AttributeValue>,
    children: Vec<NodeId>,
}

impl VirtualNode {
    fn new(id: NodeId, tag_name: String) -> Self {
        Self {
            id,
            tag_name,
            style_props: HashMap::new(),
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    #[must_use]
    pub fn id(&self) -> NodeId {
        self.id
    }

    #[must_use]
    pub fn tag_name(&self) -> &str {
        &self.tag_name
    }

    #[must_use]
    pub fn style_props(&self) -> &HashMap<String, AttributeValue> {
        &self.style_props
    }

    #[must_use]
    pub fn attributes(&self) -> &HashMap<String, AttributeValue> {
        &self.attributes
    }

    /// Ordered child handles. Order is append order, not insertion-sorted.
    #[must_use]
    pub fn children(&self) -> &[NodeId] {
        &self.children
    }
}

/// Every fallible [`VirtualTree`] operation fails this way — an id that
/// doesn't (or no longer) resolves to a node. Returned as a `Result` rather
/// than panicking so a caller further up the stack can turn it into
/// whatever "unknown id" handling it needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeError {
    NodeNotFound(NodeId),
}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeError::NodeNotFound(id) => write!(f, "unknown node id: {id}"),
        }
    }
}

impl std::error::Error for TreeError {}

/// Owns the whole retained tree. One instance per `QuickJS` engine/document —
/// ids from one `VirtualTree` are meaningless in another.
#[derive(Debug, Default)]
pub struct VirtualTree {
    nodes: HashMap<NodeId, VirtualNode>,
    next_id: NodeId,
}

impl VirtualTree {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocates a new node and returns its id. Ids are assigned
    /// sequentially starting at 0 and are never reused, even after the node
    /// they named is detached from every parent.
    ///
    /// # Panics
    ///
    /// Panics if the id space is exhausted (`u32::MAX` nodes ever created).
    pub fn create_node(&mut self, tag_name: impl Into<String>) -> NodeId {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("VirtualTree node id space exhausted");
        self.nodes.insert(id, VirtualNode::new(id, tag_name.into()));
        id
    }

    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&VirtualNode> {
        self.nodes.get(&id)
    }

    /// Appends `child_id` to `parent_id`'s children. Thin wrapper over
    /// [`insert_before`](Self::insert_before) with no anchor.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::NodeNotFound`] if `parent_id` or `child_id`
    /// names no node.
    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<(), TreeError> {
        self.insert_before(parent_id, child_id, None)
    }

    /// Inserts `child_id` into `parent_id`'s children, before `anchor_id` if
    /// given, or at the end if `None`. If `anchor_id` names a real node that
    /// isn't (or is no longer) among `parent_id`'s children, falls back to
    /// appending at the end, same as an absent `remove_child` target — only
    /// a truly unknown/never-allocated `anchor_id` is an error. Does not
    /// check whether `child_id` is already a child elsewhere (or already a
    /// child of `parent_id`) — the tree is a plain multi-parent graph at
    /// this layer, not a DOM-style single-parent tree; that invariant, if
    /// wanted, belongs to a higher layer.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::NodeNotFound`] if `parent_id` or `child_id`
    /// names no node, or if `anchor_id` is `Some` and names no node.
    pub fn insert_before(
        &mut self,
        parent_id: NodeId,
        child_id: NodeId,
        anchor_id: Option<NodeId>,
    ) -> Result<(), TreeError> {
        if !self.nodes.contains_key(&child_id) {
            return Err(TreeError::NodeNotFound(child_id));
        }
        if let Some(anchor_id) = anchor_id
            && !self.nodes.contains_key(&anchor_id)
        {
            return Err(TreeError::NodeNotFound(anchor_id));
        }
        let parent = self
            .nodes
            .get_mut(&parent_id)
            .ok_or(TreeError::NodeNotFound(parent_id))?;
        let index = anchor_id
            .and_then(|anchor_id| parent.children.iter().position(|&id| id == anchor_id))
            .unwrap_or(parent.children.len());
        parent.children.insert(index, child_id);
        Ok(())
    }

    /// Unlinks `child_id` from `parent_id`'s children, if present. `child_id`
    /// not being (or no longer being) a child of `parent_id` is a no-op, not
    /// an error — only an unknown `parent_id` is. The child node itself is
    /// never deallocated by this call.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::NodeNotFound`] if `parent_id` names no node.
    pub fn remove_child(&mut self, parent_id: NodeId, child_id: NodeId) -> Result<(), TreeError> {
        let parent = self
            .nodes
            .get_mut(&parent_id)
            .ok_or(TreeError::NodeNotFound(parent_id))?;
        parent.children.retain(|&id| id != child_id);
        Ok(())
    }

    /// Sets (inserting or overwriting) one attribute prop.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::NodeNotFound`] if `node_id` names no node.
    pub fn set_attribute(
        &mut self,
        node_id: NodeId,
        key: impl Into<String>,
        value: impl Into<AttributeValue>,
    ) -> Result<(), TreeError> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(TreeError::NodeNotFound(node_id))?;
        node.attributes.insert(key.into(), value.into());
        Ok(())
    }

    /// Sets (inserting or overwriting) one style prop.
    ///
    /// # Errors
    ///
    /// Returns [`TreeError::NodeNotFound`] if `node_id` names no node.
    pub fn set_style(
        &mut self,
        node_id: NodeId,
        key: impl Into<String>,
        value: impl Into<AttributeValue>,
    ) -> Result<(), TreeError> {
        let node = self
            .nodes
            .get_mut(&node_id)
            .ok_or(TreeError::NodeNotFound(node_id))?;
        node.style_props.insert(key.into(), value.into());
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn id_uniqueness() {
        let mut tree = VirtualTree::new();
        let ids: Vec<NodeId> = (0..100).map(|_| tree.create_node("div")).collect();

        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            ids.len(),
            "create_node must never reuse an id"
        );
    }

    #[test]
    fn append_order() {
        let mut tree = VirtualTree::new();
        let parent = tree.create_node("div");
        let a = tree.create_node("span");
        let b = tree.create_node("span");
        let c = tree.create_node("span");

        tree.append_child(parent, a).unwrap();
        tree.append_child(parent, b).unwrap();
        tree.append_child(parent, c).unwrap();

        assert_eq!(tree.get(parent).unwrap().children(), &[a, b, c]);
    }

    #[test]
    fn insert_before_at_start_middle_end() {
        let mut tree = VirtualTree::new();
        let parent = tree.create_node("div");
        let a = tree.create_node("span");
        let b = tree.create_node("span");
        let c = tree.create_node("span");

        tree.insert_before(parent, b, None).unwrap(); // end: [b]
        tree.insert_before(parent, a, Some(b)).unwrap(); // start: [a, b]
        tree.insert_before(parent, c, None).unwrap(); // end: [a, b, c]

        assert_eq!(tree.get(parent).unwrap().children(), &[a, b, c]);

        let d = tree.create_node("span");
        tree.insert_before(parent, d, Some(b)).unwrap(); // middle: [a, d, b, c]
        assert_eq!(tree.get(parent).unwrap().children(), &[a, d, b, c]);
    }

    #[test]
    fn insert_before_unknown_anchor_falls_back_to_append() {
        let mut tree = VirtualTree::new();
        let parent = tree.create_node("div");
        let a = tree.create_node("span");
        let stray_anchor = tree.create_node("span"); // real node, never attached here

        tree.insert_before(parent, a, Some(stray_anchor)).unwrap();

        assert_eq!(tree.get(parent).unwrap().children(), &[a]);
    }

    #[test]
    fn insert_before_unknown_ids_error() {
        let mut tree = VirtualTree::new();
        let parent = tree.create_node("div");
        let child = tree.create_node("span");

        assert_eq!(
            tree.insert_before(999, child, None).unwrap_err(),
            TreeError::NodeNotFound(999)
        );
        assert_eq!(
            tree.insert_before(parent, 999, None).unwrap_err(),
            TreeError::NodeNotFound(999)
        );
        assert_eq!(
            tree.insert_before(parent, child, Some(999)).unwrap_err(),
            TreeError::NodeNotFound(999)
        );
    }

    #[test]
    fn detach_keeps_node_alive() {
        let mut tree = VirtualTree::new();
        let parent = tree.create_node("div");
        let child = tree.create_node("span");
        tree.append_child(parent, child).unwrap();

        tree.remove_child(parent, child).unwrap();

        assert!(tree.get(parent).unwrap().children().is_empty());
        assert!(
            tree.get(child).is_some(),
            "detaching a child must not deallocate it"
        );
    }

    #[test]
    fn remove_of_absent_child_is_a_no_op() {
        let mut tree = VirtualTree::new();
        let parent = tree.create_node("div");
        let never_appended = tree.create_node("span");

        // Absent child that exists elsewhere in the tree.
        assert!(tree.remove_child(parent, never_appended).is_ok());
        assert!(tree.get(parent).unwrap().children().is_empty());

        // Absent child id that was never allocated at all.
        assert!(tree.remove_child(parent, 9999).is_ok());
    }

    #[test]
    fn set_attribute_overwrites() {
        let mut tree = VirtualTree::new();
        let node = tree.create_node("input");

        tree.set_attribute(node, "value", "first").unwrap();
        tree.set_attribute(node, "value", "second").unwrap();

        assert_eq!(
            tree.get(node).unwrap().attributes().get("value"),
            Some(&AttributeValue::String("second".into()))
        );
    }

    #[test]
    fn unknown_id_lookup_returns_none() {
        let tree = VirtualTree::new();
        assert!(tree.get(42).is_none());
    }

    #[test]
    fn operations_on_unknown_ids_error_instead_of_panicking() {
        let mut tree = VirtualTree::new();
        let node = tree.create_node("div");

        assert_eq!(
            tree.append_child(node, 999).unwrap_err(),
            TreeError::NodeNotFound(999)
        );
        assert_eq!(
            tree.append_child(999, node).unwrap_err(),
            TreeError::NodeNotFound(999)
        );
        assert_eq!(
            tree.remove_child(999, node).unwrap_err(),
            TreeError::NodeNotFound(999)
        );
        assert_eq!(
            tree.set_attribute(999, "k", "v").unwrap_err(),
            TreeError::NodeNotFound(999)
        );
        assert_eq!(
            tree.set_style(999, "k", "v").unwrap_err(),
            TreeError::NodeNotFound(999)
        );
    }

    #[test]
    fn set_style_is_independent_of_attributes() {
        let mut tree = VirtualTree::new();
        let node = tree.create_node("div");

        tree.set_style(node, "color", "red").unwrap();

        let got = tree.get(node).unwrap();
        assert_eq!(
            got.style_props().get("color"),
            Some(&AttributeValue::String("red".into()))
        );
        assert!(got.attributes().get("color").is_none());
    }
}
