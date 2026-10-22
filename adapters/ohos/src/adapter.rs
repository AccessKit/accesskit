// Copyright 2026 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use std::{
    collections::{HashMap, HashSet},
    ffi::CStr,
    sync::{Arc, Mutex, MutexGuard},
};

use accesskit::{
    Action, ActionData, ActionHandler, ActionRequest, ActivationHandler, Node as NodeData,
    NodeId as LocalNodeId, Role, TextSelection, TreeId, TreeInfo as TreeData, TreeUpdate,
};
use accesskit_consumer::{
    FilterResult, FullNodeId as NodeId, NodeRef as Node, Tree, TreeChangeHandler,
};
use ohos_accessibility_binding::{
    AccessibilityError, ActionArguments, ActionType, ElementInfo, ElementInfoList, EventInfo,
    EventType, FocusMoveDirection, FocusType, Provider, ProviderCallbacks, ProviderRegistration,
    Result, SearchMode,
};

use crate::node::{filter, populate_element, usize_to_i32};

const ROOT_QUERY_ID: i64 = -1;
const ROOT_ELEMENT_ID: i32 = 0;
pub(crate) const ROOT_PARENT_ID: i32 = -2_100_000;
const PLACEHOLDER_ROOT_ID: LocalNodeId = LocalNodeId(0);

struct IgnoreChanges;

impl TreeChangeHandler for IgnoreChanges {
    fn node_added(&mut self, _node: &Node) {}
    fn node_updated(&mut self, _old_node: &Node, _new_node: &Node) {}
    fn focus_moved(&mut self, _old_node: Option<&Node>, _new_node: Option<&Node>) {}
    fn node_removed(&mut self, _node: &Node) {}
}

#[derive(Debug, Default)]
pub(crate) struct NodeIdMap {
    platform_to_accesskit: HashMap<i32, NodeId>,
    accesskit_to_platform: HashMap<NodeId, i32>,
    next_platform_id: i32,
}

impl NodeIdMap {
    pub(crate) fn get_or_create(&mut self, node: &Node) -> i32 {
        if node.is_root() {
            self.platform_to_accesskit
                .insert(ROOT_ELEMENT_ID, node.id());
            self.accesskit_to_platform
                .insert(node.id(), ROOT_ELEMENT_ID);
            self.next_platform_id = self.next_platform_id.max(1);
            return ROOT_ELEMENT_ID;
        }
        if let Some(id) = self.accesskit_to_platform.get(&node.id()) {
            return *id;
        }
        let id = self.next_platform_id.max(1);
        self.next_platform_id = id.saturating_add(1);
        self.platform_to_accesskit.insert(id, node.id());
        self.accesskit_to_platform.insert(node.id(), id);
        id
    }

    fn get_accesskit(&self, platform_id: i64, root: NodeId) -> Option<NodeId> {
        if platform_id == ROOT_QUERY_ID || platform_id == i64::from(ROOT_ELEMENT_ID) {
            return Some(root);
        }
        let platform_id = i32::try_from(platform_id).ok()?;
        self.platform_to_accesskit.get(&platform_id).copied()
    }
}

enum TreeLifecycle {
    Inactive,
    Placeholder(Tree),
    Active(Tree),
}

impl TreeLifecycle {
    fn get(&self) -> Option<&Tree> {
        match self {
            Self::Inactive => None,
            Self::Placeholder(tree) | Self::Active(tree) => Some(tree),
        }
    }

    fn get_active_mut(&mut self) -> Option<&mut Tree> {
        match self {
            Self::Active(tree) => Some(tree),
            Self::Inactive | Self::Placeholder(_) => None,
        }
    }
}

#[derive(Clone, Copy)]
struct ProviderAddress(usize);

impl ProviderAddress {
    fn send(
        self,
        event_type: EventType,
        node: Option<&Node>,
        ids: &mut NodeIdMap,
        accessibility_focus: Option<NodeId>,
        screen_offset: (i32, i32),
    ) -> Result<()> {
        let element = if let Some(node) = node {
            let mut value = ElementInfo::new()?;
            populate_element(node, ids, accessibility_focus, screen_offset, &mut value)?;
            Some(value)
        } else {
            None
        };
        // The event borrows this final element allocation until it is dropped.
        let mut event = EventInfo::new()?;
        event.set_event_type(event_type)?;
        if let Some(element) = element.as_ref() {
            event.set_element_info(element)?;
        }

        let provider = unsafe { Provider::from_raw(self.0 as *mut _) }?;
        provider.send_event(&event, None);
        Ok(())
    }
}

struct State {
    tree: TreeLifecycle,
    activation_handler: Box<dyn ActivationHandler + Send>,
    ids: NodeIdMap,
    accessibility_focus: Option<NodeId>,
    host_is_focused: bool,
    screen_offset: (i32, i32),
    provider: ProviderAddress,
}

impl State {
    fn initialize_tree(&mut self) {
        if !matches!(self.tree, TreeLifecycle::Inactive) {
            return;
        }
        self.tree = match self.activation_handler.request_initial_tree() {
            Some(update) => TreeLifecycle::Active(Tree::new(update, self.host_is_focused)),
            None => {
                let update = TreeUpdate {
                    nodes: vec![(PLACEHOLDER_ROOT_ID, NodeData::new(Role::Window))],
                    tree: Some(TreeData::new(PLACEHOLDER_ROOT_ID)),
                    tree_id: TreeId::ROOT,
                    focus: PLACEHOLDER_ROOT_ID,
                };
                TreeLifecycle::Placeholder(Tree::new(update, self.host_is_focused))
            }
        };
    }
}

struct Callbacks {
    state: Arc<Mutex<State>>,
    action_handler: Arc<Mutex<Box<dyn ActionHandler + Send>>>,
}

impl Callbacks {
    fn lock(&self) -> Result<MutexGuard<'_, State>> {
        self.state
            .lock()
            .map_err(|_| AccessibilityError::LockPoisoned)
    }
}

impl ProviderCallbacks for Callbacks {
    fn find_node_infos_by_id(
        &self,
        element_id: i64,
        mode: SearchMode,
        _request_id: i32,
        elements: &mut ElementInfoList<'_>,
    ) -> Result<()> {
        let mut state = self.lock()?;
        state.initialize_tree();
        let State {
            tree,
            ids,
            accessibility_focus,
            screen_offset,
            ..
        } = &mut *state;
        let tree = tree.get().expect("tree initialized above");
        let root = tree.state().root_id();
        let target = ids
            .get_accesskit(element_id, root)
            .and_then(|id| tree.state().node_by_id(id))
            .ok_or(AccessibilityError::BadParameter)?;
        let node_ids = search_node_ids(&target, mode);
        for id in node_ids {
            let node = tree
                .state()
                .node_by_id(id)
                .ok_or(AccessibilityError::Failed)?;
            let mut element = elements.add()?;
            populate_element(
                &node,
                ids,
                *accessibility_focus,
                *screen_offset,
                &mut element,
            )?;
        }
        Ok(())
    }

    fn find_node_infos_by_text(
        &self,
        element_id: i64,
        text: &CStr,
        _request_id: i32,
        elements: &mut ElementInfoList<'_>,
    ) -> Result<()> {
        let needle = text.to_string_lossy().to_lowercase();
        let mut state = self.lock()?;
        state.initialize_tree();
        let State {
            tree,
            ids,
            accessibility_focus,
            screen_offset,
            ..
        } = &mut *state;
        let tree = tree.get().expect("tree initialized above");
        let root = tree.state().root_id();
        let target = ids
            .get_accesskit(element_id, root)
            .and_then(|id| tree.state().node_by_id(id))
            .ok_or(AccessibilityError::BadParameter)?;
        let node_ids = subtree_node_ids(&target);
        for id in node_ids {
            let node = tree
                .state()
                .node_by_id(id)
                .ok_or(AccessibilityError::Failed)?;
            if !node_search_text(&node).to_lowercase().contains(&needle) {
                continue;
            }
            let mut element = elements.add()?;
            populate_element(
                &node,
                ids,
                *accessibility_focus,
                *screen_offset,
                &mut element,
            )?;
        }
        Ok(())
    }

    fn find_focused_node(
        &self,
        _element_id: i64,
        focus_type: FocusType,
        _request_id: i32,
        element: &mut ElementInfo<'_>,
    ) -> Result<()> {
        let mut state = self.lock()?;
        state.initialize_tree();
        let State {
            tree,
            ids,
            accessibility_focus,
            screen_offset,
            ..
        } = &mut *state;
        let tree = tree.get().expect("tree initialized above");
        let id = match focus_type {
            FocusType::Input => tree.state().focus_id().ok_or(AccessibilityError::Failed)?,
            FocusType::Accessibility => accessibility_focus.ok_or(AccessibilityError::Failed)?,
            FocusType::Invalid => return Err(AccessibilityError::BadParameter),
        };
        let node = tree
            .state()
            .node_by_id(id)
            .ok_or(AccessibilityError::Failed)?;
        populate_element(&node, ids, *accessibility_focus, *screen_offset, element)
    }

    fn find_next_focus_node(
        &self,
        element_id: i64,
        direction: FocusMoveDirection,
        _request_id: i32,
        element: &mut ElementInfo<'_>,
    ) -> Result<()> {
        let mut state = self.lock()?;
        state.initialize_tree();
        let State {
            tree,
            ids,
            accessibility_focus,
            screen_offset,
            ..
        } = &mut *state;
        let tree = tree.get().expect("tree initialized above");
        let root = tree.state().root_id();
        let current = ids
            .get_accesskit(element_id, root)
            .ok_or(AccessibilityError::BadParameter)?;
        let focusable = subtree_node_ids(&tree.state().root())
            .into_iter()
            .filter(|id| {
                tree.state()
                    .node_by_id(*id)
                    .is_some_and(|node| node.is_focusable(&filter))
            })
            .collect::<Vec<_>>();
        let current_index = focusable.iter().position(|id| *id == current);
        let next = match direction {
            FocusMoveDirection::Forward | FocusMoveDirection::Right | FocusMoveDirection::Down => {
                current_index.map_or_else(|| focusable.first(), |index| focusable.get(index + 1))
            }
            FocusMoveDirection::Backward | FocusMoveDirection::Left | FocusMoveDirection::Up => {
                current_index.map_or_else(
                    || focusable.last(),
                    |index| index.checked_sub(1).and_then(|index| focusable.get(index)),
                )
            }
            FocusMoveDirection::Invalid => None,
        }
        .copied()
        .ok_or(AccessibilityError::Failed)?;
        let node = tree
            .state()
            .node_by_id(next)
            .ok_or(AccessibilityError::Failed)?;
        populate_element(&node, ids, *accessibility_focus, *screen_offset, element)
    }

    fn execute_action(
        &self,
        element_id: i64,
        action: ActionType,
        arguments: &ActionArguments<'_>,
        _request_id: i32,
    ) -> Result<()> {
        let mut state = self.lock()?;
        state.initialize_tree();
        let root = state
            .tree
            .get()
            .expect("tree initialized above")
            .state()
            .root_id();
        let target = state
            .ids
            .get_accesskit(element_id, root)
            .ok_or(AccessibilityError::BadParameter)?;

        match action {
            ActionType::GainAccessibilityFocus => {
                state.accessibility_focus = Some(target);
                return send_target_event(&mut state, target, EventType::AccessibilityFocused);
            }
            ActionType::ClearAccessibilityFocus => {
                state.accessibility_focus = None;
                return send_target_event(&mut state, target, EventType::AccessibilityFocusCleared);
            }
            #[cfg(feature = "api-15")]
            ActionType::NextHtmlItem | ActionType::PreviousHtmlItem => {
                let forward = action == ActionType::NextHtmlItem;
                let next = adjacent_focusable_node(&state, target, forward)
                    .ok_or(AccessibilityError::Failed)?;
                state.accessibility_focus = Some(next);
                return send_target_event(&mut state, next, EventType::AccessibilityFocused);
            }
            _ => {}
        }

        let request = action_request(&state, target, action, arguments)?;
        drop(state);
        self.action_handler
            .lock()
            .map_err(|_| AccessibilityError::LockPoisoned)?
            .do_action(request);
        Ok(())
    }

    fn clear_focused_node(&self) -> Result<()> {
        let mut state = self.lock()?;
        let Some(target) = state.accessibility_focus.take() else {
            return Ok(());
        };
        send_target_event(&mut state, target, EventType::AccessibilityFocusCleared)
    }

    fn cursor_position(&self, element_id: i64, _request_id: i32) -> Result<i32> {
        let mut state = self.lock()?;
        state.initialize_tree();
        let State { tree, ids, .. } = &mut *state;
        let tree = tree.get().expect("tree initialized above");
        let root = tree.state().root_id();
        let target = ids
            .get_accesskit(element_id, root)
            .ok_or(AccessibilityError::BadParameter)?;
        let node = tree
            .state()
            .node_by_id(target)
            .ok_or(AccessibilityError::BadParameter)?;
        node.text_selection_focus()
            .map(|position| position.to_global_utf16_index())
            .map(usize_to_i32)
            .transpose()?
            .ok_or(AccessibilityError::Failed)
    }
}

/// AccessKit adapter for ArkUI native accessibility providers.
pub struct Adapter<'a> {
    _registration: ProviderRegistration<'a>,
    state: Arc<Mutex<State>>,
}

impl<'a> Adapter<'a> {
    pub fn new(
        provider: Provider<'a>,
        activation_handler: impl ActivationHandler + Send + 'static,
        action_handler: impl ActionHandler + Send + 'static,
    ) -> Result<Self> {
        Self::new_with_registrar(
            provider,
            activation_handler,
            action_handler,
            |provider, callbacks| provider.register_callbacks(callbacks),
        )
    }

    /// Register the multi-instance callback shape introduced in API 15.
    ///
    /// Give each registration a unique ID. This keeps callbacks for multiple
    /// XComponent windows and recreated surfaces isolated in the Rust binding.
    #[cfg(feature = "api-15")]
    pub fn new_with_instance(
        provider: Provider<'a>,
        instance_id: &str,
        activation_handler: impl ActivationHandler + Send + 'static,
        action_handler: impl ActionHandler + Send + 'static,
    ) -> Result<Self> {
        Self::new_with_registrar(
            provider,
            activation_handler,
            action_handler,
            |provider, callbacks| provider.register_callbacks_with_instance(instance_id, callbacks),
        )
    }

    fn new_with_registrar(
        provider: Provider<'a>,
        activation_handler: impl ActivationHandler + Send + 'static,
        action_handler: impl ActionHandler + Send + 'static,
        register: impl FnOnce(Provider<'a>, Callbacks) -> Result<ProviderRegistration<'a>>,
    ) -> Result<Self> {
        let action_handler: Arc<Mutex<Box<dyn ActionHandler + Send>>> =
            Arc::new(Mutex::new(Box::new(action_handler)));
        let state = Arc::new(Mutex::new(State {
            tree: TreeLifecycle::Inactive,
            activation_handler: Box::new(activation_handler),
            ids: NodeIdMap::default(),
            accessibility_focus: None,
            host_is_focused: true,
            screen_offset: (0, 0),
            provider: ProviderAddress(provider.as_raw() as usize),
        }));
        let callbacks = Callbacks {
            state: state.clone(),
            action_handler,
        };
        let registration = register(provider, callbacks)?;
        Ok(Self {
            _registration: registration,
            state,
        })
    }

    /// Apply an update only after ArkUI has activated this adapter by querying
    /// its tree.
    pub fn update_if_active(&self, update_factory: impl FnOnce() -> TreeUpdate) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AccessibilityError::LockPoisoned)?;
        let old_focus = state.tree.get().map(|tree| tree.state().focus_id_in_tree());
        match &mut state.tree {
            TreeLifecycle::Inactive => return Ok(()),
            TreeLifecycle::Placeholder(_) => {
                state.tree =
                    TreeLifecycle::Active(Tree::new(update_factory(), state.host_is_focused));
            }
            TreeLifecycle::Active(tree) => {
                tree.update_and_process_changes(update_factory(), &mut IgnoreChanges)
            }
        }
        let new_focus = state.tree.get().map(|tree| tree.state().focus_id_in_tree());
        if state.accessibility_focus.is_some_and(|id| {
            state
                .tree
                .get()
                .is_none_or(|tree| tree.state().node_by_id(id).is_none())
        }) {
            state.accessibility_focus = None;
        }
        send_root_event(&mut state, EventType::PageContentUpdate)?;
        if old_focus != new_focus
            && let Some(new_focus) = new_focus
        {
            send_target_event(&mut state, new_focus, EventType::FocusNodeUpdate)?;
        }
        Ok(())
    }

    pub fn set_host_focus_state(&self, is_focused: bool) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AccessibilityError::LockPoisoned)?;
        state.host_is_focused = is_focused;
        if let Some(tree) = state.tree.get_active_mut() {
            tree.update_host_focus_state_and_process_changes(is_focused, &mut IgnoreChanges);
        }
        Ok(())
    }

    /// Apply an additional provider-local pixel offset to AccessKit bounds.
    ///
    /// ArkUI positions the provider's child tree at the XComponent/custom-node
    /// origin automatically, so the default `(0, 0)` is correct for normal
    /// hosts. Use this only when the AccessKit tree has a nested local origin.
    pub fn set_screen_offset(&self, x: i32, y: i32) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AccessibilityError::LockPoisoned)?;
        if state.screen_offset == (x, y) {
            return Ok(());
        }
        state.screen_offset = (x, y);
        if state.tree.get().is_some() {
            send_root_event(&mut state, EventType::PageStateUpdate)?;
        }
        Ok(())
    }

    /// Drop the retained tree. It will be requested again on the next native
    /// accessibility query.
    pub fn deactivate(&self) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| AccessibilityError::LockPoisoned)?;
        state.tree = TreeLifecycle::Inactive;
        state.accessibility_focus = None;
        Ok(())
    }
}

fn search_node_ids(target: &Node, mode: SearchMode) -> Vec<NodeId> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    let push = |node: Node, result: &mut Vec<NodeId>, seen: &mut HashSet<NodeId>| {
        if filter(&node) == FilterResult::Include && seen.insert(node.id()) {
            result.push(node.id());
        }
    };

    // CURRENT is represented by zero. ArkUI's recursive example also expects
    // the queried node itself in the result.
    push(*target, &mut result, &mut seen);
    if mode.contains(SearchMode::PREDECESSORS) {
        let mut parent = target.filtered_parent(&filter);
        while let Some(node) = parent {
            push(node, &mut result, &mut seen);
            parent = node.filtered_parent(&filter);
        }
    }
    if mode.contains(SearchMode::SIBLINGS)
        && let Some(parent) = target.filtered_parent(&filter)
    {
        for node in parent.filtered_children(&filter) {
            push(node, &mut result, &mut seen);
        }
    }
    if mode.contains(SearchMode::CHILDREN) {
        for node in target.filtered_children(&filter) {
            push(node, &mut result, &mut seen);
        }
    }
    if mode.contains(SearchMode::RECURSIVE_CHILDREN) {
        for id in subtree_node_ids(target) {
            if seen.insert(id) {
                result.push(id);
            }
        }
    }
    result
}

fn subtree_node_ids(root: &Node) -> Vec<NodeId> {
    let mut result = Vec::new();
    let mut stack = vec![*root];
    while let Some(node) = stack.pop() {
        match filter(&node) {
            FilterResult::Include => result.push(node.id()),
            FilterResult::ExcludeSubtree => continue,
            FilterResult::ExcludeNode => {}
        }
        let children = node.children().collect::<Vec<_>>();
        stack.extend(children.into_iter().rev());
    }
    result
}

fn node_search_text(node: &Node) -> String {
    [node.label(), node.value(), node.description()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(feature = "api-15")]
fn adjacent_focusable_node(state: &State, target: NodeId, forward: bool) -> Option<NodeId> {
    let tree = state.tree.get()?;
    let nodes = subtree_node_ids(&tree.state().root())
        .into_iter()
        .filter(|id| {
            tree.state()
                .node_by_id(*id)
                .is_some_and(|node| node.is_focusable(&filter))
        })
        .collect::<Vec<_>>();
    let index = nodes.iter().position(|id| *id == target)?;
    if forward {
        nodes.get(index + 1).copied()
    } else {
        index
            .checked_sub(1)
            .and_then(|index| nodes.get(index).copied())
    }
}

fn action_request(
    state: &State,
    target: NodeId,
    action: ActionType,
    arguments: &ActionArguments<'_>,
) -> Result<ActionRequest> {
    let tree = state.tree.get().ok_or(AccessibilityError::Failed)?;
    let node = tree
        .state()
        .node_by_id(target)
        .ok_or(AccessibilityError::BadParameter)?;
    let (action, data) = match action {
        ActionType::Click => (Action::Click, None),
        ActionType::ScrollForward => {
            let action = [Action::ScrollDown, Action::ScrollRight]
                .into_iter()
                .find(|action| node.supports_action(*action, &filter))
                .ok_or(AccessibilityError::Unsupported)?;
            (action, None)
        }
        ActionType::ScrollBackward => {
            let action = [Action::ScrollUp, Action::ScrollLeft]
                .into_iter()
                .find(|action| node.supports_action(*action, &filter))
                .ok_or(AccessibilityError::Unsupported)?;
            (action, None)
        }
        ActionType::SetText => {
            let value = argument_string(
                arguments,
                &["ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE", "setText", "text"],
            )
            .ok_or(AccessibilityError::BadParameter)?;
            (Action::SetValue, Some(ActionData::Value(value.into())))
        }
        ActionType::SelectText | ActionType::SetCursorPosition => {
            let start = argument_usize(
                arguments,
                &["ACTION_ARGUMENT_SELECTION_START_INT", "selectTextBegin"],
            )
            .ok_or(AccessibilityError::BadParameter)?;
            let end = if action == ActionType::SetCursorPosition {
                start
            } else {
                argument_usize(
                    arguments,
                    &[
                        "ACTION_ARGUMENT_SELECTION_END_INT",
                        "selectTextEnd",
                        "TextEnd",
                    ],
                )
                .ok_or(AccessibilityError::BadParameter)?
            };
            let anchor = node
                .text_position_from_global_utf16_index(start)
                .ok_or(AccessibilityError::BadParameter)?;
            let focus = node
                .text_position_from_global_utf16_index(end)
                .ok_or(AccessibilityError::BadParameter)?;
            (
                Action::SetTextSelection,
                Some(ActionData::SetTextSelection(TextSelection {
                    anchor: anchor.to_raw(),
                    focus: focus.to_raw(),
                })),
            )
        }
        _ => return Err(AccessibilityError::Unsupported),
    };
    if !node.supports_action(action, &filter) {
        return Err(AccessibilityError::Unsupported);
    }
    let (target_node, target_tree) = node.locate();
    Ok(ActionRequest {
        action,
        target_tree,
        target_node,
        data,
    })
}

fn argument_string(arguments: &ActionArguments<'_>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        arguments
            .get(key)
            .ok()
            .flatten()
            .map(|value| value.to_string_lossy().into_owned())
    })
}

fn argument_usize(arguments: &ActionArguments<'_>, keys: &[&str]) -> Option<usize> {
    argument_string(arguments, keys)?.parse().ok()
}

fn send_target_event(state: &mut State, target: NodeId, event_type: EventType) -> Result<()> {
    let State {
        tree,
        ids,
        accessibility_focus,
        screen_offset,
        provider,
        ..
    } = state;
    let tree = tree.get().ok_or(AccessibilityError::Failed)?;
    let node = tree
        .state()
        .node_by_id(target)
        .ok_or(AccessibilityError::BadParameter)?;
    provider.send(
        event_type,
        Some(&node),
        ids,
        *accessibility_focus,
        *screen_offset,
    )
}

fn send_root_event(state: &mut State, event_type: EventType) -> Result<()> {
    let State {
        tree,
        ids,
        accessibility_focus,
        screen_offset,
        provider,
        ..
    } = state;
    let tree = tree.get().ok_or(AccessibilityError::Failed)?;
    let root = tree.state().root();
    provider.send(
        event_type,
        Some(&root),
        ids,
        *accessibility_focus,
        *screen_offset,
    )
}
