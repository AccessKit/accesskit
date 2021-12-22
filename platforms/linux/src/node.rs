// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::atspi::{
    interfaces::{Accessible, Application, Interface, Interfaces},
    ObjectId, ObjectRef, OwnedObjectAddress, Role as AtspiRole, State, StateSet,
};
use accesskit::{AriaCurrent, CheckedState, InvalidState, Orientation, Role};
use accesskit_consumer::{Node, Tree, WeakNode};
use std::sync::Arc;

#[derive(Clone)]
pub struct PlatformNode(WeakNode);

impl PlatformNode {
    pub(crate) fn new(node: &Node) -> Self {
        Self(node.downgrade())
    }
}

impl Accessible for PlatformNode {
    fn name(&self) -> String {
        self.0
            .map(|node| node.name().map(|name| name.to_string()))
            .flatten()
            .unwrap_or(String::new())
    }

    fn description(&self) -> String {
        String::new()
    }

    fn parent(&self) -> Option<ObjectRef> {
        Some(
            self.0
                .map(|node| node.parent().map(|parent| parent.id().into()))
                .flatten()
                .unwrap_or(ObjectId::root().into()),
        )
    }

    fn child_count(&self) -> usize {
        self.0.map(|node| node.children().count()).unwrap_or(0)
    }

    fn locale(&self) -> String {
        String::new()
    }

    fn id(&self) -> ObjectId<'static> {
        self.0.map(|node| node.id().into()).unwrap()
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectRef> {
        self.0
            .map(|node| node.children().nth(index).map(|child| child.id().into()))
            .flatten()
    }

    fn children(&self) -> Vec<ObjectRef> {
        self.0
            .map(|node| node.children().map(|child| child.id().into()).collect())
            .unwrap_or(Vec::new())
    }

    fn index_in_parent(&self) -> Option<usize> {
        self.0
            .map(|node| node.parent_and_index().map(|(_, index)| index))
            .flatten()
    }

    fn role(&self) -> AtspiRole {
        self.0
            .map(|node| {
                match node.role() {
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
                    Role::DocBackLink
                    | Role::DocBiblioRef
                    | Role::DocGlossRef
                    | Role::DocNoteRef => AtspiRole::Link,
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
                        if node.unignored_children().next().is_some() {
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
                        if node.unignored_children().next().is_none() {
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
                        if node
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
                        if node.data().protected {
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
            })
            .unwrap_or(AtspiRole::Invalid)
    }

    fn state(&self) -> StateSet {
        let platform_role = self.role();
        self.0
            .map(|node| {
                let data = node.data();
                let mut state = StateSet::empty();
                if let Ok(current_active) = crate::adapter::CURRENT_ACTIVE_WINDOW.lock() {
                    if node.role() == Role::Window && *current_active == Some(data.id) {
                        state.insert(State::Active);
                    }
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
                if !node.is_invisible_or_ignored() {
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
                    if !node.is_disabled() {
                        state.insert(State::Selectable);
                    }
                    if selected {
                        state.insert(State::Selected);
                    }
                }
                if node.is_text_field() {
                    state.insert(State::SelectableText);
                    match node.data().multiline {
                        true => state.insert(State::MultiLine),
                        false => state.insert(State::SingleLine)
                    }
                }

                // Special case for indeterminate progressbar.
                if node.role() == Role::ProgressIndicator && data.value_for_range.is_none() {
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

                if node.is_read_only_supported() && node.is_read_only_or_disabled() {
                    state.insert(State::ReadOnly);
                } else {
                    state.insert(State::Enabled);
                    state.insert(State::Sensitive);
                }

                if node.is_focused() {
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
            })
            .unwrap_or(State::Defunct.into())
    }

    fn interfaces(&self) -> Interfaces {
        self.0
            .map(|node| {
                let mut interfaces: Interfaces = Interface::Accessible | Interface::ObjectEvents;
                if node.role() == Role::Window {
                    interfaces.insert(Interface::WindowEvents);
                }
                if node.data().focusable {
                    interfaces.insert(Interface::FocusEvents);
                }
                interfaces
            })
            .unwrap()
    }
}

#[derive(Clone)]
pub struct RootPlatformNode {
    app_name: String,
    app_id: Option<i32>,
    desktop_address: Option<OwnedObjectAddress>,
    tree: Arc<Tree>,
    toolkit_name: String,
    toolkit_version: String,
}

impl RootPlatformNode {
    pub fn new(
        app_name: String,
        toolkit_name: String,
        toolkit_version: String,
        tree: Arc<Tree>,
    ) -> Self {
        Self {
            app_name,
            app_id: None,
            desktop_address: None,
            tree,
            toolkit_name,
            toolkit_version,
        }
    }
}

impl Application for RootPlatformNode {
    fn name(&self) -> String {
        self.app_name.clone()
    }

    fn child_count(&self) -> usize {
        1
    }

    fn child_at_index(&self, index: usize) -> Option<ObjectRef> {
        if index == 0 {
            Some(self.tree.read().root().id().into())
        } else {
            None
        }
    }

    fn children(&self) -> Vec<ObjectRef> {
        vec![self.tree.read().root().id().into()]
    }

    fn toolkit_name(&self) -> String {
        self.toolkit_name.clone()
    }

    fn toolkit_version(&self) -> String {
        self.toolkit_version.clone()
    }

    fn id(&self) -> Option<i32> {
        self.app_id
    }

    fn set_id(&mut self, id: i32) {
        self.app_id = Some(id);
    }

    fn locale(&self, lctype: u32) -> String {
        String::new()
    }

    fn desktop(&self) -> Option<OwnedObjectAddress> {
        self.desktop_address.clone()
    }

    fn set_desktop(&mut self, address: OwnedObjectAddress) {
        self.desktop_address = Some(address);
    }

    fn register_event_listener(&mut self, _: String) {}

    fn deregister_event_listener(&mut self, _: String) {}
}
