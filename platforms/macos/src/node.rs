// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#![allow(non_upper_case_globals)]

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Weak;

use accesskit::{NodeId, Role};
use accesskit_consumer::{FilterResult, Node, Tree};
use cocoa::appkit::NSWindow;
use cocoa::base::{id, nil, BOOL, NO, YES};
use cocoa::foundation::{NSArray, NSPoint, NSRect, NSSize, NSValue};
use objc::declare::ClassDecl;
use objc::rc::{StrongPtr, WeakPtr};
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

use crate::util::{from_nsstring, make_nsstring, nsstrings_equal};

struct Attribute(*const id, fn(&State, &Node) -> id);
unsafe impl Sync for Attribute {}

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
    if nsstrings_equal(ns_role, unsafe { NSAccessibilityUnknownRole }) {
        return FilterResult::ExcludeNode;
    }

    FilterResult::Include
}

fn get_parent(state: &State, node: &Node) -> id {
    let view = state.view.load();
    if view.is_null() {
        return nil;
    }

    if let Some(parent) = node.filtered_parent(&filter) {
        PlatformNode::get_or_create(&parent, state.tree.clone(), &view).autorelease()
    } else {
        view.autorelease()
    }
}

fn get_children(state: &State, node: &Node) -> id {
    let view = state.view.load();
    if view.is_null() {
        return nil;
    }

    let platform_nodes = node
        .filtered_children(filter)
        .map(|child| PlatformNode::get_or_create(&child, state.tree.clone(), &view).autorelease())
        .collect::<Vec<id>>();
    unsafe { NSArray::arrayWithObjects(nil, &platform_nodes) }
}

fn get_screen_bounding_box(state: &State, node: &Node) -> Option<NSRect> {
    let view = state.view.load();
    if view.is_null() {
        return None;
    }

    node.bounding_box().map(|rect| {
        let rect = NSRect {
            origin: NSPoint {
                x: rect.x0,
                y: rect.y0,
            },
            size: NSSize {
                width: rect.width(),
                height: rect.height(),
            },
        };
        let rect: NSRect = unsafe { msg_send![*view, convertRect:rect toView:nil] };
        let window: id = unsafe { msg_send![*view, window] };
        unsafe { window.convertRectToScreen_(rect) }
    })
}

fn get_position(state: &State, node: &Node) -> id {
    if let Some(rect) = get_screen_bounding_box(state, node) {
        unsafe { NSValue::valueWithPoint(nil, rect.origin) }
    } else {
        nil
    }
}

fn get_size(state: &State, node: &Node) -> id {
    if let Some(rect) = get_screen_bounding_box(state, node) {
        unsafe { NSValue::valueWithSize(nil, rect.size) }
    } else {
        nil
    }
}

fn ns_role(node: &Node) -> id {
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
            Role::ListMarker => make_nsstring("AXListMarker"),
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
            Role::Date => make_nsstring("AXDateField"),
            Role::DateTime => make_nsstring("AXDateField"),
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
            Role::Heading => make_nsstring("Heading"),
            Role::Iframe => NSAccessibilityGroupRole,
            Role::IframePresentational => NSAccessibilityGroupRole,
            Role::ImeCandidate => NSAccessibilityUnknownRole,
            Role::InputTime => make_nsstring("AXTimeField"),
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
            Role::RootWebArea => make_nsstring("AXWebArea"),
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
            Role::DocSubtitle => make_nsstring("AXHeading"),
            Role::DocTip => NSAccessibilityGroupRole,
            Role::DocToc => NSAccessibilityGroupRole,
            Role::ListGrid => NSAccessibilityUnknownRole,
        }
    }
}

fn get_role(_state: &State, node: &Node) -> id {
    ns_role(node)
}

fn get_title(_state: &State, node: &Node) -> id {
    // TODO: implement proper logic for title, description, and value;
    // see Chromium's content/browser/accessibility/browser_accessibility_cocoa.mm
    let name = node.name().unwrap_or_else(|| "".into());
    make_nsstring(&name)
}

static ATTRIBUTE_MAP: &[Attribute] = unsafe {
    &[
        Attribute(&NSAccessibilityParentAttribute, get_parent),
        Attribute(&NSAccessibilityChildrenAttribute, get_children),
        Attribute(&NSAccessibilityPositionAttribute, get_position),
        Attribute(&NSAccessibilitySizeAttribute, get_size),
        Attribute(&NSAccessibilityRoleAttribute, get_role),
        Attribute(&NSAccessibilityTitleAttribute, get_title),
    ]
};

struct State {
    tree: Weak<Tree>,
    node_id: NodeId,
    view: WeakPtr,
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

    fn attribute_names(&self) -> id {
        let names = ATTRIBUTE_MAP
            .iter()
            .map(|Attribute(name_ptr, _)| unsafe { **name_ptr })
            .collect::<Vec<id>>();
        // TODO: role-specific attributes
        unsafe { NSArray::arrayWithObjects(nil, &names) }
    }

    fn attribute_value(&self, attribute_name: id) -> id {
        self.resolve(|node| {
            println!(
                "get attribute value {} on {:?}",
                from_nsstring(attribute_name),
                node.id()
            );

            for Attribute(test_name_ptr, f) in ATTRIBUTE_MAP {
                let test_name = unsafe { **test_name_ptr };
                if nsstrings_equal(attribute_name, test_name) {
                    return f(self, node);
                }
            }

            nil
        })
        .unwrap_or(nil)
    }

    fn is_ignored(&self) -> BOOL {
        self.resolve(|node| if is_ignored(self, node) { YES } else { NO })
            .unwrap_or(YES)
    }
}

pub(crate) struct PlatformNode;

impl PlatformNode {
    pub(crate) fn get_or_create(node: &Node, tree: Weak<Tree>, view: &StrongPtr) -> StrongPtr {
        let mut platform_nodes = PLATFORM_NODES.lock();
        let key = PlatformNodeKey((**view, node.id()));
        if let Some(result) = platform_nodes.get(&key) {
            return result.0.clone();
        }

        let state = Box::new(State {
            tree,
            node_id: node.id(),
            view: view.weak(),
        });
        let result = unsafe {
            let object: id = msg_send![PLATFORM_NODE_CLASS.0, alloc];
            let () = msg_send![object, init];
            let state_ptr = Box::into_raw(state);
            (*object).set_ivar(STATE_IVAR, state_ptr as *mut c_void);
            StrongPtr::new(object)
        };

        platform_nodes.insert(key, PlatformNodePtr(result.clone()));
        result
    }

    // TODO: clean up platform nodes when underlying nodes are deleted
}

static STATE_IVAR: &str = "accessKitPlatformNodeState";

struct PlatformNodeClass(*const Class);
unsafe impl Send for PlatformNodeClass {}
unsafe impl Sync for PlatformNodeClass {}

#[derive(PartialEq, Eq, Hash)]
struct PlatformNodeKey((id, NodeId));
unsafe impl Send for PlatformNodeKey {}
unsafe impl Sync for PlatformNodeKey {}

struct PlatformNodePtr(StrongPtr);
unsafe impl Send for PlatformNodePtr {}
unsafe impl Sync for PlatformNodePtr {}

static PLATFORM_NODE_CLASS: Lazy<PlatformNodeClass> = Lazy::new(|| unsafe {
    let mut decl = ClassDecl::new("AccessKitPlatformNode", class!(NSObject))
        .expect("platform node class definition failed");
    decl.add_ivar::<*mut c_void>(STATE_IVAR);

    // TODO: methods

    decl.add_method(
        sel!(accessibilityAttributeNames),
        attribute_names as extern "C" fn(&Object, Sel) -> id,
    );
    extern "C" fn attribute_names(this: &Object, _sel: Sel) -> id {
        unsafe {
            let state: *mut c_void = *this.get_ivar(STATE_IVAR);
            let state = &mut *(state as *mut State);
            state.attribute_names()
        }
    }

    decl.add_method(
        sel!(accessibilityAttributeValue:),
        attribute_value as extern "C" fn(&Object, Sel, id) -> id,
    );
    extern "C" fn attribute_value(this: &Object, _sel: Sel, attribute_name: id) -> id {
        unsafe {
            let state: *mut c_void = *this.get_ivar(STATE_IVAR);
            let state = &mut *(state as *mut State);
            state.attribute_value(attribute_name)
        }
    }

    decl.add_method(
        sel!(accessibilityIsIgnored),
        is_ignored as extern "C" fn(&Object, Sel) -> BOOL,
    );
    extern "C" fn is_ignored(this: &Object, _sel: Sel) -> BOOL {
        unsafe {
            let state: *mut c_void = *this.get_ivar(STATE_IVAR);
            let state = &mut *(state as *mut State);
            state.is_ignored()
        }
    }

    decl.add_method(sel!(dealloc), dealloc as extern "C" fn(&Object, Sel));
    extern "C" fn dealloc(this: &Object, _sel: Sel) {
        unsafe {
            let state: *mut c_void = *this.get_ivar(STATE_IVAR);
            drop(Box::from_raw(state as *mut State));
        }
    }

    PlatformNodeClass(decl.register())
});

static PLATFORM_NODES: Lazy<Mutex<HashMap<PlatformNodeKey, PlatformNodePtr>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

// Constants declared in AppKit
#[link(name = "AppKit", kind = "framework")]
extern "C" {
    // Attributes
    static NSAccessibilityChildrenAttribute: id;
    static NSAccessibilityParentAttribute: id;
    static NSAccessibilityPositionAttribute: id;
    static NSAccessibilityRoleAttribute: id;
    static NSAccessibilitySizeAttribute: id;
    static NSAccessibilityTitleAttribute: id;

    // Roles
    static NSAccessibilityButtonRole: id;
    static NSAccessibilityCheckBoxRole: id;
    static NSAccessibilityCellRole: id;
    static NSAccessibilityColorWellRole: id;
    static NSAccessibilityColumnRole: id;
    static NSAccessibilityComboBoxRole: id;
    static NSAccessibilityGroupRole: id;
    static NSAccessibilityImageRole: id;
    static NSAccessibilityIncrementorRole: id;
    static NSAccessibilityLevelIndicatorRole: id;
    static NSAccessibilityLinkRole: id;
    static NSAccessibilityListRole: id;
    static NSAccessibilityMenuRole: id;
    static NSAccessibilityMenuBarRole: id;
    static NSAccessibilityMenuItemRole: id;
    static NSAccessibilityOutlineRole: id;
    static NSAccessibilityPopUpButtonRole: id;
    static NSAccessibilityProgressIndicatorRole: id;
    static NSAccessibilityRadioButtonRole: id;
    static NSAccessibilityRadioGroupRole: id;
    static NSAccessibilityRowRole: id;
    static NSAccessibilityScrollBarRole: id;
    static NSAccessibilitySliderRole: id;
    static NSAccessibilitySplitterRole: id;
    static NSAccessibilityStaticTextRole: id;
    static NSAccessibilityTabGroupRole: id;
    static NSAccessibilityTableRole: id;
    static NSAccessibilityTextFieldRole: id;
    static NSAccessibilityToolbarRole: id;
    static NSAccessibilityUnknownRole: id;
}
