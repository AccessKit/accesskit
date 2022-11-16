// Copyright 2022 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#![allow(non_upper_case_globals)]

use std::collections::HashMap;
use std::ptr::null_mut;
use std::sync::Weak;

use accesskit::{NodeId, Role};
use accesskit_consumer::{FilterResult, Node, Tree};
use objc2::{
    declare::{Ivar, IvarDrop},
    declare_class,
    foundation::{NSArray, NSCopying, NSObject, NSPoint, NSRect, NSSize, NSString, NSValue},
    msg_send_id, ns_string,
    rc::{Id, Owned, Shared, WeakId},
    runtime::Bool,
    ClassType,
};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::appkit::*;

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
            Role::TextField => NSAccessibilityTextFieldRole,
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
    if node.is_focused() {
        return FilterResult::Include;
    }

    if node.is_hidden() {
        return FilterResult::ExcludeSubtree;
    }

    if node.is_root() && node.role() == Role::Window {
        // If the root element is a window, ignore it.
        return FilterResult::ExcludeNode;
    }

    let ns_role = ns_role(node);
    if ns_role == unsafe { NSAccessibilityUnknownRole } {
        return FilterResult::ExcludeNode;
    }

    FilterResult::Include
}

fn get_parent(state: &State, node: &Node) -> *mut NSObject {
    let view = match state.view.load() {
        Some(view) => view,
        None => {
            return null_mut();
        }
    };

    if let Some(parent) = node.filtered_parent(&filter) {
        Id::autorelease_return(PlatformNode::get_or_create(
            &parent,
            state.tree.clone(),
            &view,
        )) as *mut _
    } else {
        Id::autorelease_return(view) as *mut _
    }
}

fn get_children(state: &State, node: &Node) -> *mut NSObject {
    let view = match state.view.load() {
        Some(view) => view,
        None => {
            return null_mut();
        }
    };

    let platform_nodes = node
        .filtered_children(filter)
        .map(|child| PlatformNode::get_or_create(&child, state.tree.clone(), &view))
        .collect::<Vec<Id<PlatformNode, Shared>>>();
    Id::autorelease_return(NSArray::from_vec(platform_nodes)) as *mut _
}

fn get_screen_bounding_box(state: &State, node: &Node) -> Option<NSRect> {
    let view = match state.view.load() {
        Some(view) => view,
        None => {
            return None;
        }
    };

    node.bounding_box().map(|rect| {
        let view_bounds = view.bounds();
        let rect = NSRect {
            origin: NSPoint {
                x: rect.x0,
                y: view_bounds.size.height - rect.y1,
            },
            size: NSSize {
                width: rect.width(),
                height: rect.height(),
            },
        };
        let rect = view.convert_rect_to_view(rect, None);
        let window = view.window().unwrap();
        window.convert_rect_to_screen(rect)
    })
}

fn get_position(state: &State, node: &Node) -> *mut NSObject {
    if let Some(rect) = get_screen_bounding_box(state, node) {
        Id::autorelease_return(NSValue::new(rect.origin)) as *mut _
    } else {
        null_mut()
    }
}

fn get_size(state: &State, node: &Node) -> *mut NSObject {
    if let Some(rect) = get_screen_bounding_box(state, node) {
        Id::autorelease_return(NSValue::new(rect.size)) as *mut _
    } else {
        null_mut()
    }
}

fn get_role(_state: &State, node: &Node) -> *mut NSObject {
    Id::autorelease_return(ns_role(node).copy()) as *mut _
}

fn get_title(_state: &State, node: &Node) -> *mut NSObject {
    // TODO: implement proper logic for title, description, and value;
    // see Chromium's content/browser/accessibility/browser_accessibility_cocoa.mm
    let name = node.name().unwrap_or_else(|| "".into());
    Id::autorelease_return(NSString::from_str(&name)) as *mut _
}

struct Attribute(&'static NSString, fn(&State, &Node) -> *mut NSObject);

static ATTRIBUTE_MAP: Lazy<Vec<Attribute>> = Lazy::new(|| unsafe {
    vec![
        Attribute(NSAccessibilityParentAttribute, get_parent),
        Attribute(NSAccessibilityChildrenAttribute, get_children),
        Attribute(NSAccessibilityPositionAttribute, get_position),
        Attribute(NSAccessibilitySizeAttribute, get_size),
        Attribute(NSAccessibilityRoleAttribute, get_role),
        Attribute(NSAccessibilityTitleAttribute, get_title),
    ]
});

struct State {
    tree: Weak<Tree>,
    node_id: NodeId,
    view: WeakId<NSView>,
}

fn is_ignored(_state: &State, node: &Node) -> bool {
    filter(node) != FilterResult::Include
}

impl State {
    fn resolve<F, T>(&self, f: F) -> Option<T>
    where
        F: FnOnce(&Node) -> T,
    {
        if let Some(tree) = self.tree.upgrade() {
            let state = tree.read();
            if let Some(node) = state.node_by_id(self.node_id) {
                return Some(f(&node));
            }
        }
        None
    }

    fn attribute_names(&self) -> *mut NSArray<NSString> {
        let names = ATTRIBUTE_MAP
            .iter()
            .map(|Attribute(name, _)| name.copy())
            .collect::<Vec<Id<NSString, Shared>>>();
        // TODO: role-specific attributes
        Id::autorelease_return(NSArray::from_vec(names))
    }

    fn attribute_value(&self, attribute_name: &NSString) -> *mut NSObject {
        self.resolve(|node| {
            println!("get attribute value {} on {:?}", attribute_name, node.id());

            for Attribute(test_name, f) in ATTRIBUTE_MAP.iter() {
                if attribute_name == *test_name {
                    return f(self, node);
                }
            }

            null_mut()
        })
        .unwrap_or_else(null_mut)
    }

    fn is_ignored(&self) -> Bool {
        self.resolve(|node| {
            if is_ignored(self, node) {
                Bool::YES
            } else {
                Bool::NO
            }
        })
        .unwrap_or(Bool::YES)
    }
}

declare_class!(
    pub(crate) struct PlatformNode {
        state: IvarDrop<Box<State>>,
    }

    unsafe impl ClassType for PlatformNode {
        type Super = NSObject;
        const NAME: &'static str = "AccessKitNode";
    }

    unsafe impl PlatformNode {
        #[sel(accessibilityAttributeNames)]
        fn attribute_names(&self) -> *mut NSArray<NSString> {
            self.state.attribute_names()
        }

        #[sel(accessibilityAttributeValue:)]
        fn attribute_value(&self, attribute_name: &NSString) -> *mut NSObject {
            self.state.attribute_value(attribute_name)
        }

        #[sel(accessibilityIsIgnored)]
        fn is_ignored(&self) -> Bool {
            self.state.is_ignored()
        }
    }
);

#[derive(PartialEq, Eq, Hash)]
struct PlatformNodeKey((*const NSView, NodeId));
unsafe impl Send for PlatformNodeKey {}
unsafe impl Sync for PlatformNodeKey {}

struct PlatformNodePtr(Id<PlatformNode, Shared>);
unsafe impl Send for PlatformNodePtr {}

static PLATFORM_NODES: Lazy<Mutex<HashMap<PlatformNodeKey, PlatformNodePtr>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

impl PlatformNode {
    pub(crate) fn get_or_create(
        node: &Node,
        tree: Weak<Tree>,
        view: &Id<NSView, Shared>,
    ) -> Id<Self, Shared> {
        let mut platform_nodes = PLATFORM_NODES.lock();
        let key = PlatformNodeKey((Id::as_ptr(view), node.id()));
        if let Some(result) = platform_nodes.get(&key) {
            return result.0.clone();
        }

        let state = Box::new(State {
            tree,
            node_id: node.id(),
            view: WeakId::new(view),
        });
        let result: Id<Self, Shared> = unsafe {
            let mut object: Id<Self, Owned> = msg_send_id![Self::class(), new];
            Ivar::write(&mut object.state, state);
            object.into()
        };

        platform_nodes.insert(key, PlatformNodePtr(result.clone()));
        result
    }

    // TODO: clean up platform nodes when underlying nodes are deleted
}
