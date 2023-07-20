// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2021 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#![allow(non_upper_case_globals)]

use accesskit::{
    Action, ActionData, ActionRequest, CheckedState, Live, NodeId, NodeIdContent, Point, Role,
};
use accesskit_consumer::{DetachedNode, FilterResult, Node, NodeState, TreeState};
use arrayvec::ArrayVec;
use paste::paste;
use std::sync::{Arc, Weak};
use windows::{
    core::*,
    Win32::{Foundation::*, System::Com::*, UI::Accessibility::*},
};

use crate::{context::Context, text::PlatformRange as PlatformTextRange, util::*};

fn runtime_id_from_node_id(id: NodeId) -> impl std::ops::Deref<Target = [i32]> {
    let mut result = ArrayVec::<i32, { std::mem::size_of::<NodeIdContent>() + 1 }>::new();
    result.push(UiaAppendRuntimeId as i32);
    let id = id.0;
    let id_bytes = id.get().to_be_bytes();
    let start_index: usize = (id.leading_zeros() / 8) as usize;
    for byte in &id_bytes[start_index..] {
        result.push((*byte).into());
    }
    result
}

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

fn filter_with_root_exception(node: &Node) -> FilterResult {
    if node.is_root() {
        return FilterResult::Include;
    }
    filter(node)
}

pub(crate) enum NodeWrapper<'a> {
    Node(&'a Node<'a>),
    DetachedNode(&'a DetachedNode),
}

impl<'a> NodeWrapper<'a> {
    fn node_state(&self) -> &'a NodeState {
        match self {
            Self::Node(node) => node.state(),
            Self::DetachedNode(node) => node.state(),
        }
    }

    fn control_type(&self) -> UIA_CONTROLTYPE_ID {
        let role = self.node_state().role();
        // TODO: Handle special cases. (#14)
        match role {
            Role::Unknown => UIA_CustomControlTypeId,
            Role::InlineTextBox => UIA_CustomControlTypeId,
            Role::Cell => UIA_DataItemControlTypeId,
            Role::StaticText => UIA_TextControlTypeId,
            Role::Image => UIA_ImageControlTypeId,
            Role::Link => UIA_HyperlinkControlTypeId,
            Role::Row => UIA_DataItemControlTypeId,
            Role::ListItem => UIA_ListItemControlTypeId,
            Role::ListMarker => UIA_GroupControlTypeId,
            Role::TreeItem => UIA_TreeItemControlTypeId,
            Role::ListBoxOption => UIA_ListItemControlTypeId,
            Role::MenuItem => UIA_MenuItemControlTypeId,
            Role::MenuListOption => UIA_ListItemControlTypeId,
            Role::Paragraph => UIA_GroupControlTypeId,
            Role::GenericContainer => UIA_GroupControlTypeId,
            Role::Presentation => UIA_GroupControlTypeId,
            Role::CheckBox => UIA_CheckBoxControlTypeId,
            Role::RadioButton => UIA_RadioButtonControlTypeId,
            Role::TextField => UIA_EditControlTypeId,
            Role::Button => UIA_ButtonControlTypeId,
            Role::LabelText => UIA_TextControlTypeId,
            Role::Pane => UIA_PaneControlTypeId,
            Role::RowHeader => UIA_DataItemControlTypeId,
            Role::ColumnHeader => UIA_DataItemControlTypeId,
            Role::Column => UIA_GroupControlTypeId,
            Role::RowGroup => UIA_GroupControlTypeId,
            Role::List => UIA_ListControlTypeId,
            Role::Table => UIA_TableControlTypeId,
            Role::TableHeaderContainer => UIA_GroupControlTypeId,
            Role::LayoutTableCell => UIA_DataItemControlTypeId,
            Role::LayoutTableRow => UIA_DataItemControlTypeId,
            Role::LayoutTable => UIA_TableControlTypeId,
            Role::Switch => UIA_ButtonControlTypeId,
            Role::ToggleButton => UIA_ButtonControlTypeId,
            Role::Menu => UIA_MenuControlTypeId,
            Role::Abbr => UIA_TextControlTypeId,
            Role::Alert => UIA_TextControlTypeId,
            Role::AlertDialog => {
                // Chromium's implementation suggests the use of
                // UIA_TextControlTypeId, not UIA_PaneControlTypeId, because some
                // Windows screen readers are not compatible with
                // Role::AlertDialog yet.
                UIA_TextControlTypeId
            }
            Role::Application => UIA_PaneControlTypeId,
            Role::Article => UIA_GroupControlTypeId,
            Role::Audio => UIA_GroupControlTypeId,
            Role::Banner => UIA_GroupControlTypeId,
            Role::Blockquote => UIA_GroupControlTypeId,
            Role::Canvas => UIA_ImageControlTypeId,
            Role::Caption => UIA_TextControlTypeId,
            Role::Caret => UIA_GroupControlTypeId,
            Role::Client => UIA_PaneControlTypeId,
            Role::Code => UIA_TextControlTypeId,
            Role::ColorWell => UIA_ButtonControlTypeId,
            Role::ComboBoxGrouping => UIA_ComboBoxControlTypeId,
            Role::ComboBoxMenuButton => UIA_ComboBoxControlTypeId,
            Role::Complementary => UIA_GroupControlTypeId,
            Role::Comment => UIA_GroupControlTypeId,
            Role::ContentDeletion => UIA_GroupControlTypeId,
            Role::ContentInsertion => UIA_GroupControlTypeId,
            Role::ContentInfo => UIA_GroupControlTypeId,
            Role::Date => UIA_EditControlTypeId,
            Role::DateTime => UIA_EditControlTypeId,
            Role::Definition => UIA_GroupControlTypeId,
            Role::DescriptionList => UIA_ListControlTypeId,
            Role::DescriptionListDetail => UIA_TextControlTypeId,
            Role::DescriptionListTerm => UIA_ListItemControlTypeId,
            Role::Details => UIA_GroupControlTypeId,
            Role::Dialog => UIA_PaneControlTypeId,
            Role::Directory => UIA_ListControlTypeId,
            Role::DisclosureTriangle => UIA_ButtonControlTypeId,
            Role::Document => UIA_DocumentControlTypeId,
            Role::EmbeddedObject => UIA_PaneControlTypeId,
            Role::Emphasis => UIA_TextControlTypeId,
            Role::Feed => UIA_GroupControlTypeId,
            Role::FigureCaption => UIA_TextControlTypeId,
            Role::Figure => UIA_GroupControlTypeId,
            Role::Footer => UIA_GroupControlTypeId,
            Role::FooterAsNonLandmark => UIA_GroupControlTypeId,
            Role::Form => UIA_GroupControlTypeId,
            Role::Grid => UIA_DataGridControlTypeId,
            Role::Group => UIA_GroupControlTypeId,
            Role::Header => UIA_GroupControlTypeId,
            Role::HeaderAsNonLandmark => UIA_GroupControlTypeId,
            Role::Heading => UIA_TextControlTypeId,
            Role::Iframe => UIA_DocumentControlTypeId,
            Role::IframePresentational => UIA_GroupControlTypeId,
            Role::ImeCandidate => UIA_PaneControlTypeId,
            Role::InputTime => UIA_GroupControlTypeId,
            Role::Keyboard => UIA_PaneControlTypeId,
            Role::Legend => UIA_TextControlTypeId,
            Role::LineBreak => UIA_TextControlTypeId,
            Role::ListBox => UIA_ListControlTypeId,
            Role::Log => UIA_GroupControlTypeId,
            Role::Main => UIA_GroupControlTypeId,
            Role::Mark => UIA_TextControlTypeId,
            Role::Marquee => UIA_TextControlTypeId,
            Role::Math => UIA_GroupControlTypeId,
            Role::MenuBar => UIA_MenuBarControlTypeId,
            Role::MenuItemCheckBox => UIA_CheckBoxControlTypeId,
            Role::MenuItemRadio => UIA_RadioButtonControlTypeId,
            Role::MenuListPopup => UIA_ListControlTypeId,
            Role::Meter => UIA_ProgressBarControlTypeId,
            Role::Navigation => UIA_GroupControlTypeId,
            Role::Note => UIA_GroupControlTypeId,
            Role::PluginObject => UIA_GroupControlTypeId,
            Role::PopupButton => {
                // TODO: handle combo-box special case. (#25)
                UIA_ButtonControlTypeId
            }
            Role::Portal => UIA_ButtonControlTypeId,
            Role::Pre => UIA_GroupControlTypeId,
            Role::ProgressIndicator => UIA_ProgressBarControlTypeId,
            Role::RadioGroup => UIA_GroupControlTypeId,
            Role::Region => UIA_GroupControlTypeId,
            Role::RootWebArea => UIA_DocumentControlTypeId,
            Role::Ruby => UIA_GroupControlTypeId,
            Role::RubyAnnotation => {
                // Generally exposed as description on <ruby> (Role::Ruby)
                // element, not as its own object in the tree.
                // However, it's possible to make a RubyAnnotation element
                // show up in the AX tree, for example by adding tabindex="0"
                // to the source <rp> or <rt> element or making the source element
                // the target of an aria-owns. Therefore, browser side needs to
                // gracefully handle it if it actually shows up in the tree.
                UIA_TextControlTypeId
            }
            Role::ScrollBar => UIA_ScrollBarControlTypeId,
            Role::ScrollView => UIA_PaneControlTypeId,
            Role::Search => UIA_GroupControlTypeId,
            Role::SearchBox => UIA_EditControlTypeId,
            Role::Section => UIA_GroupControlTypeId,
            Role::Slider => UIA_SliderControlTypeId,
            Role::SpinButton => UIA_SpinnerControlTypeId,
            Role::Splitter => UIA_SeparatorControlTypeId,
            Role::Status => UIA_StatusBarControlTypeId,
            Role::Strong => UIA_TextControlTypeId,
            Role::Suggestion => UIA_GroupControlTypeId,
            Role::SvgRoot => UIA_ImageControlTypeId,
            Role::Tab => UIA_TabItemControlTypeId,
            Role::TabList => UIA_TabControlTypeId,
            Role::TabPanel => UIA_PaneControlTypeId,
            Role::Term => UIA_ListItemControlTypeId,
            Role::TextFieldWithComboBox => UIA_ComboBoxControlTypeId,
            Role::Time => UIA_TextControlTypeId,
            Role::Timer => UIA_PaneControlTypeId,
            Role::TitleBar => UIA_PaneControlTypeId,
            Role::Toolbar => UIA_ToolBarControlTypeId,
            Role::Tooltip => UIA_ToolTipControlTypeId,
            Role::Tree => UIA_TreeControlTypeId,
            Role::TreeGrid => UIA_DataGridControlTypeId,
            Role::Video => UIA_GroupControlTypeId,
            Role::WebView => UIA_DocumentControlTypeId,
            Role::Window => {
                // TODO: determine whether to use Window or Pane.
                // It may be good to use Pane for nested windows,
                // as Chromium does. (#14)
                UIA_WindowControlTypeId
            }
            Role::PdfActionableHighlight => UIA_CustomControlTypeId,
            Role::PdfRoot => UIA_DocumentControlTypeId,
            Role::GraphicsDocument => UIA_DocumentControlTypeId,
            Role::GraphicsObject => UIA_PaneControlTypeId,
            Role::GraphicsSymbol => UIA_ImageControlTypeId,
            Role::DocAbstract => UIA_GroupControlTypeId,
            Role::DocAcknowledgements => UIA_GroupControlTypeId,
            Role::DocAfterword => UIA_GroupControlTypeId,
            Role::DocAppendix => UIA_GroupControlTypeId,
            Role::DocBackLink => UIA_HyperlinkControlTypeId,
            Role::DocBiblioEntry => UIA_ListItemControlTypeId,
            Role::DocBibliography => UIA_GroupControlTypeId,
            Role::DocBiblioRef => UIA_HyperlinkControlTypeId,
            Role::DocChapter => UIA_GroupControlTypeId,
            Role::DocColophon => UIA_GroupControlTypeId,
            Role::DocConclusion => UIA_GroupControlTypeId,
            Role::DocCover => UIA_ImageControlTypeId,
            Role::DocCredit => UIA_GroupControlTypeId,
            Role::DocCredits => UIA_GroupControlTypeId,
            Role::DocDedication => UIA_GroupControlTypeId,
            Role::DocEndnote => UIA_ListItemControlTypeId,
            Role::DocEndnotes => UIA_GroupControlTypeId,
            Role::DocEpigraph => UIA_GroupControlTypeId,
            Role::DocEpilogue => UIA_GroupControlTypeId,
            Role::DocErrata => UIA_GroupControlTypeId,
            Role::DocExample => UIA_GroupControlTypeId,
            Role::DocFootnote => UIA_ListItemControlTypeId,
            Role::DocForeword => UIA_GroupControlTypeId,
            Role::DocGlossary => UIA_GroupControlTypeId,
            Role::DocGlossRef => UIA_HyperlinkControlTypeId,
            Role::DocIndex => UIA_GroupControlTypeId,
            Role::DocIntroduction => UIA_GroupControlTypeId,
            Role::DocNoteRef => UIA_HyperlinkControlTypeId,
            Role::DocNotice => UIA_GroupControlTypeId,
            Role::DocPageBreak => UIA_SeparatorControlTypeId,
            Role::DocPageFooter => UIA_GroupControlTypeId,
            Role::DocPageHeader => UIA_GroupControlTypeId,
            Role::DocPageList => UIA_GroupControlTypeId,
            Role::DocPart => UIA_GroupControlTypeId,
            Role::DocPreface => UIA_GroupControlTypeId,
            Role::DocPrologue => UIA_GroupControlTypeId,
            Role::DocPullquote => UIA_GroupControlTypeId,
            Role::DocQna => UIA_GroupControlTypeId,
            Role::DocSubtitle => UIA_GroupControlTypeId,
            Role::DocTip => UIA_GroupControlTypeId,
            Role::DocToc => UIA_GroupControlTypeId,
            Role::ListGrid => UIA_DataGridControlTypeId,
        }
    }

    fn name(&self) -> Option<String> {
        match self {
            Self::Node(node) => node.name(),
            Self::DetachedNode(node) => node.name(),
        }
    }

    fn is_content_element(&self) -> bool {
        let result = match self {
            Self::Node(node) => filter(node),
            Self::DetachedNode(node) => filter_detached(node),
        };
        result == FilterResult::Include
    }

    fn is_enabled(&self) -> bool {
        !self.node_state().is_disabled()
    }

    fn is_focusable(&self) -> bool {
        self.node_state().is_focusable()
    }

    fn is_focused(&self) -> bool {
        match self {
            Self::Node(node) => node.is_focused(),
            Self::DetachedNode(node) => node.is_focused(),
        }
    }

    fn live_setting(&self) -> LiveSetting {
        let live = match self {
            Self::Node(node) => node.live(),
            Self::DetachedNode(node) => node.live(),
        };
        match live {
            Live::Off => Off,
            Live::Polite => Polite,
            Live::Assertive => Assertive,
        }
    }

    fn is_toggle_pattern_supported(&self) -> bool {
        self.node_state().checked_state().is_some() && !self.is_selection_item_pattern_supported()
    }

    fn toggle_state(&self) -> ToggleState {
        match self.node_state().checked_state().unwrap() {
            CheckedState::False => ToggleState_Off,
            CheckedState::True => ToggleState_On,
            CheckedState::Mixed => ToggleState_Indeterminate,
        }
    }

    fn is_invoke_pattern_supported(&self) -> bool {
        self.node_state().is_invocable()
    }

    fn is_value_pattern_supported(&self) -> bool {
        self.node_state().value().is_some()
    }

    fn is_range_value_pattern_supported(&self) -> bool {
        self.node_state().numeric_value().is_some()
    }

    fn value(&self) -> &str {
        self.node_state().value().unwrap()
    }

    fn is_read_only(&self) -> bool {
        self.node_state().is_read_only()
    }

    fn numeric_value(&self) -> f64 {
        self.node_state().numeric_value().unwrap()
    }

    fn min_numeric_value(&self) -> f64 {
        self.node_state().min_numeric_value().unwrap_or(0.0)
    }

    fn max_numeric_value(&self) -> f64 {
        self.node_state().max_numeric_value().unwrap_or(0.0)
    }

    fn numeric_value_step(&self) -> f64 {
        self.node_state().numeric_value_step().unwrap_or(0.0)
    }

    fn numeric_value_jump(&self) -> f64 {
        self.node_state()
            .numeric_value_jump()
            .unwrap_or_else(|| self.numeric_value_step())
    }

    fn is_selection_item_pattern_supported(&self) -> bool {
        match self.node_state().role() {
            // TODO: tables (#29)
            // https://www.w3.org/TR/core-aam-1.1/#mapping_state-property_table
            // SelectionItem.IsSelected is exposed when aria-checked is True or
            // False, for 'radio' and 'menuitemradio' roles.
            Role::RadioButton | Role::MenuItemRadio => matches!(
                self.node_state().checked_state(),
                Some(CheckedState::True | CheckedState::False)
            ),
            // https://www.w3.org/TR/wai-aria-1.1/#aria-selected
            // SelectionItem.IsSelected is exposed when aria-select is True or False.
            Role::ListBoxOption
            | Role::ListItem
            | Role::MenuListOption
            | Role::Tab
            | Role::TreeItem => self.node_state().is_selected().is_some(),
            _ => false,
        }
    }

    fn is_selected(&self) -> bool {
        match self.node_state().role() {
            // https://www.w3.org/TR/core-aam-1.1/#mapping_state-property_table
            // SelectionItem.IsSelected is set according to the True or False
            // value of aria-checked for 'radio' and 'menuitemradio' roles.
            Role::RadioButton | Role::MenuItemRadio => {
                self.node_state().checked_state() == Some(CheckedState::True)
            }
            // https://www.w3.org/TR/wai-aria-1.1/#aria-selected
            // SelectionItem.IsSelected is set according to the True or False
            // value of aria-selected.
            _ => self.node_state().is_selected().unwrap_or(false),
        }
    }

    fn is_text_pattern_supported(&self) -> bool {
        match self {
            Self::Node(node) => node.supports_text_ranges(),
            Self::DetachedNode(node) => node.supports_text_ranges(),
        }
    }

    pub(crate) fn enqueue_property_changes(
        &self,
        queue: &mut Vec<QueuedEvent>,
        element: &IRawElementProviderSimple,
        old: &NodeWrapper,
    ) {
        self.enqueue_simple_property_changes(queue, element, old);
        self.enqueue_pattern_property_changes(queue, element, old);
        self.enqueue_property_implied_events(queue, element, old);
    }

    fn enqueue_property_implied_events(
        &self,
        queue: &mut Vec<QueuedEvent>,
        element: &IRawElementProviderSimple,
        old: &NodeWrapper,
    ) {
        if self.is_selection_item_pattern_supported()
            && self.is_selected()
            && !(old.is_selection_item_pattern_supported() && old.is_selected())
        {
            queue.push(QueuedEvent::Simple {
                element: element.clone(),
                event_id: UIA_SelectionItem_ElementSelectedEventId,
            });
        }
        if self.is_text_pattern_supported()
            && old.is_text_pattern_supported()
            && self.node_state().raw_text_selection() != old.node_state().raw_text_selection()
        {
            queue.push(QueuedEvent::Simple {
                element: element.clone(),
                event_id: UIA_Text_TextSelectionChangedEventId,
            });
        }
    }

    fn enqueue_property_change(
        &self,
        queue: &mut Vec<QueuedEvent>,
        element: &IRawElementProviderSimple,
        property_id: UIA_PROPERTY_ID,
        old_value: VariantFactory,
        new_value: VariantFactory,
    ) {
        let old_value: VARIANT = old_value.into();
        let new_value: VARIANT = new_value.into();
        queue.push(QueuedEvent::PropertyChanged {
            element: element.clone(),
            property_id,
            old_value,
            new_value,
        });
    }
}

#[implement(
    IRawElementProviderSimple,
    IRawElementProviderFragment,
    IRawElementProviderFragmentRoot,
    IToggleProvider,
    IInvokeProvider,
    IValueProvider,
    IRangeValueProvider,
    ISelectionItemProvider,
    ITextProvider
)]
pub(crate) struct PlatformNode {
    pub(crate) context: Weak<Context>,
    pub(crate) node_id: NodeId,
}

impl PlatformNode {
    pub(crate) fn new(context: &Arc<Context>, node_id: NodeId) -> Self {
        Self {
            context: Arc::downgrade(context),
            node_id,
        }
    }

    fn upgrade_context(&self) -> Result<Arc<Context>> {
        upgrade(&self.context)
    }

    fn with_tree_state_and_context<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&TreeState, &Context) -> Result<T>,
    {
        let context = self.upgrade_context()?;
        let tree = context.read_tree();
        f(tree.state(), &context)
    }

    fn with_tree_state<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&TreeState) -> Result<T>,
    {
        self.with_tree_state_and_context(|state, _| f(state))
    }

    fn resolve_with_context<F, T>(&self, f: F) -> Result<T>
    where
        for<'a> F: FnOnce(Node<'a>, &Context) -> Result<T>,
    {
        self.with_tree_state_and_context(|state, context| {
            if let Some(node) = state.node_by_id(self.node_id) {
                f(node, context)
            } else {
                Err(element_not_available())
            }
        })
    }

    fn resolve<F, T>(&self, f: F) -> Result<T>
    where
        for<'a> F: FnOnce(Node<'a>) -> Result<T>,
    {
        self.resolve_with_context(|node, _| f(node))
    }

    fn resolve_with_context_for_text_pattern<F, T>(&self, f: F) -> Result<T>
    where
        for<'a> F: FnOnce(Node<'a>, &Context) -> Result<T>,
    {
        self.with_tree_state_and_context(|state, context| {
            if let Some(node) = state
                .node_by_id(self.node_id)
                .filter(Node::supports_text_ranges)
            {
                f(node, context)
            } else {
                Err(element_not_available())
            }
        })
    }

    fn resolve_for_text_pattern<F, T>(&self, f: F) -> Result<T>
    where
        for<'a> F: FnOnce(Node<'a>) -> Result<T>,
    {
        self.resolve_with_context_for_text_pattern(|node, _| f(node))
    }

    fn do_action<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce() -> ActionRequest,
    {
        let context = self.upgrade_context()?;
        let tree = context.read_tree();
        if tree.state().has_node(self.node_id) {
            drop(tree);
            let request = f();
            context.action_handler.do_action(request);
            Ok(())
        } else {
            Err(element_not_available())
        }
    }

    fn do_default_action(&self) -> Result<()> {
        self.do_action(|| ActionRequest {
            action: Action::Default,
            target: self.node_id,
            data: None,
        })
    }

    fn relative(&self, node_id: NodeId) -> Self {
        Self {
            context: self.context.clone(),
            node_id,
        }
    }
}

impl IRawElementProviderSimple_Impl for PlatformNode {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider)
    }

    fn GetPatternProvider(&self, pattern_id: UIA_PATTERN_ID) -> Result<IUnknown> {
        self.pattern_provider(pattern_id)
    }

    fn GetPropertyValue(&self, property_id: UIA_PROPERTY_ID) -> Result<VARIANT> {
        self.resolve_with_context(|node, context| {
            let wrapper = NodeWrapper::Node(&node);
            let mut result = wrapper.get_property_value(property_id);
            if result.is_empty() && node.is_root() {
                match property_id {
                    UIA_NamePropertyId => {
                        result = window_title(context.hwnd).into();
                    }
                    UIA_NativeWindowHandlePropertyId => {
                        result = (context.hwnd.0 as i32).into();
                    }
                    _ => (),
                }
            }
            Ok(result.into())
        })
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        self.with_tree_state_and_context(|state, context| {
            if self.node_id == state.root_id() {
                unsafe { UiaHostProviderFromHwnd(context.hwnd) }
            } else {
                Err(Error::OK)
            }
        })
    }
}

impl IRawElementProviderFragment_Impl for PlatformNode {
    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        self.resolve(|node| {
            let result = match direction {
                NavigateDirection_Parent => node.filtered_parent(&filter_with_root_exception),
                NavigateDirection_NextSibling => node.following_filtered_siblings(&filter).next(),
                NavigateDirection_PreviousSibling => {
                    node.preceding_filtered_siblings(&filter).next()
                }
                NavigateDirection_FirstChild => node.filtered_children(&filter).next(),
                NavigateDirection_LastChild => node.filtered_children(&filter).next_back(),
                _ => None,
            };
            match result {
                Some(result) => Ok(self.relative(result.id()).into()),
                None => Err(Error::OK),
            }
        })
    }

    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        let runtime_id = runtime_id_from_node_id(self.node_id);
        Ok(safe_array_from_i32_slice(&runtime_id))
    }

    fn BoundingRectangle(&self) -> Result<UiaRect> {
        self.resolve_with_context(|node, context| {
            let rect = node.bounding_box().map_or(UiaRect::default(), |rect| {
                let client_top_left = context.client_top_left();
                UiaRect {
                    left: rect.x0 + client_top_left.x,
                    top: rect.y0 + client_top_left.y,
                    width: rect.width(),
                    height: rect.height(),
                }
            });
            Ok(rect)
        })
    }

    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        Ok(std::ptr::null_mut())
    }

    fn SetFocus(&self) -> Result<()> {
        self.do_action(|| ActionRequest {
            action: Action::Focus,
            target: self.node_id,
            data: None,
        })
    }

    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        self.with_tree_state(|state| {
            let root_id = state.root_id();
            if root_id == self.node_id {
                // SAFETY: We know &self is inside a full COM implementation.
                unsafe { self.cast() }
            } else {
                Ok(self.relative(root_id).into())
            }
        })
    }
}

impl IRawElementProviderFragmentRoot_Impl for PlatformNode {
    fn ElementProviderFromPoint(&self, x: f64, y: f64) -> Result<IRawElementProviderFragment> {
        self.resolve_with_context(|node, context| {
            let client_top_left = context.client_top_left();
            let point = Point::new(x - client_top_left.x, y - client_top_left.y);
            let point = node.transform().inverse() * point;
            node.node_at_point(point, &filter).map_or_else(
                || Err(Error::OK),
                |node| Ok(self.relative(node.id()).into()),
            )
        })
    }

    fn GetFocus(&self) -> Result<IRawElementProviderFragment> {
        self.with_tree_state(|state| {
            if let Some(id) = state.focus_id() {
                if id != self.node_id {
                    return Ok(self.relative(id).into());
                }
            }
            Err(Error::OK)
        })
    }
}

macro_rules! properties {
    ($(($base_id:ident, $m:ident)),+) => {
        impl NodeWrapper<'_> {
            fn get_property_value(&self, property_id: UIA_PROPERTY_ID) -> VariantFactory {
                match property_id {
                    $(paste! { [< UIA_ $base_id PropertyId>] } => {
                        self.$m().into()
                    })*
                    _ => VariantFactory::empty()
                }
            }
            fn enqueue_simple_property_changes(
                &self,
                queue: &mut Vec<QueuedEvent>,
                element: &IRawElementProviderSimple,
                old: &NodeWrapper,
            ) {
                $({
                    let old_value = old.$m();
                    let new_value = self.$m();
                    if old_value != new_value {
                        self.enqueue_property_change(
                            queue,
                            element,
                            paste! { [<UIA_ $base_id PropertyId>] },
                            old_value.into(),
                            new_value.into(),
                        );
                    }
                })*
            }
        }
    };
}

macro_rules! patterns {
    ($(($base_pattern_id:ident, $is_supported:ident, (
        $(($base_property_id:ident, $getter:ident, $com_type:ident)),*
    ), (
        $($extra_trait_method:item),*
    ))),+) => {
        impl PlatformNode {
            fn pattern_provider(&self, pattern_id: UIA_PATTERN_ID) -> Result<IUnknown> {
                self.resolve(|node| {
                    let wrapper = NodeWrapper::Node(&node);
                    match pattern_id {
                        $(paste! { [< UIA_ $base_pattern_id PatternId>] } => {
                            if wrapper.$is_supported() {
                                // SAFETY: We know we're running inside a full COM implementation.
                                let intermediate: paste! { [< I $base_pattern_id Provider>] } =
                                    unsafe { self.cast() }?;
                                return intermediate.cast();
                            }
                        })*
                        _ => (),
                    }
                    Err(Error::OK)
                })
            }
        }
        impl NodeWrapper<'_> {
            fn enqueue_pattern_property_changes(
                &self,
                queue: &mut Vec<QueuedEvent>,
                element: &IRawElementProviderSimple,
                old: &NodeWrapper,
            ) {
                $(if self.$is_supported() && old.$is_supported() {
                    $({
                        let old_value = old.$getter();
                        let new_value = self.$getter();
                        if old_value != new_value {
                            self.enqueue_property_change(
                                queue,
                                element,
                                paste! { [<UIA_ $base_pattern_id $base_property_id PropertyId>] },
                                old_value.into(),
                                new_value.into(),
                            );
                        }
                    })*
                })*
            }
        }
        paste! {
            $(impl [< I $base_pattern_id Provider_Impl>] for PlatformNode {
                $(fn $base_property_id(&self) -> Result<$com_type> {
                    self.resolve(|node| {
                        let wrapper = NodeWrapper::Node(&node);
                        Ok(wrapper.$getter().into())
                    })
                })*
                $($extra_trait_method)*
            })*
        }
    };
}

properties! {
    (ControlType, control_type),
    (Name, name),
    (IsContentElement, is_content_element),
    (IsControlElement, is_content_element),
    (IsEnabled, is_enabled),
    (IsKeyboardFocusable, is_focusable),
    (HasKeyboardFocus, is_focused),
    (LiveSetting, live_setting)
}

patterns! {
    (Toggle, is_toggle_pattern_supported, (
        (ToggleState, toggle_state, ToggleState)
    ), (
        fn Toggle(&self) -> Result<()> {
            self.do_default_action()
        }
    )),
    (Invoke, is_invoke_pattern_supported, (), (
        fn Invoke(&self) -> Result<()> {
            self.do_default_action()
        }
    )),
    (Value, is_value_pattern_supported, (
        (Value, value, BSTR),
        (IsReadOnly, is_read_only, BOOL)
    ), (
        fn SetValue(&self, value: &PCWSTR) -> Result<()> {
            self.do_action(|| {
                let value = unsafe { value.to_string() }.unwrap();
                ActionRequest {
                    action: Action::SetValue,
                    target: self.node_id,
                    data: Some(ActionData::Value(value.into())),
                }
            })
        }
    )),
    (RangeValue, is_range_value_pattern_supported, (
        (Value, numeric_value, f64),
        (IsReadOnly, is_read_only, BOOL),
        (Minimum, min_numeric_value, f64),
        (Maximum, max_numeric_value, f64),
        (SmallChange, numeric_value_step, f64),
        (LargeChange, numeric_value_jump, f64)
    ), (
        fn SetValue(&self, value: f64) -> Result<()> {
            self.do_action(|| {
                ActionRequest {
                    action: Action::SetValue,
                    target: self.node_id,
                    data: Some(ActionData::NumericValue(value)),
                }
            })
        }
    )),
    (SelectionItem, is_selection_item_pattern_supported, (
        (IsSelected, is_selected, BOOL)
    ), (
        fn Select(&self) -> Result<()> {
            self.do_default_action()
        },

        fn AddToSelection(&self) -> Result<()> {
            // TODO: implement when we work on list boxes (#23)
            Err(not_implemented())
        },

        fn RemoveFromSelection(&self) -> Result<()> {
            // TODO: implement when we work on list boxes (#23)
            Err(not_implemented())
        },

        fn SelectionContainer(&self) -> Result<IRawElementProviderSimple> {
            // TODO: implement when we work on list boxes (#23)
            // We return E_FAIL here because that's what Chromium does
            // if it can't find a container.
            Err(Error::new(E_FAIL, "".into()))
        }
    )),
    (Text, is_text_pattern_supported, (), (
        fn GetSelection(&self) -> Result<*mut SAFEARRAY> {
            self.resolve_for_text_pattern(|node| {
                if let Some(range) = node.text_selection() {
                    let platform_range: ITextRangeProvider = PlatformTextRange::new(&self.context, range).into();
                    let iunknown: IUnknown = platform_range.cast()?;
                    Ok(safe_array_from_com_slice(&[iunknown]))
                } else {
                    Ok(std::ptr::null_mut())
                }
            })
        },

        fn GetVisibleRanges(&self) -> Result<*mut SAFEARRAY> {
            // TBD: Do we need this? The Quorum GUI toolkit, which is our
            // current point of comparison for text functionality,
            // doesn't implement it.
            Ok(std::ptr::null_mut())
        },

        fn RangeFromChild(&self, _child: Option<&IRawElementProviderSimple>) -> Result<ITextRangeProvider> {
            // We don't support embedded objects in text.
            Err(not_implemented())
        },

        fn RangeFromPoint(&self, point: &UiaPoint) -> Result<ITextRangeProvider> {
            self.resolve_with_context_for_text_pattern(|node, context| {
                let client_top_left = context.client_top_left();
                let point = Point::new(point.x - client_top_left.x, point.y - client_top_left.y);
                let point = node.transform().inverse() * point;
                let pos = node.text_position_at_point(point);
                let range = pos.to_degenerate_range();
                Ok(PlatformTextRange::new(&self.context, range).into())
            })
        },

        fn DocumentRange(&self) -> Result<ITextRangeProvider> {
            self.resolve_for_text_pattern(|node| {
                let range = node.document_range();
                Ok(PlatformTextRange::new(&self.context, range).into())
            })
        },

        fn SupportedTextSelection(&self) -> Result<SupportedTextSelection> {
            self.resolve_for_text_pattern(|node| {
                if node.has_text_selection() {
                    Ok(SupportedTextSelection_Single)
                } else {
                    Ok(SupportedTextSelection_None)
                }
            })
        }
    ))
}

// Ensures that `PlatformNode` is actually safe to use in the free-threaded
// manner that we advertise via `ProviderOptions`.
#[test]
fn platform_node_impl_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<PlatformNode>();
}
