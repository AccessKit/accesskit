// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#![allow(non_upper_case_globals)]

use accesskit::{Action, ActionData, ActionRequest, Checked, NodeId, Role, TextSelection};
use accesskit_consumer::{DetachedNode, FilterResult, Node, NodeState};
use icrate::{
    AppKit::*,
    Foundation::{
        ns_string, NSArray, NSCopying, NSInteger, NSNumber, NSObject, NSPoint, NSRange, NSRect,
        NSString,
    },
};
use objc2::{
    declare_class, msg_send_id,
    mutability::InteriorMutable,
    rc::Id,
    runtime::{AnyObject, Sel},
    sel, ClassType, DeclaredClass,
};
use std::rc::{Rc, Weak};

use crate::{context::Context, filters::filter, util::*};

fn ns_role(node_state: &NodeState) -> &'static NSAccessibilityRole {
    let role = node_state.role();
    // TODO: Handle special cases.
    unsafe {
        match role {
            Role::Unknown => NSAccessibilityUnknownRole,
            Role::InlineTextBox => NSAccessibilityUnknownRole,
            Role::Cell => NSAccessibilityCellRole,
            Role::StaticText => NSAccessibilityStaticTextRole,
            Role::Image => NSAccessibilityImageRole,
            Role::Link => NSAccessibilityLinkRole,
            Role::Row => NSAccessibilityRowRole,
            Role::ListItem => NSAccessibilityGroupRole,
            Role::ListMarker => ns_string!("AXListMarker"),
            Role::TreeItem => NSAccessibilityRowRole,
            Role::ListBoxOption => NSAccessibilityStaticTextRole,
            Role::MenuItem => NSAccessibilityMenuItemRole,
            Role::MenuListOption => NSAccessibilityMenuItemRole,
            Role::Paragraph => NSAccessibilityGroupRole,
            Role::GenericContainer => NSAccessibilityUnknownRole,
            Role::CheckBox => NSAccessibilityCheckBoxRole,
            Role::RadioButton => NSAccessibilityRadioButtonRole,
            Role::TextInput
            | Role::SearchInput
            | Role::EmailInput
            | Role::NumberInput
            | Role::PasswordInput
            | Role::PhoneNumberInput
            | Role::UrlInput => NSAccessibilityTextFieldRole,
            Role::Button | Role::DefaultButton => NSAccessibilityButtonRole,
            Role::Pane => NSAccessibilityUnknownRole,
            Role::RowHeader => NSAccessibilityCellRole,
            Role::ColumnHeader => NSAccessibilityCellRole,
            Role::Column => NSAccessibilityColumnRole,
            Role::RowGroup => NSAccessibilityGroupRole,
            Role::List => NSAccessibilityListRole,
            Role::Table => NSAccessibilityTableRole,
            Role::TableHeaderContainer => NSAccessibilityGroupRole,
            Role::LayoutTableCell => NSAccessibilityGroupRole,
            Role::LayoutTableRow => NSAccessibilityGroupRole,
            Role::LayoutTable => NSAccessibilityGroupRole,
            Role::Switch => NSAccessibilityCheckBoxRole,
            Role::ToggleButton => NSAccessibilityCheckBoxRole,
            Role::Menu => NSAccessibilityMenuRole,
            Role::MultilineTextInput => NSAccessibilityTextAreaRole,
            Role::DateInput | Role::DateTimeInput | Role::WeekInput | Role::MonthInput => {
                ns_string!("AXDateField")
            }
            Role::TimeInput => ns_string!("AXTimeField"),
            Role::Abbr => NSAccessibilityGroupRole,
            Role::Alert => NSAccessibilityGroupRole,
            Role::AlertDialog => NSAccessibilityGroupRole,
            Role::Application => NSAccessibilityGroupRole,
            Role::Article => NSAccessibilityGroupRole,
            Role::Audio => NSAccessibilityGroupRole,
            Role::Banner => NSAccessibilityGroupRole,
            Role::Blockquote => NSAccessibilityGroupRole,
            Role::Canvas => NSAccessibilityImageRole,
            Role::Caption => NSAccessibilityGroupRole,
            Role::Caret => NSAccessibilityUnknownRole,
            Role::Code => NSAccessibilityGroupRole,
            Role::ColorWell => NSAccessibilityColorWellRole,
            Role::ComboBox => NSAccessibilityPopUpButtonRole,
            Role::EditableComboBox => NSAccessibilityComboBoxRole,
            Role::Complementary => NSAccessibilityGroupRole,
            Role::Comment => NSAccessibilityGroupRole,
            Role::ContentDeletion => NSAccessibilityGroupRole,
            Role::ContentInsertion => NSAccessibilityGroupRole,
            Role::ContentInfo => NSAccessibilityGroupRole,
            Role::Definition => NSAccessibilityGroupRole,
            Role::DescriptionList => NSAccessibilityListRole,
            Role::DescriptionListDetail => NSAccessibilityGroupRole,
            Role::DescriptionListTerm => NSAccessibilityGroupRole,
            Role::Details => NSAccessibilityGroupRole,
            Role::Dialog => NSAccessibilityGroupRole,
            Role::Directory => NSAccessibilityListRole,
            Role::DisclosureTriangle => NSAccessibilityButtonRole,
            Role::Document => NSAccessibilityGroupRole,
            Role::EmbeddedObject => NSAccessibilityGroupRole,
            Role::Emphasis => NSAccessibilityGroupRole,
            Role::Feed => NSAccessibilityUnknownRole,
            Role::FigureCaption => NSAccessibilityGroupRole,
            Role::Figure => NSAccessibilityGroupRole,
            Role::Footer => NSAccessibilityGroupRole,
            Role::FooterAsNonLandmark => NSAccessibilityGroupRole,
            Role::Form => NSAccessibilityGroupRole,
            Role::Grid => NSAccessibilityTableRole,
            Role::Group => NSAccessibilityGroupRole,
            Role::Header => NSAccessibilityGroupRole,
            Role::HeaderAsNonLandmark => NSAccessibilityGroupRole,
            Role::Heading => ns_string!("Heading"),
            Role::Iframe => NSAccessibilityGroupRole,
            Role::IframePresentational => NSAccessibilityGroupRole,
            Role::ImeCandidate => NSAccessibilityUnknownRole,
            Role::Keyboard => NSAccessibilityUnknownRole,
            Role::Legend => NSAccessibilityGroupRole,
            Role::LineBreak => NSAccessibilityGroupRole,
            Role::ListBox => NSAccessibilityListRole,
            Role::Log => NSAccessibilityGroupRole,
            Role::Main => NSAccessibilityGroupRole,
            Role::Mark => NSAccessibilityGroupRole,
            Role::Marquee => NSAccessibilityGroupRole,
            Role::Math => NSAccessibilityGroupRole,
            Role::MenuBar => NSAccessibilityMenuBarRole,
            Role::MenuItemCheckBox => NSAccessibilityMenuItemRole,
            Role::MenuItemRadio => NSAccessibilityMenuItemRole,
            Role::MenuListPopup => NSAccessibilityMenuRole,
            Role::Meter => NSAccessibilityLevelIndicatorRole,
            Role::Navigation => NSAccessibilityGroupRole,
            Role::Note => NSAccessibilityGroupRole,
            Role::PluginObject => NSAccessibilityGroupRole,
            Role::Portal => NSAccessibilityButtonRole,
            Role::Pre => NSAccessibilityGroupRole,
            Role::ProgressIndicator => NSAccessibilityProgressIndicatorRole,
            Role::RadioGroup => NSAccessibilityRadioGroupRole,
            Role::Region => NSAccessibilityGroupRole,
            Role::RootWebArea => ns_string!("AXWebArea"),
            Role::Ruby => NSAccessibilityGroupRole,
            Role::RubyAnnotation => NSAccessibilityUnknownRole,
            Role::ScrollBar => NSAccessibilityScrollBarRole,
            Role::ScrollView => NSAccessibilityUnknownRole,
            Role::Search => NSAccessibilityGroupRole,
            Role::Section => NSAccessibilityGroupRole,
            Role::Slider => NSAccessibilitySliderRole,
            Role::SpinButton => NSAccessibilityIncrementorRole,
            Role::Splitter => NSAccessibilitySplitterRole,
            Role::Status => NSAccessibilityGroupRole,
            Role::Strong => NSAccessibilityGroupRole,
            Role::Suggestion => NSAccessibilityGroupRole,
            Role::SvgRoot => NSAccessibilityGroupRole,
            Role::Tab => NSAccessibilityRadioButtonRole,
            Role::TabList => NSAccessibilityTabGroupRole,
            Role::TabPanel => NSAccessibilityGroupRole,
            Role::Term => NSAccessibilityGroupRole,
            Role::Time => NSAccessibilityGroupRole,
            Role::Timer => NSAccessibilityGroupRole,
            Role::TitleBar => NSAccessibilityStaticTextRole,
            Role::Toolbar => NSAccessibilityToolbarRole,
            Role::Tooltip => NSAccessibilityGroupRole,
            Role::Tree => NSAccessibilityOutlineRole,
            Role::TreeGrid => NSAccessibilityTableRole,
            Role::Video => NSAccessibilityGroupRole,
            Role::WebView => NSAccessibilityUnknownRole,
            // Use the group role for Role::Window, since the NSWindow
            // provides the top-level accessibility object for the window.
            Role::Window => NSAccessibilityGroupRole,
            Role::PdfActionableHighlight => NSAccessibilityButtonRole,
            Role::PdfRoot => NSAccessibilityGroupRole,
            Role::GraphicsDocument => NSAccessibilityGroupRole,
            Role::GraphicsObject => NSAccessibilityGroupRole,
            Role::GraphicsSymbol => NSAccessibilityImageRole,
            Role::DocAbstract => NSAccessibilityGroupRole,
            Role::DocAcknowledgements => NSAccessibilityGroupRole,
            Role::DocAfterword => NSAccessibilityGroupRole,
            Role::DocAppendix => NSAccessibilityGroupRole,
            Role::DocBackLink => NSAccessibilityLinkRole,
            Role::DocBiblioEntry => NSAccessibilityGroupRole,
            Role::DocBibliography => NSAccessibilityGroupRole,
            Role::DocBiblioRef => NSAccessibilityGroupRole,
            Role::DocChapter => NSAccessibilityGroupRole,
            Role::DocColophon => NSAccessibilityGroupRole,
            Role::DocConclusion => NSAccessibilityGroupRole,
            Role::DocCover => NSAccessibilityImageRole,
            Role::DocCredit => NSAccessibilityGroupRole,
            Role::DocCredits => NSAccessibilityGroupRole,
            Role::DocDedication => NSAccessibilityGroupRole,
            Role::DocEndnote => NSAccessibilityGroupRole,
            Role::DocEndnotes => NSAccessibilityGroupRole,
            Role::DocEpigraph => NSAccessibilityGroupRole,
            Role::DocEpilogue => NSAccessibilityGroupRole,
            Role::DocErrata => NSAccessibilityGroupRole,
            Role::DocExample => NSAccessibilityGroupRole,
            Role::DocFootnote => NSAccessibilityGroupRole,
            Role::DocForeword => NSAccessibilityGroupRole,
            Role::DocGlossary => NSAccessibilityGroupRole,
            Role::DocGlossRef => NSAccessibilityLinkRole,
            Role::DocIndex => NSAccessibilityGroupRole,
            Role::DocIntroduction => NSAccessibilityGroupRole,
            Role::DocNoteRef => NSAccessibilityLinkRole,
            Role::DocNotice => NSAccessibilityGroupRole,
            Role::DocPageBreak => NSAccessibilitySplitterRole,
            Role::DocPageFooter => NSAccessibilityGroupRole,
            Role::DocPageHeader => NSAccessibilityGroupRole,
            Role::DocPageList => NSAccessibilityGroupRole,
            Role::DocPart => NSAccessibilityGroupRole,
            Role::DocPreface => NSAccessibilityGroupRole,
            Role::DocPrologue => NSAccessibilityGroupRole,
            Role::DocPullquote => NSAccessibilityGroupRole,
            Role::DocQna => NSAccessibilityGroupRole,
            Role::DocSubtitle => ns_string!("AXHeading"),
            Role::DocTip => NSAccessibilityGroupRole,
            Role::DocToc => NSAccessibilityGroupRole,
            Role::ListGrid => NSAccessibilityUnknownRole,
            Role::Terminal => NSAccessibilityTextAreaRole,
        }
    }
}

pub(crate) fn can_be_focused(node: &Node) -> bool {
    filter(node) == FilterResult::Include && node.role() != Role::Window
}

#[derive(PartialEq)]
pub(crate) enum Value {
    Bool(bool),
    Number(f64),
    String(String),
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

    fn is_root(&self) -> bool {
        match self {
            Self::Node(node) => node.is_root(),
            Self::DetachedNode(node) => node.is_root(),
        }
    }

    fn name(&self) -> Option<String> {
        if self.is_root() && self.node_state().role() == Role::Window {
            // If the group element that we expose for the top-level window
            // includes a title, VoiceOver behavior is broken.
            return None;
        }
        match self {
            Self::Node(node) => node.name(),
            Self::DetachedNode(node) => node.name(),
        }
    }

    fn node_value(&self) -> Option<String> {
        match self {
            Self::Node(node) => node.value(),
            Self::DetachedNode(node) => node.value(),
        }
    }

    // TODO: implement proper logic for title, description, and value;
    // see Chromium's content/browser/accessibility/browser_accessibility_cocoa.mm
    // and figure out how this is different in the macOS 10.10+ protocol

    pub(crate) fn title(&self) -> Option<String> {
        let state = self.node_state();
        if state.role() == Role::StaticText && state.raw_value().is_none() {
            // In this case, macOS wants the text to be the value, not title.
            return None;
        }
        self.name()
    }

    pub(crate) fn value(&self) -> Option<Value> {
        let state = self.node_state();
        if let Some(checked) = state.checked() {
            return Some(Value::Bool(checked != Checked::False));
        }
        if let Some(value) = self.node_value() {
            return Some(Value::String(value));
        }
        if let Some(value) = state.numeric_value() {
            return Some(Value::Number(value));
        }
        if state.role() == Role::StaticText {
            if let Some(name) = self.name() {
                return Some(Value::String(name));
            }
        }
        None
    }

    pub(crate) fn supports_text_ranges(&self) -> bool {
        match self {
            Self::Node(node) => node.supports_text_ranges(),
            Self::DetachedNode(node) => node.supports_text_ranges(),
        }
    }

    pub(crate) fn raw_text_selection(&self) -> Option<&TextSelection> {
        self.node_state().raw_text_selection()
    }
}

pub(crate) struct PlatformNodeIvars {
    context: Weak<Context>,
    node_id: NodeId,
}

declare_class!(
    pub(crate) struct PlatformNode;

    unsafe impl ClassType for PlatformNode {
        #[inherits(NSObject)]
        type Super = NSAccessibilityElement;
        type Mutability = InteriorMutable;
        const NAME: &'static str = "AccessKitNode";
    }

    impl DeclaredClass for PlatformNode {
        type Ivars = PlatformNodeIvars;
    }

    unsafe impl PlatformNode {
        #[method_id(accessibilityParent)]
        fn parent(&self) -> Option<Id<AnyObject>> {
            self.resolve_with_context(|node, context| {
                if let Some(parent) = node.filtered_parent(&filter) {
                    Some(Id::into_super(Id::into_super(Id::into_super(context.get_or_create_platform_node(parent.id())))))
                } else {
                    context
                        .view
                        .load()
                        .and_then(|view| unsafe { NSAccessibility::accessibilityParent(&*view) })
                }
            })
            .flatten()
        }

        #[method_id(accessibilityChildren)]
        fn children(&self) -> Option<Id<NSArray<PlatformNode>>> {
            self.children_internal()
        }

        #[method_id(accessibilityChildrenInNavigationOrder)]
        fn children_in_navigation_order(&self) -> Option<Id<NSArray<PlatformNode>>> {
            // For now, we assume the children are in navigation order.
            self.children_internal()
        }

        #[method(accessibilityFrame)]
        fn frame(&self) -> NSRect {
            self.resolve_with_context(|node, context| {
                let view = match context.view.load() {
                    Some(view) => view,
                    None => {
                        return NSRect::ZERO;
                    }
                };

                node.bounding_box().map_or_else(
                    || {
                        if node.is_root() {
                            unsafe { NSAccessibility::accessibilityFrame(&*view) }
                        } else {
                            NSRect::ZERO
                        }
                    },
                    |rect| to_ns_rect(&view, rect),
                )
            })
            .unwrap_or(NSRect::ZERO)
        }

        #[method_id(accessibilityRole)]
        fn role(&self) -> Id<NSAccessibilityRole> {
            self.resolve(|node| ns_role(node.state()))
                .unwrap_or(unsafe { NSAccessibilityUnknownRole })
                .copy()
        }

        #[method_id(accessibilityRoleDescription)]
        fn role_description(&self) -> Option<Id<NSString>> {
            self.resolve(|node| {
                if let Some(role_description) = node.role_description() {
                    Some(NSString::from_str(&role_description))
                } else {
                    unsafe { msg_send_id![super(self), accessibilityRoleDescription] }
                }
            })
            .flatten()
        }

        #[method_id(accessibilityTitle)]
        fn title(&self) -> Option<Id<NSString>> {
            self.resolve(|node| {
                let wrapper = NodeWrapper::Node(node);
                wrapper.title().map(|title| NSString::from_str(&title))
            })
            .flatten()
        }

        #[method_id(accessibilityValue)]
        fn value(&self) -> Option<Id<NSObject>> {
            self.resolve(|node| {
                let wrapper = NodeWrapper::Node(node);
                wrapper.value().map(|value| match value {
                    Value::Bool(value) => {
                        Id::into_super(Id::into_super(NSNumber::new_bool(value)))
                    }
                    Value::Number(value) => {
                        Id::into_super(Id::into_super(NSNumber::new_f64(value)))
                    }
                    Value::String(value) => {
                        Id::into_super(NSString::from_str(&value))
                    }
                })
            })
            .flatten()
        }

        #[method(setAccessibilityValue:)]
        fn set_value(&self, _value: &NSObject) {
            // This isn't yet implemented. See the comment on this selector
            // in `is_selector_allowed`.
        }

        #[method_id(accessibilityMinValue)]
        fn min_value(&self) -> Option<Id<NSNumber>> {
            self.resolve(|node| {
                node.min_numeric_value().map(NSNumber::new_f64)
            })
            .flatten()
        }

        #[method_id(accessibilityMaxValue)]
        fn max_value(&self) -> Option<Id<NSNumber>> {
            self.resolve(|node| {
                node.max_numeric_value().map(NSNumber::new_f64)
            })
            .flatten()
        }

        #[method(isAccessibilityElement)]
        fn is_accessibility_element(&self) -> bool {
            self.resolve(|node| filter(node) == FilterResult::Include)
                .unwrap_or(false)
        }

        #[method(isAccessibilityFocused)]
        fn is_focused(&self) -> bool {
            self.resolve(|node| node.is_focused() && can_be_focused(node))
                .unwrap_or(false)
        }

        #[method(setAccessibilityFocused:)]
        fn set_focused(&self, focused: bool) {
            self.resolve_with_context(|node, context| {
                if focused {
                    if node.is_focusable() {
                        context.do_action(ActionRequest {
                            action: Action::Focus,
                            target: node.id(),
                            data: None,
                        });
                    }
                } else {
                    let root = node.tree_state.root();
                    if root.is_focusable() {
                        context.do_action(ActionRequest {
                            action: Action::Focus,
                            target: root.id(),
                            data: None,
                        });
                    }
                }
            });
        }

        #[method(accessibilityPerformPress)]
        fn press(&self) -> bool {
            self.resolve_with_context(|node, context| {
                let clickable = node.is_clickable();
                if clickable {
                    context.do_action(ActionRequest {
                        action: Action::Default,
                        target: node.id(),
                        data: None,
                    });
                }
                clickable
            })
            .unwrap_or(false)
        }

        #[method(accessibilityPerformIncrement)]
        fn increment(&self) -> bool {
            self.resolve_with_context(|node, context| {
                let supports_increment = node.supports_increment();
                if supports_increment {
                    context.do_action(ActionRequest {
                        action: Action::Increment,
                        target: node.id(),
                        data: None,
                    });
                }
                supports_increment
            })
            .unwrap_or(false)
        }

        #[method(accessibilityPerformDecrement)]
        fn decrement(&self) -> bool {
            self.resolve_with_context(|node, context| {
                let supports_decrement = node.supports_decrement();
                if supports_decrement {
                    context.do_action(ActionRequest {
                        action: Action::Decrement,
                        target: node.id(),
                        data: None,
                    });
                }
                supports_decrement
            })
            .unwrap_or(false)
        }

        #[method(accessibilityNotifiesWhenDestroyed)]
        fn notifies_when_destroyed(&self) -> bool {
            true
        }

        #[method(accessibilityNumberOfCharacters)]
        fn number_of_characters(&self) -> NSInteger {
            self.resolve(|node| {
                if node.supports_text_ranges() {
                    node.document_range().end().to_global_utf16_index() as _
                } else {
                    0
                }
            })
            .unwrap_or(0)
        }

        #[method_id(accessibilitySelectedText)]
        fn selected_text(&self) -> Option<Id<NSString>> {
            self.resolve(|node| {
                if node.supports_text_ranges() {
                    if let Some(range) = node.text_selection() {
                        let text = range.text();
                        return Some(NSString::from_str(&text));
                    }
                }
                None
            })
            .flatten()
        }

        #[method(accessibilitySelectedTextRange)]
        fn selected_text_range(&self) -> NSRange {
            self.resolve(|node| {
                if node.supports_text_ranges() {
                    if let Some(range) = node.text_selection() {
                        return to_ns_range(&range);
                    }
                }
                NSRange::new(0, 0)
            })
            .unwrap_or_else(|| NSRange::new(0, 0))
        }

        #[method(accessibilityInsertionPointLineNumber)]
        fn insertion_point_line_number(&self) -> NSInteger {
            self.resolve(|node| {
                if node.supports_text_ranges() {
                    if let Some(pos) = node.text_selection_focus() {
                        return pos.to_line_index() as _;
                    }
                }
                0
            })
            .unwrap_or(0)
        }

        #[method(accessibilityRangeForLine:)]
        fn range_for_line(&self, line_index: NSInteger) -> NSRange {
            self.resolve(|node| {
                if node.supports_text_ranges() && line_index >= 0 {
                    if let Some(range) = node.line_range_from_index(line_index as _) {
                        return to_ns_range(&range);
                    }
                }
                NSRange::new(0, 0)
            })
            .unwrap_or_else(|| NSRange::new(0, 0))
        }

        #[method(accessibilityRangeForPosition:)]
        fn range_for_position(&self, point: NSPoint) -> NSRange {
            self.resolve_with_context(|node, context| {
                let view = match context.view.load() {
                    Some(view) => view,
                    None => {
                        return NSRange::new(0, 0);
                    }
                };

                if node.supports_text_ranges() {
                    let point = from_ns_point(&view, node, point);
                    let pos = node.text_position_at_point(point);
                    return to_ns_range_for_character(&pos);
                }
                NSRange::new(0, 0)
            })
            .unwrap_or_else(|| NSRange::new(0, 0))
        }

        #[method_id(accessibilityStringForRange:)]
        fn string_for_range(&self, range: NSRange) -> Option<Id<NSString>> {
            self.resolve(|node| {
                if node.supports_text_ranges() {
                    if let Some(range) = from_ns_range(node, range) {
                        let text = range.text();
                        return Some(NSString::from_str(&text));
                    }
                }
                None
            })
            .flatten()
        }

        #[method(accessibilityFrameForRange:)]
        fn frame_for_range(&self, range: NSRange) -> NSRect {
            self.resolve_with_context(|node, context| {
                let view = match context.view.load() {
                    Some(view) => view,
                    None => {
                        return NSRect::ZERO;
                    }
                };

                if node.supports_text_ranges() {
                    if let Some(range) = from_ns_range(node, range) {
                        let rects = range.bounding_boxes();
                        if let Some(rect) =
                            rects.into_iter().reduce(|rect1, rect2| rect1.union(rect2))
                        {
                            return to_ns_rect(&view, rect);
                        }
                    }
                }
                NSRect::ZERO
            })
            .unwrap_or(NSRect::ZERO)
        }

        #[method(accessibilityLineForIndex:)]
        fn line_for_index(&self, index: NSInteger) -> NSInteger {
            self.resolve(|node| {
                if node.supports_text_ranges() && index >= 0 {
                    if let Some(pos) = node.text_position_from_global_utf16_index(index as _) {
                        return pos.to_line_index() as _;
                    }
                }
                0
            })
            .unwrap_or(0)
        }

        #[method(accessibilityRangeForIndex:)]
        fn range_for_index(&self, index: NSInteger) -> NSRange {
            self.resolve(|node| {
                if node.supports_text_ranges() && index >= 0 {
                    if let Some(pos) = node.text_position_from_global_utf16_index(index as _) {
                        return to_ns_range_for_character(&pos);
                    }
                }
                NSRange::new(0, 0)
            })
            .unwrap_or_else(|| NSRange::new(0, 0))
        }

        #[method(setAccessibilitySelectedTextRange:)]
        fn set_selected_text_range(&self, range: NSRange) {
            self.resolve_with_context(|node, context| {
                if node.supports_text_ranges() {
                    if let Some(range) = from_ns_range(node, range) {
                        context.do_action(ActionRequest {
                            action: Action::SetTextSelection,
                            target: node.id(),
                            data: Some(ActionData::SetTextSelection(range.to_text_selection())),
                        });
                    }
                }
            });
        }

        #[method(isAccessibilitySelectorAllowed:)]
        fn is_selector_allowed(&self, selector: Sel) -> bool {
            self.resolve(|node| {
                if selector == sel!(setAccessibilityFocused:) {
                    return node.is_focusable();
                }
                if selector == sel!(accessibilityPerformPress) {
                    return node.is_clickable();
                }
                if selector == sel!(accessibilityPerformIncrement) {
                    return node.supports_increment();
                }
                if selector == sel!(accessibilityPerformDecrement) {
                    return node.supports_decrement();
                }
                if selector == sel!(accessibilityNumberOfCharacters)
                    || selector == sel!(accessibilitySelectedText)
                    || selector == sel!(accessibilitySelectedTextRange)
                    || selector == sel!(accessibilityInsertionPointLineNumber)
                    || selector == sel!(accessibilityRangeForLine:)
                    || selector == sel!(accessibilityRangeForPosition:)
                    || selector == sel!(accessibilityStringForRange:)
                    || selector == sel!(accessibilityFrameForRange:)
                    || selector == sel!(accessibilityLineForIndex:)
                    || selector == sel!(accessibilityRangeForIndex:)
                    || selector == sel!(setAccessibilitySelectedTextRange:)
                {
                    return node.supports_text_ranges();
                }
                if selector == sel!(setAccessibilityValue:) {
                    // Our implementation of this currently does nothing,
                    // and it's not clear if VoiceOver ever actually uses it,
                    // but it must be allowed for editable text in order to get
                    // the expected VoiceOver behavior.
                    return node.supports_text_ranges() && !node.is_read_only();
                }
                selector == sel!(accessibilityParent)
                    || selector == sel!(accessibilityChildren)
                    || selector == sel!(accessibilityChildrenInNavigationOrder)
                    || selector == sel!(accessibilityFrame)
                    || selector == sel!(accessibilityRole)
                    || selector == sel!(accessibilityRoleDescription)
                    || selector == sel!(accessibilityTitle)
                    || selector == sel!(accessibilityValue)
                    || selector == sel!(accessibilityMinValue)
                    || selector == sel!(accessibilityMaxValue)
                    || selector == sel!(isAccessibilityElement)
                    || selector == sel!(isAccessibilityFocused)
                    || selector == sel!(accessibilityNotifiesWhenDestroyed)
                    || selector == sel!(isAccessibilitySelectorAllowed:)
            })
            .unwrap_or(false)
        }
    }
);

impl PlatformNode {
    pub(crate) fn new(context: Weak<Context>, node_id: NodeId) -> Id<Self> {
        let this = Self::alloc().set_ivars(PlatformNodeIvars { context, node_id });

        unsafe { msg_send_id![super(this), init] }
    }

    fn resolve_with_context<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&Node, &Rc<Context>) -> T,
    {
        let context = self.ivars().context.upgrade()?;
        let tree = context.tree.borrow();
        let state = tree.state();
        let node = state.node_by_id(self.ivars().node_id)?;
        Some(f(&node, &context))
    }

    fn resolve<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&Node) -> T,
    {
        self.resolve_with_context(|node, _| f(node))
    }

    fn children_internal(&self) -> Option<Id<NSArray<PlatformNode>>> {
        self.resolve_with_context(|node, context| {
            let platform_nodes = node
                .filtered_children(filter)
                .map(|child| context.get_or_create_platform_node(child.id()))
                .collect::<Vec<Id<PlatformNode>>>();
            NSArray::from_vec(platform_nodes)
        })
    }
}
