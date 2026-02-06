// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from the Flutter engine.
// Copyright 2013 The Flutter Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use accesskit::{Action, ActionRequest, Rect, Role, Toggled};
use accesskit_consumer::{FilterResult, Node, NodeId, Tree};
use objc2::{
    ClassType, DeclaredClass, declare_class, msg_send_id, mutability::MainThreadOnly, rc::Retained,
    runtime::AnyObject,
};
use objc2_foundation::{CGRect, NSArray, NSObject, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    UIAccessibilityContainerType, UIAccessibilityElement, UIAccessibilityTraitAdjustable,
    UIAccessibilityTraitButton, UIAccessibilityTraitHeader, UIAccessibilityTraitImage,
    UIAccessibilityTraitLink, UIAccessibilityTraitNone, UIAccessibilityTraitNotEnabled,
    UIAccessibilityTraitSelected, UIAccessibilityTraitStaticText, UIAccessibilityTraits,
};
use std::rc::{Rc, Weak};

use crate::{
    context::Context,
    filters::{filter, filter_for_is_accessibility_element},
    util::{UIAccessibilityExpandedStatus, to_cg_rect},
};

#[derive(Debug, PartialEq)]
enum Value {
    Bool(bool),
    Number(f64),
    String(String),
}

impl From<Value> for String {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(true) => "1".into(),
            Value::Bool(false) => "0".into(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s,
        }
    }
}

#[derive(Debug, PartialEq)]
enum FrameSource {
    ViewBounds,
    Rect(Rect),
    Zero,
}

pub(crate) struct NodeWrapper<'a>(pub(crate) &'a Node<'a>);

impl NodeWrapper<'_> {
    fn label(&self) -> Option<String> {
        self.0.label()
    }

    fn hint(&self) -> Option<String> {
        self.0.description()
    }

    fn value(&self) -> Option<Value> {
        if let Some(toggled) = self.0.toggled() {
            return Some(Value::Bool(toggled != Toggled::False));
        }
        if let Some(value) = self.0.value() {
            return Some(Value::String(value));
        }
        if let Some(value) = self.0.numeric_value() {
            return Some(Value::Number(value));
        }
        None
    }

    fn frame_source(&self) -> FrameSource {
        if let Some(rect) = self.0.bounding_box() {
            FrameSource::Rect(rect)
        } else if self.0.is_root() {
            FrameSource::ViewBounds
        } else {
            FrameSource::Zero
        }
    }

    pub(crate) fn has_non_scroll_action(&self) -> bool {
        self.0.supports_action(Action::Click, &filter)
            || self.0.supports_action(Action::Focus, &filter)
            || self.0.supports_action(Action::Increment, &filter)
            || self.0.supports_action(Action::Decrement, &filter)
            || self.0.supports_action(Action::Expand, &filter)
            || self.0.supports_action(Action::Collapse, &filter)
            || self.0.supports_action(Action::CustomAction, &filter)
            || self.0.supports_action(Action::ReplaceSelectedText, &filter)
            || self.0.supports_action(Action::SetTextSelection, &filter)
            || self.0.supports_action(Action::SetValue, &filter)
    }

    pub(crate) fn can_be_focused(&self) -> bool {
        self.0.has_label()
            || self.0.toggled().is_some()
            || self.0.has_value()
            || self.0.numeric_value().is_some()
            || self.0.has_description()
            || self.has_non_scroll_action()
    }

    fn traits(&self) -> UIAccessibilityTraits {
        let mut traits = match self.0.role() {
            Role::Button | Role::DefaultButton | Role::DisclosureTriangle => unsafe {
                UIAccessibilityTraitButton
            },
            Role::Link => unsafe { UIAccessibilityTraitLink },
            Role::Image => unsafe { UIAccessibilityTraitImage },
            Role::Label => unsafe { UIAccessibilityTraitStaticText },
            Role::Heading => unsafe { UIAccessibilityTraitHeader },
            Role::Slider | Role::SpinButton => unsafe { UIAccessibilityTraitAdjustable },
            _ => unsafe { UIAccessibilityTraitNone },
        };

        if self.0.is_disabled() {
            traits |= unsafe { UIAccessibilityTraitNotEnabled };
        }

        if self.0.is_selected() == Some(true) {
            traits |= unsafe { UIAccessibilityTraitSelected };
        }

        traits
    }

    fn container_type(&self) -> UIAccessibilityContainerType {
        match self.0.role() {
            Role::Table | Role::Grid | Role::TreeGrid | Role::ListGrid => {
                UIAccessibilityContainerType::DataTable
            }
            Role::List | Role::ListBox | Role::DescriptionList | Role::Tree => {
                UIAccessibilityContainerType::List
            }
            Role::Article
            | Role::Banner
            | Role::Complementary
            | Role::ContentInfo
            | Role::Footer
            | Role::Form
            | Role::Main
            | Role::Navigation
            | Role::Region
            | Role::Search => UIAccessibilityContainerType::Landmark,
            Role::Group => UIAccessibilityContainerType::SemanticGroup,
            _ => UIAccessibilityContainerType::None,
        }
    }
}

pub(crate) struct PlatformNodeIvars {
    context: Weak<Context>,
    node_id: NodeId,
}

declare_class!(
    #[derive(Debug)]
    pub(crate) struct PlatformNode;

    unsafe impl ClassType for PlatformNode {
        #[inherits(NSObject)]
        type Super = UIAccessibilityElement;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "AccessKitNode";
    }

    impl DeclaredClass for PlatformNode {
        type Ivars = PlatformNodeIvars;
    }

    unsafe impl NSObjectProtocol for PlatformNode {}

    #[allow(non_snake_case)]
    unsafe impl PlatformNode {
        #[method_id(accessibilityContainer)]
        fn container(&self) -> Option<Retained<AnyObject>> {
            self.resolve_container()
        }

        // Explicit no-op. The container is computed dynamically in the
        // `accessibilityContainer` getter. If we let UIAccessibilityElement's
        // implementation stash the init-time placeholder, internal UIKit
        // paths can return it and bypass our getter override.
        // See https://github.com/flutter/flutter/issues/54366.
        #[method(setAccessibilityContainer:)]
        fn set_container(&self, _container: Option<&AnyObject>) {}

        #[method(isAccessibilityElement)]
        fn is_element(&self) -> bool {
            self.resolve(|node| filter_for_is_accessibility_element(node) == FilterResult::Include)
                .unwrap_or(false)
        }

        #[method_id(accessibilityLabel)]
        fn label(&self) -> Option<Retained<NSString>> {
            self.resolve(|node| {
                let wrapper = NodeWrapper(node);
                wrapper.label().map(|s| NSString::from_str(&s))
            })
            .flatten()
        }

        #[method_id(accessibilityHint)]
        fn hint(&self) -> Option<Retained<NSString>> {
            self.resolve(|node| {
                let wrapper = NodeWrapper(node);
                wrapper.hint().map(|s| NSString::from_str(&s))
            })
            .flatten()
        }

        #[method_id(accessibilityValue)]
        fn value(&self) -> Option<Retained<NSString>> {
            self.resolve(|node| {
                let wrapper = NodeWrapper(node);
                wrapper
                    .value()
                    .map(|v| NSString::from_str(&String::from(v)))
            })
            .flatten()
        }

        #[method(accessibilityTraits)]
        fn traits(&self) -> UIAccessibilityTraits {
            self.resolve(|node| NodeWrapper(node).traits())
                .unwrap_or(unsafe { UIAccessibilityTraitNone })
        }

        #[method(accessibilityFrame)]
        fn frame(&self) -> CGRect {
            self.resolve_with_context(|node, _, context| {
                let view = context.view.load()?;
                Some(match NodeWrapper(node).frame_source() {
                    FrameSource::Rect(rect) => to_cg_rect(&view, rect),
                    FrameSource::ViewBounds => {
                        let bounds = view.bounds();
                        unsafe { view.convertRect_toView(bounds, None) }
                    }
                    FrameSource::Zero => CGRect::ZERO,
                })
            })
            .flatten()
            .unwrap_or(CGRect::ZERO)
        }

        #[method_id(accessibilityLanguage)]
        fn language(&self) -> Option<Retained<NSString>> {
            self.resolve(|node| node.language().map(NSString::from_str))
                .flatten()
        }

        #[method(accessibilityExpandedStatus)]
        fn expanded_status(&self) -> UIAccessibilityExpandedStatus {
            self.resolve(|node| match node.data().is_expanded() {
                Some(true) => UIAccessibilityExpandedStatus::Expanded,
                Some(false) => UIAccessibilityExpandedStatus::Collapsed,
                None => UIAccessibilityExpandedStatus::Unsupported,
            })
            .unwrap_or(UIAccessibilityExpandedStatus::Unsupported)
        }

        #[method_id(accessibilityElements)]
        fn elements(&self) -> Option<Retained<NSArray<NSObject>>> {
            self.resolve_with_context(|node, _, context| {
                // If this node is itself a leaf accessibility element, hide
                // its descendants — they contribute to the node's label, not
                // independent focus targets.
                if filter_for_is_accessibility_element(node) == FilterResult::Include {
                    return NSArray::new();
                }
                let children: Vec<Retained<NSObject>> = node
                    .filtered_children(&filter)
                    .filter_map(|child| context.get_or_create_platform_node(child.id()))
                    .map(PlatformNode::into_ns_object)
                    .collect();
                NSArray::from_vec(children)
            })
        }

        #[method(accessibilityContainerType)]
        fn container_type(&self) -> UIAccessibilityContainerType {
            self.resolve(|node| {
                let wrapper = NodeWrapper(node);
                wrapper.container_type()
            })
            .unwrap_or(UIAccessibilityContainerType::None)
        }

        #[method(accessibilityElementDidBecomeFocused)]
        fn element_did_become_focused(&self) {
            self.resolve_with_context(|node, tree, context| {
                let node_id = node.id();
                *context.platform_focus.borrow_mut() = Some(node_id);
                if let Some((target_node, target_tree)) = tree.state().locate_node(node_id) {
                    context.do_action(ActionRequest {
                        action: Action::Focus,
                        target_tree,
                        target_node,
                        data: None,
                    });
                }
            });
        }
    }
);

impl PlatformNode {
    pub(crate) fn into_ns_object(this: Retained<Self>) -> Retained<NSObject> {
        let element = Retained::into_super(this);
        let responder = Retained::into_super(element);
        Retained::into_super(responder)
    }

    pub(crate) fn into_any_object(this: Retained<Self>) -> Retained<AnyObject> {
        Retained::into_super(Self::into_ns_object(this))
    }

    pub(crate) fn new(context: &Rc<Context>, node_id: NodeId) -> Option<Retained<Self>> {
        // UIAccessibilityElement's designated initializer is
        // `initWithAccessibilityContainer:`; plain `init` raises
        // NSInvalidArgumentException at runtime. Following Flutter's iOS
        // adapter, we pass the backing view as the init-time container
        // regardless of the node's real position in the tree, and report the
        // actual parent dynamically via the `accessibilityContainer` override.
        let view = context.view.load()?;
        let container = Retained::into_super(Retained::into_super(Retained::into_super(view)));
        let this = context.mtm.alloc::<Self>().set_ivars(PlatformNodeIvars {
            context: Rc::downgrade(context),
            node_id,
        });
        Some(unsafe { msg_send_id![super(this), initWithAccessibilityContainer: &*container] })
    }

    fn resolve<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&Node) -> T,
    {
        let context = self.ivars().context.upgrade()?;
        let tree = context.tree.borrow();
        let tree_state = tree.state();
        let node = tree_state.node_by_id(self.ivars().node_id)?;
        Some(f(&node))
    }

    fn resolve_with_context<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&Node, &Tree, &Rc<Context>) -> T,
    {
        let context = self.ivars().context.upgrade()?;
        let tree = context.tree.borrow();
        let tree_state = tree.state();
        let node = tree_state.node_by_id(self.ivars().node_id)?;
        Some(f(&node, &tree, &context))
    }

    fn resolve_container(&self) -> Option<Retained<AnyObject>> {
        let context = self.ivars().context.upgrade()?;
        let parent_id = {
            let tree = context.tree.borrow();
            let node = tree.state().node_by_id(self.ivars().node_id)?;
            node.parent().map(|p| p.id())
        };
        match parent_id {
            Some(parent_id) => context
                .get_or_create_platform_node(parent_id)
                .map(PlatformNode::into_any_object),
            None => {
                let view = context.view.load()?;
                Some(Retained::into_super(Retained::into_super(
                    Retained::into_super(view),
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::{Action, Node as NodeBuilder, NodeId, Rect, Toggled, Tree, TreeId, TreeUpdate};

    const ROOT_ID: NodeId = NodeId(0);

    fn build_tree(nodes: Vec<(NodeId, NodeBuilder)>) -> accesskit_consumer::Tree {
        let update = TreeUpdate {
            nodes,
            tree: Some(Tree::new(ROOT_ID)),
            tree_id: TreeId::ROOT,
            focus: ROOT_ID,
        };
        accesskit_consumer::Tree::new(update, false)
    }

    fn with_single<F, R>(node: &NodeBuilder, f: F) -> R
    where
        F: FnOnce(&Node) -> R,
    {
        let tree = build_tree(vec![(ROOT_ID, node.clone())]);
        let state = tree.state();
        let tree_node = state.node_by_tree_local_id(ROOT_ID, TreeId::ROOT).unwrap();
        f(&tree_node)
    }

    fn wrapper_value(node: &NodeBuilder) -> Option<Value> {
        with_single(node, |n| NodeWrapper(n).value())
    }

    fn wrapper_label(node: &NodeBuilder) -> Option<String> {
        with_single(node, |n| NodeWrapper(n).label())
    }

    fn wrapper_hint(node: &NodeBuilder) -> Option<String> {
        with_single(node, |n| NodeWrapper(n).hint())
    }

    fn node_traits(node: &NodeBuilder) -> UIAccessibilityTraits {
        with_single(node, |n| NodeWrapper(n).traits())
    }

    fn node_container_type(node: &NodeBuilder) -> UIAccessibilityContainerType {
        with_single(node, |n| NodeWrapper(n).container_type())
    }

    fn node_can_be_focused(nodes: Vec<(NodeId, NodeBuilder)>, target: NodeId) -> bool {
        let tree = build_tree(nodes);
        let state = tree.state();
        let node = state.node_by_tree_local_id(target, TreeId::ROOT).unwrap();
        NodeWrapper(&node).can_be_focused()
    }

    // ---- label ----

    #[test]
    fn label_present() {
        let mut node = NodeBuilder::new(Role::Button);
        node.set_label("OK");
        assert_eq!(wrapper_label(&node), Some("OK".into()));
    }

    #[test]
    fn label_absent() {
        let node = NodeBuilder::new(Role::Button);
        assert_eq!(wrapper_label(&node), None);
    }

    // ---- hint ----

    #[test]
    fn hint_present() {
        let mut node = NodeBuilder::new(Role::Button);
        node.set_description("Confirms the action");
        assert_eq!(wrapper_hint(&node), Some("Confirms the action".into()));
    }

    #[test]
    fn hint_absent() {
        let node = NodeBuilder::new(Role::Button);
        assert_eq!(wrapper_hint(&node), None);
    }

    // ---- value ----

    #[test]
    fn value_toggled_true() {
        let mut node = NodeBuilder::new(Role::CheckBox);
        node.set_toggled(Toggled::True);
        assert_eq!(wrapper_value(&node), Some(Value::Bool(true)));
    }

    #[test]
    fn value_toggled_false() {
        let mut node = NodeBuilder::new(Role::CheckBox);
        node.set_toggled(Toggled::False);
        assert_eq!(wrapper_value(&node), Some(Value::Bool(false)));
    }

    #[test]
    fn value_toggled_mixed() {
        let mut node = NodeBuilder::new(Role::CheckBox);
        node.set_toggled(Toggled::Mixed);
        assert_eq!(wrapper_value(&node), Some(Value::Bool(true)));
    }

    #[test]
    fn value_text_string() {
        let mut node = NodeBuilder::new(Role::Label);
        node.set_value("hello");
        assert_eq!(wrapper_value(&node), Some(Value::String("hello".into())));
    }

    #[test]
    fn value_numeric() {
        let mut node = NodeBuilder::new(Role::Slider);
        node.set_numeric_value(42.5);
        assert_eq!(wrapper_value(&node), Some(Value::Number(42.5)));
    }

    #[test]
    fn value_toggled_takes_priority() {
        let mut node = NodeBuilder::new(Role::CheckBox);
        node.set_toggled(Toggled::True);
        node.set_value("ignored");
        node.set_numeric_value(99.0);
        assert_eq!(wrapper_value(&node), Some(Value::Bool(true)));
    }

    #[test]
    fn value_string_over_numeric() {
        let mut node = NodeBuilder::new(Role::Label);
        node.set_value("text");
        node.set_numeric_value(1.0);
        assert_eq!(wrapper_value(&node), Some(Value::String("text".into())));
    }

    #[test]
    fn value_none() {
        let node = NodeBuilder::new(Role::Button);
        assert_eq!(wrapper_value(&node), None);
    }

    // ---- String::from(Value) ----

    #[test]
    fn rendered_value_bool_true_is_one() {
        assert_eq!(String::from(Value::Bool(true)), "1");
    }

    #[test]
    fn rendered_value_bool_false_is_zero() {
        assert_eq!(String::from(Value::Bool(false)), "0");
    }

    #[test]
    fn rendered_value_number_uses_display() {
        assert_eq!(String::from(Value::Number(42.5)), "42.5");
    }

    #[test]
    fn rendered_value_string_passthrough() {
        assert_eq!(String::from(Value::String("hello".into())), "hello");
    }

    // ---- can_be_focused ----

    #[test]
    fn focusable_button() {
        let mut node = NodeBuilder::new(Role::Button);
        node.set_label("OK");
        node.add_action(Action::Click);
        assert!(node_can_be_focused(vec![(ROOT_ID, node)], ROOT_ID));
    }

    #[test]
    fn window_not_focusable() {
        let node = NodeBuilder::new(Role::Window);
        assert!(!node_can_be_focused(vec![(ROOT_ID, node)], ROOT_ID));
    }

    // ---- traits ----

    #[test]
    fn traits_button() {
        let node = NodeBuilder::new(Role::Button);
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitButton } != 0);
    }

    #[test]
    fn traits_default_button() {
        let node = NodeBuilder::new(Role::DefaultButton);
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitButton } != 0);
    }

    #[test]
    fn traits_disclosure_triangle() {
        let node = NodeBuilder::new(Role::DisclosureTriangle);
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitButton } != 0);
    }

    #[test]
    fn traits_link() {
        let node = NodeBuilder::new(Role::Link);
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitLink } != 0);
    }

    #[test]
    fn traits_image() {
        let node = NodeBuilder::new(Role::Image);
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitImage } != 0);
    }

    #[test]
    fn traits_label() {
        let node = NodeBuilder::new(Role::Label);
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitStaticText } != 0);
    }

    #[test]
    fn traits_heading() {
        let node = NodeBuilder::new(Role::Heading);
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitHeader } != 0);
    }

    #[test]
    fn traits_slider() {
        let node = NodeBuilder::new(Role::Slider);
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitAdjustable } != 0);
    }

    #[test]
    fn traits_spin_button() {
        let node = NodeBuilder::new(Role::SpinButton);
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitAdjustable } != 0);
    }

    #[test]
    fn traits_disabled() {
        let mut node = NodeBuilder::new(Role::Button);
        node.set_disabled();
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitNotEnabled } != 0);
    }

    #[test]
    fn traits_selected() {
        let mut node = NodeBuilder::new(Role::Tab);
        node.set_selected(true);
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitSelected } != 0);
    }

    #[test]
    fn traits_selected_false_does_not_set_selected() {
        let mut node = NodeBuilder::new(Role::Tab);
        node.set_selected(false);
        assert!(node_traits(&node) & unsafe { UIAccessibilityTraitSelected } == 0);
    }

    #[test]
    fn traits_plain_button_has_no_modifiers() {
        let node = NodeBuilder::new(Role::Button);
        let t = node_traits(&node);
        assert!(t & unsafe { UIAccessibilityTraitButton } != 0);
        assert!(t & unsafe { UIAccessibilityTraitNotEnabled } == 0);
        assert!(t & unsafe { UIAccessibilityTraitSelected } == 0);
    }

    #[test]
    fn traits_disabled_and_selected_accumulate() {
        let mut node = NodeBuilder::new(Role::Button);
        node.set_disabled();
        node.set_selected(true);
        let t = node_traits(&node);
        assert!(t & unsafe { UIAccessibilityTraitButton } != 0);
        assert!(t & unsafe { UIAccessibilityTraitNotEnabled } != 0);
        assert!(t & unsafe { UIAccessibilityTraitSelected } != 0);
    }

    #[test]
    fn traits_none_for_group() {
        let node = NodeBuilder::new(Role::Group);
        assert_eq!(node_traits(&node), unsafe { UIAccessibilityTraitNone });
    }

    // ---- container_type ----

    #[test]
    fn container_type_data_table_roles() {
        for role in [Role::Table, Role::Grid, Role::TreeGrid, Role::ListGrid] {
            let node = NodeBuilder::new(role);
            assert_eq!(
                node_container_type(&node),
                UIAccessibilityContainerType::DataTable,
                "role {role:?}",
            );
        }
    }

    #[test]
    fn container_type_list_roles() {
        for role in [Role::List, Role::ListBox, Role::DescriptionList, Role::Tree] {
            let node = NodeBuilder::new(role);
            assert_eq!(
                node_container_type(&node),
                UIAccessibilityContainerType::List,
                "role {role:?}",
            );
        }
    }

    #[test]
    fn container_type_landmark_roles() {
        for role in [
            Role::Article,
            Role::Banner,
            Role::Complementary,
            Role::ContentInfo,
            Role::Footer,
            Role::Form,
            Role::Main,
            Role::Navigation,
            Role::Region,
            Role::Search,
        ] {
            let node = NodeBuilder::new(role);
            assert_eq!(
                node_container_type(&node),
                UIAccessibilityContainerType::Landmark,
                "role {role:?}",
            );
        }
    }

    #[test]
    fn container_type_semantic_group_for_group() {
        let node = NodeBuilder::new(Role::Group);
        assert_eq!(
            node_container_type(&node),
            UIAccessibilityContainerType::SemanticGroup,
        );
    }

    #[test]
    fn container_type_none_for_button() {
        let node = NodeBuilder::new(Role::Button);
        assert_eq!(
            node_container_type(&node),
            UIAccessibilityContainerType::None,
        );
    }

    // ---- frame_source ----

    fn node_frame_source(nodes: Vec<(NodeId, NodeBuilder)>, target: NodeId) -> FrameSource {
        let tree = build_tree(nodes);
        let state = tree.state();
        let node = state.node_by_tree_local_id(target, TreeId::ROOT).unwrap();
        NodeWrapper(&node).frame_source()
    }

    #[test]
    fn frame_source_uses_bounding_box_when_present() {
        let mut node = NodeBuilder::new(Role::Button);
        node.set_bounds(Rect {
            x0: 1.0,
            y0: 2.0,
            x1: 3.0,
            y1: 4.0,
        });
        assert_eq!(
            node_frame_source(vec![(ROOT_ID, node)], ROOT_ID),
            FrameSource::Rect(Rect {
                x0: 1.0,
                y0: 2.0,
                x1: 3.0,
                y1: 4.0,
            }),
        );
    }

    #[test]
    fn frame_source_root_without_bounds_uses_view_bounds() {
        let node = NodeBuilder::new(Role::Window);
        assert_eq!(
            node_frame_source(vec![(ROOT_ID, node)], ROOT_ID),
            FrameSource::ViewBounds,
        );
    }

    #[test]
    fn frame_source_non_root_without_bounds_is_zero() {
        const CHILD_ID: NodeId = NodeId(1);
        let mut root = NodeBuilder::new(Role::Window);
        root.set_children(vec![CHILD_ID]);
        let child = NodeBuilder::new(Role::Button);
        assert_eq!(
            node_frame_source(vec![(ROOT_ID, root), (CHILD_ID, child)], CHILD_ID),
            FrameSource::Zero,
        );
    }

    #[test]
    fn frame_source_bounding_box_takes_priority_on_root() {
        let mut node = NodeBuilder::new(Role::Window);
        node.set_bounds(Rect {
            x0: 0.0,
            y0: 0.0,
            x1: 10.0,
            y1: 10.0,
        });
        assert!(matches!(
            node_frame_source(vec![(ROOT_ID, node)], ROOT_ID),
            FrameSource::Rect(_),
        ));
    }
}
