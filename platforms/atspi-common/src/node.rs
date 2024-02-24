// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2017 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use accesskit::{
    Action, ActionData, ActionRequest, Affine, Checked, DefaultActionVerb, Live, NodeId, Point,
    Rect, Role,
};
use accesskit_consumer::{DetachedNode, FilterResult, Node, NodeState, TreeState};
use atspi_common::{
    CoordType, Interface, InterfaceSet, Layer, Live as AtspiLive, Role as AtspiRole, State,
    StateSet,
};
use std::{
    hash::{Hash, Hasher},
    iter::FusedIterator,
    sync::{Arc, RwLock, RwLockReadGuard, Weak},
};

use crate::{
    adapter::Adapter,
    context::{AppContext, Context},
    filters::{filter, filter_detached},
    util::WindowBounds,
    Action as AtspiAction, Error, ObjectEvent, Property, Rect as AtspiRect, Result,
};

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

    pub fn name(&self) -> Option<String> {
        match self {
            Self::Node(node) => node.name(),
            Self::DetachedNode(node) => node.name(),
        }
    }

    pub fn description(&self) -> String {
        String::new()
    }

    pub fn parent_id(&self) -> Option<NodeId> {
        self.node_state().parent_id()
    }

    pub fn id(&self) -> NodeId {
        self.node_state().id()
    }

    fn child_ids(
        &self,
    ) -> impl DoubleEndedIterator<Item = NodeId>
           + ExactSizeIterator<Item = NodeId>
           + FusedIterator<Item = NodeId>
           + '_ {
        self.node_state().child_ids()
    }

    fn filtered_child_ids(
        &self,
    ) -> impl DoubleEndedIterator<Item = NodeId> + FusedIterator<Item = NodeId> + '_ {
        match self {
            Self::Node(node) => node.filtered_children(&filter).map(|child| child.id()),
            _ => unreachable!(),
        }
    }

    pub fn role(&self) -> AtspiRole {
        if self.node_state().has_role_description() {
            return AtspiRole::Extended;
        }

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
            Role::Button | Role::DefaultButton => AtspiRole::PushButton,
            Role::Canvas => AtspiRole::Canvas,
            Role::Caption => AtspiRole::Caption,
            Role::Cell => AtspiRole::TableCell,
            Role::CheckBox => AtspiRole::CheckBox,
            Role::Switch => AtspiRole::ToggleButton,
            Role::ColorWell => AtspiRole::PushButton,
            Role::Column => AtspiRole::Unknown,
            Role::ColumnHeader => AtspiRole::ColumnHeader,
            Role::ComboBox | Role::EditableComboBox => AtspiRole::ComboBox,
            Role::Complementary => AtspiRole::Landmark,
            Role::ContentDeletion => AtspiRole::ContentDeletion,
            Role::ContentInsertion => AtspiRole::ContentInsertion,
            Role::ContentInfo | Role::Footer => AtspiRole::Landmark,
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
            Role::Legend => AtspiRole::Label,
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
            Role::TextInput
            | Role::MultilineTextInput
            | Role::SearchInput
            | Role::EmailInput
            | Role::NumberInput
            | Role::PhoneNumberInput
            | Role::UrlInput => AtspiRole::Entry,
            Role::DateInput
            | Role::DateTimeInput
            | Role::WeekInput
            | Role::MonthInput
            | Role::TimeInput => AtspiRole::DateEditor,
            Role::PasswordInput => AtspiRole::PasswordText,
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
            Role::WebView => AtspiRole::Panel,
            Role::FigureCaption => AtspiRole::Caption,
            // TODO: Are there special cases to consider?
            Role::Unknown => AtspiRole::Unknown,
            Role::ImeCandidate | Role::Keyboard => AtspiRole::RedundantObject,
            Role::Terminal => AtspiRole::Terminal,
        }
    }

    fn is_focused(&self) -> bool {
        match self {
            Self::Node(node) => node.is_focused(),
            Self::DetachedNode(node) => node.is_focused(),
        }
    }

    pub fn state(&self, is_window_focused: bool) -> StateSet {
        let state = self.node_state();
        let atspi_role = self.role();
        let mut atspi_state = StateSet::empty();
        if state.parent_id().is_none() && state.role() == Role::Window && is_window_focused {
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
        if atspi_role != AtspiRole::ToggleButton && state.checked().is_some() {
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
        if state.is_text_input() {
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
        match state.checked() {
            Some(Checked::Mixed) => atspi_state.insert(State::Indeterminate),
            Some(Checked::True) if atspi_role == AtspiRole::ToggleButton => {
                atspi_state.insert(State::Pressed)
            }
            Some(Checked::True) => atspi_state.insert(State::Checked),
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

    fn supports_action(&self) -> bool {
        self.node_state().default_action_verb().is_some()
    }

    fn supports_component(&self) -> bool {
        self.node_state().raw_bounds().is_some() || self.is_root()
    }

    fn supports_value(&self) -> bool {
        self.current_value().is_some()
    }

    pub fn interfaces(&self) -> InterfaceSet {
        let mut interfaces = InterfaceSet::new(Interface::Accessible);
        if self.supports_action() {
            interfaces.insert(Interface::Action);
        }
        if self.supports_component() {
            interfaces.insert(Interface::Component);
        }
        if self.supports_value() {
            interfaces.insert(Interface::Value);
        }
        interfaces
    }

    pub(crate) fn live(&self) -> AtspiLive {
        let live = match self {
            Self::Node(node) => node.live(),
            Self::DetachedNode(node) => node.live(),
        };
        match live {
            Live::Off => AtspiLive::None,
            Live::Polite => AtspiLive::Polite,
            Live::Assertive => AtspiLive::Assertive,
        }
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
            Some(DefaultActionVerb::Unselect) => "unselect",
            None => "",
        })
    }

    fn raw_bounds_and_transform(&self) -> (Option<Rect>, Affine) {
        let state = self.node_state();
        (state.raw_bounds(), state.direct_transform())
    }

    fn extents(&self, window_bounds: &WindowBounds) -> AtspiRect {
        if self.is_root() {
            return window_bounds.outer.into();
        }
        match self {
            Self::Node(node) => node.bounding_box().map_or_else(
                || AtspiRect::INVALID,
                |bounds| {
                    let window_top_left = window_bounds.inner.origin();
                    let node_origin = bounds.origin();
                    let new_origin = Point::new(
                        window_top_left.x + node_origin.x,
                        window_top_left.y + node_origin.y,
                    );
                    bounds.with_origin(new_origin).into()
                },
            ),
            _ => unreachable!(),
        }
    }

    fn current_value(&self) -> Option<f64> {
        self.node_state().numeric_value()
    }

    pub(crate) fn notify_changes(
        &self,
        window_bounds: &WindowBounds,
        adapter: &Adapter,
        old: &NodeWrapper<'_>,
    ) {
        self.notify_state_changes(adapter, old);
        self.notify_property_changes(adapter, old);
        self.notify_bounds_changes(window_bounds, adapter, old);
        self.notify_children_changes(adapter, old);
    }

    fn notify_state_changes(&self, adapter: &Adapter, old: &NodeWrapper<'_>) {
        let old_state = old.state(true);
        let new_state = self.state(true);
        let changed_states = old_state ^ new_state;
        for state in changed_states.iter() {
            adapter.emit_object_event(
                self.id(),
                ObjectEvent::StateChanged(state, new_state.contains(state)),
            );
        }
    }

    fn notify_property_changes(&self, adapter: &Adapter, old: &NodeWrapper<'_>) {
        let name = self.name();
        if name != old.name() {
            let name = name.unwrap_or_default();
            adapter.emit_object_event(
                self.id(),
                ObjectEvent::PropertyChanged(Property::Name(name.clone())),
            );

            let live = self.live();
            if live != AtspiLive::None {
                adapter.emit_object_event(self.id(), ObjectEvent::Announcement(name, live));
            }
        }
        let description = self.description();
        if description != old.description() {
            adapter.emit_object_event(
                self.id(),
                ObjectEvent::PropertyChanged(Property::Description(description)),
            );
        }
        let parent_id = self.parent_id();
        if parent_id != old.parent_id() {
            let node = match self {
                NodeWrapper::Node(node) => node,
                _ => unreachable!(),
            };
            let parent = node
                .filtered_parent(&filter)
                .map_or(NodeIdOrRoot::Root, |node| NodeIdOrRoot::Node(node.id()));
            adapter.emit_object_event(
                self.id(),
                ObjectEvent::PropertyChanged(Property::Parent(parent)),
            );
        }
        let role = self.role();
        if role != old.role() {
            adapter.emit_object_event(
                self.id(),
                ObjectEvent::PropertyChanged(Property::Role(role)),
            );
        }
        if let Some(value) = self.current_value() {
            if Some(value) != old.current_value() {
                adapter.emit_object_event(
                    self.id(),
                    ObjectEvent::PropertyChanged(Property::Value(value)),
                );
            }
        }
    }

    fn notify_bounds_changes(
        &self,
        window_bounds: &WindowBounds,
        adapter: &Adapter,
        old: &NodeWrapper<'_>,
    ) {
        if self.raw_bounds_and_transform() != old.raw_bounds_and_transform() {
            adapter.emit_object_event(
                self.id(),
                ObjectEvent::BoundsChanged(self.extents(window_bounds)),
            );
        }
    }

    fn notify_children_changes(&self, adapter: &Adapter, old: &NodeWrapper<'_>) {
        let old_children = old.child_ids().collect::<Vec<NodeId>>();
        let filtered_children = self.filtered_child_ids().collect::<Vec<NodeId>>();
        for (index, child) in filtered_children.iter().enumerate() {
            if !old_children.contains(child) {
                adapter.emit_object_event(self.id(), ObjectEvent::ChildAdded(index, *child));
            }
        }
        for child in old_children.into_iter() {
            if !filtered_children.contains(&child) {
                adapter.emit_object_event(self.id(), ObjectEvent::ChildRemoved(child));
            }
        }
    }
}

#[derive(Clone)]
pub struct PlatformNode {
    context: Weak<Context>,
    adapter_id: usize,
    id: NodeId,
}

impl PlatformNode {
    pub(crate) fn new(context: &Arc<Context>, adapter_id: usize, id: NodeId) -> Self {
        Self {
            context: Arc::downgrade(context),
            adapter_id,
            id,
        }
    }

    fn from_adapter_root(adapter_id_and_context: &(usize, Arc<Context>)) -> Self {
        let (adapter_id, context) = adapter_id_and_context;
        Self::new(context, *adapter_id, context.read_tree().state().root_id())
    }

    fn upgrade_context(&self) -> Result<Arc<Context>> {
        if let Some(context) = self.context.upgrade() {
            Ok(context)
        } else {
            Err(Error::Defunct)
        }
    }

    fn with_tree_state<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&TreeState) -> Result<T>,
    {
        let context = self.upgrade_context()?;
        let tree = context.read_tree();
        f(tree.state())
    }

    fn with_tree_state_and_context<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&TreeState, &Context) -> Result<T>,
    {
        let context = self.upgrade_context()?;
        let tree = context.read_tree();
        f(tree.state(), &context)
    }

    fn resolve_with_context<F, T>(&self, f: F) -> Result<T>
    where
        for<'a> F: FnOnce(Node<'a>, &Context) -> Result<T>,
    {
        self.with_tree_state_and_context(|state, context| {
            if let Some(node) = state.node_by_id(self.id) {
                f(node, context)
            } else {
                Err(Error::Defunct)
            }
        })
    }

    fn resolve<F, T>(&self, f: F) -> Result<T>
    where
        for<'a> F: FnOnce(Node<'a>) -> Result<T>,
    {
        self.resolve_with_context(|node, _| f(node))
    }

    fn do_action_internal<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&TreeState, &Context) -> ActionRequest,
    {
        let context = self.upgrade_context()?;
        let tree = context.read_tree();
        if tree.state().has_node(self.id) {
            let request = f(tree.state(), &context);
            drop(tree);
            context.do_action(request);
            Ok(())
        } else {
            Err(Error::Defunct)
        }
    }

    pub fn name(&self) -> Result<String> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.name().unwrap_or_default())
        })
    }

    pub fn description(&self) -> Result<String> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.description())
        })
    }

    pub fn relative(&self, id: NodeId) -> Self {
        Self {
            context: self.context.clone(),
            adapter_id: self.adapter_id,
            id,
        }
    }

    pub fn root(&self) -> Result<PlatformRoot> {
        let context = self.upgrade_context()?;
        Ok(PlatformRoot::new(&context.app_context))
    }

    pub fn toolkit_name(&self) -> Result<String> {
        self.with_tree_state(|state| Ok(state.toolkit_name().unwrap_or_default()))
    }

    pub fn toolkit_version(&self) -> Result<String> {
        self.with_tree_state(|state| Ok(state.toolkit_version().unwrap_or_default()))
    }

    pub fn parent(&self) -> Result<NodeIdOrRoot> {
        self.resolve(|node| {
            let parent = node
                .filtered_parent(&filter)
                .map_or(NodeIdOrRoot::Root, |node| NodeIdOrRoot::Node(node.id()));
            Ok(parent)
        })
    }

    pub fn child_count(&self) -> Result<i32> {
        self.resolve(|node| {
            i32::try_from(node.filtered_children(&filter).count())
                .map_err(|_| Error::TooManyChildren)
        })
    }

    pub fn adapter_id(&self) -> usize {
        self.adapter_id
    }

    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn child_at_index(&self, index: usize) -> Result<Option<NodeId>> {
        self.resolve(|node| {
            let child = node
                .filtered_children(&filter)
                .nth(index)
                .map(|child| child.id());
            Ok(child)
        })
    }

    pub fn map_children<T, I>(&self, f: impl Fn(NodeId) -> I) -> Result<T>
    where
        T: FromIterator<I>,
    {
        self.resolve(|node| {
            let children = node
                .filtered_children(&filter)
                .map(|child| child.id())
                .map(f)
                .collect();
            Ok(children)
        })
    }

    pub fn index_in_parent(&self) -> Result<i32> {
        self.resolve(|node| {
            i32::try_from(node.preceding_filtered_siblings(&filter).count())
                .map_err(|_| Error::IndexOutOfRange)
        })
    }

    pub fn role(&self) -> Result<AtspiRole> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.role())
        })
    }

    pub fn localized_role_name(&self) -> Result<String> {
        self.resolve(|node| Ok(node.state().role_description().unwrap_or_default()))
    }

    pub fn state(&self) -> StateSet {
        self.resolve_with_context(|node, context| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.state(context.read_tree().state().focus_id().is_some()))
        })
        .unwrap_or(State::Defunct.into())
    }

    pub fn supports_action(&self) -> Result<bool> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.supports_action())
        })
    }

    pub fn supports_component(&self) -> Result<bool> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.supports_component())
        })
    }

    pub fn supports_value(&self) -> Result<bool> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.supports_value())
        })
    }

    pub fn interfaces(&self) -> Result<InterfaceSet> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.interfaces())
        })
    }

    pub fn n_actions(&self) -> Result<i32> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.n_actions())
        })
    }

    pub fn action_name(&self, index: i32) -> Result<String> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.get_action_name(index))
        })
    }

    pub fn actions(&self) -> Result<Vec<AtspiAction>> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            let n_actions = wrapper.n_actions() as usize;
            let mut actions = Vec::with_capacity(n_actions);
            for i in 0..n_actions {
                actions.push(AtspiAction {
                    localized_name: wrapper.get_action_name(i as i32),
                    description: "".into(),
                    key_binding: "".into(),
                });
            }
            Ok(actions)
        })
    }

    pub fn do_action(&self, index: i32) -> Result<bool> {
        if index != 0 {
            return Ok(false);
        }
        self.do_action_internal(|_, _| ActionRequest {
            action: Action::Default,
            target: self.id,
            data: None,
        })?;
        Ok(true)
    }

    pub fn contains(&self, x: i32, y: i32, coord_type: CoordType) -> Result<bool> {
        self.resolve_with_context(|node, context| {
            let window_bounds = context.read_root_window_bounds();
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
                _ => return Err(Error::UnsupportedInterface),
            };
            Ok(bounds.contains(Point::new(x.into(), y.into())))
        })
    }

    pub fn accessible_at_point(
        &self,
        x: i32,
        y: i32,
        coord_type: CoordType,
    ) -> Result<Option<NodeId>> {
        self.resolve_with_context(|node, context| {
            let window_bounds = context.read_root_window_bounds();
            let top_left = window_bounds.top_left(coord_type, node.is_root());
            let point = Point::new(f64::from(x) - top_left.x, f64::from(y) - top_left.y);
            let point = node.transform().inverse() * point;
            Ok(node.node_at_point(point, &filter).map(|node| node.id()))
        })
    }

    pub fn extents(&self, coord_type: CoordType) -> Result<AtspiRect> {
        self.resolve_with_context(|node, context| {
            let window_bounds = context.read_root_window_bounds();
            match node.bounding_box() {
                Some(node_bounds) => {
                    let top_left = window_bounds.top_left(coord_type, node.is_root());
                    let new_origin =
                        Point::new(top_left.x + node_bounds.x0, top_left.y + node_bounds.y0);
                    Ok(node_bounds.with_origin(new_origin).into())
                }
                None if node.is_root() => {
                    let bounds = window_bounds.outer;
                    Ok(match coord_type {
                        CoordType::Screen => bounds.into(),
                        CoordType::Window => bounds.with_origin(Point::ZERO).into(),
                        _ => unimplemented!(),
                    })
                }
                _ => Err(Error::UnsupportedInterface),
            }
        })
    }

    pub fn layer(&self) -> Result<Layer> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            if wrapper.role() == AtspiRole::Window {
                Ok(Layer::Window)
            } else {
                Ok(Layer::Widget)
            }
        })
    }

    pub fn grab_focus(&self) -> Result<bool> {
        self.do_action_internal(|_, _| ActionRequest {
            action: Action::Focus,
            target: self.id,
            data: None,
        })?;
        Ok(true)
    }

    pub fn scroll_to_point(&self, coord_type: CoordType, x: i32, y: i32) -> Result<bool> {
        self.do_action_internal(|tree_state, context| {
            let window_bounds = context.read_root_window_bounds();
            let is_root = self.id == tree_state.root_id();
            let top_left = window_bounds.top_left(coord_type, is_root);
            let point = Point::new(f64::from(x) - top_left.x, f64::from(y) - top_left.y);
            ActionRequest {
                action: Action::ScrollToPoint,
                target: self.id,
                data: Some(ActionData::ScrollToPoint(point)),
            }
        })?;
        Ok(true)
    }

    pub fn minimum_value(&self) -> Result<f64> {
        self.resolve(|node| Ok(node.state().min_numeric_value().unwrap_or(std::f64::MIN)))
    }

    pub fn maximum_value(&self) -> Result<f64> {
        self.resolve(|node| Ok(node.state().max_numeric_value().unwrap_or(std::f64::MAX)))
    }

    pub fn minimum_increment(&self) -> Result<f64> {
        self.resolve(|node| Ok(node.state().numeric_value_step().unwrap_or(0.0)))
    }

    pub fn current_value(&self) -> Result<f64> {
        self.resolve(|node| {
            let wrapper = NodeWrapper::Node(&node);
            Ok(wrapper.current_value().unwrap_or(0.0))
        })
    }

    pub fn set_current_value(&self, value: f64) -> Result<()> {
        self.do_action_internal(|_, _| ActionRequest {
            action: Action::SetValue,
            target: self.id,
            data: Some(ActionData::NumericValue(value)),
        })
    }
}

impl PartialEq for PlatformNode {
    fn eq(&self, other: &Self) -> bool {
        self.adapter_id == other.adapter_id && self.id == other.id
    }
}

impl Eq for PlatformNode {}

impl Hash for PlatformNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.adapter_id.hash(state);
        self.id.hash(state);
    }
}

#[derive(Clone)]
pub struct PlatformRoot {
    app_context: Weak<RwLock<AppContext>>,
}

impl PlatformRoot {
    pub fn new(app_context: &Arc<RwLock<AppContext>>) -> Self {
        Self {
            app_context: Arc::downgrade(app_context),
        }
    }

    fn resolve_app_context<F, T>(&self, f: F) -> Result<T>
    where
        for<'a> F: FnOnce(RwLockReadGuard<'a, AppContext>) -> Result<T>,
    {
        let app_context = match self.app_context.upgrade() {
            Some(context) => context,
            None => return Err(Error::Defunct),
        };
        let app_context = app_context.read().unwrap();
        f(app_context)
    }

    pub fn name(&self) -> Result<String> {
        self.resolve_app_context(|context| Ok(context.name.clone().unwrap_or_default()))
    }

    pub fn child_count(&self) -> Result<i32> {
        self.resolve_app_context(|context| {
            i32::try_from(context.adapters.len()).map_err(|_| Error::TooManyChildren)
        })
    }

    pub fn child_at_index(&self, index: usize) -> Result<Option<PlatformNode>> {
        self.resolve_app_context(|context| {
            let child = context
                .adapters
                .get(index)
                .map(PlatformNode::from_adapter_root);
            Ok(child)
        })
    }

    pub fn child_id_at_index(&self, index: usize) -> Result<Option<(usize, NodeId)>> {
        self.resolve_app_context(|context| {
            let child = context
                .adapters
                .get(index)
                .map(|(adapter_id, context)| (*adapter_id, context.read_tree().state().root_id()));
            Ok(child)
        })
    }

    pub fn map_children<T, I>(&self, f: impl Fn(PlatformNode) -> I) -> Result<T>
    where
        T: FromIterator<I>,
    {
        self.resolve_app_context(|context| {
            let children = context
                .adapters
                .iter()
                .map(PlatformNode::from_adapter_root)
                .map(f)
                .collect();
            Ok(children)
        })
    }

    pub fn map_child_ids<T, I>(&self, f: impl Fn((usize, NodeId)) -> I) -> Result<T>
    where
        T: FromIterator<I>,
    {
        self.resolve_app_context(|context| {
            let children = context
                .adapters
                .iter()
                .map(|(adapter_id, context)| (*adapter_id, context.read_tree().state().root_id()))
                .map(f)
                .collect();
            Ok(children)
        })
    }

    pub fn toolkit_name(&self) -> Result<String> {
        self.resolve_app_context(|context| Ok(context.toolkit_name.clone().unwrap_or_default()))
    }

    pub fn toolkit_version(&self) -> Result<String> {
        self.resolve_app_context(|context| Ok(context.toolkit_version.clone().unwrap_or_default()))
    }

    pub fn id(&self) -> Result<i32> {
        self.resolve_app_context(|context| Ok(context.id.unwrap_or(-1)))
    }

    pub fn set_id(&mut self, id: i32) -> Result<()> {
        let app_context = match self.app_context.upgrade() {
            Some(context) => context,
            None => return Err(Error::Defunct),
        };
        let mut app_context = app_context.write().unwrap();
        app_context.id = Some(id);
        Ok(())
    }
}

impl PartialEq for PlatformRoot {
    fn eq(&self, other: &Self) -> bool {
        self.app_context.ptr_eq(&other.app_context)
    }
}

impl Hash for PlatformRoot {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.app_context.as_ptr().hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeIdOrRoot {
    Node(NodeId),
    Root,
}
