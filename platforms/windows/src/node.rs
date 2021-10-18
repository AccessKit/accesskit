// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

#![allow(non_upper_case_globals)]

use accesskit_consumer::{Node, WeakNode};
use accesskit_schema::{NodeIdContent, Role};
use accesskit_windows_bindings::Windows::Win32::{
    Foundation::*, Graphics::Gdi::*, System::OleAutomation::*, UI::Accessibility::*,
};
use accesskit_windows_bindings::*;
use arrayvec::ArrayVec;
use windows::*;

use crate::util::*;

struct ResolvedPlatformNode<'a> {
    node: Node<'a>,
    hwnd: HWND,
}

impl ResolvedPlatformNode<'_> {
    fn new(node: Node, hwnd: HWND) -> ResolvedPlatformNode {
        ResolvedPlatformNode { node, hwnd }
    }

    fn relative<'a>(&self, node: Node<'a>) -> ResolvedPlatformNode<'a> {
        ResolvedPlatformNode::new(node, self.hwnd)
    }

    fn downgrade(&self) -> PlatformNode {
        PlatformNode::new(&self.node, self.hwnd)
    }

    fn provider_options(&self) -> ProviderOptions {
        ProviderOptions_ServerSideProvider
    }

    fn is_pattern_supported(&self, _pattern_id: i32) -> bool {
        // TODO: add patterns
        false
    }

    fn control_type(&self) -> i32 {
        let role = self.node.role();
        // TODO: Handle special cases.
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
                // TODO: handle combo-box special case.
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
                // as Chromium does.
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

    fn get_property_value(&self, property_id: i32) -> VARIANT {
        // TODO: add properties
        match property_id {
            UIA_ControlTypePropertyId => {
                return variant_from_i32(self.control_type());
            }
            UIA_NamePropertyId => {
                if let Some(name) = self.node.name() {
                    return variant_from_bstr(name.into());
                }
            }
            UIA_IsKeyboardFocusablePropertyId => {
                return variant_from_bool(self.node.is_focusable());
            }
            UIA_HasKeyboardFocusPropertyId => {
                return variant_from_bool(self.node.is_focused());
            }
            _ => (),
        }
        empty_variant()
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
            NavigateDirection_Parent => self.node.unignored_parent(),
            NavigateDirection_NextSibling => self.node.following_unignored_siblings().next(),
            NavigateDirection_PreviousSibling => self.node.preceding_unignored_siblings().next(),
            NavigateDirection_FirstChild => self.node.unignored_children().next(),
            NavigateDirection_LastChild => self.node.unignored_children().next_back(),
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
        self.node.bounds().map_or(UiaRect::default(), |rect| {
            let mut result = UiaRect {
                left: rect.left.into(),
                top: rect.top.into(),
                width: rect.width.into(),
                height: rect.height.into(),
            };
            let mut client_top_left = POINT::default();
            unsafe { ClientToScreen(self.hwnd, &mut client_top_left) }.unwrap();
            result.left += f64::from(client_top_left.x);
            result.top += f64::from(client_top_left.y);
            result
        })
    }

    fn set_focus(&self) {
        // TODO: request action
    }

    fn hit_test(&self, _x: f64, _y: f64) -> Option<ResolvedPlatformNode> {
        // TODO: Either request a hit test from the toolkit, or do our own.
        None
    }

    fn focus(&self) -> Option<ResolvedPlatformNode> {
        if let Some(node) = self.node.tree_reader.focus() {
            if node.id() != self.node.id() {
                return Some(self.relative(node));
            }
        }
        None
    }
}

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
            .unwrap_or_else(|| Err(Error::new(HRESULT(UIA_E_ELEMENTNOTAVAILABLE), "")))
    }

    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        // We don't currently have to resolve the node to implement this.
        // But we might have to in the future. So to avoid leaking
        // implementation details that might change, we'll resolve
        // the node and just ignore it. There's precedent for this;
        // Chromium's implementation of this method validates the node
        // even though the return value is hard-coded.
        self.resolve(|resolved| Ok(resolved.provider_options()))
    }

    fn GetPatternProvider(&mut self, pattern_id: i32) -> Result<IUnknown> {
        let supported = self.resolve(|resolved| Ok(resolved.is_pattern_supported(pattern_id)))?;
        if supported {
            let intermediate: IRawElementProviderSimple = self.into();
            Ok(intermediate.into())
        } else {
            Err(Error::OK)
        }
    }

    fn GetPropertyValue(&self, property_id: i32) -> Result<VARIANT> {
        self.resolve(|resolved| Ok(resolved.get_property_value(property_id)))
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        self.resolve(|resolved| resolved.host_provider())
    }

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

    fn FragmentRoot(&mut self) -> Result<IRawElementProviderFragmentRoot> {
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
            FragmentRootResult::This => Ok(self.into()),
            FragmentRootResult::Other(node) => Ok(node.into()),
        }
    }

    fn ElementProviderFromPoint(&self, x: f64, y: f64) -> Result<IRawElementProviderFragment> {
        self.resolve(|resolved| match resolved.hit_test(x, y) {
            Some(result) => Ok(result.downgrade().into()),
            None => Err(Error::OK),
        })
    }

    fn GetFocus(&self) -> Result<IRawElementProviderFragment> {
        self.resolve(|resolved| match resolved.focus() {
            Some(result) => Ok(result.downgrade().into()),
            None => Err(Error::OK),
        })
    }
}
