// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2017 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use crate::{
    atspi::{
        interfaces::{Action, ObjectEvent, Property, QueuedEvent},
        ObjectId, ObjectRef, Rect as AtspiRect, ACCESSIBLE_PATH_PREFIX,
    },
    util::{AppContext, WindowBounds},
};
use accesskit::{kurbo::Point, CheckedState, DefaultActionVerb, NodeId, Role};
use accesskit_consumer::{DetachedNode, FilterResult, Node, NodeState, Tree, TreeState};
use atspi::{accessible::Role as AtspiRole, CoordType, Interface, InterfaceSet, State, StateSet};
use parking_lot::RwLock;
use std::{
    iter::FusedIterator,
    sync::{Arc, Weak},
};
use zbus::fdo;

fn filter_common(node: &NodeState) -> FilterResult {
    if node.is_hidden() {
        return FilterResult::ExcludeSubtree;
    }

    let role = node.role();
    if role == Role::Presentation || role == Role::GenericContainer || role == Role::InlineTextBox {
        return FilterResult::ExcludeNode;
    }

    FilterResult::Include
}

pub(crate) fn filter(node: &Node) -> FilterResult {
    if node.is_focused() {
        return FilterResult::Include;
    }

    filter_common(node.state())
}

pub(crate) fn filter_detached(node: &DetachedNode) -> FilterResult {
    if node.is_focused() {
        return FilterResult::Include;
    }

    filter_common(node.state())
}

pub(crate) enum NodeWrapper<'a> {
    Node(&'a Node<'a>),
    DetachedNode(&'a DetachedNode),
}

impl<'a> NodeWrapper<'a> {
    fn node_state(&self) -> &NodeState {
        match self {
            Self::Node(node) => node.state(),
            Self::DetachedNode(node) => node.state(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Self::Node(node) => node.name(),
            Self::DetachedNode(node) => node.name(),
        }
        .unwrap_or_default()
    }

    pub fn description(&self) -> String {
        String::new()
    }

    pub fn parent_id(&self) -> Option<NodeId> {
        self.node_state().parent_id()
    }

    pub fn filtered_parent(&self) -> Option<ObjectRef> {
        match self {
            Self::Node(node) => node
                .filtered_parent(&filter)
                .map(|parent| parent.id().into()),
            _ => unreachable!(),
        }
    }

    pub fn id(&self) -> ObjectId<'static> {
        self.node_state().id().into()
    }

    pub fn child_ids(
        &self,
    ) -> impl DoubleEndedIterator<Item = NodeId>
           + ExactSizeIterator<Item = NodeId>
           + FusedIterator<Item = NodeId>
           + '_ {
        self.node_state().child_ids()
    }

    pub fn role(&self) -> AtspiRole {
        match self.node_state().role() {
            Role::Alert => AtspiRole::Notification,
            Role::AlertDialog => AtspiRole::Alert,
            Role::Comment | Role::Suggestion => AtspiRole::Section,
            // TODO: See how to represent ARIA role="application"
            Role::Application => AtspiRole::Embedded,
            Role::Article => AtspiRole::Article,
            Role::Audio => AtspiRole::Audio,
            Role::Banner | Role::Header => AtspiRole::Landmark,
            Role::Blockquote => AtspiRole::BlockQuote,
            Role::Caret => AtspiRole::Unknown,
            Role::Button => AtspiRole::PushButton,
            Role::Canvas => AtspiRole::Canvas,
            Role::Caption => AtspiRole::Caption,
            Role::Cell => AtspiRole::TableCell,
            Role::CheckBox => AtspiRole::CheckBox,
            Role::Switch => AtspiRole::ToggleButton,
            Role::ColorWell => AtspiRole::PushButton,
            Role::Column => AtspiRole::Unknown,
            Role::ColumnHeader => AtspiRole::ColumnHeader,
            Role::ComboBoxGrouping | Role::ComboBoxMenuButton => AtspiRole::ComboBox,
            Role::Complementary => AtspiRole::Landmark,
            Role::ContentDeletion => AtspiRole::ContentDeletion,
            Role::ContentInsertion => AtspiRole::ContentInsertion,
            Role::ContentInfo | Role::Footer => AtspiRole::Landmark,
            Role::Date | Role::DateTime => AtspiRole::DateEditor,
            Role::Definition | Role::DescriptionListDetail => AtspiRole::DescriptionValue,
            Role::DescriptionList => AtspiRole::DescriptionList,
            Role::DescriptionListTerm => AtspiRole::DescriptionTerm,
            Role::Details => AtspiRole::Panel,
            Role::Dialog => AtspiRole::Dialog,
            Role::Directory => AtspiRole::List,
            Role::DisclosureTriangle => AtspiRole::ToggleButton,
            Role::DocCover => AtspiRole::Image,
            Role::DocBackLink | Role::DocBiblioRef | Role::DocGlossRef | Role::DocNoteRef => {
                AtspiRole::Link
            }
            Role::DocBiblioEntry | Role::DocEndnote => AtspiRole::ListItem,
            Role::DocNotice | Role::DocTip => AtspiRole::Comment,
            Role::DocFootnote => AtspiRole::Footnote,
            Role::DocPageBreak => AtspiRole::Separator,
            Role::DocPageFooter => AtspiRole::Footer,
            Role::DocPageHeader => AtspiRole::Header,
            Role::DocAcknowledgements
            | Role::DocAfterword
            | Role::DocAppendix
            | Role::DocBibliography
            | Role::DocChapter
            | Role::DocConclusion
            | Role::DocCredits
            | Role::DocEndnotes
            | Role::DocEpilogue
            | Role::DocErrata
            | Role::DocForeword
            | Role::DocGlossary
            | Role::DocIndex
            | Role::DocIntroduction
            | Role::DocPageList
            | Role::DocPart
            | Role::DocPreface
            | Role::DocPrologue
            | Role::DocToc => AtspiRole::Landmark,
            Role::DocAbstract
            | Role::DocColophon
            | Role::DocCredit
            | Role::DocDedication
            | Role::DocEpigraph
            | Role::DocExample
            | Role::DocPullquote
            | Role::DocQna => AtspiRole::Section,
            Role::DocSubtitle => AtspiRole::Heading,
            Role::Document => AtspiRole::DocumentFrame,
            Role::EmbeddedObject => AtspiRole::Embedded,
            // TODO: Forms which lack an accessible name are no longer
            // exposed as forms. Forms which have accessible
            // names should be exposed as `AtspiRole::Landmark` according to Core AAM.
            Role::Form => AtspiRole::Form,
            Role::Figure | Role::Feed => AtspiRole::Panel,
            Role::GenericContainer
            | Role::FooterAsNonLandmark
            | Role::HeaderAsNonLandmark
            | Role::Ruby => AtspiRole::Section,
            Role::GraphicsDocument => AtspiRole::DocumentFrame,
            Role::GraphicsObject => AtspiRole::Panel,
            Role::GraphicsSymbol => AtspiRole::Image,
            Role::Grid => AtspiRole::Table,
            Role::Group => AtspiRole::Panel,
            Role::Heading => AtspiRole::Heading,
            Role::Iframe | Role::IframePresentational => AtspiRole::InternalFrame,
            // TODO: If there are unignored children, then it should be AtspiRole::ImageMap.
            Role::Image => AtspiRole::Image,
            Role::InlineTextBox => AtspiRole::Static,
            Role::InputTime => AtspiRole::DateEditor,
            Role::LabelText | Role::Legend => AtspiRole::Label,
            // Layout table objects are treated the same as `Role::GenericContainer`.
            Role::LayoutTable => AtspiRole::Section,
            Role::LayoutTableCell => AtspiRole::Section,
            Role::LayoutTableRow => AtspiRole::Section,
            // TODO: Having a separate accessible object for line breaks
            // is inconsistent with other implementations.
            Role::LineBreak => AtspiRole::Static,
            Role::Link => AtspiRole::Link,
            Role::List => AtspiRole::List,
            Role::ListBox => AtspiRole::ListBox,
            // TODO: Use `AtspiRole::MenuItem' inside a combo box.
            Role::ListBoxOption => AtspiRole::ListItem,
            Role::ListGrid => AtspiRole::Table,
            Role::ListItem => AtspiRole::ListItem,
            // Regular list markers only expose their alternative text, but do not
            // expose their descendants, and the descendants should be ignored. This
            // is because the alternative text depends on the counter style and can
            // be different from the actual (visual) marker text, and hence,
            // inconsistent with the descendants. We treat a list marker as non-text
            // only if it still has non-ignored descendants, which happens only when =>
            // - The list marker itself is ignored but the descendants are not
            // - Or the list marker contains images
            // TODO: How to check for unignored children when the node is detached?
            Role::ListMarker => AtspiRole::Static,
            Role::Log => AtspiRole::Log,
            Role::Main => AtspiRole::Landmark,
            Role::Mark => AtspiRole::Static,
            Role::Math => AtspiRole::Math,
            Role::Marquee => AtspiRole::Marquee,
            Role::Menu | Role::MenuListPopup => AtspiRole::Menu,
            Role::MenuBar => AtspiRole::MenuBar,
            Role::MenuItem | Role::MenuListOption => AtspiRole::MenuItem,
            Role::MenuItemCheckBox => AtspiRole::CheckMenuItem,
            Role::MenuItemRadio => AtspiRole::RadioMenuItem,
            Role::Meter => AtspiRole::LevelBar,
            Role::Navigation => AtspiRole::Landmark,
            Role::Note => AtspiRole::Comment,
            Role::Pane | Role::ScrollView => AtspiRole::Panel,
            Role::Paragraph => AtspiRole::Paragraph,
            Role::PdfActionableHighlight => AtspiRole::PushButton,
            Role::PdfRoot => AtspiRole::DocumentFrame,
            Role::PluginObject => AtspiRole::Embedded,
            Role::PopupButton => AtspiRole::PushButton,
            Role::Portal => AtspiRole::PushButton,
            Role::Pre => AtspiRole::Section,
            Role::ProgressIndicator => AtspiRole::ProgressBar,
            Role::RadioButton => AtspiRole::RadioButton,
            Role::RadioGroup => AtspiRole::Panel,
            Role::Region => AtspiRole::Landmark,
            Role::RootWebArea => AtspiRole::DocumentWeb,
            Role::Row => AtspiRole::TableRow,
            Role::RowGroup => AtspiRole::Panel,
            Role::RowHeader => AtspiRole::RowHeader,
            // TODO: Generally exposed as description on <ruby> (`Role::Ruby`) element, not
            // as its own object in the tree.
            // However, it's possible to make a `Role::RubyAnnotation` element show up in the
            // tree, for example by adding tabindex="0" to the source <rp> or <rt>
            // element or making the source element the target of an aria-owns.
            // Therefore, we need to gracefully handle it if it actually
            // shows up in the tree.
            Role::RubyAnnotation => AtspiRole::Static,
            Role::Section => AtspiRole::Section,
            Role::ScrollBar => AtspiRole::ScrollBar,
            Role::Search => AtspiRole::Landmark,
            Role::Slider => AtspiRole::Slider,
            Role::SpinButton => AtspiRole::SpinButton,
            Role::Splitter => AtspiRole::Separator,
            Role::StaticText => AtspiRole::Static,
            Role::Status => AtspiRole::StatusBar,
            Role::SvgRoot => AtspiRole::DocumentFrame,
            Role::Tab => AtspiRole::PageTab,
            Role::Table => AtspiRole::Table,
            // TODO: This mapping is correct, but it doesn't seem to be
            // used. We don't necessarily want to always expose these containers, but
            // we must do so if they are focusable.
            Role::TableHeaderContainer => AtspiRole::Panel,
            Role::TabList => AtspiRole::PageTabList,
            Role::TabPanel => AtspiRole::ScrollPane,
            // TODO: This mapping should also be applied to the dfn
            // element.
            Role::Term => AtspiRole::DescriptionTerm,
            Role::TitleBar => AtspiRole::TitleBar,
            Role::TextField | Role::SearchBox => {
                if self.node_state().is_protected() {
                    AtspiRole::PasswordText
                } else {
                    AtspiRole::Entry
                }
            }
            Role::TextFieldWithComboBox => AtspiRole::ComboBox,
            Role::Abbr | Role::Code | Role::Emphasis | Role::Strong | Role::Time => {
                AtspiRole::Static
            }
            Role::Timer => AtspiRole::Timer,
            Role::ToggleButton => AtspiRole::ToggleButton,
            Role::Toolbar => AtspiRole::ToolBar,
            Role::Tooltip => AtspiRole::ToolTip,
            Role::Tree => AtspiRole::Tree,
            Role::TreeItem => AtspiRole::TreeItem,
            Role::TreeGrid => AtspiRole::TreeTable,
            Role::Video => AtspiRole::Video,
            // In AT-SPI, elements with `AtspiRole::Frame` are windows with titles and
            // buttons, while those with `AtspiRole::Window` are windows without those
            // elements.
            Role::Window => AtspiRole::Frame,
            Role::Client | Role::WebView => AtspiRole::Panel,
            Role::FigureCaption => AtspiRole::Caption,
            // TODO: Are there special cases to consider?
            Role::Presentation | Role::Unknown => AtspiRole::Unknown,
            Role::ImeCandidate | Role::Keyboard => AtspiRole::RedundantObject,
        }
    }

    fn is_focused(&self) -> bool {
        match self {
            Self::Node(node) => node.is_focused(),
            Self::DetachedNode(node) => node.is_focused(),
        }
    }

    pub fn state(&self) -> StateSet {
        let state = self.node_state();
        let atspi_role = self.role();
        let mut atspi_state = StateSet::empty();
        if state.role() == Role::Window && state.parent_id().is_none() {
            atspi_state.insert(State::Active);
        }
        // TODO: Focus and selection.
        if state.is_focusable() {
            atspi_state.insert(State::Focusable);
        }
        let filter_result = match self {
            Self::Node(node) => filter(node),
            Self::DetachedNode(node) => filter_detached(node),
        };
        if filter_result == FilterResult::Include {
            atspi_state.insert(State::Visible | State::Showing);
        }
        if atspi_role != AtspiRole::ToggleButton && state.checked_state().is_some() {
            atspi_state.insert(State::Checkable);
        }
        if let Some(selected) = state.is_selected() {
            if !state.is_disabled() {
                atspi_state.insert(State::Selectable);
            }
            if selected {
                atspi_state.insert(State::Selected);
            }
        }
        if state.is_text_field() {
            atspi_state.insert(State::SelectableText);
            atspi_state.insert(match state.is_multiline() {
                true => State::MultiLine,
                false => State::SingleLine,
            });
        }

        // Special case for indeterminate progressbar.
        if state.role() == Role::ProgressIndicator && state.numeric_value().is_none() {
            atspi_state.insert(State::Indeterminate);
        }

        // Checked state
        match state.checked_state() {
            Some(CheckedState::Mixed) => atspi_state.insert(State::Indeterminate),
            Some(CheckedState::True) if atspi_role == AtspiRole::ToggleButton => {
                atspi_state.insert(State::Pressed)
            }
            Some(CheckedState::True) => atspi_state.insert(State::Checked),
            _ => {}
        }

        if state.is_read_only_supported() && state.is_read_only_or_disabled() {
            atspi_state.insert(State::ReadOnly);
        } else {
            atspi_state.insert(State::Enabled | State::Sensitive);
        }

        if self.is_focused() {
            atspi_state.insert(State::Focused);
        }

        atspi_state
    }

    fn is_root(&self) -> bool {
        match self {
            Self::Node(node) => node.is_root(),
            Self::DetachedNode(node) => node.is_root(),
        }
    }

    pub fn interfaces(&self) -> InterfaceSet {
        let state = self.node_state();
        let mut interfaces = InterfaceSet::new(Interface::Accessible);
        if state.default_action_verb().is_some() {
            interfaces.insert(Interface::Action);
        }
        if state.raw_bounds().is_some() || self.is_root() {
            interfaces.insert(Interface::Component);
        }
        if self.current_value().is_some() {
            interfaces.insert(Interface::Value);
        }
        interfaces
    }

    fn n_actions(&self) -> i32 {
        match self.node_state().default_action_verb() {
            Some(_) => 1,
            None => 0,
        }
    }

    fn get_action_name(&self, index: i32) -> String {
        if index != 0 {
            return String::new();
        }
        String::from(match self.node_state().default_action_verb() {
            Some(DefaultActionVerb::Click) => "click",
            Some(DefaultActionVerb::Focus) => "focus",
            Some(DefaultActionVerb::Check) => "check",
            Some(DefaultActionVerb::Uncheck) => "uncheck",
            Some(DefaultActionVerb::ClickAncestor) => "clickAncestor",
            Some(DefaultActionVerb::Jump) => "jump",
            Some(DefaultActionVerb::Open) => "open",
            Some(DefaultActionVerb::Press) => "press",
            Some(DefaultActionVerb::Select) => "select",
            None => "",
        })
    }

    fn current_value(&self) -> Option<f64> {
        self.node_state().numeric_value()
    }

    pub fn enqueue_changes(&self, queue: &mut Vec<QueuedEvent>, old: &NodeWrapper) {
        self.enqueue_state_changes(queue, old);
        self.enqueue_property_changes(queue, old);
    }

    fn enqueue_state_changes(&self, queue: &mut Vec<QueuedEvent>, old: &NodeWrapper) {
        let old_state = old.state();
        let new_state = self.state();
        let changed_states = old_state ^ new_state;
        for state in changed_states.iter() {
            queue.push(QueuedEvent::Object {
                target: self.id(),
                event: ObjectEvent::StateChanged(state, new_state.contains(state)),
            });
        }
    }

    fn enqueue_property_changes(&self, queue: &mut Vec<QueuedEvent>, old: &NodeWrapper) {
        let name = self.name();
        if name != old.name() {
            queue.push(QueuedEvent::Object {
                target: self.id(),
                event: ObjectEvent::PropertyChanged(Property::Name(name)),
            });
        }
        let description = self.description();
        if description != old.description() {
            queue.push(QueuedEvent::Object {
                target: self.id(),
                event: ObjectEvent::PropertyChanged(Property::Description(description)),
            });
        }
        let parent_id = self.parent_id();
        if parent_id != old.parent_id() {
            queue.push(QueuedEvent::Object {
                target: self.id(),
                event: ObjectEvent::PropertyChanged(Property::Parent(self.filtered_parent())),
            });
        }
        let role = self.role();
        if role != old.role() {
            queue.push(QueuedEvent::Object {
                target: self.id(),
                event: ObjectEvent::PropertyChanged(Property::Role(role)),
            });
        }
    }
}

pub(crate) fn unknown_object(id: &ObjectId) -> fdo::Error {
    fdo::Error::UnknownObject(format!("{}{}", ACCESSIBLE_PATH_PREFIX, id.as_str()))
}

#[derive(Clone)]
pub(crate) struct PlatformNode {
    tree: Weak<Tree>,
    node_id: NodeId,
}

impl PlatformNode {
    pub(crate) fn new(tree: &Arc<Tree>, node_id: NodeId) -> Self {
        Self {
            tree: Arc::downgrade(tree),
            node_id,
        }
    }

    fn upgrade_tree(&self) -> fdo::Result<Arc<Tree>> {
        if let Some(tree) = self.tree.upgrade() {
            Ok(tree)
        } else {
            Err(unknown_object(&self.accessible_id()))
        }
    }

    fn with_tree_state<F, T>(&self, f: F) -> fdo::Result<T>
    where
        F: FnOnce(&TreeState) -> fdo::Result<T>,
    {
        let tree = self.upgrade_tree()?;
        let state = tree.read();
        f(&state)
    }

    fn resolve<F, T>(&self, f: F) -> fdo::Result<T>
    where
        for<'a> F: FnOnce(Node<'a>) -> fdo::Result<T>,
    {
        self.with_tree_state(|state| {
            if let Some(node) = state.node_by_id(self.node_id) {
                f(node)
            } else {
                Err(unknown_object(&self.accessible_id()))
            }
        })
    }

    fn validate_for_action(&self) -> fdo::Result<Arc<Tree>> {
        let tree = self.upgrade_tree()?;
        let state = tree.read();
        if state.has_node(self.node_id) {
            drop(state);
            Ok(tree)
        } else {
            Err(unknown_object(&self.accessible_id()))
        }
    }

    pub fn name(&self) -> fdo::Result<String> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.name())
        })
    }

    pub fn description(&self) -> fdo::Result<String> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.description())
        })
    }

    pub fn parent(&self) -> fdo::Result<ObjectRef> {
        self.resolve(|node| {
            Ok(node
                .filtered_parent(&filter)
                .map(|parent| parent.id().into())
                .unwrap_or_else(|| ObjectRef::Managed(ObjectId::root())))
        })
    }

    pub fn child_count(&self) -> fdo::Result<i32> {
        self.resolve(|node| {
            i32::try_from(node.state().child_ids().count())
                .map_err(|_| fdo::Error::Failed("Too many children.".into()))
        })
    }

    pub fn accessible_id(&self) -> ObjectId<'static> {
        self.node_id.into()
    }

    pub fn child_at_index(&self, index: usize) -> fdo::Result<Option<ObjectRef>> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            let child = wrapper.child_ids().nth(index).map(ObjectRef::from);
            Ok(child)
        })
    }

    pub fn children(&self) -> fdo::Result<Vec<ObjectRef>> {
        self.resolve(|node| {
            Ok(node
                .filtered_children(&filter)
                .map(|child| child.id().into())
                .collect())
        })
    }

    pub fn index_in_parent(&self) -> fdo::Result<i32> {
        self.resolve(|node| {
            i32::try_from(node.preceding_filtered_siblings(&filter).count())
                .map_err(|_| fdo::Error::Failed("Index is too big.".into()))
        })
    }

    pub fn role(&self) -> fdo::Result<AtspiRole> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.role())
        })
    }

    pub fn state(&self) -> fdo::Result<StateSet> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.state())
        })
    }

    pub fn interfaces(&self) -> fdo::Result<InterfaceSet> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.interfaces())
        })
    }

    pub fn n_actions(&self) -> fdo::Result<i32> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.n_actions())
        })
    }

    pub fn get_action_name(&self, index: i32) -> fdo::Result<String> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.get_action_name(index))
        })
    }

    pub fn get_actions(&self) -> fdo::Result<Vec<Action>> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            let n_actions = wrapper.n_actions() as usize;
            let mut actions = Vec::with_capacity(n_actions);
            for i in 0..n_actions {
                actions.push(Action {
                    localized_name: wrapper.get_action_name(i as i32),
                    description: "".into(),
                    key_binding: "".into(),
                });
            }
            Ok(actions)
        })
    }

    pub fn do_action(&self, index: i32) -> fdo::Result<bool> {
        if index != 0 {
            return Ok(false);
        }
        let tree = self.validate_for_action()?;
        tree.do_default_action(self.node_id);
        Ok(true)
    }

    pub fn contains(
        &self,
        window_bounds: &WindowBounds,
        x: i32,
        y: i32,
        coord_type: CoordType,
    ) -> fdo::Result<bool> {
        self.resolve(|node| {
            let bounds = match node.bounding_box() {
                Some(node_bounds) => {
                    let top_left = window_bounds.top_left(coord_type, node.is_root());
                    let new_origin =
                        Point::new(top_left.x + node_bounds.x0, top_left.y + node_bounds.y0);
                    node_bounds.with_origin(new_origin)
                }
                None if node.is_root() => {
                    let bounds = window_bounds.outer;
                    match coord_type {
                        CoordType::Screen => bounds,
                        CoordType::Window => bounds.with_origin(Point::ZERO),
                        _ => unimplemented!(),
                    }
                }
                _ => return Err(unknown_object(&self.accessible_id())),
            };
            Ok(bounds.contains(Point::new(x.into(), y.into())))
        })
    }

    pub fn get_accessible_at_point(
        &self,
        window_bounds: &WindowBounds,
        x: i32,
        y: i32,
        coord_type: CoordType,
    ) -> fdo::Result<Option<ObjectRef>> {
        self.resolve(|node| {
            let top_left = window_bounds.top_left(coord_type, node.is_root());
            let point = Point::new(f64::from(x) - top_left.x, f64::from(y) - top_left.y);
            let point = node.transform().inverse() * point;
            Ok(node
                .node_at_point(point, &filter)
                .map(|node| ObjectRef::Managed(NodeWrapper::Node(&node).id())))
        })
    }

    pub fn get_extents(
        &self,
        window_bounds: &WindowBounds,
        coord_type: CoordType,
    ) -> fdo::Result<(AtspiRect,)> {
        self.resolve(|node| match node.bounding_box() {
            Some(node_bounds) => {
                let top_left = window_bounds.top_left(coord_type, node.is_root());
                let new_origin =
                    Point::new(top_left.x + node_bounds.x0, top_left.y + node_bounds.y0);
                Ok((node_bounds.with_origin(new_origin).into(),))
            }
            None if node.is_root() => {
                let bounds = window_bounds.outer;
                Ok((match coord_type {
                    CoordType::Screen => bounds.into(),
                    CoordType::Window => bounds.with_origin(Point::ZERO).into(),
                    _ => unimplemented!(),
                },))
            }
            _ => Err(unknown_object(&self.accessible_id())),
        })
    }

    pub fn grab_focus(&self) -> fdo::Result<bool> {
        let tree = self.validate_for_action()?;
        tree.set_focus(self.node_id);
        Ok(true)
    }

    pub fn minimum_value(&self) -> fdo::Result<f64> {
        self.resolve(|node| Ok(node.state().min_numeric_value().unwrap_or(std::f64::MIN)))
    }

    pub fn maximum_value(&self) -> fdo::Result<f64> {
        self.resolve(|node| Ok(node.state().max_numeric_value().unwrap_or(std::f64::MAX)))
    }

    pub fn minimum_increment(&self) -> fdo::Result<f64> {
        self.resolve(|node| Ok(node.state().numeric_value_step().unwrap_or(0.0)))
    }

    pub fn current_value(&self) -> fdo::Result<f64> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.current_value().unwrap_or(0.0))
        })
    }

    pub fn set_current_value(&self, value: f64) -> fdo::Result<()> {
        let tree = self.validate_for_action()?;
        tree.set_numeric_value(self.node_id, value);
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct PlatformRootNode {
    pub(crate) context: Weak<RwLock<AppContext>>,
    pub(crate) tree: Weak<Tree>,
}

impl PlatformRootNode {
    pub fn new(context: &Arc<RwLock<AppContext>>, tree: &Arc<Tree>) -> Self {
        Self {
            context: Arc::downgrade(context),
            tree: Arc::downgrade(tree),
        }
    }
}
