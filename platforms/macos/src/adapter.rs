// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use accesskit::{kurbo::Point, ActionHandler, TreeUpdate};
use accesskit_consumer::{FilterResult, Tree};
use objc2::{
    foundation::{NSArray, NSObject, NSPoint},
    rc::{Id, Shared, WeakId},
};
use once_cell::sync::Lazy;
use std::{ffi::c_void, ptr::null_mut, sync::Arc};

use crate::{appkit::NSView, context::Context, event::QueuedEvents, node::filter};

pub struct Adapter {
    context: Lazy<Arc<Context>, Box<dyn FnOnce() -> Arc<Context>>>,
}

impl Adapter {
    /// Create a new macOS adapter.
    ///
    /// # Safety
    ///
    /// `view` must be a valid, unreleased pointer to an `NSView`.
    pub unsafe fn new(
        view: *mut c_void,
        source: Box<dyn FnOnce() -> TreeUpdate>,
        action_handler: Box<dyn ActionHandler>,
    ) -> Self {
        let view = Id::retain(view as *mut NSView).unwrap();
        let view = WeakId::new(&view);
        Self {
            context: Lazy::new(Box::new(move || {
                let tree = Tree::new(source(), action_handler);
                Context::new(view, tree)
            })),
        }
    }

    /// Initialize the tree if it hasn't been initialized already, then apply
    /// the provided update.
    ///
    /// The caller must call [`QueuedEvents::raise`] on the return value.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    pub fn update(&self, update: TreeUpdate) -> QueuedEvents {
        let context = Lazy::force(&self.context);
        context.update(update)
    }

    /// If and only if the tree has been initialized, call the provided function
    /// and apply the resulting update.
    ///
    /// If a [`QueuedEvents`] instance is returned, the caller must call
    /// [`QueuedEvents::raise`] on it.
    ///
    /// This method may be safely called on any thread, but refer to
    /// [`QueuedEvents::raise`] for restrictions on the context in which
    /// it should be called.
    pub fn update_if_active(&self, updater: impl FnOnce() -> TreeUpdate) -> Option<QueuedEvents> {
        Lazy::get(&self.context).map(|context| context.update(updater()))
    }

    pub fn view_children(&self) -> *mut NSArray<NSObject> {
        let context = Lazy::force(&self.context);
        let state = context.tree.read();
        let node = state.root();
        let platform_nodes = if filter(&node) == FilterResult::Include {
            vec![Id::into_super(Id::into_super(
                context.get_or_create_platform_node(node.id()),
            ))]
        } else {
            node.filtered_children(filter)
                .map(|node| {
                    Id::into_super(Id::into_super(
                        context.get_or_create_platform_node(node.id()),
                    ))
                })
                .collect::<Vec<Id<NSObject, Shared>>>()
        };
        let array = NSArray::from_vec(platform_nodes);
        Id::autorelease_return(array)
    }

    pub fn focus(&self) -> *mut NSObject {
        let context = Lazy::force(&self.context);
        let state = context.tree.read();
        if let Some(node) = state.focus() {
            if filter(&node) == FilterResult::Include {
                return Id::autorelease_return(context.get_or_create_platform_node(node.id()))
                    as *mut _;
            }
        }
        null_mut()
    }

    pub fn hit_test(&self, point: NSPoint) -> *mut NSObject {
        let context = Lazy::force(&self.context);
        let view = match context.view.load() {
            Some(view) => view,
            None => {
                return null_mut();
            }
        };

        let window = view.window().unwrap();
        let point = window.convert_point_from_screen(point);
        let point = view.convert_point_from_view(point, None);
        let view_bounds = view.bounds();
        let point = Point::new(point.x, view_bounds.size.height - point.y);

        let state = context.tree.read();
        let root = state.root();
        let point = root.transform().inverse() * point;
        if let Some(node) = root.node_at_point(point, &filter) {
            return Id::autorelease_return(context.get_or_create_platform_node(node.id()))
                as *mut _;
        }
        null_mut()
    }
}
