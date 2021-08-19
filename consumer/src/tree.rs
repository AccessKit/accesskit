// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use accesskit_schema::{NodeId, TreeId, TreeUpdate};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, RwLockReadGuard};

use crate::{Node, NodeData, TreeData};

pub(crate) struct ParentAndIndex(pub(crate) NodeId, pub(crate) usize);

pub(crate) struct NodeState {
    pub(crate) parent_and_index: Option<ParentAndIndex>,
    pub(crate) data: NodeData,
}

pub(crate) struct State {
    pub(crate) nodes: HashMap<NodeId, NodeState>,
    pub(crate) root: NodeId,
    pub(crate) data: TreeData,
}

impl State {
    fn validate_global(&self) {
        assert!(self.nodes.contains_key(&self.root));
        if let Some(id) = self.data.focus {
            assert!(self.nodes.contains_key(&id));
        }
        if let Some(id) = self.data.root_scroller {
            assert!(self.nodes.contains_key(&id));
        }
    }

    fn update(&mut self, update: TreeUpdate) {
        // TODO: handle TreeUpdate::clear
        assert!(update.clear.is_none());

        let root = update.root.unwrap_or(self.root);
        let mut pending_nodes: HashMap<NodeId, _> = HashMap::new();
        let mut pending_children = HashMap::new();
        let mut orphans = HashSet::new();

        if root != self.root {
            orphans.insert(self.root);
            self.root = root;
        }

        for node_data in update.nodes {
            let node_id = node_data.id;
            orphans.remove(&node_id);

            let mut seen_child_ids = HashSet::new();
            for (child_index, child_id) in node_data.children.iter().enumerate() {
                assert!(!seen_child_ids.contains(child_id));
                orphans.remove(child_id);
                let parent_and_index = ParentAndIndex(node_id, child_index);
                if let Some(child_state) = self.nodes.get_mut(child_id) {
                    child_state.parent_and_index = Some(parent_and_index);
                } else if let Some(child_data) = pending_nodes.remove(child_id) {
                    let node_state = NodeState {
                        parent_and_index: Some(parent_and_index),
                        data: child_data,
                    };
                    self.nodes.insert(*child_id, node_state);
                } else {
                    pending_children.insert(*child_id, parent_and_index);
                }
                seen_child_ids.insert(child_id);
            }

            if let Some(node_state) = self.nodes.get_mut(&node_id) {
                if node_id == root {
                    node_state.parent_and_index = None
                }
                for child_id in &node_state.data.children {
                    if !seen_child_ids.contains(child_id) {
                        orphans.insert(*child_id);
                    }
                }
                node_state.data = node_data;
            } else if let Some(parent_and_index) = pending_children.remove(&node_id) {
                let node_state = NodeState {
                    parent_and_index: Some(parent_and_index),
                    data: node_data,
                };
                self.nodes.insert(node_id, node_state);
            } else if node_id == root {
                let node_state = NodeState {
                    parent_and_index: None,
                    data: node_data,
                };
                self.nodes.insert(node_id, node_state);
            } else {
                pending_nodes.insert(node_id, node_data);
            }
        }

        if !pending_nodes.is_empty() {
            for (node_id, data) in &pending_nodes {
                println!("unattached: {:?} {:?}", node_id, data.role);
            }
            panic!("unattached nodes");
        }

        assert_eq!(pending_children.len(), 0);

        if !orphans.is_empty() {
            let mut to_remove = HashSet::new();

            fn traverse_orphan(
                nodes: &HashMap<NodeId, NodeState>,
                to_remove: &mut HashSet<NodeId>,
                id: NodeId,
            ) {
                to_remove.insert(id);
                let node = nodes.get(&id).unwrap();
                for child_id in &node.data.children {
                    traverse_orphan(nodes, to_remove, *child_id);
                }
            }

            for id in orphans {
                traverse_orphan(&self.nodes, &mut to_remove, id);
            }

            for id in to_remove {
                self.nodes.remove(&id);
            }
        }

        if let Some(tree) = update.tree {
            assert_eq!(tree.id, self.data.id);
            self.data = tree;
        }

        self.validate_global();
    }

    fn serialize(&self) -> TreeUpdate {
        let mut nodes = Vec::new();

        fn traverse(state: &State, nodes: &mut Vec<NodeData>, id: NodeId) {
            let node = state.nodes.get(&id).unwrap();
            nodes.push(node.data.clone());

            for child_id in &node.data.children {
                traverse(state, nodes, *child_id);
            }
        }

        traverse(self, &mut nodes, self.root);
        assert_eq!(nodes.len(), self.nodes.len());

        TreeUpdate {
            clear: None,
            nodes,
            tree: Some(self.data.clone()),
            root: Some(self.root),
        }
    }
}

pub struct Reader<'a> {
    pub(crate) tree: &'a Arc<Tree>,
    pub(crate) state: RwLockReadGuard<'a, State>,
}

impl Reader<'_> {
    pub fn node_by_id(&self, id: NodeId) -> Option<Node<'_>> {
        self.state.nodes.get(&id).map(|node_state| Node {
            tree_reader: &self,
            state: node_state,
        })
    }

    pub fn root(&self) -> Node<'_> {
        self.node_by_id(self.state.root).unwrap()
    }

    pub fn id(&self) -> &TreeId {
        &self.state.data.id
    }
}

pub struct Tree {
    state: RwLock<State>,
}

impl Tree {
    pub fn new(mut initial_state: TreeUpdate) -> Arc<Self> {
        assert!(initial_state.clear.is_none());

        let mut state = State {
            nodes: HashMap::new(),
            root: initial_state.root.take().unwrap(),
            data: initial_state.tree.take().unwrap(),
        };
        state.update(initial_state);
        Arc::new(Self {
            state: RwLock::new(state),
        })
    }

    pub fn update(&self, update: TreeUpdate) {
        let mut state = self.state.write().unwrap();
        state.update(update)
    }

    // Intended for debugging.
    pub fn serialize(&self) -> TreeUpdate {
        let state = self.state.read().unwrap();
        state.serialize()
    }

    // https://github.com/rust-lang/rust-clippy/issues/7296
    #[allow(clippy::needless_lifetimes)]
    pub fn read<'a>(self: &'a Arc<Tree>) -> Reader<'a> {
        Reader {
            tree: self,
            state: self.state.read().unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use accesskit_schema::{Node, NodeId, Role, StringEncoding, Tree, TreeId, TreeUpdate};
    use std::collections::BTreeSet;
    use std::num::NonZeroU64;

    const ONE: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(1) };
    const TWO: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(2) };
    const THREE: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(3) };
    const FOUR: NonZeroU64 = unsafe { NonZeroU64::new_unchecked(4) };

    #[test]
    fn init_tree_with_root_node() {
        let update = TreeUpdate {
            clear: None,
            nodes: vec![
                new_node(NodeId(ONE), Role::Application),
            ],
            tree: Some(Tree {
                id: TreeId("test_tree".to_string()),
                source_string_encoding: StringEncoding::Utf16,
                parent: None,
                focus: None,
                root_scroller: None,
            }),
            root: Some(NodeId(ONE)),
        };
        let tree = super::Tree::new(update);
        assert_eq!(&TreeId("test_tree".to_string()), tree.read().id());
        assert_eq!(NodeId(ONE), tree.read().root().id());
        assert_eq!(true, tree.read().root().parent().is_none());
    }

    #[test]
    fn root_node_has_children() {
        let update = TreeUpdate {
            clear: None,
            nodes: vec![
                Node {
                    children: vec![NodeId(TWO), NodeId(THREE)],
                    ..new_node(NodeId(ONE), Role::Application)
                },
                new_node(NodeId(TWO), Role::Window),
                new_node(NodeId(THREE), Role::Window),
            ],
            tree: Some(Tree {
                id: TreeId("test_tree".to_string()),
                source_string_encoding: StringEncoding::Utf16,
                parent: None,
                focus: None,
                root_scroller: None,
            }),
            root: Some(NodeId(ONE)),
        };
        let tree = super::Tree::new(update);
        let reader = tree.read();
        assert_eq!(NodeId(ONE), reader.node_by_id(NodeId(TWO)).unwrap().parent().unwrap().id());
        assert_eq!(NodeId(ONE), reader.node_by_id(NodeId(THREE)).unwrap().parent().unwrap().id());
        assert_eq!(2, reader.root().children().count());
    }

    #[test]
    fn add_child_to_root_node() {
        let root_node = new_node(NodeId(ONE), Role::Application);
        let first_update = TreeUpdate {
            clear: None,
            nodes: vec![
                root_node.clone()
            ],
            tree: Some(Tree {
                id: TreeId("test_tree".to_string()),
                source_string_encoding: StringEncoding::Utf16,
                parent: None,
                focus: None,
                root_scroller: None,
            }),
            root: Some(NodeId(ONE)),
        };
        let tree = super::Tree::new(first_update);
        assert_eq!(0, tree.read().root().children().count());
        let second_update = TreeUpdate {
            clear: None,
            nodes: vec![
                Node {
                    children: vec![NodeId(TWO)],
                    ..root_node
                },
                new_node(NodeId(TWO), Role::RootWebArea)
            ],
            tree: None,
            root: None,
        };
        tree.update(second_update);
        assert_eq!(1, tree.read().root().children().count());
        assert_eq!(NodeId(TWO), tree.read().root().children().next().unwrap().id());
    }

    #[test]
    fn remove_child_from_root_node() {
        let root_node = new_node(NodeId(ONE), Role::Application);
        let first_update = TreeUpdate {
            clear: None,
            nodes: vec![
                Node {
                    children: vec![NodeId(TWO)],
                    ..root_node.clone()
                },
                new_node(NodeId(TWO), Role::RootWebArea)
            ],
            tree: Some(Tree {
                id: TreeId("test_tree".to_string()),
                source_string_encoding: StringEncoding::Utf16,
                parent: None,
                focus: None,
                root_scroller: None,
            }),
            root: Some(NodeId(ONE)),
        };
        let tree = super::Tree::new(first_update);
        assert_eq!(1, tree.read().root().children().count());
        let second_update = TreeUpdate {
            clear: None,
            nodes: vec![
                root_node
            ],
            tree: None,
            root: None,
        };
        tree.update(second_update);
        assert_eq!(0, tree.read().root().children().count());
        assert_eq!(true, tree.read().node_by_id(NodeId(TWO)).is_none());
    }

    #[test]
    fn move_focus_between_siblings() {
        let first_update = TreeUpdate {
            clear: None,
            nodes: vec![
                Node {
                    children: vec![NodeId(TWO)],
                    ..new_node(NodeId(ONE), Role::Application)
                },
                Node {
                    children: vec![NodeId(THREE), NodeId(FOUR)],
                    ..new_node(NodeId(TWO), Role::Window)
                },
                new_node(NodeId(THREE), Role::Button),
                new_node(NodeId(FOUR), Role::Button)
            ],
            tree: Some(Tree {
                id: TreeId("test_tree".to_string()),
                source_string_encoding: StringEncoding::Utf16,
                parent: None,
                focus: Some(NodeId(THREE)),
                root_scroller: None,
            }),
            root: Some(NodeId(ONE)),
        };
        let tree = super::Tree::new(first_update);
        assert_eq!(true, tree.read().node_by_id(NodeId(THREE)).unwrap().is_focused());
        let second_update = TreeUpdate {
            clear: None,
            nodes: vec![],
            tree: Some(Tree {
                id: TreeId("test_tree".to_string()),
                source_string_encoding: StringEncoding::Utf16,
                parent: None,
                focus: Some(NodeId(FOUR)),
                root_scroller: None,
            }),
            root: None,
        };
        tree.update(second_update);
        assert_eq!(true, tree.read().node_by_id(NodeId(FOUR)).unwrap().is_focused());
        assert_eq!(false, tree.read().node_by_id(NodeId(THREE)).unwrap().is_focused());
    }

    fn new_node(id: NodeId, role: Role) -> Node {
        Node {
            id,
            role,
            bounds: None,
            children: vec![],
            actions: BTreeSet::new(),
            name: None,
            name_from: None,
            description: None,
            description_from: None,
            value: None,
            autofill_available: false,
            expanded: None,
            default: false,
            editable: false,
            focusable: false,
            orientation: None,
            hovered: false,
            ignored: false,
            invisible: false,
            linked: false,
            multiline: false,
            multiselectable: false,
            protected: false,
            required: false,
            visited: false,
            busy: false,
            nonatomic_text_field_root: false,
            container_live_atomic: false,
            container_live_busy: false,
            live_atomic: false,
            modal: false,
            canvas_has_fallback: false,
            scrollable: false,
            clickable: false,
            clips_children: false,
            not_user_selectable_style: false,
            selected: None,
            selected_from_focus: false,
            grabbed: None,
            drop_effects: BTreeSet::new(),
            is_line_breaking_object: false,
            is_page_breaking_object: false,
            has_aria_attribute: false,
            touch_pass_through: false,
            indirect_children: vec![],
            active_descendant: None,
            error_message: None,
            in_page_link_target: None,
            member_of: None,
            next_on_line: None,
            previous_on_line: None,
            popup_for: None,
            controls: vec![],
            details: vec![],
            described_by: vec![],
            flow_to: vec![],
            labelled_by: vec![],
            radio_groups: vec![],
            markers: vec![],
            text_direction: None,
            character_offsets: vec![],
            words: vec![],
            custom_actions: vec![],
            access_key: None,
            invalid_state: None,
            auto_complete: None,
            checked_state: None,
            checked_state_description: None,
            child_tree: None,
            class_name: None,
            container_live_relevant: None,
            container_live_status: None,
            css_display: None,
            font_family: None,
            html_tag: None,
            inner_html: None,
            input_type: None,
            key_shortcuts: None,
            language: None,
            live_relevant: None,
            live_status: None,
            placeholder: None,
            aria_role: None,
            role_description: None,
            tooltip: None,
            url: None,
            default_action_verb: None,
            scroll_x: None,
            scroll_x_min: None,
            scroll_x_max: None,
            scroll_y: None,
            scroll_y_min: None,
            scroll_y_max: None,
            text_selection: None,
            aria_column_count: None,
            aria_cell_column_index: None,
            aria_cell_column_span: None,
            aria_row_count: None,
            aria_cell_row_index: None,
            aria_cell_row_span: None,
            table_row_count: None,
            table_column_count: None,
            table_header: None,
            table_row_index: None,
            table_row_header: None,
            table_column_index: None,
            table_column_header: None,
            table_cell_column_index: None,
            table_cell_column_span: None,
            table_cell_row_index: None,
            table_cell_row_span: None,
            sort_direction: None,
            hierarchical_level: None,
            read_only: false,
            disabled: false,
            set_size: None,
            pos_in_set: None,
            color_value: None,
            aria_current: None,
            background_color: None,
            foreground_color: None,
            has_popup: None,
            list_style: None,
            text_align: None,
            vertical_offset: None,
            bold: false,
            italic: false,
            overline: None,
            strikethrough: None,
            underline: None,
            previous_focus: None,
            next_focus: None,
            value_for_range: None,
            min_value_for_range: None,
            max_value_for_range: None,
            step_value_for_range: None,
            font_size: None,
            font_weight: None,
            text_indent: None,
        }
    }
}
