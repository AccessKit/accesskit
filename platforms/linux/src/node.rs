// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{
    interfaces::{Action, ObjectEvent, Property, QueuedEvent},
    ObjectId, ObjectRef, OwnedObjectAddress, Rect as AtspiRect,
};
use accesskit::{
    kurbo::{Point, Rect},
    CheckedState, DefaultActionVerb, NodeId, Role,
};
use accesskit_consumer::{FilterResult, Node, Tree, TreeState};
use atspi::{accessible::Role as AtspiRole, CoordType, Interface, InterfaceSet, State, StateSet};
use parking_lot::RwLock;
use std::{
    convert::TryFrom,
    sync::{Arc, Weak},
};
use zbus::fdo;

pub(crate) fn filter(node: &Node) -> FilterResult {
    if node.is_focused() {
        return FilterResult::Include;
    }

    if node.is_hidden() {
        return FilterResult::ExcludeSubtree;
    }

    let role = node.role();
    if role == Role::Presentation || role == Role::GenericContainer || role == Role::InlineTextBox {
        return FilterResult::ExcludeNode;
    }

    FilterResult::Include
}

pub(crate) struct NodeWrapper<'a> {
    node: &'a Node<'a>,
    app_state: Arc<RwLock<AppState>>,
}

impl<'a> NodeWrapper<'a> {
    pub(crate) fn new(node: &'a Node<'a>, app_state: &Arc<RwLock<AppState>>) -> Self {
        NodeWrapper {
            node,
            app_state: app_state.clone(),
        }
    }

    pub fn name(&self) -> String {
        self.node.name().unwrap_or_default()
    }

    pub fn description(&self) -> String {
        String::new()
    }

    pub fn parent(&self) -> Option<ObjectRef> {
        self.node.parent().map(|parent| parent.id().into())
    }

    pub fn child_count(&self) -> usize {
        self.node.child_ids().count()
    }

    pub fn locale(&self) -> String {
        String::new()
    }

    pub fn id(&self) -> ObjectId<'static> {
        self.node.id().into()
    }

    pub fn role(&self) -> AtspiRole {
        match self.node.role() {
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
            // exposed as forms. http://crbug.com/874384. Forms which have accessible
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
            Role::Image => {
                if self.node.filtered_children(&filter).next().is_some() {
                    AtspiRole::ImageMap
                } else {
                    AtspiRole::Image
                }
            }
            Role::InlineTextBox => AtspiRole::Static,
            Role::InputTime => AtspiRole::DateEditor,
            Role::LabelText | Role::Legend => AtspiRole::Label,
            // Layout table objects are treated the same as `Role::GenericContainer`.
            Role::LayoutTable => AtspiRole::Section,
            Role::LayoutTableCell => AtspiRole::Section,
            Role::LayoutTableRow => AtspiRole::Section,
            // TODO: Having a separate accessible object for line breaks
            // is inconsistent with other implementations. http://crbug.com/873144#c1.
            Role::LineBreak => AtspiRole::Static,
            Role::Link => AtspiRole::Link,
            Role::List => AtspiRole::List,
            Role::ListBox => AtspiRole::ListBox,
            // TODO: Use `AtspiRole::MenuItem' inside a combo box, see how
            // ax_platform_node_win.cc code does this.
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
            Role::ListMarker => {
                if self.node.filtered_children(&filter).next().is_none() {
                    AtspiRole::Static
                } else {
                    AtspiRole::Panel
                }
            }
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
            Role::PopupButton => {
                // TODO: Add a getter for html_tag
                //if self
                //    .node
                //    .data()
                //    .html_tag
                //    .as_ref()
                //    .map_or(false, |tag| tag.as_ref() == "select")
                //{
                //    AtspiRole::ComboBox
                //} else {
                AtspiRole::PushButton
                //}
            }
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
            // AX tree, for example by adding tabindex="0" to the source <rp> or <rt>
            // element or making the source element the target of an aria-owns.
            // Therefore, browser side needs to gracefully handle it if it actually
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
            // ax::mojom::Role::kSubscript =>
            // AtspiRole::Subscript,
            // ax::mojom::Role::kSuperscript =>
            // AtspiRole::Superscript,
            Role::SvgRoot => AtspiRole::DocumentFrame,
            Role::Tab => AtspiRole::PageTab,
            Role::Table => AtspiRole::Table,
            // TODO: This mapping is correct, but it doesn't seem to be
            // used. We don't necessarily want to always expose these containers, but
            // we must do so if they are focusable. http://crbug.com/874043
            Role::TableHeaderContainer => AtspiRole::Panel,
            Role::TabList => AtspiRole::PageTabList,
            Role::TabPanel => AtspiRole::ScrollPane,
            // TODO: This mapping should also be applied to the dfn
            // element. http://crbug.com/874411
            Role::Term => AtspiRole::DescriptionTerm,
            Role::TitleBar => AtspiRole::TitleBar,
            Role::TextField | Role::SearchBox => {
                // TODO: Add a getter for protected
                //if self.node.data().protected {
                //    AtspiRole::PasswordText
                //} else {
                AtspiRole::Entry
                //}
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

    pub fn state(&self) -> StateSet {
        let platform_role = self.role();
        //let data = self.node.data();
        let mut state = StateSet::empty();
        if self.node.role() == Role::Window && self.node.parent().is_none() {
            state.insert(State::Active);
        }
        //if let Some(expanded) = data.expanded {
        //    state.insert(State::Expandable);
        //    if expanded {
        //        state.insert(State::Expanded);
        //    }
        //}
        //if data.default {
        //    state.insert(State::IsDefault);
        //}
        //if data.editable && !data.read_only {
        //    state.insert(State::Editable);
        //}
        // TODO: Focus and selection.
        if self.node.is_focusable() {
            state.insert(State::Focusable);
        }
        //match data.orientation {
        //    Some(Orientation::Horizontal) => state.insert(State::Horizontal),
        //    Some(Orientation::Vertical) => state.insert(State::Vertical),
        //    _ => {}
        //}
        if filter(self.node) == FilterResult::Include {
            state.insert(State::Visible);
            // if (!delegate_->IsOffscreen() && !is_minimized)
            state.insert(State::Showing);
        }
        //if data.multiselectable {
        //    state.insert(State::Multiselectable);
        //}
        //if data.required {
        //    state.insert(State::Required);
        //}
        //if data.visited {
        //    state.insert(State::Visited);
        //}
        //if let Some(InvalidState::True | InvalidState::Other(_)) = data.invalid_state {
        //    state.insert(State::InvalidEntry);
        //}
        //match data.aria_current {
        //    None | Some(AriaCurrent::False) => {}
        //    _ => state.insert(State::Active),
        //}
        if platform_role != AtspiRole::ToggleButton && self.node.checked_state().is_some() {
            state.insert(State::Checkable);
        }
        //if data.has_popup.is_some() {
        //    state.insert(State::HasPopup);
        //}
        //if data.busy {
        //    state.insert(State::Busy);
        //}
        //if data.modal {
        //    state.insert(State::Modal);
        //}
        if let Some(selected) = self.node.is_selected() {
            if !self.node.is_disabled() {
                state.insert(State::Selectable);
            }
            if selected {
                state.insert(State::Selected);
            }
        }
        if self.node.is_text_field() {
            state.insert(State::SelectableText);
            //match self.node.data().multiline {
            //    true => state.insert(State::MultiLine),
            //    false => state.insert(State::SingleLine),
            //}
        }

        // Special case for indeterminate progressbar.
        if self.node.role() == Role::ProgressIndicator && self.node.numeric_value().is_none() {
            state.insert(State::Indeterminate);
        }

        //let has_suggestion = data
        //    .auto_complete
        //    .as_ref()
        //    .map_or(false, |a| !a.as_ref().is_empty());
        //if has_suggestion || data.autofill_available {
        //    state.insert(State::SupportsAutocompletion);
        //}

        // Checked state
        match self.node.checked_state() {
            Some(CheckedState::Mixed) => state.insert(State::Indeterminate),
            Some(CheckedState::True) => {
                if platform_role == AtspiRole::ToggleButton {
                    state.insert(State::Pressed);
                } else {
                    state.insert(State::Checked);
                }
            }
            _ => {}
        }

        if self.node.is_read_only_supported() && self.node.is_read_only_or_disabled() {
            state.insert(State::ReadOnly);
        } else {
            state.insert(State::Enabled);
            state.insert(State::Sensitive);
        }

        if self.node.is_focused() {
            state.insert(State::Focused);
        }

        // It is insufficient to compare with g_current_activedescendant due to both
        // timing and event ordering for objects which implement AtkSelection and also
        // have an active descendant. For instance, if we check the state set of a
        // selectable child, it will only have ATK_STATE_FOCUSED if we've processed
        // the activedescendant change.
        // if (GetActiveDescendantOfCurrentFocused() == atk_object)
        //     state.insert(State::Focused);
        state
    }

    pub fn interfaces(&self) -> InterfaceSet {
        let mut interfaces = InterfaceSet::new(Interface::Accessible);
        if self.node.default_action_verb().is_some() {
            interfaces.insert(Interface::Action);
        }
        if self.node.bounding_box().is_some() || self.node.is_root() {
            interfaces.insert(Interface::Component);
        }
        if self.node.numeric_value().is_some() {
            interfaces.insert(Interface::Value);
        }
        interfaces
    }

    pub fn n_actions(&self) -> i32 {
        self.node.default_action_verb().map_or(0, |_| 1)
    }

    pub fn get_action_name(&self, index: i32) -> String {
        if index != 0 {
            return String::new();
        }
        String::from(match self.node.default_action_verb() {
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
        let parent = self.parent();
        if parent != old.parent() {
            queue.push(QueuedEvent::Object {
                target: self.id(),
                event: ObjectEvent::PropertyChanged(Property::Parent(parent)),
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

fn unknown_object() -> fdo::Error {
    fdo::Error::UnknownObject("".into())
}

#[derive(Clone)]
pub(crate) struct PlatformNode {
    tree: Weak<Tree>,
    node_id: NodeId,
    app_state: Weak<RwLock<AppState>>,
}

impl PlatformNode {
    pub(crate) fn new(
        tree: &Arc<Tree>,
        node_id: NodeId,
        app_state: &Arc<RwLock<AppState>>,
    ) -> Self {
        Self {
            tree: Arc::downgrade(tree),
            node_id,
            app_state: Arc::downgrade(app_state),
        }
    }

    fn upgrade_tree(&self) -> fdo::Result<Arc<Tree>> {
        if let Some(tree) = self.tree.upgrade() {
            Ok(tree)
        } else {
            Err(unknown_object())
        }
    }

    fn upgrade_app_state(&self) -> fdo::Result<Arc<RwLock<AppState>>> {
        if let Some(state) = self.app_state.upgrade() {
            Ok(state)
        } else {
            Err(unknown_object())
        }
    }

    fn with_state<F, T>(&self, f: F) -> fdo::Result<T>
    where
        F: FnOnce((&TreeState, &Arc<RwLock<AppState>>)) -> fdo::Result<T>,
    {
        let tree = self.upgrade_tree()?;
        let app_state = self.upgrade_app_state()?;
        let state = tree.read();
        f((&state, &app_state))
    }

    fn resolve<F, T>(&self, f: F) -> fdo::Result<T>
    where
        for<'a> F: FnOnce(NodeWrapper<'a>) -> fdo::Result<T>,
    {
        self.with_state(|(tree, app)| {
            if let Some(node) = tree.node_by_id(self.node_id) {
                f(NodeWrapper::new(&node, app))
            } else {
                Err(unknown_object())
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
            Err(unknown_object())
        }
    }

    pub fn name(&self) -> fdo::Result<String> {
        self.resolve(|resolved| Ok(resolved.name()))
    }

    pub fn description(&self) -> fdo::Result<String> {
        self.resolve(|resolved| Ok(resolved.description()))
    }

    pub fn parent(&self) -> fdo::Result<ObjectRef> {
        self.resolve(|resolved| {
            Ok(resolved
                .parent()
                .unwrap_or_else(|| ObjectRef::Managed(ObjectId::root())))
        })
    }

    pub fn child_count(&self) -> fdo::Result<i32> {
        self.resolve(|resolved| {
            i32::try_from(resolved.child_count())
                .map_err(|_| fdo::Error::Failed("Too many children.".into()))
        })
    }

    pub fn locale(&self) -> fdo::Result<String> {
        self.resolve(|resolved| Ok(resolved.locale()))
    }

    pub fn accessible_id(&self) -> fdo::Result<ObjectId> {
        self.resolve(|resolved| Ok(resolved.id()))
    }

    pub fn child_at_index(&self, index: usize) -> fdo::Result<Option<ObjectRef>> {
        self.resolve(|resolved| Ok(resolved.node.child_ids().nth(index).map(ObjectRef::from)))
    }

    pub fn children(&self) -> fdo::Result<Vec<ObjectRef>> {
        self.resolve(|resolved| Ok(resolved.node.child_ids().map(ObjectRef::from).collect()))
    }

    pub fn index_in_parent(&self) -> fdo::Result<i32> {
        self.resolve(|resolved| {
            resolved
                .node
                .parent_and_index()
                .map_or(Ok(-1), |(_, index)| {
                    i32::try_from(index).map_err(|_| fdo::Error::Failed("Index is too big.".into()))
                })
        })
    }

    pub fn role(&self) -> fdo::Result<AtspiRole> {
        self.resolve(|resolved| Ok(resolved.role()))
    }

    pub fn state(&self) -> fdo::Result<StateSet> {
        self.resolve(|resolved| Ok(resolved.state()))
    }

    pub fn interfaces(&self) -> fdo::Result<InterfaceSet> {
        self.resolve(|resolved| Ok(resolved.interfaces()))
    }

    pub fn n_actions(&self) -> fdo::Result<i32> {
        self.resolve(|resolved| Ok(resolved.n_actions()))
    }

    pub fn get_action_name(&self, index: i32) -> fdo::Result<String> {
        self.resolve(|resolved| Ok(resolved.get_action_name(index)))
    }

    pub fn get_actions(&self) -> fdo::Result<Vec<Action>> {
        self.resolve(|resolved| {
            let n_actions = resolved.n_actions() as usize;
            let mut actions = Vec::with_capacity(n_actions);
            for i in 0..n_actions {
                actions.push(Action {
                    localized_name: resolved.get_action_name(i as i32),
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

    pub fn contains(&self, x: i32, y: i32, coord_type: CoordType) -> fdo::Result<bool> {
        self.resolve(|wrapper| {
            let app_state = wrapper.app_state.read();
            let bounds = match wrapper.node.bounding_box() {
                Some(node_bounds) => {
                    let top_left = match coord_type {
                        CoordType::Screen => app_state.inner_bounds.origin(),
                        CoordType::Window => {
                            let outer_position = app_state.outer_bounds.origin();
                            let inner_position = app_state.inner_bounds.origin();
                            Point::new(
                                inner_position.x - outer_position.x,
                                inner_position.y - outer_position.y,
                            )
                        }
                        _ => unimplemented!(),
                    };
                    let new_origin =
                        Point::new(top_left.x + node_bounds.x0, top_left.y + node_bounds.y0);
                    node_bounds.with_origin(new_origin)
                }
                None if wrapper.node.is_root() => {
                    let window_bounds = app_state.outer_bounds;
                    match coord_type {
                        CoordType::Screen => window_bounds,
                        CoordType::Window => window_bounds.with_origin(Point::ZERO),
                        _ => unimplemented!(),
                    }
                }
                _ => return Err(unknown_object()),
            };
            Ok(bounds.contains(Point::new(x.into(), y.into())))
        })
    }

    pub fn get_accessible_at_point(
        &self,
        x: i32,
        y: i32,
        coord_type: CoordType,
    ) -> fdo::Result<Option<ObjectRef>> {
        self.resolve(|wrapper| {
            let app_state = wrapper.app_state.read();
            let is_root = wrapper.node.is_root();
            let top_left = match coord_type {
                CoordType::Screen if is_root => app_state.outer_bounds.origin(),
                CoordType::Screen => app_state.inner_bounds.origin(),
                CoordType::Window if is_root => Point::ZERO,
                CoordType::Window => app_state.inner_bounds.origin(),
                _ => unimplemented!(),
            };
            let point = Point::new(f64::from(x) - top_left.x, f64::from(y) - top_left.y);
            Ok(wrapper
                .node
                .node_at_point(point, &filter)
                .map(|node| ObjectRef::Managed(node.id().into())))
        })
    }

    pub fn get_extents(&self, coord_type: CoordType) -> fdo::Result<(AtspiRect,)> {
        self.resolve(|wrapper| {
            let app_state = wrapper.app_state.read();
            match wrapper.node.bounding_box() {
                Some(node_bounds) => {
                    let top_left = match coord_type {
                        CoordType::Screen => app_state.inner_bounds.origin(),
                        CoordType::Window => {
                            let outer_position = app_state.outer_bounds.origin();
                            let inner_position = app_state.inner_bounds.origin();
                            Point::new(
                                inner_position.x - outer_position.x,
                                inner_position.y - outer_position.y,
                            )
                        }
                        _ => unimplemented!(),
                    };
                    let new_origin =
                        Point::new(top_left.x + node_bounds.x0, top_left.y + node_bounds.y0);
                    Ok((node_bounds.with_origin(new_origin).into(),))
                }
                None if wrapper.node.is_root() => {
                    let window_bounds = app_state.outer_bounds;
                    Ok((match coord_type {
                        CoordType::Screen => window_bounds.into(),
                        CoordType::Window => window_bounds.with_origin(Point::ZERO).into(),
                        _ => unimplemented!(),
                    },))
                }
                _ => Err(unknown_object()),
            }
        })
    }

    pub fn grab_focus(&self) -> fdo::Result<bool> {
        let tree = self.validate_for_action()?;
        tree.set_focus(self.node_id);
        Ok(true)
    }

    pub fn minimum_value(&self) -> fdo::Result<f64> {
        self.resolve(|resolved| Ok(resolved.node.min_numeric_value().unwrap_or(std::f64::MIN)))
    }

    pub fn maximum_value(&self) -> fdo::Result<f64> {
        self.resolve(|resolved| Ok(resolved.node.max_numeric_value().unwrap_or(std::f64::MAX)))
    }

    pub fn minimum_increment(&self) -> fdo::Result<f64> {
        self.resolve(|resolved| Ok(resolved.node.numeric_value_step().unwrap_or(0.0)))
    }

    pub fn current_value(&self) -> fdo::Result<f64> {
        self.resolve(|resolved| Ok(resolved.node.numeric_value().unwrap_or(0.0)))
    }

    pub fn set_current_value(&self, value: f64) -> fdo::Result<()> {
        let tree = self.validate_for_action()?;
        tree.set_numeric_value(self.node_id, value);
        Ok(())
    }
}

pub(crate) struct AppState {
    pub name: String,
    pub toolkit_name: String,
    pub toolkit_version: String,
    pub id: Option<i32>,
    pub desktop_address: Option<OwnedObjectAddress>,
    pub outer_bounds: Rect,
    pub inner_bounds: Rect,
}

impl AppState {
    pub fn new(name: String, toolkit_name: String, toolkit_version: String) -> Self {
        Self {
            name,
            toolkit_name,
            toolkit_version,
            id: None,
            desktop_address: None,
            outer_bounds: Rect::default(),
            inner_bounds: Rect::default(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct PlatformRootNode {
    pub state: Weak<RwLock<AppState>>,
    pub tree: Weak<Tree>,
}

impl PlatformRootNode {
    pub fn new(state: Weak<RwLock<AppState>>, tree: Weak<Tree>) -> Self {
        Self { state, tree }
    }
}
