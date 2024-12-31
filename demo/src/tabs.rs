use accesskit::{Action, ActionRequest, Node, NodeId, Orientation, Role};

use crate::{node_id, Key, Widget};

pub(crate) trait Tab: Widget {
    fn title(&self) -> &str;
}

pub(crate) struct TabView {
    id: &'static str,
    tabs: Vec<Box<dyn Tab + Send>>,
    has_focus: bool,
    focused_tab: NodeId,
    selected_tab_index: usize,
    tab_ids: Vec<NodeId>,
    tab_panel_ids: Vec<NodeId>,
}

impl TabView {
    pub(crate) fn new(id: &'static str, tabs: Vec<Box<dyn Tab + Send>>) -> Self {
        let tab_ids = (0..tabs.len())
            .map(|i| node_id!(id, i))
            .collect::<Vec<NodeId>>();
        let focused_tab = tab_ids[0];
        let tab_panel_ids = (0..tabs.len()).map(|i| node_id!(id, i)).collect();
        Self {
            id,
            tabs,
            has_focus: false,
            focused_tab,
            selected_tab_index: 0,
            tab_ids,
            tab_panel_ids,
        }
    }

    fn current_tab(&self) -> &dyn Tab {
        self.tabs[self.selected_tab_index].as_ref()
    }

    fn current_tab_mut(&mut self) -> &mut dyn Tab {
        self.tabs[self.selected_tab_index].as_mut()
    }
}

impl Widget for TabView {
    fn key_pressed(&mut self, key: Key) -> bool {
        if !self.has_focus {
            let tab = self.current_tab_mut();
            if tab.key_pressed(key) {
                return true;
            } else {
                return false;
            }
        }

        match key {
            Key::Tab => {
                self.has_focus = false;
                let tab = self.current_tab_mut();
                tab.set_focused(true);
                true
            }
            Key::Left => {
                self.selected_tab_index = self.selected_tab_index.saturating_sub(1);
                self.focused_tab = self.tab_ids[self.selected_tab_index];
                true
            }
            Key::Right => {
                self.selected_tab_index = (self.selected_tab_index + 1).min(self.tabs.len() - 1);
                self.focused_tab = self.tab_ids[self.selected_tab_index];
                true
            }
            _ => false,
        }
    }

    fn has_focus(&self) -> bool {
        self.has_focus
    }

    fn set_focused(&mut self, has_focus: bool) {
        self.has_focus = has_focus;
    }

    fn render(&self, update: &mut accesskit::TreeUpdate) -> accesskit::NodeId {
        let container_id = node_id!(self.id);
        let mut container = Node::new(Role::GenericContainer);
        let tab_list_id = node_id!(self.id);
        container.set_children(vec![
            tab_list_id,
            self.tab_panel_ids[self.selected_tab_index],
        ]);
        update.nodes.push((container_id, container));
        let mut tab_list = Node::new(Role::TabList);
        tab_list.set_children(self.tab_ids.iter().copied().collect::<Vec<NodeId>>());
        tab_list.set_orientation(Orientation::Horizontal);
        tab_list.set_size_of_set(self.tabs.len());
        update.nodes.push((tab_list_id, tab_list));
        for (i, tab) in self.tabs.iter().enumerate() {
            let mut tab_node = Node::new(Role::Tab);
            tab_node.set_controls(vec![self.tab_panel_ids[self.selected_tab_index]]);
            tab_node.set_label(tab.title());
            tab_node.set_position_in_set(i);
            tab_node.set_selected(i == self.selected_tab_index);
            tab_node.add_action(Action::Click);
            tab_node.add_action(Action::Focus);
            update.nodes.push((self.tab_ids[i], tab_node));
        }
        let tab = self.current_tab();
        let mut tab_panel = Node::new(Role::TabPanel);
        tab_panel.set_children(vec![tab.render(update)]);
        tab_panel.set_labelled_by(vec![self.tab_ids[self.selected_tab_index]]);
        update
            .nodes
            .push((self.tab_panel_ids[self.selected_tab_index], tab_panel));
        if self.has_focus() {
            update.focus = self.focused_tab;
        }
        container_id
    }

    fn do_action(&mut self, request: &ActionRequest) -> bool {
        let tab = self.current_tab_mut();
        if tab.do_action(request) {
            true
        } else if let Some(i) = self.tab_ids.iter().position(|x| *x == request.target) {
            match request.action {
                Action::Focus => {
                    self.has_focus = true;
                    self.focused_tab = request.target;
                }
                Action::Click => {
                    self.selected_tab_index = i;
                }
                _ => (),
            }
            true
        } else {
            false
        }
    }
}
