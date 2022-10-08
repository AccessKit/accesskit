// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2021 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#![allow(non_upper_case_globals)]

use accesskit::kurbo::Point;
use accesskit::{CheckedState, Live, NodeIdContent, Role};
use accesskit_consumer::{Node, WeakNode};
use arrayvec::ArrayVec;
use paste::paste;
use windows as Windows;
use windows::{
    core::*,
    Win32::{Foundation::*, Graphics::Gdi::*, System::Com::*, UI::Accessibility::*},
};

use crate::util::*;

pub(crate) struct ResolvedPlatformNode<'a> {
    node: Node<'a>,
    hwnd: HWND,
}

impl ResolvedPlatformNode<'_> {
    pub(crate) fn new(node: Node, hwnd: HWND) -> ResolvedPlatformNode {
        ResolvedPlatformNode { node, hwnd }
    }

    fn relative<'a>(&self, node: Node<'a>) -> ResolvedPlatformNode<'a> {
        ResolvedPlatformNode::new(node, self.hwnd)
    }

    pub(crate) fn downgrade(&self) -> PlatformNode {
        PlatformNode::new(&self.node, self.hwnd)
    }

    fn provider_options(&self) -> ProviderOptions {
        ProviderOptions_ServerSideProvider
    }

    fn control_type(&self) -> i32 {
        let role = self.node.role();
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
        self.node.name()
    }

    fn is_content_element(&self) -> bool {
        !self.node.is_invisible_or_ignored()
    }

    fn is_enabled(&self) -> bool {
        !self.node.is_disabled()
    }

    fn is_focusable(&self) -> bool {
        self.node.is_focusable()
    }

    fn is_focused(&self) -> bool {
        self.node.is_focused()
    }

    fn live_setting(&self) -> LiveSetting {
        match self.node.live() {
            Live::Off => Off,
            Live::Polite => Polite,
            Live::Assertive => Assertive,
        }
    }

    fn is_toggle_pattern_supported(&self) -> bool {
        self.node.checked_state().is_some() && !self.is_selection_item_pattern_supported()
    }

    fn toggle_state(&self) -> ToggleState {
        match self.node.checked_state().unwrap() {
            CheckedState::False => ToggleState_Off,
            CheckedState::True => ToggleState_On,
            CheckedState::Mixed => ToggleState_Indeterminate,
        }
    }

    fn is_invoke_pattern_supported(&self) -> bool {
        self.node.is_invocable()
    }

    fn is_value_pattern_supported(&self) -> bool {
        self.node.value().is_some()
    }

    fn is_range_value_pattern_supported(&self) -> bool {
        self.node.numeric_value().is_some()
    }

    fn value(&self) -> &str {
        self.node.value().unwrap()
    }

    fn is_read_only(&self) -> bool {
        self.node.is_read_only()
    }

    fn numeric_value(&self) -> f64 {
        self.node.numeric_value().unwrap()
    }

    fn min_numeric_value(&self) -> f64 {
        self.node.min_numeric_value().unwrap_or(std::f64::MIN)
    }

    fn max_numeric_value(&self) -> f64 {
        self.node.max_numeric_value().unwrap_or(std::f64::MAX)
    }

    fn numeric_value_step(&self) -> f64 {
        self.node.numeric_value_step().unwrap_or(0.0)
    }

    fn numeric_value_jump(&self) -> f64 {
        self.node
            .numeric_value_jump()
            .unwrap_or_else(|| self.numeric_value_step())
    }

    fn is_selection_item_pattern_supported(&self) -> bool {
        match self.node.role() {
            // TODO: tables (#29)
            // https://www.w3.org/TR/core-aam-1.1/#mapping_state-property_table
            // SelectionItem.IsSelected is exposed when aria-checked is True or
            // False, for 'radio' and 'menuitemradio' roles.
            Role::RadioButton | Role::MenuItemRadio => matches!(
                self.node.checked_state(),
                Some(CheckedState::True | CheckedState::False)
            ),
            // https://www.w3.org/TR/wai-aria-1.1/#aria-selected
            // SelectionItem.IsSelected is exposed when aria-select is True or False.
            Role::ListBoxOption
            | Role::ListItem
            | Role::MenuListOption
            | Role::Tab
            | Role::TreeItem => self.node.is_selected().is_some(),
            _ => false,
        }
    }

    fn select(&self) {
        self.node.do_default_action()
    }

    fn add_to_selection(&self) {
        // TODO: implement when we work on list boxes (#23)
    }

    fn remove_from_selection(&self) {
        // TODO: implement when we work on list boxes (#23)
    }

    fn is_selected(&self) -> bool {
        match self.node.role() {
            // https://www.w3.org/TR/core-aam-1.1/#mapping_state-property_table
            // SelectionItem.IsSelected is set according to the True or False
            // value of aria-checked for 'radio' and 'menuitemradio' roles.
            Role::RadioButton | Role::MenuItemRadio => {
                self.node.checked_state() == Some(CheckedState::True)
            }
            // https://www.w3.org/TR/wai-aria-1.1/#aria-selected
            // SelectionItem.IsSelected is set according to the True or False
            // value of aria-selected.
            _ => self.node.is_selected().unwrap_or(false),
        }
    }

    pub(crate) fn enqueue_property_changes(
        &self,
        queue: &mut Vec<QueuedEvent>,
        old: &ResolvedPlatformNode,
    ) {
        self.enqueue_simple_property_changes(queue, old);
        self.enqueue_pattern_property_changes(queue, old);
        self.enqueue_property_implied_events(queue, old);
    }

    fn enqueue_property_implied_events(
        &self,
        queue: &mut Vec<QueuedEvent>,
        old: &ResolvedPlatformNode,
    ) {
        if self.is_selection_item_pattern_supported()
            && self.is_selected()
            && !(old.is_selection_item_pattern_supported() && old.is_selected())
        {
            let element: IRawElementProviderSimple = self.downgrade().into();
            queue.push(QueuedEvent::Simple {
                element,
                event_id: UIA_SelectionItem_ElementSelectedEventId,
            });
        }
    }

    fn enqueue_property_change(
        &self,
        queue: &mut Vec<QueuedEvent>,
        property_id: i32,
        old_value: VariantFactory,
        new_value: VariantFactory,
    ) {
        let element: IRawElementProviderSimple = self.downgrade().into();
        let old_value: VARIANT = old_value.into();
        let new_value: VARIANT = new_value.into();
        queue.push(QueuedEvent::PropertyChanged {
            element,
            property_id,
            old_value,
            new_value,
        });
    }

    fn host_provider(&self) -> Result<IRawElementProviderSimple> {
        if self.node.is_root() {
            unsafe { UiaHostProviderFromHwnd(self.hwnd) }
        } else {
            Err(Error::OK)
        }
    }

    fn navigate(&self, direction: NavigateDirection) -> Option<ResolvedPlatformNode> {
        let result = match direction {
            NavigateDirection_Parent => self.node.parent(),
            NavigateDirection_NextSibling => self.node.following_siblings().next(),
            NavigateDirection_PreviousSibling => self.node.preceding_siblings().next(),
            NavigateDirection_FirstChild => self.node.children().next(),
            NavigateDirection_LastChild => self.node.children().next_back(),
            _ => None,
        };
        result.map(|node| self.relative(node))
    }

    fn runtime_id(&self) -> impl std::ops::Deref<Target = [i32]> {
        let mut result = ArrayVec::<i32, { std::mem::size_of::<NodeIdContent>() + 1 }>::new();
        result.push(UiaAppendRuntimeId as i32);
        let id = self.node.id().0;
        let id_bytes = id.get().to_be_bytes();
        let start_index: usize = (id.leading_zeros() / 8) as usize;
        for byte in &id_bytes[start_index..] {
            result.push((*byte).into());
        }
        result
    }

    fn bounding_rectangle(&self) -> UiaRect {
        self.node.bounding_box().map_or(UiaRect::default(), |rect| {
            let mut client_top_left = POINT::default();
            unsafe { ClientToScreen(self.hwnd, &mut client_top_left) }.unwrap();
            UiaRect {
                left: rect.x0 + f64::from(client_top_left.x),
                top: rect.y0 + f64::from(client_top_left.y),
                width: rect.width(),
                height: rect.height(),
            }
        })
    }

    fn set_focus(&self) {
        self.node.set_focus()
    }

    fn node_at_point(&self, point: Point) -> Option<ResolvedPlatformNode> {
        let mut client_top_left = POINT::default();
        unsafe { ClientToScreen(self.hwnd, &mut client_top_left) }.unwrap();
        let point = self.node.transform().inverse()
            * Point {
                x: point.x - f64::from(client_top_left.x),
                y: point.y - f64::from(client_top_left.y),
            };
        self.node
            .node_at_point(point)
            .map(|node| self.relative(node))
    }

    fn focus(&self) -> Option<ResolvedPlatformNode> {
        if let Some(node) = self.node.tree_reader.focus() {
            if node.id() != self.node.id() {
                return Some(self.relative(node));
            }
        }
        None
    }

    fn toggle(&self) {
        self.node.do_default_action()
    }

    fn invoke(&self) {
        self.node.do_default_action()
    }

    fn set_value(&self, value: impl Into<Box<str>>) {
        self.node.set_value(value)
    }

    fn set_numeric_value(&self, value: f64) {
        self.node.set_numeric_value(value)
    }
}

#[derive(Clone)]
#[implement(
    Windows::Win32::UI::Accessibility::IRawElementProviderSimple,
    Windows::Win32::UI::Accessibility::IRawElementProviderFragment,
    Windows::Win32::UI::Accessibility::IRawElementProviderFragmentRoot
)]
pub(crate) struct PlatformNode {
    node: WeakNode,
    hwnd: HWND,
}

#[allow(non_snake_case)]
impl PlatformNode {
    pub(crate) fn new(node: &Node, hwnd: HWND) -> Self {
        Self {
            node: node.downgrade(),
            hwnd,
        }
    }

    fn resolve<F, T>(&self, f: F) -> Result<T>
    where
        for<'a> F: FnOnce(ResolvedPlatformNode<'a>) -> Result<T>,
    {
        self.node
            .map(|node| f(ResolvedPlatformNode::new(node, self.hwnd)))
            .unwrap_or_else(|| {
                Err(Error::new(
                    HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32),
                    "".into(),
                ))
            })
    }
}

impl IRawElementProviderSimple_Impl for PlatformNode {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        // We don't currently have to resolve the node to implement this.
        // But we might have to in the future. So to avoid leaking
        // implementation details that might change, we'll resolve
        // the node and just ignore it. There's precedent for this;
        // Chromium's implementation of this method validates the node
        // even though the return value is hard-coded.
        self.resolve(|resolved| Ok(resolved.provider_options()))
    }

    fn GetPatternProvider(&self, pattern_id: i32) -> Result<IUnknown> {
        let provider = self.resolve(|resolved| Ok(resolved.pattern_provider(pattern_id)))?;
        provider.map_or_else(|| Err(Error::OK), Ok)
    }

    fn GetPropertyValue(&self, property_id: i32) -> Result<VARIANT> {
        self.resolve(|resolved| {
            let result = resolved.get_property_value(property_id);
            Ok(result.into())
        })
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        self.resolve(|resolved| resolved.host_provider())
    }
}

impl IRawElementProviderFragment_Impl for PlatformNode {
    fn Navigate(&self, direction: NavigateDirection) -> Result<IRawElementProviderFragment> {
        self.resolve(|resolved| match resolved.navigate(direction) {
            Some(result) => Ok(result.downgrade().into()),
            None => Err(Error::OK),
        })
    }

    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        self.resolve(|resolved| {
            let runtime_id = resolved.runtime_id();
            Ok(safe_array_from_i32_slice(&runtime_id))
        })
    }

    fn BoundingRectangle(&self) -> Result<UiaRect> {
        self.resolve(|resolved| Ok(resolved.bounding_rectangle()))
    }

    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        // As with ProviderOptions above, avoid leaking implementation details.
        self.resolve(|_resolved| Ok(std::ptr::null_mut()))
    }

    fn SetFocus(&self) -> Result<()> {
        self.resolve(|resolved| {
            resolved.set_focus();
            Ok(())
        })
    }

    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        enum FragmentRootResult {
            This,
            Other(PlatformNode),
        }
        let result = self.resolve(|resolved| {
            if resolved.node.is_root() {
                Ok(FragmentRootResult::This)
            } else {
                let root = resolved.node.tree_reader.root();
                Ok(FragmentRootResult::Other(
                    resolved.relative(root).downgrade(),
                ))
            }
        })?;
        match result {
            FragmentRootResult::This => Ok(self.clone().into()),
            FragmentRootResult::Other(node) => Ok(node.into()),
        }
    }
}

impl IRawElementProviderFragmentRoot_Impl for PlatformNode {
    fn ElementProviderFromPoint(&self, x: f64, y: f64) -> Result<IRawElementProviderFragment> {
        self.resolve(|resolved| {
            let point = Point::new(x, y);
            resolved
                .node_at_point(point)
                .map_or_else(|| Err(Error::OK), |node| Ok(node.downgrade().into()))
        })
    }

    fn GetFocus(&self) -> Result<IRawElementProviderFragment> {
        self.resolve(|resolved| match resolved.focus() {
            Some(result) => Ok(result.downgrade().into()),
            None => Err(Error::OK),
        })
    }
}

macro_rules! properties {
    ($(($base_id:ident, $m:ident)),+) => {
        impl ResolvedPlatformNode<'_> {
            fn get_property_value(&self, property_id: i32) -> VariantFactory {
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
                old: &ResolvedPlatformNode,
            ) {
                $({
                    let old_value = old.$m();
                    let new_value = self.$m();
                    if old_value != new_value {
                        self.enqueue_property_change(
                            queue,
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
        impl ResolvedPlatformNode<'_> {
            fn pattern_provider(&self, pattern_id: i32) -> Option<IUnknown> {
                match pattern_id {
                    $(paste! { [< UIA_ $base_pattern_id PatternId>] } => {
                        self.$is_supported().then(|| {
                            let intermediate: paste! { [< I $base_pattern_id Provider>] } =
                                (paste! { [< $base_pattern_id Provider>] })(self.downgrade()).into();
                            intermediate.into()
                        })
                    })*
                    _ => None,
                }
            }
            fn enqueue_pattern_property_changes(
                &self,
                queue: &mut Vec<QueuedEvent>,
                old: &ResolvedPlatformNode,
            ) {
                $(if self.$is_supported() && old.$is_supported() {
                    $({
                        let old_value = old.$getter();
                        let new_value = self.$getter();
                        if old_value != new_value {
                            self.enqueue_property_change(
                                queue,
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
            $(#[allow(non_snake_case)]
            impl [< I $base_pattern_id Provider_Impl>] for [< $base_pattern_id Provider>] {
                $(fn $base_property_id(&self) -> Result<$com_type> {
                    self.0.resolve(|resolved| Ok(resolved.$getter().into()))
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

#[implement(Windows::Win32::UI::Accessibility::IToggleProvider)]
struct ToggleProvider(PlatformNode);

#[implement(Windows::Win32::UI::Accessibility::IInvokeProvider)]
struct InvokeProvider(PlatformNode);

#[implement(Windows::Win32::UI::Accessibility::IValueProvider)]
struct ValueProvider(PlatformNode);

#[implement(Windows::Win32::UI::Accessibility::IRangeValueProvider)]
struct RangeValueProvider(PlatformNode);

#[implement(Windows::Win32::UI::Accessibility::ISelectionItemProvider)]
struct SelectionItemProvider(PlatformNode);

patterns! {
    (Toggle, is_toggle_pattern_supported, (
        (ToggleState, toggle_state, ToggleState)
    ), (
        fn Toggle(&self) -> Result<()> {
            self.0.resolve(|resolved| {
                resolved.toggle();
                Ok(())
            })
        }
    )),
    (Invoke, is_invoke_pattern_supported, (), (
        fn Invoke(&self) -> Result<()> {
            self.0.resolve(|resolved| {
                resolved.invoke();
                Ok(())
            })
        }
    )),
    (Value, is_value_pattern_supported, (
        (Value, value, BSTR),
        (IsReadOnly, is_read_only, BOOL)
    ), (
        fn SetValue(&self, value: &PCWSTR) -> Result<()> {
            self.0.resolve(|resolved| {
                let value = unsafe { value.to_string() }.unwrap();
                resolved.set_value(value);
                Ok(())
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
            self.0.resolve(|resolved| {
                resolved.set_numeric_value(value);
                Ok(())
            })
        }
    )),
    (SelectionItem, is_selection_item_pattern_supported, (
        (IsSelected, is_selected, BOOL)
    ), (
        fn Select(&self) -> Result<()> {
            self.0.resolve(|resolved| {
                resolved.select();
                Ok(())
            })
        },

        fn AddToSelection(&self) -> Result<()> {
            self.0.resolve(|resolved| {
                resolved.add_to_selection();
                Ok(())
            })
        },

        fn RemoveFromSelection(&self) -> Result<()> {
            self.0.resolve(|resolved| {
                resolved.remove_from_selection();
                Ok(())
            })
        },

        fn SelectionContainer(&self) -> Result<IRawElementProviderSimple> {
            self.0.resolve(|_resolved| {
                // TODO: implement when we work on list boxes (#23)
                // We return E_FAIL here because that's what Chromium does
                // if it can't find a container.
                Err(Error::new(E_FAIL, "".into()))
            })
        }
    ))
}
