// Copyright 2021 The AccessKit Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#![allow(non_upper_case_globals)]

use std::ffi::c_void;

use accesskit_consumer::{Node, WeakNode};
use accesskit_schema::Role;
use cocoa::base::{id, nil, BOOL, NO, YES};
use cocoa::foundation::{NSArray, NSPoint, NSSize, NSValue};
use lazy_static::lazy_static;
use objc::declare::ClassDecl;
use objc::rc::{StrongPtr, WeakPtr};
use objc::runtime::{Class, Object, Sel};
use objc::{class, msg_send, sel, sel_impl};

use crate::util::from_nsstring;

struct Attribute(*const id, fn(&State, &Node) -> id);
unsafe impl Sync for Attribute {}

fn get_parent(state: &State, node: &Node) -> id {
    let view = state.view.load();
    if view.is_null() {
        return nil;
    }

    if let Some(parent) = node.parent() {
        PlatformNode::new(&parent, &view).autorelease()
    } else {
        view.autorelease()
    }
}

fn get_position(_state: &State, node: &Node) -> id {
    if let Some(bounds) = &node.data().bounds {
        // TODO: implement for real
        let ns_point = NSPoint { x: 100., y: 100. };
        unsafe { NSValue::valueWithPoint(nil, ns_point) }
    } else {
        nil
    }
}

fn get_role(_state: &State, node: &Node) -> id {
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
            Role::ListMarker => NSAccessibilityUnknownRole,
            Role::TreeItem => NSAccessibilityRowRole,
            Role::ListBoxOption => NSAccessibilityStaticTextRole,
            Role::MenuItem => NSAccessibilityMenuItemRole,
            Role::MenuListOption => NSAccessibilityMenuItemRole,
            Role::Paragraph => NSAccessibilityGroupRole,
            Role::GenericContainer => NSAccessibilityGroupRole,
            Role::Presentation => NSAccessibilityGroupRole,
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
            // TODO: continue mapping here
            Role::Abbr => NSAccessibilityUnknownRole,
            Role::Alert => NSAccessibilityUnknownRole,
            Role::AlertDialog => NSAccessibilityUnknownRole,
            Role::Application => NSAccessibilityUnknownRole,
            Role::Article => NSAccessibilityUnknownRole,
            Role::Audio => NSAccessibilityUnknownRole,
            Role::Banner => NSAccessibilityUnknownRole,
            Role::Blockquote => NSAccessibilityUnknownRole,
            Role::Canvas => NSAccessibilityUnknownRole,
            Role::Caption => NSAccessibilityUnknownRole,
            Role::Caret => NSAccessibilityUnknownRole,
            Role::Client => NSAccessibilityUnknownRole,
            Role::Code => NSAccessibilityUnknownRole,
            Role::ColorWell => NSAccessibilityUnknownRole,
            Role::ComboBoxGrouping => NSAccessibilityUnknownRole,
            Role::ComboBoxMenuButton => NSAccessibilityUnknownRole,
            Role::Complementary => NSAccessibilityUnknownRole,
            Role::Comment => NSAccessibilityUnknownRole,
            Role::ContentDeletion => NSAccessibilityUnknownRole,
            Role::ContentInsertion => NSAccessibilityUnknownRole,
            Role::ContentInfo => NSAccessibilityUnknownRole,
            Role::Date => NSAccessibilityUnknownRole,
            Role::DateTime => NSAccessibilityUnknownRole,
            Role::Definition => NSAccessibilityUnknownRole,
            Role::DescriptionList => NSAccessibilityUnknownRole,
            Role::DescriptionListDetail => NSAccessibilityUnknownRole,
            Role::DescriptionListTerm => NSAccessibilityUnknownRole,
            Role::Details => NSAccessibilityUnknownRole,
            Role::Dialog => NSAccessibilityUnknownRole,
            Role::Directory => NSAccessibilityUnknownRole,
            Role::DisclosureTriangle => NSAccessibilityUnknownRole,
            Role::Document => NSAccessibilityUnknownRole,
            Role::EmbeddedObject => NSAccessibilityUnknownRole,
            Role::Emphasis => NSAccessibilityUnknownRole,
            Role::Feed => NSAccessibilityUnknownRole,
            Role::FigureCaption => NSAccessibilityUnknownRole,
            Role::Figure => NSAccessibilityUnknownRole,
            Role::Footer => NSAccessibilityUnknownRole,
            Role::FooterAsNonLandmark => NSAccessibilityUnknownRole,
            Role::Form => NSAccessibilityUnknownRole,
            Role::Grid => NSAccessibilityUnknownRole,
            Role::Group => NSAccessibilityGroupRole,
            Role::Header => NSAccessibilityUnknownRole,
            Role::HeaderAsNonLandmark => NSAccessibilityUnknownRole,
            Role::Heading => NSAccessibilityUnknownRole,
            Role::Iframe => NSAccessibilityUnknownRole,
            Role::IframePresentational => NSAccessibilityUnknownRole,
            Role::ImeCandidate => NSAccessibilityUnknownRole,
            Role::InputTime => NSAccessibilityUnknownRole,
            Role::Keyboard => NSAccessibilityUnknownRole,
            Role::Legend => NSAccessibilityUnknownRole,
            Role::LineBreak => NSAccessibilityUnknownRole,
            Role::ListBox => NSAccessibilityUnknownRole,
            Role::Log => NSAccessibilityUnknownRole,
            Role::Main => NSAccessibilityUnknownRole,
            Role::Mark => NSAccessibilityUnknownRole,
            Role::Marquee => NSAccessibilityUnknownRole,
            Role::Math => NSAccessibilityUnknownRole,
            Role::MenuBar => NSAccessibilityUnknownRole,
            Role::MenuItemCheckBox => NSAccessibilityUnknownRole,
            Role::MenuItemRadio => NSAccessibilityUnknownRole,
            Role::MenuListPopup => NSAccessibilityUnknownRole,
            Role::Meter => NSAccessibilityUnknownRole,
            Role::Navigation => NSAccessibilityUnknownRole,
            Role::Note => NSAccessibilityUnknownRole,
            Role::PluginObject => NSAccessibilityUnknownRole,
            Role::PopupButton => NSAccessibilityUnknownRole,
            Role::Portal => NSAccessibilityUnknownRole,
            Role::Pre => NSAccessibilityUnknownRole,
            Role::ProgressIndicator => NSAccessibilityUnknownRole,
            Role::RadioGroup => NSAccessibilityUnknownRole,
            Role::Region => NSAccessibilityUnknownRole,
            Role::RootWebArea => NSAccessibilityUnknownRole,
            Role::Ruby => NSAccessibilityUnknownRole,
            Role::RubyAnnotation => NSAccessibilityUnknownRole,
            Role::ScrollBar => NSAccessibilityUnknownRole,
            Role::ScrollView => NSAccessibilityUnknownRole,
            Role::Search => NSAccessibilityUnknownRole,
            Role::SearchBox => NSAccessibilityUnknownRole,
            Role::Section => NSAccessibilityUnknownRole,
            Role::Slider => NSAccessibilityUnknownRole,
            Role::SpinButton => NSAccessibilityUnknownRole,
            Role::Splitter => NSAccessibilityUnknownRole,
            Role::Status => NSAccessibilityUnknownRole,
            Role::Strong => NSAccessibilityUnknownRole,
            Role::Suggestion => NSAccessibilityUnknownRole,
            Role::SvgRoot => NSAccessibilityUnknownRole,
            Role::Tab => NSAccessibilityUnknownRole,
            Role::TabList => NSAccessibilityUnknownRole,
            Role::TabPanel => NSAccessibilityUnknownRole,
            Role::Term => NSAccessibilityUnknownRole,
            Role::TextFieldWithComboBox => NSAccessibilityUnknownRole,
            Role::Time => NSAccessibilityUnknownRole,
            Role::Timer => NSAccessibilityUnknownRole,
            Role::TitleBar => NSAccessibilityUnknownRole,
            Role::Toolbar => NSAccessibilityUnknownRole,
            Role::Tooltip => NSAccessibilityUnknownRole,
            Role::Tree => NSAccessibilityUnknownRole,
            Role::TreeGrid => NSAccessibilityUnknownRole,
            Role::Video => NSAccessibilityUnknownRole,
            Role::WebView => NSAccessibilityUnknownRole,
            // Use the group role for Role::Window, since the NSWindow
            // provides the top-level accessibility object for the window.
            Role::Window => NSAccessibilityGroupRole,
            Role::PdfActionableHighlight => NSAccessibilityUnknownRole,
            Role::PdfRoot => NSAccessibilityUnknownRole,
            Role::GraphicsDocument => NSAccessibilityUnknownRole,
            Role::GraphicsObject => NSAccessibilityUnknownRole,
            Role::GraphicsSymbol => NSAccessibilityUnknownRole,
            Role::DocAbstract => NSAccessibilityUnknownRole,
            Role::DocAcknowledgements => NSAccessibilityUnknownRole,
            Role::DocAfterword => NSAccessibilityUnknownRole,
            Role::DocAppendix => NSAccessibilityUnknownRole,
            Role::DocBackLink => NSAccessibilityUnknownRole,
            Role::DocBiblioEntry => NSAccessibilityUnknownRole,
            Role::DocBibliography => NSAccessibilityUnknownRole,
            Role::DocBiblioRef => NSAccessibilityUnknownRole,
            Role::DocChapter => NSAccessibilityUnknownRole,
            Role::DocColophon => NSAccessibilityUnknownRole,
            Role::DocConclusion => NSAccessibilityUnknownRole,
            Role::DocCover => NSAccessibilityUnknownRole,
            Role::DocCredit => NSAccessibilityUnknownRole,
            Role::DocCredits => NSAccessibilityUnknownRole,
            Role::DocDedication => NSAccessibilityUnknownRole,
            Role::DocEndnote => NSAccessibilityUnknownRole,
            Role::DocEndnotes => NSAccessibilityUnknownRole,
            Role::DocEpigraph => NSAccessibilityUnknownRole,
            Role::DocEpilogue => NSAccessibilityUnknownRole,
            Role::DocErrata => NSAccessibilityUnknownRole,
            Role::DocExample => NSAccessibilityUnknownRole,
            Role::DocFootnote => NSAccessibilityUnknownRole,
            Role::DocForeword => NSAccessibilityUnknownRole,
            Role::DocGlossary => NSAccessibilityUnknownRole,
            Role::DocGlossRef => NSAccessibilityUnknownRole,
            Role::DocIndex => NSAccessibilityUnknownRole,
            Role::DocIntroduction => NSAccessibilityUnknownRole,
            Role::DocNoteRef => NSAccessibilityUnknownRole,
            Role::DocNotice => NSAccessibilityUnknownRole,
            Role::DocPageBreak => NSAccessibilityUnknownRole,
            Role::DocPageFooter => NSAccessibilityUnknownRole,
            Role::DocPageHeader => NSAccessibilityUnknownRole,
            Role::DocPageList => NSAccessibilityUnknownRole,
            Role::DocPart => NSAccessibilityUnknownRole,
            Role::DocPreface => NSAccessibilityUnknownRole,
            Role::DocPrologue => NSAccessibilityUnknownRole,
            Role::DocPullquote => NSAccessibilityUnknownRole,
            Role::DocQna => NSAccessibilityUnknownRole,
            Role::DocSubtitle => NSAccessibilityUnknownRole,
            Role::DocTip => NSAccessibilityUnknownRole,
            Role::DocToc => NSAccessibilityUnknownRole,
            Role::ListGrid => NSAccessibilityUnknownRole,
        }
    }
}

fn get_size(_state: &State, node: &Node) -> id {
    if let Some(bounds) = &node.data().bounds {
        let ns_size = NSSize {
            width: bounds.rect.width as f64,
            height: bounds.rect.height as f64,
        };
        unsafe { NSValue::valueWithSize(nil, ns_size) }
    } else {
        nil
    }
}

static ATTRIBUTE_MAP: &[Attribute] = unsafe {
    &[
        Attribute(&NSAccessibilityParentAttribute, get_parent),
        Attribute(&NSAccessibilityPositionAttribute, get_position),
        Attribute(&NSAccessibilityRoleAttribute, get_role),
        Attribute(&NSAccessibilitySizeAttribute, get_size),
    ]
};

struct State {
    node: WeakNode,
    view: WeakPtr,
}

impl State {
    fn attribute_names(&self) -> id {
        let names = ATTRIBUTE_MAP
            .iter()
            .map(|Attribute(name_ptr, _)| unsafe { **name_ptr })
            .collect::<Vec<id>>();
        // TODO: role-specific attributes
        println!("returning attribute names {:?}", names);
        unsafe { NSArray::arrayWithObjects(nil, &names) }
    }

    fn attribute_value(&self, attribute_name: id) -> id {
        self.node
            .map(|node| {
                println!("get attribute value {}", from_nsstring(attribute_name));

                for Attribute(test_name_ptr, f) in ATTRIBUTE_MAP {
                    let equal: BOOL = unsafe {
                        let test_name: id = **test_name_ptr;
                        msg_send![attribute_name, isEqualToString: test_name]
                    };
                    if equal == YES {
                        return f(&self, &node);
                    }
                }

                nil
            })
            .unwrap_or(nil)
    }

    fn is_ignored(&self) -> BOOL {
        self.node
            .map(|node| {
                if node.is_invisible_or_ignored() {
                    YES
                } else {
                    NO
                }
            })
            .unwrap_or(YES)
    }
}

pub(crate) struct PlatformNode;

impl PlatformNode {
    pub(crate) fn new(node: &Node, view: &StrongPtr) -> StrongPtr {
        let state = Box::new(State {
            node: node.downgrade(),
            view: view.weak(),
        });
        unsafe {
            let object: id = msg_send![PLATFORM_NODE_CLASS.0, alloc];
            let () = msg_send![object, init];
            let state_ptr = Box::into_raw(state);
            (*object).set_ivar(STATE_IVAR, state_ptr as *mut c_void);
            StrongPtr::new(object)
        }
    }
}

static STATE_IVAR: &str = "accessKitPlatformNodeState";

struct PlatformNodeClass(*const Class);
unsafe impl Sync for PlatformNodeClass {}

lazy_static! {
    static ref PLATFORM_NODE_CLASS: PlatformNodeClass = unsafe {
        let mut decl = ClassDecl::new("AccessKitPlatformNode", class!(NSObject))
            .expect("platform node class definition failed");
        decl.add_ivar::<*mut c_void>(STATE_IVAR);

        // TODO: methods

        decl.add_method(sel!(accessibilityAttributeNames), attribute_names as extern "C" fn(&Object, Sel) -> id);
        extern "C" fn attribute_names(this: &Object, _sel: Sel) -> id {
            unsafe {
                let state: *mut c_void = *this.get_ivar(STATE_IVAR);
                let state = &mut *(state as *mut State);
                state.attribute_names()
            }
        }

        decl.add_method(sel!(accessibilityAttributeValue:), attribute_value as extern "C" fn(&Object, Sel, id) -> id);
        extern "C" fn attribute_value(this: &Object, _sel: Sel, attribute_name: id) -> id {
            unsafe {
                let state: *mut c_void = *this.get_ivar(STATE_IVAR);
                let state = &mut *(state as *mut State);
                state.attribute_value(attribute_name)
            }
        }

        decl.add_method(sel!(accessibilityIsIgnored), is_ignored as extern "C" fn(&Object, Sel) -> BOOL);
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
                Box::from_raw(state as *mut State); // drops the state
            }
        }

        PlatformNodeClass(decl.register())
    };
}

// Constants declared in AppKit
#[link(name = "AppKit", kind = "framework")]
extern "C" {
    // Attributes
    static NSAccessibilityParentAttribute: id;
    static NSAccessibilityPositionAttribute: id;
    static NSAccessibilityRoleAttribute: id;
    static NSAccessibilitySizeAttribute: id;

    // Roles
    static NSAccessibilityButtonRole: id;
    static NSAccessibilityCheckBoxRole: id;
    static NSAccessibilityCellRole: id;
    static NSAccessibilityColumnRole: id;
    static NSAccessibilityGroupRole: id;
    static NSAccessibilityImageRole: id;
    static NSAccessibilityLinkRole: id;
    static NSAccessibilityListRole: id;
    static NSAccessibilityMenuRole: id;
    static NSAccessibilityMenuItemRole: id;
    static NSAccessibilityRadioButtonRole: id;
    static NSAccessibilityRowRole: id;
    static NSAccessibilityStaticTextRole: id;
    static NSAccessibilityTableRole: id;
    static NSAccessibilityTextFieldRole: id;
    static NSAccessibilityUnknownRole: id;
}
