// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#![allow(non_upper_case_globals)]

use accesskit::{kurbo::Point, CheckedState, NodeId, Role, TextSelection};
use accesskit_consumer::{DetachedNode, FilterResult, Node, NodeState, TextRange, Tree};
use objc2::{
    declare::{Ivar, IvarDrop},
    declare_class,
    foundation::{
        NSArray, NSAttributedString, NSCopying, NSInteger, NSNumber, NSObject, NSPoint, NSRange,
        NSRect, NSSize, NSString,
    },
    msg_send_id, ns_string,
    rc::{Id, Owned, Shared},
    runtime::Sel,
    sel, ClassType,
};
use std::{
    ptr::null_mut,
    rc::{Rc, Weak},
};

use crate::{appkit::*, context::Context};

fn ns_role(node: &Node) -> &'static NSString {
    let role = node.role();
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
            Role::Presentation => NSAccessibilityUnknownRole,
            Role::CheckBox => NSAccessibilityCheckBoxRole,
            Role::RadioButton => NSAccessibilityRadioButtonRole,
            Role::TextField => {
                if node.is_multiline() {
                    NSAccessibilityTextAreaRole
                } else {
                    NSAccessibilityTextFieldRole
                }
            }
            Role::Button => NSAccessibilityButtonRole,
            Role::LabelText => NSAccessibilityGroupRole,
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
            Role::Client => NSAccessibilityUnknownRole,
            Role::Code => NSAccessibilityGroupRole,
            Role::ColorWell => NSAccessibilityColorWellRole,
            Role::ComboBoxGrouping => NSAccessibilityComboBoxRole,
            Role::ComboBoxMenuButton => NSAccessibilityComboBoxRole,
            Role::Complementary => NSAccessibilityGroupRole,
            Role::Comment => NSAccessibilityGroupRole,
            Role::ContentDeletion => NSAccessibilityGroupRole,
            Role::ContentInsertion => NSAccessibilityGroupRole,
            Role::ContentInfo => NSAccessibilityGroupRole,
            Role::Date => ns_string!("AXDateField"),
            Role::DateTime => ns_string!("AXDateField"),
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
            Role::InputTime => ns_string!("AXTimeField"),
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
            Role::PopupButton => NSAccessibilityPopUpButtonRole,
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
            Role::SearchBox => NSAccessibilityTextFieldRole,
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
            Role::TextFieldWithComboBox => NSAccessibilityComboBoxRole,
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
        }
    }
}

pub(crate) fn filter(node: &Node) -> FilterResult {
    let ns_role = ns_role(node);
    if ns_role == unsafe { NSAccessibilityUnknownRole } {
        return FilterResult::ExcludeNode;
    }

    if node.is_hidden() && !node.is_focused() {
        return FilterResult::ExcludeSubtree;
    }

    FilterResult::Include
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

    fn name(&self) -> Option<String> {
        match self {
            Self::Node(node) => node.name(),
            Self::DetachedNode(node) => node.name(),
        }
    }

    // TODO: implement proper logic for title, description, and value;
    // see Chromium's content/browser/accessibility/browser_accessibility_cocoa.mm
    // and figure out how this is different in the macOS 10.10+ protocol

    pub(crate) fn title(&self) -> Option<String> {
        let state = self.node_state();
        if state.role() == Role::StaticText && state.value().is_none() {
            // In this case, macOS wants the text to be the value, not title.
            return None;
        }
        self.name()
    }

    pub(crate) fn value(&self) -> Option<Value> {
        let state = self.node_state();
        if let Some(state) = state.checked_state() {
            return Some(Value::Bool(state != CheckedState::False));
        }
        if let Some(value) = state.value() {
            return Some(Value::String(value.into()));
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

struct BoxedData {
    context: Weak<Context>,
    node_id: NodeId,
}

fn text_range_from_ns_range<'a>(node: &'a Node<'a>, ns_range: NSRange) -> Option<TextRange<'a>> {
    let pos = node.text_position_from_global_utf16_index(ns_range.location)?;
    let mut range = pos.to_degenerate_range();
    if ns_range.length > 0 {
        let end =
            node.text_position_from_global_utf16_index(ns_range.location + ns_range.length)?;
        range.set_end(end);
    }
    Some(range)
}

declare_class!(
    pub(crate) struct PlatformNode {
        // SAFETY: This is set in `PlatformNode::new` immediately after
        // the object is created.
        boxed: IvarDrop<Box<BoxedData>>,
    }

    unsafe impl ClassType for PlatformNode {
        #[inherits(NSObject)]
        type Super = NSAccessibilityElement;
        const NAME: &'static str = "AccessKitNode";
    }

    unsafe impl PlatformNode {
        #[sel(accessibilityParent)]
        fn parent(&self) -> *mut NSObject {
            self.resolve_with_context(|node, context| {
                if let Some(parent) = node.filtered_parent(&filter) {
                    Id::autorelease_return(context.get_or_create_platform_node(parent.id()))
                        as *mut _
                } else {
                    context
                        .view
                        .load()
                        .map_or_else(null_mut, |view| Id::autorelease_return(view) as *mut _)
                }
            })
            .unwrap_or_else(null_mut)
        }

        #[sel(accessibilityChildren)]
        fn children(&self) -> *mut NSArray<PlatformNode> {
            self.children_internal()
        }

        #[sel(accessibilityChildrenInNavigationOrder)]
        fn children_in_navigation_order(&self) -> *mut NSArray<PlatformNode> {
            // For now, we assume the children are in navigation order.
            self.children_internal()
        }

        #[sel(accessibilityFrame)]
        fn frame(&self) -> NSRect {
            self.resolve_with_context(|node, context| {
                let view = match context.view.load() {
                    Some(view) => view,
                    None => {
                        return NSRect::ZERO;
                    }
                };

                node.bounding_box().map_or(NSRect::ZERO, |rect| {
                    // AccessKit coordinates are in physical (DPI-dependent)
                    // pixels, but macOS expects logical (DPI-independent)
                    // coordinates here.
                    let factor = view.backing_scale_factor();
                    let rect = NSRect {
                        origin: NSPoint {
                            x: rect.x0 / factor,
                            y: if view.is_flipped() {
                                rect.y0 / factor
                            } else {
                                let view_bounds = view.bounds();
                                view_bounds.size.height - rect.y1 / factor
                            },
                        },
                        size: NSSize {
                            width: rect.width() / factor,
                            height: rect.height() / factor,
                        },
                    };
                    let rect = view.convert_rect_to_view(rect, None);
                    let window = view.window().unwrap();
                    window.convert_rect_to_screen(rect)
                })
            })
            .unwrap_or(NSRect::ZERO)
        }

        #[sel(accessibilityRole)]
        fn role(&self) -> *mut NSString {
            let role = self
                .resolve(ns_role)
                .unwrap_or(unsafe { NSAccessibilityUnknownRole });
            Id::autorelease_return(role.copy())
        }

        #[sel(accessibilityTitle)]
        fn title(&self) -> *mut NSString {
            let result = self
                .resolve(|node| {
                    let wrapper = NodeWrapper::Node(node);
                    wrapper.title()
                })
                .flatten();
            result.map_or_else(null_mut, |result| {
                Id::autorelease_return(NSString::from_str(&result))
            })
        }

        #[sel(accessibilityValue)]
        fn value(&self) -> *mut NSObject {
            self.resolve(|node| {
                let wrapper = NodeWrapper::Node(node);
                wrapper.value().map_or_else(null_mut, |value| match value {
                    Value::Bool(value) => {
                        Id::autorelease_return(NSNumber::new_bool(value)) as *mut _
                    }
                    Value::Number(value) => {
                        Id::autorelease_return(NSNumber::new_f64(value)) as *mut _
                    }
                    Value::String(value) => {
                        Id::autorelease_return(NSString::from_str(&value)) as *mut _
                    }
                })
            })
            .unwrap_or_else(null_mut)
        }

        #[sel(accessibilityMinValue)]
        fn min_value(&self) -> *mut NSNumber {
            self.resolve(|node| {
                node.min_numeric_value().map_or_else(null_mut, |value| {
                    Id::autorelease_return(NSNumber::new_f64(value))
                })
            })
            .unwrap_or_else(null_mut)
        }

        #[sel(accessibilityMaxValue)]
        fn max_value(&self) -> *mut NSNumber {
            self.resolve(|node| {
                node.max_numeric_value().map_or_else(null_mut, |value| {
                    Id::autorelease_return(NSNumber::new_f64(value))
                })
            })
            .unwrap_or_else(null_mut)
        }

        #[sel(isAccessibilityElement)]
        fn is_accessibility_element(&self) -> bool {
            self.resolve(|node| filter(node) == FilterResult::Include)
                .unwrap_or(false)
        }

        #[sel(isAccessibilityFocused)]
        fn is_focused(&self) -> bool {
            self.resolve(|node| node.is_focused() && can_be_focused(node))
                .unwrap_or(false)
        }

        #[sel(setAccessibilityFocused:)]
        fn set_focused(&self, focused: bool) {
            self.resolve_with_tree(|node, tree| {
                if focused {
                    if node.is_focusable() {
                        tree.set_focus(node.id());
                    }
                } else {
                    let root = node.tree_state.root();
                    if root.is_focusable() {
                        tree.set_focus(root.id());
                    }
                }
            });
        }

        #[sel(accessibilityPerformPress)]
        fn press(&self) -> bool {
            self.resolve_with_tree(|node, tree| {
                let clickable = node.is_clickable();
                if clickable {
                    tree.do_default_action(node.id());
                }
                clickable
            })
            .unwrap_or(false)
        }

        #[sel(accessibilityPerformIncrement)]
        fn increment(&self) -> bool {
            self.resolve_with_tree(|node, tree| {
                let supports_increment = node.supports_increment();
                if supports_increment {
                    tree.increment(node.id());
                }
                supports_increment
            })
            .unwrap_or(false)
        }

        #[sel(accessibilityPerformDecrement)]
        fn decrement(&self) -> bool {
            self.resolve_with_tree(|node, tree| {
                let supports_decrement = node.supports_decrement();
                if supports_decrement {
                    tree.decrement(node.id());
                }
                supports_decrement
            })
            .unwrap_or(false)
        }

        #[sel(accessibilityNotifiesWhenDestroyed)]
        fn notifies_when_destroyed(&self) -> bool {
            true
        }

        #[sel(accessibilityNumberOfCharacters)]
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

        #[sel(accessibilitySelectedText)]
        fn selected_text(&self) -> *mut NSString {
            self.resolve(|node| {
                if node.supports_text_ranges() {
                    if let Some(range) = node.text_selection() {
                        let text = range.text();
                        return Id::autorelease_return(NSString::from_str(&text));
                    }
                }
                null_mut()
            })
            .unwrap_or_else(null_mut)
        }

        #[sel(accessibilitySelectedTextRange)]
        fn selected_text_range(&self) -> NSRange {
            self.resolve(|node| {
                if node.supports_text_ranges() {
                    if let Some(range) = node.text_selection() {
                        let start = range.start().to_global_utf16_index();
                        let end = range.end().to_global_utf16_index();
                        return NSRange::from(start..end);
                    }
                }
                NSRange::new(0, 0)
            })
            .unwrap_or_else(|| NSRange::new(0, 0))
        }

        #[sel(accessibilityInsertionPointLineNumber)]
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

        #[sel(accessibilityRangeForLine:)]
        fn range_for_line(&self, line_index: NSInteger) -> NSRange {
            self.resolve(|node| {
                if node.supports_text_ranges() && line_index >= 0 {
                    if let Some(range) = node.line_range_from_index(line_index as _) {
                        let start = range.start().to_global_utf16_index();
                        let end = range.end().to_global_utf16_index();
                        return NSRange::from(start..end);
                    }
                }
                NSRange::new(0, 0)
            })
            .unwrap_or_else(|| NSRange::new(0, 0))
        }

        #[sel(accessibilityRangeForPosition:)]
        fn range_for_position(&self, point: NSPoint) -> NSRange {
            self.resolve_with_context(|node, context| {
                let view = match context.view.load() {
                    Some(view) => view,
                    None => {
                        return NSRange::new(0, 0);
                    }
                };

                let window = view.window().unwrap();
                let point = window.convert_point_from_screen(point);
                let point = view.convert_point_from_view(point, None);
                // AccessKit coordinates are in physical (DPI-dependent) pixels, but
                // macOS provides logical (DPI-independent) coordinates here.
                let factor = view.backing_scale_factor();
                let point = Point::new(
                    point.x * factor,
                    if view.is_flipped() {
                        point.y * factor
                    } else {
                        let view_bounds = view.bounds();
                        (view_bounds.size.height - point.y) * factor
                    },
                );
                let point = node.transform().inverse() * point;

                if node.supports_text_ranges() {
                    let start = node.text_position_at_point(point);
                    let end = if start.is_document_end() {
                        start
                    } else {
                        start.forward_by_character()
                    };
                    let start = start.to_global_utf16_index();
                    let end = end.to_global_utf16_index();
                    return NSRange::from(start..end);
                }
                NSRange::new(0, 0)
            })
            .unwrap_or_else(|| NSRange::new(0, 0))
        }

        #[sel(accessibilityStringForRange:)]
        fn string_for_range(&self, range: NSRange) -> *mut NSString {
            self.resolve(|node| {
                if node.supports_text_ranges() {
                    if let Some(range) = text_range_from_ns_range(node, range) {
                        let text = range.text();
                        return Id::autorelease_return(NSString::from_str(&text));
                    }
                }
                null_mut()
            })
            .unwrap_or_else(null_mut)
        }

        #[sel(accessibilityAttributedStringForRange:)]
        fn attributed_string_for_range(&self, range: NSRange) -> *mut NSAttributedString {
            self.resolve(|node| {
                if node.supports_text_ranges() {
                    if let Some(range) = text_range_from_ns_range(node, range) {
                        let text = range.text();
                        // TODO: Expose formatting information.
                        let ns_string = NSString::from_str(&text);
                        return Id::autorelease_return(NSAttributedString::from_nsstring(
                            &ns_string,
                        ));
                    }
                }
                null_mut()
            })
            .unwrap_or_else(null_mut)
        }

        #[sel(accessibilityFrameForRange:)]
        fn frame_for_range(&self, range: NSRange) -> NSRect {
            self.resolve_with_context(|node, context| {
                let view = match context.view.load() {
                    Some(view) => view,
                    None => {
                        return NSRect::ZERO;
                    }
                };

                if node.supports_text_ranges() {
                    if let Some(range) = text_range_from_ns_range(node, range) {
                        let rects = range.bounding_boxes();
                        if let Some(rect) =
                            rects.into_iter().reduce(|rect1, rect2| rect1.union(rect2))
                        {
                            // AccessKit coordinates are in physical (DPI-dependent)
                            // pixels, but macOS expects logical (DPI-independent)
                            // coordinates here.
                            let factor = view.backing_scale_factor();
                            let rect = NSRect {
                                origin: NSPoint {
                                    x: rect.x0 / factor,
                                    y: if view.is_flipped() {
                                        rect.y0 / factor
                                    } else {
                                        let view_bounds = view.bounds();
                                        view_bounds.size.height - rect.y1 / factor
                                    },
                                },
                                size: NSSize {
                                    width: rect.width() / factor,
                                    height: rect.height() / factor,
                                },
                            };
                            let rect = view.convert_rect_to_view(rect, None);
                            let window = view.window().unwrap();
                            return window.convert_rect_to_screen(rect);
                        }
                    }
                }
                NSRect::ZERO
            })
            .unwrap_or(NSRect::ZERO)
        }

        #[sel(accessibilityLineForIndex:)]
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

        #[sel(accessibilityRangeForIndex:)]
        fn range_for_index(&self, index: NSInteger) -> NSRange {
            self.resolve(|node| {
                if node.supports_text_ranges() && index >= 0 {
                    if let Some(start) = node.text_position_from_global_utf16_index(index as _) {
                        let end = if start.is_document_end() {
                            start
                        } else {
                            start.forward_by_character()
                        };
                        let start = start.to_global_utf16_index();
                        let end = end.to_global_utf16_index();
                        return NSRange::from(start..end);
                    }
                }
                NSRange::new(0, 0)
            })
            .unwrap_or_else(|| NSRange::new(0, 0))
        }

        #[sel(setAccessibilitySelectedTextRange:)]
        fn set_selected_text_range(&self, range: NSRange) {
            self.resolve_with_tree(|node, tree| {
                if node.supports_text_ranges() {
                    if let Some(range) = text_range_from_ns_range(node, range) {
                        tree.select_text_range(&range);
                    }
                }
            });
        }

        #[sel(isAccessibilitySelectorAllowed:)]
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
                    || selector == sel!(accessibilityAttributedStringForRange:)
                    || selector == sel!(accessibilityFrameForRange:)
                    || selector == sel!(accessibilityLineForIndex:)
                    || selector == sel!(accessibilityRangeForIndex:)
                    // This adapter doesn't yet actually support the SetValue
                    // action, but we must claim to support it in order
                    // to get the expected VoiceOver behavior in edits.
                    || selector == sel!(setAccessibilityValue:)
                    || selector == sel!(setAccessibilitySelectedTextRange:)
                {
                    return node.supports_text_ranges();
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
    pub(crate) fn new(context: Weak<Context>, node_id: NodeId) -> Id<Self, Shared> {
        let boxed = Box::new(BoxedData { context, node_id });
        unsafe {
            let mut object: Id<Self, Owned> = msg_send_id![Self::class(), new];
            Ivar::write(&mut object.boxed, boxed);
            object.into()
        }
    }

    fn resolve_with_context<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&Node, &Rc<Context>) -> T,
    {
        let context = self.boxed.context.upgrade()?;
        let state = context.tree.read();
        let node = state.node_by_id(self.boxed.node_id)?;
        Some(f(&node, &context))
    }

    fn resolve<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&Node) -> T,
    {
        self.resolve_with_context(|node, _| f(node))
    }

    fn resolve_with_tree<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&Node, &Tree) -> T,
    {
        self.resolve_with_context(|node, context| f(node, &context.tree))
    }

    fn children_internal(&self) -> *mut NSArray<PlatformNode> {
        self.resolve_with_context(|node, context| {
            let platform_nodes = node
                .filtered_children(filter)
                .map(|child| context.get_or_create_platform_node(child.id()))
                .collect::<Vec<Id<PlatformNode, Shared>>>();
            Id::autorelease_return(NSArray::from_vec(platform_nodes))
        })
        .unwrap_or_else(null_mut)
    }
}
