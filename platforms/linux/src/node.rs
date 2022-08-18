// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{
    interfaces::{EventKind, Interface, Interfaces, ObjectEvent, Property, QueuedEvent},
    ObjectId, ObjectRef, OwnedObjectAddress, Role as AtspiRole, State, StateSet,
};
use accesskit::{AriaCurrent, CheckedState, InvalidState, Orientation, Role};
use accesskit_consumer::{Node, Tree, WeakNode};
use parking_lot::RwLock;
use std::sync::Weak;
use zbus::fdo;
use zvariant::Value;

pub(crate) struct ResolvedPlatformNode<'a> {
    node: Node<'a>,
}

impl ResolvedPlatformNode<'_> {
    pub fn new(node: Node) -> ResolvedPlatformNode {
        ResolvedPlatformNode { node }
    }

    pub fn downgrade(&self) -> PlatformNode {
        PlatformNode::new(&self.node)
    }

    pub fn name(&self) -> String {
        self.node
            .name()
            .map(|name| name.to_string())
            .unwrap_or(String::new())
    }

    pub fn description(&self) -> String {
        String::new()
    }

    pub fn parent(&self) -> Option<ObjectRef> {
        self.node.parent().map(|parent| parent.id().into())
    }

    pub fn child_count(&self) -> usize {
        self.node.children().count()
    }

    pub fn locale(&self) -> String {
        String::new()
    }

    pub fn id(&self) -> ObjectId<'static> {
        self.node.id().into()
    }

    pub fn child_at_index(&self, index: usize) -> Option<ObjectRef> {
        self.node
            .children()
            .nth(index)
            .map(|child| child.id().into())
    }

    pub fn children(&self) -> Vec<ObjectRef> {
        self.node
            .children()
            .map(|child| child.id().into())
            .collect()
    }

    pub fn index_in_parent(&self) -> Option<usize> {
        self.node.parent_and_index().map(|(_, index)| index)
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
                if self.node.unignored_children().next().is_some() {
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
                if self.node.unignored_children().next().is_none() {
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
                if self
                    .node
                    .data()
                    .html_tag
                    .as_ref()
                    .map_or(false, |tag| tag.as_ref() == "select")
                {
                    AtspiRole::ComboBox
                } else {
                    AtspiRole::PushButton
                }
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
                if self.node.data().protected {
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

    pub fn state(&self) -> StateSet {
        let platform_role = self.role();
        let data = self.node.data();
        let mut state = StateSet::empty();
        if self.node.role() == Role::Window && self.node.parent().is_none() {
            state.insert(State::Active);
        }
        if let Some(expanded) = data.expanded {
            state.insert(State::Expandable);
            if expanded {
                state.insert(State::Expanded);
            }
        }
        if data.default {
            state.insert(State::IsDefault);
        }
        if data.editable && !data.read_only {
            state.insert(State::Editable);
        }
        // TODO: Focus and selection.
        if data.focusable {
            state.insert(State::Focusable);
        }
        match data.orientation {
            Some(Orientation::Horizontal) => state.insert(State::Horizontal),
            Some(Orientation::Vertical) => state.insert(State::Vertical),
            _ => {}
        }
        if !self.node.is_invisible_or_ignored() {
            state.insert(State::Visible);
            // if (!delegate_->IsOffscreen() && !is_minimized)
            state.insert(State::Showing);
        }
        if data.multiselectable {
            state.insert(State::Multiselectable);
        }
        if data.required {
            state.insert(State::Required);
        }
        if data.visited {
            state.insert(State::Visited);
        }
        if let Some(InvalidState::True | InvalidState::Other(_)) = data.invalid_state {
            state.insert(State::InvalidEntry);
        }
        match data.aria_current {
            None | Some(AriaCurrent::False) => {}
            _ => state.insert(State::Active),
        }
        if platform_role != AtspiRole::ToggleButton && data.checked_state.is_some() {
            state.insert(State::Checkable);
        }
        if data.has_popup.is_some() {
            state.insert(State::HasPopup);
        }
        if data.busy {
            state.insert(State::Busy);
        }
        if data.modal {
            state.insert(State::Modal);
        }
        if let Some(selected) = data.selected {
            if !self.node.is_disabled() {
                state.insert(State::Selectable);
            }
            if selected {
                state.insert(State::Selected);
            }
        }
        if self.node.is_text_field() {
            state.insert(State::SelectableText);
            match self.node.data().multiline {
                true => state.insert(State::MultiLine),
                false => state.insert(State::SingleLine),
            }
        }

        // Special case for indeterminate progressbar.
        if self.node.role() == Role::ProgressIndicator && data.numeric_value.is_none() {
            state.insert(State::Indeterminate);
        }

        let has_suggestion = data
            .auto_complete
            .as_ref()
            .map_or(false, |a| !a.as_ref().is_empty());
        if has_suggestion || data.autofill_available {
            state.insert(State::SupportsAutocompletion);
        }

        // Checked state
        match data.checked_state {
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

    pub fn interfaces(&self) -> Interfaces {
        let mut interfaces = Interfaces::new(Interface::Accessible);
        if self.node.numeric_value().is_some() {
            interfaces.insert(Interface::Value);
        }
        interfaces
    }

    pub fn minimum_value(&self) -> f64 {
        self.node.min_numeric_value().unwrap_or(std::f64::MIN)
    }

    pub fn maximum_value(&self) -> f64 {
        self.node.max_numeric_value().unwrap_or(std::f64::MAX)
    }

    pub fn minimum_increment(&self) -> f64 {
        self.node.numeric_value_step().unwrap_or(0.0)
    }

    pub fn current_value(&self) -> f64 {
        self.node.numeric_value().unwrap_or(0.0)
    }

    pub fn set_current_value(&self, value: f64) {
        self.node.set_numeric_value(value)
    }

    pub fn enqueue_changes(&self, queue: &mut Vec<QueuedEvent>, old: &ResolvedPlatformNode) {
        let old_state = old.state();
        let new_state = self.state();
        let changed_states = old_state ^ new_state;
        for state in changed_states.iter() {
            queue.push(QueuedEvent {
                target: self.id(),
                kind: EventKind::Object(ObjectEvent::StateChanged(
                    state,
                    new_state.contains(state),
                )),
            });
        }
        let name = self.name();
        if name != old.name() {
            queue.push(QueuedEvent {
                target: self.id(),
                kind: EventKind::Object(ObjectEvent::PropertyChanged(
                    Property::AccessibleName,
                    Value::from(name).into(),
                )),
            });
        }
        let description = self.description();
        if description != old.description() {
            queue.push(QueuedEvent {
                target: self.id(),
                kind: EventKind::Object(ObjectEvent::PropertyChanged(
                    Property::AccessibleDescription,
                    Value::from(description).into(),
                )),
            });
        }
        let role = self.role();
        if role != old.role() {
            queue.push(QueuedEvent {
                target: self.id(),
                kind: EventKind::Object(ObjectEvent::PropertyChanged(
                    Property::AccessibleRole,
                    Value::from(role as u32).into(),
                )),
            });
        }
    }
}

#[derive(Clone)]
pub(crate) struct PlatformNode(WeakNode);

impl PlatformNode {
    pub fn new(node: &Node) -> Self {
        Self(node.downgrade())
    }

    pub fn resolve<F, T>(&self, f: F) -> fdo::Result<T>
    where
        for<'a> F: FnOnce(ResolvedPlatformNode<'a>) -> T,
    {
        self.0
            .map(|node| f(ResolvedPlatformNode::new(node)))
            .ok_or(fdo::Error::UnknownObject("".into()))
    }
}

pub(crate) struct AppState {
    pub name: String,
    pub toolkit_name: String,
    pub toolkit_version: String,
    pub id: Option<i32>,
    pub desktop_address: Option<OwnedObjectAddress>,
}

impl AppState {
    pub fn new(name: String, toolkit_name: String, toolkit_version: String) -> Self {
        Self {
            name,
            toolkit_name,
            toolkit_version,
            id: None,
            desktop_address: None,
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
