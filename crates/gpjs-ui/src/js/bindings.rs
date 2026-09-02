// Copyright (c) 2026 tom96da
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Binds `globalThis.__gpjsui_native__`: the small set of native functions
//! JS calls to mutate the retained virtual tree and register input-event
//! callbacks.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use rquickjs::{Ctx, Exception, Function, Object, Result as JsResult, Value};

use crate::tree::{AttributeValue, NodeId, TreeError, VirtualTree};

/// Every `(node id, event name)` a JS caller has registered through
/// `addEventListener`, mapped to the plain integer callback handles it gave
/// us. Only ever stores owned Rust data — never a JS value or function —
/// so it stays readable by Rust code running outside of any JS call, such
/// as a future input-event dispatcher.
#[derive(Debug, Default)]
pub struct EventListeners {
    by_node: HashMap<NodeId, HashMap<String, Vec<u32>>>,
}

impl EventListeners {
    pub fn register(&mut self, node_id: NodeId, event: impl Into<String>, callback_id: u32) {
        self.by_node
            .entry(node_id)
            .or_default()
            .entry(event.into())
            .or_default()
            .push(callback_id);
    }

    pub fn callbacks_for(&self, node_id: NodeId, event: &str) -> &[u32] {
        self.by_node
            .get(&node_id)
            .and_then(|events| events.get(event))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
}

/// Everything one `__gpjsui_native__` binding set shares: the retained tree
/// it mutates and the event-listener registrations it records.
#[derive(Debug, Default)]
pub struct Host {
    pub tree: VirtualTree,
    pub listeners: EventListeners,
}

fn throw_tree_error<'js>(ctx: &Ctx<'js>, err: TreeError) -> rquickjs::Error {
    Exception::throw_type(ctx, &err.to_string())
}

fn attribute_value_from_js<'js>(ctx: &Ctx<'js>, value: Value<'js>) -> JsResult<AttributeValue> {
    if value.is_string() {
        return value.get::<String>().map(AttributeValue::String);
    }
    if value.is_number() {
        return value.get::<f64>().map(AttributeValue::Number);
    }
    if value.is_bool() {
        return value.get::<bool>().map(AttributeValue::Bool);
    }
    Err(Exception::throw_type(
        ctx,
        "setAttribute value must be a string, number, or boolean",
    ))
}

/// Installs `globalThis.__gpjsui_native__` into `ctx`, wired to `host`.
pub fn install<'js>(ctx: &Ctx<'js>, host: Rc<RefCell<Host>>) -> JsResult<()> {
    let native = Object::new(ctx.clone())?;

    {
        let host = Rc::clone(&host);
        native.set(
            "createNode",
            Function::new(ctx.clone(), move |tag_name: String| -> NodeId {
                host.borrow_mut().tree.create_node(tag_name)
            })?,
        )?;
    }

    {
        let host = Rc::clone(&host);
        native.set(
            "appendChild",
            Function::new(
                ctx.clone(),
                move |ctx: Ctx<'js>, parent_id: NodeId, child_id: NodeId| -> JsResult<()> {
                    host.borrow_mut()
                        .tree
                        .append_child(parent_id, child_id)
                        .map_err(|err| throw_tree_error(&ctx, err))
                },
            )?,
        )?;
    }

    {
        let host = Rc::clone(&host);
        native.set(
            "removeChild",
            Function::new(
                ctx.clone(),
                move |ctx: Ctx<'js>, parent_id: NodeId, child_id: NodeId| -> JsResult<()> {
                    host.borrow_mut()
                        .tree
                        .remove_child(parent_id, child_id)
                        .map_err(|err| throw_tree_error(&ctx, err))
                },
            )?,
        )?;
    }

    {
        let host = Rc::clone(&host);
        native.set(
            "setAttribute",
            Function::new(
                ctx.clone(),
                move |ctx: Ctx<'js>,
                      node_id: NodeId,
                      key: String,
                      value: Value<'js>|
                      -> JsResult<()> {
                    let value = attribute_value_from_js(&ctx, value)?;
                    host.borrow_mut()
                        .tree
                        .set_attribute(node_id, key, value)
                        .map_err(|err| throw_tree_error(&ctx, err))
                },
            )?,
        )?;
    }

    {
        let host = Rc::clone(&host);
        native.set(
            "addEventListener",
            Function::new(
                ctx.clone(),
                move |ctx: Ctx<'js>, node_id: NodeId, event: String, callback_id: u32| -> JsResult<()> {
                    let mut host = host.borrow_mut();
                    if host.tree.get(node_id).is_none() {
                        return Err(throw_tree_error(&ctx, TreeError::NodeNotFound(node_id)));
                    }
                    host.listeners.register(node_id, event, callback_id);
                    Ok(())
                },
            )?,
        )?;
    }

    ctx.globals().set("__gpjsui_native__", native)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::js::engine::Engine;

    fn engine_with_bindings() -> (Engine, Rc<RefCell<Host>>) {
        let engine = Engine::new().unwrap();
        let host = Rc::new(RefCell::new(Host::default()));
        engine
            .with(|ctx| install(&ctx, Rc::clone(&host)))
            .unwrap();
        (engine, host)
    }

    #[test]
    fn happy_path_round_trip_through_eval() {
        let (engine, host) = engine_with_bindings();

        let child_id: NodeId = engine
            .eval(
                r#"
                const parent = __gpjsui_native__.createNode('div');
                const child = __gpjsui_native__.createNode('span');
                __gpjsui_native__.appendChild(parent, child);

                const stray = __gpjsui_native__.createNode('span');
                __gpjsui_native__.appendChild(parent, stray);
                __gpjsui_native__.removeChild(parent, stray);

                __gpjsui_native__.setAttribute(child, 'label', 'hello');
                __gpjsui_native__.setAttribute(child, 'count', 3);
                __gpjsui_native__.setAttribute(child, 'visible', true);
                __gpjsui_native__.addEventListener(child, 'click', 7);

                child;
                "#,
            )
            .unwrap();

        let host = host.borrow();
        let parent = host.tree.get(0).unwrap();
        assert_eq!(parent.tag_name(), "div");
        assert_eq!(
            parent.children(),
            &[child_id],
            "the removed stray child must not remain attached"
        );

        let child = host.tree.get(child_id).unwrap();
        assert_eq!(child.tag_name(), "span");
        assert_eq!(
            child.attributes().get("label"),
            Some(&AttributeValue::String("hello".into()))
        );
        assert_eq!(
            child.attributes().get("count"),
            Some(&AttributeValue::Number(3.0))
        );
        assert_eq!(
            child.attributes().get("visible"),
            Some(&AttributeValue::Bool(true))
        );
        assert_eq!(host.listeners.callbacks_for(child_id, "click"), &[7]);
    }

    #[test]
    fn unknown_node_id_raises_catchable_exception() {
        let (engine, _host) = engine_with_bindings();

        let caught: bool = engine
            .eval(
                r#"
                let caught = false;
                try {
                    __gpjsui_native__.appendChild(999, 1000);
                } catch (e) {
                    caught = true;
                }
                caught;
                "#,
            )
            .unwrap();

        assert!(caught, "an unknown node id must raise a catchable exception");
    }

    #[test]
    fn non_primitive_attribute_value_raises_catchable_exception() {
        let (engine, _host) = engine_with_bindings();

        let caught: bool = engine
            .eval(
                r#"
                const node = __gpjsui_native__.createNode('div');
                let caught = false;
                try {
                    __gpjsui_native__.setAttribute(node, 'bad', { nested: true });
                } catch (e) {
                    caught = true;
                }
                caught;
                "#,
            )
            .unwrap();

        assert!(
            caught,
            "a non-primitive attribute value must raise a catchable exception"
        );
    }
}
