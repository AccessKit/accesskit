// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

use enumset::{EnumSet, EnumSetType};
pub use kurbo;
use kurbo::{Affine, Point, Rect};
#[cfg(feature = "schemars")]
use schemars_lib as schemars;
#[cfg(feature = "schemars")]
use schemars_lib::JsonSchema;
#[cfg(feature = "serde")]
use serde_lib as serde;
#[cfg(feature = "serde")]
use serde_lib::{Deserialize, Serialize};
use std::{
    num::{NonZeroU128, NonZeroU64},
    sync::Arc,
};

/// The type of an accessibility node.
///
/// The majority of these roles come from the ARIA specification. Reference
/// the latest draft for proper usage.
///
/// Like the AccessKit schema as a whole, this list is largely taken
/// from Chromium. However, unlike Chromium's alphabetized list, this list
/// is ordered roughly by expected usage frequency (with the notable exception
/// of [`Role::Unknown`]). This is more efficient in serialization formats
/// where integers use a variable-length encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Role {
    Unknown,
    InlineTextBox,
    Cell,
    StaticText,
    Image,
    Link,
    Row,
    ListItem,

    /// Contains the bullet, number, or other marker for a list item.
    ListMarker,

    TreeItem,
    ListBoxOption,
    MenuItem,
    MenuListOption,
    Paragraph,
    GenericContainer,

    /// Used for ARIA role="none"/"presentation" -- ignored in platform tree.
    Presentation,

    CheckBox,
    RadioButton,
    TextField,
    Button,
    LabelText,
    Pane,
    RowHeader,
    ColumnHeader,
    Column,
    RowGroup,
    List,
    Table,
    TableHeaderContainer,
    LayoutTableCell,
    LayoutTableRow,
    LayoutTable,
    Switch,
    ToggleButton,
    Menu,

    Abbr,
    Alert,
    AlertDialog,
    Application,
    Article,
    Audio,
    Banner,
    Blockquote,
    Canvas,
    Caption,
    Caret,
    Client,
    Code,
    ColorWell,
    ComboBoxGrouping,
    ComboBoxMenuButton,
    Complementary,
    Comment,
    ContentDeletion,
    ContentInsertion,
    ContentInfo,
    Date,
    DateTime,
    Definition,
    DescriptionList,
    DescriptionListDetail,
    DescriptionListTerm,
    Details,
    Dialog,
    Directory,
    DisclosureTriangle,
    Document,
    EmbeddedObject,
    Emphasis,
    Feed,
    FigureCaption,
    Figure,
    Footer,
    FooterAsNonLandmark,
    Form,
    Grid,
    Group,
    Header,
    HeaderAsNonLandmark,
    Heading,
    Iframe,
    IframePresentational,
    ImeCandidate,
    InputTime,
    Keyboard,
    Legend,
    LineBreak,
    ListBox,
    Log,
    Main,
    Mark,
    Marquee,
    Math,
    MenuBar,
    MenuItemCheckBox,
    MenuItemRadio,
    MenuListPopup,
    Meter,
    Navigation,
    Note,
    PluginObject,
    PopupButton,
    Portal,
    Pre,
    ProgressIndicator,
    RadioGroup,
    Region,
    RootWebArea,
    Ruby,
    RubyAnnotation,
    ScrollBar,
    ScrollView,
    Search,
    SearchBox,
    Section,
    Slider,
    SpinButton,
    Splitter,
    Status,
    Strong,
    Suggestion,
    SvgRoot,
    Tab,
    TabList,
    TabPanel,
    Term,
    TextFieldWithComboBox,
    Time,
    Timer,
    TitleBar,
    Toolbar,
    Tooltip,
    Tree,
    TreeGrid,
    Video,
    WebView,
    Window,

    PdfActionableHighlight,
    PdfRoot,

    // ARIA Graphics module roles:
    // https://rawgit.com/w3c/graphics-aam/master/#mapping_role_table
    GraphicsDocument,
    GraphicsObject,
    GraphicsSymbol,

    // DPub Roles:
    // https://www.w3.org/TR/dpub-aam-1.0/#mapping_role_table
    DocAbstract,
    DocAcknowledgements,
    DocAfterword,
    DocAppendix,
    DocBackLink,
    DocBiblioEntry,
    DocBibliography,
    DocBiblioRef,
    DocChapter,
    DocColophon,
    DocConclusion,
    DocCover,
    DocCredit,
    DocCredits,
    DocDedication,
    DocEndnote,
    DocEndnotes,
    DocEpigraph,
    DocEpilogue,
    DocErrata,
    DocExample,
    DocFootnote,
    DocForeword,
    DocGlossary,
    DocGlossRef,
    DocIndex,
    DocIntroduction,
    DocNoteRef,
    DocNotice,
    DocPageBreak,
    DocPageFooter,
    DocPageHeader,
    DocPageList,
    DocPart,
    DocPreface,
    DocPrologue,
    DocPullquote,
    DocQna,
    DocSubtitle,
    DocTip,
    DocToc,

    /// Behaves similar to an ARIA grid but is primarily used by Chromium's
    /// `TableView` and its subclasses, so they can be exposed correctly
    /// on certain platforms.
    ListGrid,
}

impl Default for Role {
    fn default() -> Self {
        Self::Unknown
    }
}

/// An action to be taken on an accessibility node.
///
/// In contrast to [`DefaultActionVerb`], these describe what happens to the
/// object, e.g. "focus".
#[derive(EnumSetType, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "serde", enumset(serialize_as_list))]
pub enum Action {
    /// Do the default action for an object, typically this means "click".
    Default,

    Focus,
    Blur,

    Collapse,
    Expand,

    /// Requires [`ActionRequest::data`] to be set to [`ActionData::CustomAction`].
    CustomAction,

    /// Decrement a numeric value by one step.
    Decrement,
    /// Increment a numeric value by one step.
    Increment,

    HideTooltip,
    ShowTooltip,

    /// Request that the tree source invalidate its entire tree.
    InvalidateTree,

    /// Load inline text boxes for this subtree, providing information
    /// about word boundaries, line layout, and individual character
    /// bounding boxes.
    LoadInlineTextBoxes,

    /// Delete any selected text in the control's text value and
    /// insert the specified value in its place, like when typing or pasting.
    /// Requires [`ActionRequest::data`] to be set to [`ActionData::Value`].
    ReplaceSelectedText,

    // Scrolls by approximately one screen in a specific direction. Should be
    // called on a node that has scrollable boolean set to true.
    // TBD: Do we need a doc comment on each of the values below?
    // Or does this awkwardness suggest a refactor?
    ScrollBackward,
    ScrollDown,
    ScrollForward,
    ScrollLeft,
    ScrollRight,
    ScrollUp,

    /// Scroll any scrollable containers to make the target object visible
    /// on the screen.  Optionally set [`ActionRequest::data`] to
    /// [`ActionData::ScrollTargetRect`].
    ScrollIntoView,

    /// Scroll the given object to a specified point in the tree's container
    /// (e.g. window). Requires [`ActionRequest::data`] to be set to
    /// [`ActionData::ScrollToPoint`].
    ScrollToPoint,

    /// Requires [`ActionRequest::data`] to be set to [`ActionData::SetScrollOffset`].
    SetScrollOffset,

    /// Requires [`ActionRequest::data`] to be set to [`ActionData::SetTextSelection`].
    SetTextSelection,

    /// Don't focus this node, but set it as the sequential focus navigation
    /// starting point, so that pressing Tab moves to the next element
    /// following this one, for example.
    SetSequentialFocusNavigationStartingPoint,

    /// Replace the value of the control with the specified value and
    /// reset the selection, if applicable. Requires [`ActionRequest::data`]
    /// to be set to [`ActionData::Value`] or [`ActionData::NumericValue`].
    SetValue,

    ShowContextMenu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Orientation {
    /// E.g. most toolbars and separators.
    Horizontal,
    /// E.g. menu or combo box.
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum NameFrom {
    /// E.g. [`aria-label`].
    ///
    /// [`aria-label`]: https://www.w3.org/TR/wai-aria-1.1/#aria-label
    Attribute,
    AttributeExplicitlyEmpty,
    /// E.g. in the case of a table, from a `caption` element.
    Caption,
    Contents,
    /// E.g. from an HTML placeholder attribute on a text field.
    Placeholder,
    /// E.g. from a `figcaption` element in a figure.
    RelatedElement,
    /// E.g. `<input type="text" title="title">`.
    Title,
    /// E.g. `<input type="button" value="Button's name">`.
    Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum DescriptionFrom {
    AriaDescription,
    /// HTML-AAM 5.2.2
    ButtonLabel,
    RelatedElement,
    RubyAnnotation,
    /// HTML-AAM 5.8.2
    Summary,
    /// HTML-AAM 5.9.2
    TableCaption,
    Title,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum TextDirection {
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

/// Indicates if a form control has invalid input or if a web DOM element has an
/// [`aria-invalid`] attribute.
///
/// [`aria-invalid`]: https://www.w3.org/TR/wai-aria-1.1/#aria-invalid
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Invalid {
    True,
    Grammar,
    Spelling,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum CheckedState {
    False,
    True,
    Mixed,
}

/// Describes the action that will be performed on a given node when
/// executing the default action, which is a click.
///
/// In contrast to [`Action`], these describe what the user can do on the
/// object, e.g. "press", not what happens to the object as a result.
/// Only one verb can be used at a time to describe the default action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum DefaultActionVerb {
    Click,
    Focus,
    Check,
    Uncheck,
    /// A click will be performed on one of the node's ancestors.
    /// This happens when the node itself is not clickable, but one of its
    /// ancestors has click handlers attached which are able to capture the click
    /// as it bubbles up.
    ClickAncestor,
    Jump,
    Open,
    Press,
    Select,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum SortDirection {
    Unsorted,
    Ascending,
    Descending,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum AriaCurrent {
    False,
    True,
    Page,
    Step,
    Location,
    Date,
    Time,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Live {
    Off,
    Polite,
    Assertive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum HasPopup {
    True,
    Menu,
    Listbox,
    Tree,
    Grid,
    Dialog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum ListStyle {
    Circle,
    Disc,
    Image,
    Numeric,
    Square,
    /// Language specific ordering (alpha, roman, cjk-ideographic, etc...)
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum VerticalOffset {
    Subscript,
    Superscript,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum TextDecoration {
    Solid,
    Dotted,
    Dashed,
    Double,
    Wavy,
}

// This is NonZeroU128 because we regularly store Option<NodeId>.
// 128-bit to handle UUIDs.
pub type NodeIdContent = NonZeroU128;

/// The stable identity of a [`Node`], unique within the node's tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
pub struct NodeId(pub NodeIdContent);

impl From<NonZeroU64> for NodeId {
    fn from(inner: NonZeroU64) -> Self {
        Self(inner.into())
    }
}

/// Defines a custom action for a UI element.
///
/// For example, a list UI can allow a user to reorder items in the list by dragging the
/// items.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CustomAction {
    pub id: i32,
    pub description: Box<str>,
}

// Helper for skipping false values in serialization.
#[cfg(feature = "serde")]
fn is_false(b: &bool) -> bool {
    !b
}

// Helper for skipping empty slices in serialization.
#[cfg(feature = "serde")]
fn is_empty<T>(slice: &[T]) -> bool {
    slice.is_empty()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TextPosition {
    /// The node's role must be [`Role::InlineTextBox`].
    pub node: NodeId,
    /// The index of an item in [`Node::character_lengths`], or the length
    /// of that slice if the position is at the end of the line.
    pub character_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TextSelection {
    /// The position where the selection started, and which does not change
    /// as the selection is expanded or contracted. If there is no selection
    /// but only a caret, this must be equal to [`focus`]. This is also known
    /// as a degenerate selection.
    pub anchor: TextPosition,
    /// The active end of the selection, which changes as the selection
    /// is expanded or contracted, or the position of the caret if there is
    /// no selection.
    pub focus: TextPosition,
}

/// A single accessible object. A complete UI is represented as a tree of these.
///
/// For brevity, and to make more of the documentation usable in bindings
/// to other languages, documentation of getter methods is written as if
/// documenting fields in a struct, and such methods are referred to
/// as properties.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Node {
    role: Role,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    transform: Option<Box<Affine>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    bounds: Option<Rect>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    children: Vec<NodeId>,

    /// Unordered set of actions supported by this node.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "EnumSet::is_empty"))]
    actions: EnumSet<Action>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    name: Option<Box<str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    name_from: Option<NameFrom>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    description: Option<Box<str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    description_from: Option<DescriptionFrom>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    value: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    autofill_available: bool,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    expanded: Option<bool>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    default: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    editable: bool,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    orientation: Option<Orientation>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    hovered: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    hidden: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    linked: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    multiline: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    multiselectable: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    protected: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    required: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    visited: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    busy: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    nonatomic_text_field_root: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    live_atomic: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    modal: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    canvas_has_fallback: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    scrollable: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    clips_children: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    not_user_selectable_style: bool,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    selected: Option<bool>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    selected_from_focus: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    is_line_breaking_object: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    is_page_breaking_object: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    has_aria_attribute: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    touch_pass_through: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    indirect_children: Vec<NodeId>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    active_descendant: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    error_message: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    in_page_link_target: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    member_of: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    next_on_line: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    previous_on_line: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    popup_for: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    controls: Vec<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    details: Vec<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    described_by: Vec<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    flow_to: Vec<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    labelled_by: Vec<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    radio_group: Vec<NodeId>,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    is_spelling_error: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    is_grammar_error: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    is_search_match: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    is_suggestion: bool,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    text_direction: Option<TextDirection>,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    character_lengths: Box<[u8]>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    character_positions: Option<Box<[f32]>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    character_widths: Option<Box<[f32]>>,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    word_lengths: Box<[u8]>,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    custom_actions: Vec<CustomAction>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    access_key: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    invalid: Option<Invalid>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    auto_complete: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    checked_state: Option<CheckedState>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    checked_state_description: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    class_name: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    css_display: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    font_family: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    html_tag: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    inner_html: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    input_type: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    key_shortcuts: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    language: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    live_relevant: Option<Box<str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    live: Option<Live>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    placeholder: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    aria_role: Option<Box<str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    role_description: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    tooltip: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    url: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    default_action_verb: Option<DefaultActionVerb>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    scroll_x: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    scroll_x_min: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    scroll_x_max: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    scroll_y: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    scroll_y_min: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    scroll_y_max: Option<f32>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    text_selection: Option<TextSelection>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    aria_column_count: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    aria_cell_column_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    aria_cell_column_span: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    aria_row_count: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    aria_cell_row_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    aria_cell_row_span: Option<usize>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    table_row_count: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    table_column_count: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    table_header: Option<NodeId>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    table_row_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    table_row_header: Option<NodeId>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    table_column_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    table_column_header: Option<NodeId>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    table_cell_column_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    table_cell_column_span: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    table_cell_row_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    table_cell_row_span: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    sort_direction: Option<SortDirection>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    hierarchical_level: Option<usize>,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    read_only: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    disabled: bool,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    size_of_set: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    position_in_set: Option<usize>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    color_value: Option<u32>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    aria_current: Option<AriaCurrent>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    background_color: Option<u32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    foreground_color: Option<u32>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    has_popup: Option<HasPopup>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    list_style: Option<ListStyle>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    text_align: Option<TextAlign>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    vertical_offset: Option<VerticalOffset>,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    bold: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    italic: bool,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    overline: Option<TextDecoration>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    strikethrough: Option<TextDecoration>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    underline: Option<TextDecoration>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    previous_focus: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    next_focus: Option<NodeId>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    numeric_value: Option<f64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    min_numeric_value: Option<f64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    max_numeric_value: Option<f64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    numeric_value_step: Option<f64>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    numeric_value_jump: Option<f64>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    font_size: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    font_weight: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    text_indent: Option<f32>,
}

impl Node {
    pub fn new(role: Role) -> Self {
        Self {
            role,
            ..Default::default()
        }
    }

    pub fn role(&self) -> Role {
        self.role
    }
    pub fn set_role(&mut self, value: Role) {
        self.role = value;
    }

    /// An affine transform to apply to any coordinates within this node
    /// and its descendants, including the [`bounds`] property of this node.
    /// The combined transforms of this node and its ancestors define
    /// the coordinate space of this node. /// This should be `None` if
    /// it would be set to the identity transform, which should be the case
    /// for most nodes.
    ///
    /// AccessKit expects the final transformed coordinates to be relative
    /// to the origin of the tree's container (e.g. window), in physical
    /// pixels, with the y coordinate being top-down.
    ///
    /// [`bounds`]: NodeProvider::bounds
    pub fn transform(&self) -> Option<Affine> {
        self.transform.as_ref().map(|v| **v)
    }
    pub fn set_transform(&mut self, value: Affine) {
        self.transform = Some(Box::new(value));
    }
    pub fn clear_transform(&mut self) {
        self.transform = None;
    }

    /// The bounding box of this node, in the node's coordinate space.
    /// This property does not affect the coordinate space of either this node
    /// or its descendants; only the [`transform`] property affects that.
    /// This, along with the recommendation that most nodes should have
    /// a [`transform`] of `None`, implies that the `bounds` property
    /// of most nodes should be in the coordinate space of the nearest ancestor
    /// with a non-`None` [`transform`], or if there is no such ancestor,
    /// the tree's container (e.g. window).
    ///
    /// [`transform`]: NodeProvider::transform
    pub fn bounds(&self) -> Option<Rect> {
        self.bounds
    }
    pub fn set_bounds(&mut self, value: Rect) {
        self.bounds = Some(value);
    }
    pub fn clear_bounds(&mut self) {
        self.bounds = None;
    }

    pub fn children(&self) -> &[NodeId] {
        &self.children
    }
    pub fn set_children(&mut self, value: impl Into<Vec<NodeId>>) {
        self.children = value.into();
    }
    pub fn push_child(&mut self, id: NodeId) {
        self.children.push(id);
    }
    pub fn clear_children(&mut self) {
        self.children.clear();
    }

    pub fn supports_action(&self, action: Action) -> bool {
        self.actions.contains(action)
    }
    pub fn add_action(&mut self, action: Action) {
        self.actions.insert(action);
    }
    pub fn remove_action(&mut self, action: Action) {
        self.actions.remove(action);
    }
    pub fn clear_actions(&mut self) {
        self.actions.clear();
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
    pub fn set_name(&mut self, value: impl Into<Box<str>>) {
        self.name = Some(value.into());
    }
    pub fn clear_name(&mut self) {
        self.name = None;
    }

    /// What information was used to compute the object's name.
    pub fn name_from(&self) -> Option<NameFrom> {
        self.name_from
    }
    pub fn set_name_from(&mut self, value: NameFrom) {
        self.name_from = Some(value);
    }
    pub fn clear_name_from(&mut self) {
        self.name_from = None;
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
    pub fn set_description(&mut self, value: impl Into<Box<str>>) {
        self.description = Some(value.into());
    }
    pub fn clear_description(&mut self) {
        self.description = None;
    }

    /// What information was used to compute the object's description.
    pub fn description_from(&self) -> Option<DescriptionFrom> {
        self.description_from
    }
    pub fn set_description_from(&mut self, value: DescriptionFrom) {
        self.description_from = Some(value);
    }
    pub fn clear_description_from(&mut self) {
        self.description_from = None;
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
    pub fn set_value(&mut self, value: impl Into<Box<str>>) {
        self.value = Some(value.into());
    }
    pub fn clear_value(&mut self) {
        self.value = None;
    }

    pub fn is_autofill_available(&self) -> bool {
        self.autofill_available
    }
    pub fn set_autofill_available(&mut self, value: bool) {
        self.autofill_available = value;
    }

    /// Whether this node is expanded, collapsed, or neither.
    ///
    /// Setting this to `false` means the node is collapsed; omitting it means this state
    /// isn't applicable.
    pub fn is_expanded(&self) -> Option<bool> {
        self.expanded
    }
    pub fn set_expanded(&mut self, value: bool) {
        self.expanded = Some(value);
    }
    pub fn clear_expanded(&mut self) {
        self.expanded = None;
    }

    pub fn is_default(&self) -> bool {
        self.default
    }
    pub fn set_default(&mut self, value: bool) {
        self.default = value;
    }

    pub fn is_editable(&self) -> bool {
        self.editable
    }
    pub fn set_editable(&mut self, value: bool) {
        self.editable = value;
    }

    pub fn orientation(&self) -> Option<Orientation> {
        self.orientation
    }
    pub fn set_orientation(&mut self, value: Orientation) {
        self.orientation = Some(value);
    }
    pub fn clear_orientation(&mut self) {
        self.orientation = None;
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }
    pub fn set_hovered(&mut self, value: bool) {
        self.hovered = value;
    }

    /// Exclude this node and its descendants from the tree presented to
    /// assistive technologies, and from hit testing.
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }
    pub fn set_hidden(&mut self, value: bool) {
        self.hidden = value;
    }

    pub fn is_linked(&self) -> bool {
        self.linked
    }
    pub fn set_linked(&mut self, value: bool) {
        self.linked = value;
    }

    pub fn is_multiline(&self) -> bool {
        self.multiline
    }
    pub fn set_multiline(&mut self, value: bool) {
        self.multiline = value;
    }

    pub fn is_multiselectable(&self) -> bool {
        self.multiselectable
    }
    pub fn set_multiselectable(&mut self, value: bool) {
        self.multiselectable = value;
    }

    pub fn is_protected(&self) -> bool {
        self.protected
    }
    pub fn set_protected(&mut self, value: bool) {
        self.protected = value;
    }

    pub fn is_required(&self) -> bool {
        self.required
    }
    pub fn set_required(&mut self, value: bool) {
        self.required = value;
    }

    pub fn is_visited(&self) -> bool {
        self.visited
    }
    pub fn set_visited(&mut self, value: bool) {
        self.visited = value;
    }

    pub fn is_busy(&self) -> bool {
        self.busy
    }
    pub fn set_busy(&mut self, value: bool) {
        self.busy = value;
    }

    /// The object functions as a text field which exposes its descendants.
    ///
    /// Use cases include the root of a content-editable region, an ARIA
    /// textbox which isn't currently editable and which has interactive
    /// descendants, and a `<body>` element that has "design-mode" set to "on".
    pub fn is_nonatomic_text_field_root(&self) -> bool {
        self.nonatomic_text_field_root
    }
    pub fn set_nonatomic_text_field_root(&mut self, value: bool) {
        self.nonatomic_text_field_root = value;
    }

    pub fn is_live_atomic(&self) -> bool {
        self.live_atomic
    }
    pub fn set_live_atomic(&mut self, value: bool) {
        self.live_atomic = value;
    }

    /// If a dialog box is marked as explicitly modal.
    pub fn is_modal(&self) -> bool {
        self.modal
    }
    pub fn set_modal(&mut self, value: bool) {
        self.modal = value;
    }

    /// Set on a canvas element if it has fallback content.
    pub fn canvas_has_fallback(&self) -> bool {
        self.canvas_has_fallback
    }
    pub fn set_canvas_has_fallback(&mut self, value: bool) {
        self.canvas_has_fallback = value;
    }

    /// Indicates this node is user-scrollable, e.g. `overflow: scroll|auto`, as
    /// opposed to only programmatically scrollable, like `overflow: hidden`, or
    /// not scrollable at all, e.g. `overflow: visible`.
    pub fn is_scrollable(&self) -> bool {
        self.scrollable
    }
    pub fn set_scrollable(&mut self, value: bool) {
        self.scrollable = value;
    }

    /// Indicates that this node clips its children, i.e. may have
    /// `overflow: hidden` or clip children by default.
    pub fn clips_children(&self) -> bool {
        self.clips_children
    }
    pub fn set_clips_children(&mut self, value: bool) {
        self.clips_children = value;
    }

    /// Indicates that this node is not selectable because the style has
    /// `user-select: none`. Note that there may be other reasons why a node is
    /// not selectable - for example, bullets in a list. However, this attribute
    /// is only set on `user-select: none`.
    pub fn is_not_user_selectable_style(&self) -> bool {
        self.not_user_selectable_style
    }
    pub fn set_not_user_selectable_style(&mut self, value: bool) {
        self.not_user_selectable_style = value;
    }

    /// Indicates whether this node is selected or unselected.
    ///
    /// The absence of this flag (as opposed to a `false` setting)
    /// means that the concept of "selected" doesn't apply.
    /// When deciding whether to set the flag to false or omit it,
    /// consider whether it would be appropriate for a screen reader
    /// to announce "not selected". The ambiguity of this flag
    /// in platform accessibility APIs has made extraneous
    /// "not selected" announcements a common annoyance.
    pub fn is_selected(&self) -> Option<bool> {
        self.selected
    }
    pub fn set_selected(&mut self, value: bool) {
        self.selected = Some(value);
    }
    pub fn clear_selected(&mut self) {
        self.selected = None;
    }

    /// Indicates whether this node is selected due to selection follows focus.
    pub fn is_selected_from_focus(&self) -> bool {
        self.selected_from_focus
    }
    pub fn set_selected_from_focus(&mut self, value: bool) {
        self.selected_from_focus = value;
    }

    /// Indicates whether this node causes a hard line-break
    /// (e.g. block level elements, or `<br>`).
    pub fn is_line_breaking_object(&self) -> bool {
        self.is_line_breaking_object
    }
    pub fn set_is_line_breaking_object(&mut self, value: bool) {
        self.is_line_breaking_object = value;
    }

    /// Indicates whether this node causes a page break.
    pub fn is_page_breaking_object(&self) -> bool {
        self.is_page_breaking_object
    }
    pub fn set_is_page_breaking_object(&mut self, value: bool) {
        self.is_page_breaking_object = value;
    }

    /// True if the node has any ARIA attributes set.
    pub fn has_aria_attribute(&self) -> bool {
        self.has_aria_attribute
    }
    pub fn set_has_aria_attribute(&mut self, value: bool) {
        self.has_aria_attribute = value;
    }

    /// This element allows touches to be passed through when a screen reader
    /// is in touch exploration mode, e.g. a virtual keyboard normally
    /// behaves this way.
    pub fn is_touch_pass_through(&self) -> bool {
        self.touch_pass_through
    }
    pub fn set_touch_pass_through(&mut self, value: bool) {
        self.touch_pass_through = value;
    }

    /// Ids of nodes that are children of this node logically, but are
    /// not children of this node in the tree structure. As an example,
    /// a table cell is a child of a row, and an 'indirect' child of a
    /// column.
    pub fn indirect_children(&self) -> &[NodeId] {
        &self.indirect_children
    }
    pub fn set_indirect_children(&mut self, value: impl Into<Vec<NodeId>>) {
        self.indirect_children = value.into();
    }
    pub fn push_indirect_child(&mut self, id: NodeId) {
        self.indirect_children.push(id);
    }
    pub fn clear_indirect_children(&mut self) {
        self.indirect_children.clear();
    }

    // Relationships between this node and other nodes.

    pub fn active_descendant(&self) -> Option<NodeId> {
        self.active_descendant
    }
    pub fn set_active_descendant(&mut self, value: NodeId) {
        self.active_descendant = Some(value);
    }
    pub fn clear_active_descendant(&mut self) {
        self.active_descendant = None;
    }

    pub fn error_message(&self) -> Option<NodeId> {
        self.error_message
    }
    pub fn set_error_message(&mut self, value: NodeId) {
        self.error_message = Some(value);
    }
    pub fn clear_error_message(&mut self) {
        self.error_message = None;
    }

    pub fn in_page_link_target(&self) -> Option<NodeId> {
        self.in_page_link_target
    }
    pub fn set_in_page_link_target(&mut self, value: NodeId) {
        self.in_page_link_target = Some(value);
    }
    pub fn clear_in_page_link_target(&mut self) {
        self.in_page_link_target = None;
    }

    pub fn member_of(&self) -> Option<NodeId> {
        self.member_of
    }
    pub fn set_member_of(&mut self, value: NodeId) {
        self.member_of = Some(value);
    }
    pub fn clear_member_of(&mut self) {
        self.member_of = None;
    }

    pub fn next_on_line(&self) -> Option<NodeId> {
        self.next_on_line
    }
    pub fn set_next_on_line(&mut self, value: NodeId) {
        self.next_on_line = Some(value);
    }
    pub fn clear_next_on_line(&mut self) {
        self.next_on_line = None;
    }

    pub fn previous_on_line(&self) -> Option<NodeId> {
        self.previous_on_line
    }
    pub fn set_previous_on_line(&mut self, value: NodeId) {
        self.previous_on_line = Some(value);
    }
    pub fn clear_previous_on_line(&mut self) {
        self.previous_on_line = None;
    }

    pub fn popup_for(&self) -> Option<NodeId> {
        self.popup_for
    }
    pub fn set_popup_for(&mut self, value: NodeId) {
        self.popup_for = Some(value);
    }
    pub fn clear_popup_for(&mut self) {
        self.popup_for = None;
    }

    pub fn controls(&self) -> &[NodeId] {
        &self.controls
    }
    pub fn set_controls(&mut self, value: impl Into<Vec<NodeId>>) {
        self.controls = value.into();
    }
    pub fn push_controls(&mut self, id: NodeId) {
        self.controls.push(id);
    }
    pub fn clear_controls(&mut self) {
        self.controls.clear();
    }

    pub fn details(&self) -> &[NodeId] {
        &self.details
    }
    pub fn set_details(&mut self, value: impl Into<Vec<NodeId>>) {
        self.details = value.into();
    }
    pub fn push_details(&mut self, id: NodeId) {
        self.details.push(id);
    }
    pub fn clear_details(&mut self) {
        self.details.clear();
    }

    pub fn described_by(&self) -> &[NodeId] {
        &self.described_by
    }
    pub fn set_described_by(&mut self, value: impl Into<Vec<NodeId>>) {
        self.described_by = value.into();
    }
    pub fn push_described_by(&mut self, id: NodeId) {
        self.described_by.push(id);
    }
    pub fn clear_described_by(&mut self) {
        self.described_by.clear();
    }

    pub fn flow_to(&self) -> &[NodeId] {
        &self.flow_to
    }
    pub fn set_flow_to(&mut self, value: impl Into<Vec<NodeId>>) {
        self.flow_to = value.into();
    }
    pub fn push_flow_to(&mut self, id: NodeId) {
        self.flow_to.push(id);
    }
    pub fn clear_flow_to(&mut self) {
        self.flow_to.clear();
    }

    pub fn labelled_by(&self) -> &[NodeId] {
        &self.labelled_by
    }
    pub fn set_labelled_by(&mut self, value: impl Into<Vec<NodeId>>) {
        self.labelled_by = value.into();
    }
    pub fn push_labelled_by(&mut self, id: NodeId) {
        self.labelled_by.push(id);
    }
    pub fn clear_labelled_by(&mut self) {
        self.labelled_by.clear();
    }

    /// On radio buttons this should be set to a list of all of the buttons
    /// in the same group as this one, including this radio button itself.
    pub fn radio_group(&self) -> &[NodeId] {
        &self.radio_group
    }
    pub fn set_radio_group(&mut self, value: impl Into<Vec<NodeId>>) {
        self.radio_group = value.into();
    }
    pub fn push_to_radio_group(&mut self, id: NodeId) {
        self.radio_group.push(id);
    }
    pub fn clear_radio_group(&mut self) {
        self.radio_group.clear();
    }

    pub fn is_spelling_error(&self) -> bool {
        self.is_spelling_error
    }
    pub fn set_is_spelling_error(&mut self, value: bool) {
        self.is_spelling_error = value;
    }

    pub fn is_grammar_error(&self) -> bool {
        self.is_grammar_error
    }
    pub fn set_is_grammar_error(&mut self, value: bool) {
        self.is_grammar_error = value;
    }

    pub fn is_search_match(&self) -> bool {
        self.is_search_match
    }
    pub fn set_is_search_match(&mut self, value: bool) {
        self.is_search_match = value;
    }

    pub fn is_suggestion(&self) -> bool {
        self.is_suggestion
    }
    pub fn set_is_suggestion(&mut self, value: bool) {
        self.is_suggestion = value;
    }

    pub fn text_direction(&self) -> Option<TextDirection> {
        self.text_direction
    }
    pub fn set_text_direction(&mut self, value: TextDirection) {
        self.text_direction = Some(value);
    }
    pub fn clear_text_direction(&mut self) {
        self.text_direction = None;
    }

    /// For inline text. The length (non-inclusive) of each character
    /// in UTF-8 code units (bytes). The sum of these lengths must equal
    /// the length of [`value`], also in bytes.
    ///
    /// A character is defined as the smallest unit of text that
    /// can be selected. This isn't necessarily a single Unicode
    /// scalar value (code point). This is why AccessKit can't compute
    /// the lengths of the characters from the text itself; this information
    /// must be provided by the text editing implementation.
    ///
    /// If this node is the last text box in a line that ends with a hard
    /// line break, that line break should be included at the end of this
    /// node's value as either a CRLF or LF; in both cases, the line break
    /// should be counted as a single character for the sake of this slice.
    /// When the caret is at the end of such a line, the focus of the text
    /// selection should be on the line break, not after it.
    ///
    /// [`value`]: NodeProvider::value
    pub fn character_lengths(&self) -> &[u8] {
        &self.character_lengths
    }
    pub fn set_character_lengths(&mut self, value: impl Into<Box<[u8]>>) {
        self.character_lengths = value.into();
    }

    /// For inline text. This is the position of each character within
    /// the node's bounding box, in the direction given by
    /// [`text_direction`], in the coordinate space of this node.
    ///
    /// When present, the length of this slice should be the same as the length
    /// of [`character_lengths`], including for lines that end
    /// with a hard line break. The position of such a line break should
    /// be the position where an end-of-paragraph marker would be rendered.
    ///
    /// This property is optional. Without it, AccessKit can't support some
    /// use cases, such as screen magnifiers that track the caret position
    /// or screen readers that display a highlight cursor. However,
    /// most text functionality still works without this information.
    ///
    /// [`text_direction`]: NodeProvider::text_direction
    /// [`character_lengths`]: NodeProvider::character_lengths
    pub fn character_positions(&self) -> Option<&[f32]> {
        self.character_positions.as_deref()
    }
    pub fn set_character_positions(&mut self, value: impl Into<Box<[f32]>>) {
        self.character_positions = Some(value.into());
    }
    pub fn clear_character_positions(&mut self) {
        self.character_positions = None;
    }

    /// For inline text. This is the advance width of each character,
    /// in the direction given by [`text_direction`], in the coordinate
    /// space of this node.
    ///
    /// When present, the length of this slice should be the same as the length
    /// of [`character_lengths`], including for lines that end
    /// with a hard line break. The width of such a line break should
    /// be non-zero if selecting the line break by itself results in
    /// a visible highlight (as in Microsoft Word), or zero if not
    /// (as in Windows Notepad).
    ///
    /// This property is optional. Without it, AccessKit can't support some
    /// use cases, such as screen magnifiers that track the caret position
    /// or screen readers that display a highlight cursor. However,
    /// most text functionality still works without this information.
    ///
    /// [`text_direction`]: NodeProvider::text_direction
    /// [`character_lengths`]: NodeProvider::character_lengths
    pub fn character_widths(&self) -> Option<&[f32]> {
        self.character_widths.as_deref()
    }
    pub fn set_character_widths(&mut self, value: impl Into<Box<[f32]>>) {
        self.character_widths = Some(value.into());
    }
    pub fn clear_character_widths(&mut self) {
        self.character_widths = None;
    }

    /// For inline text. The length of each word in characters, as defined
    /// in [`character_lengths`]. The sum of these lengths must equal
    /// the length of [`character_lengths`].
    ///
    /// The end of each word is the beginning of the next word; there are no
    /// characters that are not considered part of a word. Trailing whitespace
    /// is typically considered part of the word that precedes it, while
    /// a line's leading whitespace is considered its own word. Whether
    /// punctuation is considered a separate word or part of the preceding
    /// word depends on the particular text editing implementation.
    /// Some editors may have their own definition of a word; for example,
    /// in an IDE, words may correspond to programming language tokens.
    ///
    /// Not all assistive technologies require information about word
    /// boundaries, and not all platform accessibility APIs even expose
    /// this information, but for assistive technologies that do use
    /// this information, users will get unpredictable results if the word
    /// boundaries exposed by the accessibility tree don't match
    /// the editor's behavior. This is why AccessKit does not determine
    /// word boundaries itself.
    ///
    /// [`character_lengths`]: NodeProvider::character_lengths
    pub fn word_lengths(&self) -> &[u8] {
        &self.word_lengths
    }
    pub fn set_word_lengths(&mut self, value: impl Into<Box<[u8]>>) {
        self.word_lengths = value.into();
    }

    pub fn custom_actions(&self) -> &[CustomAction] {
        &self.custom_actions
    }
    pub fn set_custom_actions(&mut self, value: impl Into<Vec<CustomAction>>) {
        self.custom_actions = value.into();
    }
    pub fn push_custom_action(&mut self, action: CustomAction) {
        self.custom_actions.push(action);
    }
    pub fn clear_custom_actions(&mut self) {
        self.custom_actions.clear();
    }

    pub fn access_key(&self) -> Option<&str> {
        self.access_key.as_deref()
    }
    pub fn set_access_key(&mut self, value: impl Into<Box<str>>) {
        self.access_key = Some(value.into());
    }
    pub fn clear_access_key(&mut self) {
        self.access_key = None;
    }

    pub fn invalid(&self) -> Option<Invalid> {
        self.invalid
    }
    pub fn set_invalid(&mut self, value: Invalid) {
        self.invalid = Some(value);
    }
    pub fn clear_invalid(&mut self) {
        self.invalid = None;
    }

    pub fn auto_complete(&self) -> Option<&str> {
        self.auto_complete.as_deref()
    }
    pub fn set_auto_complete(&mut self, value: impl Into<Box<str>>) {
        self.auto_complete = Some(value.into());
    }
    pub fn clear_auto_complete(&mut self) {
        self.auto_complete = None;
    }

    pub fn checked_state(&self) -> Option<CheckedState> {
        self.checked_state
    }
    pub fn set_checked_state(&mut self, value: CheckedState) {
        self.checked_state = Some(value);
    }
    pub fn clear_checked_state(&mut self) {
        self.checked_state = None;
    }

    pub fn checked_state_description(&self) -> Option<&str> {
        self.checked_state_description.as_deref()
    }
    pub fn set_checked_state_description(&mut self, value: impl Into<Box<str>>) {
        self.checked_state_description = Some(value.into());
    }
    pub fn clear_checked_state_description(&mut self) {
        self.checked_state_description = None;
    }

    pub fn class_name(&self) -> Option<&str> {
        self.class_name.as_deref()
    }
    pub fn set_class_name(&mut self, value: impl Into<Box<str>>) {
        self.class_name = Some(value.into());
    }
    pub fn clear_class_name(&mut self) {
        self.class_name = None;
    }

    pub fn css_display(&self) -> Option<&str> {
        self.css_display.as_deref()
    }
    pub fn set_css_display(&mut self, value: impl Into<Box<str>>) {
        self.css_display = Some(value.into());
    }
    pub fn clear_css_display(&mut self) {
        self.css_display = None;
    }

    /// Only present when different from parent.
    pub fn font_family(&self) -> Option<&str> {
        self.font_family.as_deref()
    }
    pub fn set_font_family(&mut self, value: impl Into<Box<str>>) {
        self.font_family = Some(value.into());
    }
    pub fn clear_font_family(&mut self) {
        self.font_family = None;
    }

    pub fn html_tag(&self) -> Option<&str> {
        self.html_tag.as_deref()
    }
    pub fn set_html_tag(&mut self, value: impl Into<Box<str>>) {
        self.html_tag = Some(value.into());
    }
    pub fn clear_html_tag(&mut self) {
        self.html_tag = None;
    }

    /// Inner HTML of an element. Only used for a top-level math element,
    /// to support third-party math accessibility products that parse MathML.
    pub fn inner_html(&self) -> Option<&str> {
        self.inner_html.as_deref()
    }
    pub fn set_inner_html(&mut self, value: impl Into<Box<str>>) {
        self.inner_html = Some(value.into());
    }
    pub fn clear_inner_html(&mut self) {
        self.inner_html = None;
    }

    pub fn input_type(&self) -> Option<&str> {
        self.input_type.as_deref()
    }
    pub fn set_input_type(&mut self, value: impl Into<Box<str>>) {
        self.input_type = Some(value.into());
    }
    pub fn clear_input_type(&mut self) {
        self.input_type = None;
    }

    pub fn key_shortcuts(&self) -> Option<&str> {
        self.key_shortcuts.as_deref()
    }
    pub fn set_key_shortcuts(&mut self, value: impl Into<Box<str>>) {
        self.key_shortcuts = Some(value.into());
    }
    pub fn clear_key_shortcuts(&mut self) {
        self.key_shortcuts = None;
    }

    /// Only present when different from parent.
    pub fn language(&self) -> Option<&str> {
        self.language.as_deref()
    }
    pub fn set_language(&mut self, value: impl Into<Box<str>>) {
        self.language = Some(value.into());
    }
    pub fn clear_language(&mut self) {
        self.language = None;
    }

    pub fn live_relevant(&self) -> Option<&str> {
        self.live_relevant.as_deref()
    }
    pub fn set_live_relevant(&mut self, value: impl Into<Box<str>>) {
        self.live_relevant = Some(value.into());
    }
    pub fn clear_live_relevant(&mut self) {
        self.live_relevant = None;
    }

    pub fn live(&self) -> Option<Live> {
        self.live
    }
    pub fn set_live(&mut self, value: Live) {
        self.live = Some(value);
    }
    pub fn clear_live(&mut self) {
        self.live = None;
    }

    /// Only if not already exposed in [`name`] ([`NameFrom::Placeholder`]).
    ///
    /// [`name`]: NodeProvider::name
    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }
    pub fn set_placeholder(&mut self, value: impl Into<Box<str>>) {
        self.placeholder = Some(value.into());
    }
    pub fn clear_placeholder(&mut self) {
        self.placeholder = None;
    }

    pub fn aria_role(&self) -> Option<&str> {
        self.aria_role.as_deref()
    }
    pub fn set_aria_role(&mut self, value: impl Into<Box<str>>) {
        self.aria_role = Some(value.into());
    }
    pub fn clear_aria_role(&mut self) {
        self.aria_role = None;
    }

    pub fn role_description(&self) -> Option<&str> {
        self.role_description.as_deref()
    }
    pub fn set_role_description(&mut self, value: impl Into<Box<str>>) {
        self.role_description = Some(value.into());
    }
    pub fn clear_role_description(&mut self) {
        self.role_description = None;
    }

    /// Only if not already exposed in [`name`] ([`NameFrom::Title`]).
    ///
    /// [`name`]: NodeProvider::name
    pub fn tooltip(&self) -> Option<&str> {
        self.tooltip.as_deref()
    }
    pub fn set_tooltip(&mut self, value: impl Into<Box<str>>) {
        self.tooltip = Some(value.into());
    }
    pub fn clear_tooltip(&mut self) {
        self.tooltip = None;
    }

    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }
    pub fn set_url(&mut self, value: impl Into<Box<str>>) {
        self.url = Some(value.into());
    }
    pub fn clear_url(&mut self) {
        self.url = None;
    }

    pub fn default_action_verb(&self) -> Option<DefaultActionVerb> {
        self.default_action_verb
    }
    pub fn set_default_action_verb(&mut self, value: DefaultActionVerb) {
        self.default_action_verb = Some(value);
    }
    pub fn clear_default_action_verb(&mut self) {
        self.default_action_verb = None;
    }

    // Scrollable container attributes.

    pub fn scroll_x(&self) -> Option<f32> {
        self.scroll_x
    }
    pub fn set_scroll_x(&mut self, value: f32) {
        self.scroll_x = Some(value);
    }
    pub fn clear_scroll_x(&mut self) {
        self.scroll_x = None;
    }

    pub fn scroll_x_min(&self) -> Option<f32> {
        self.scroll_x_min
    }
    pub fn set_scroll_x_min(&mut self, value: f32) {
        self.scroll_x_min = Some(value);
    }
    pub fn clear_scroll_x_min(&mut self) {
        self.scroll_x_min = None;
    }

    pub fn scroll_x_max(&self) -> Option<f32> {
        self.scroll_x_max
    }
    pub fn set_scroll_x_max(&mut self, value: f32) {
        self.scroll_x_max = Some(value);
    }
    pub fn clear_scroll_x_max(&mut self) {
        self.scroll_x_max = None;
    }

    pub fn scroll_y(&self) -> Option<f32> {
        self.scroll_y
    }
    pub fn set_scroll_y(&mut self, value: f32) {
        self.scroll_y = Some(value);
    }
    pub fn clear_scroll_y(&mut self) {
        self.scroll_y = None;
    }

    pub fn scroll_y_min(&self) -> Option<f32> {
        self.scroll_y_min
    }
    pub fn set_scroll_y_min(&mut self, value: f32) {
        self.scroll_y_min = Some(value);
    }
    pub fn clear_scroll_y_min(&mut self) {
        self.scroll_y_min = None;
    }

    pub fn scroll_y_max(&self) -> Option<f32> {
        self.scroll_y_max
    }
    pub fn set_scroll_y_max(&mut self, value: f32) {
        self.scroll_y_max = Some(value);
    }
    pub fn clear_scroll_y_max(&mut self) {
        self.scroll_y_max = None;
    }

    pub fn text_selection(&self) -> Option<TextSelection> {
        self.text_selection
    }
    pub fn set_text_selection(&mut self, value: TextSelection) {
        self.text_selection = Some(value);
    }
    pub fn clear_text_selection(&mut self) {
        self.text_selection = None;
    }

    pub fn aria_column_count(&self) -> Option<usize> {
        self.aria_column_count
    }
    pub fn set_aria_column_count(&mut self, value: usize) {
        self.aria_column_count = Some(value);
    }
    pub fn clear_aria_column_count(&mut self) {
        self.aria_column_count = None;
    }

    pub fn aria_cell_column_index(&self) -> Option<usize> {
        self.aria_cell_column_index
    }
    pub fn set_aria_cell_column_index(&mut self, value: usize) {
        self.aria_cell_column_index = Some(value);
    }
    pub fn clear_aria_cell_column_index(&mut self) {
        self.aria_cell_column_index = None;
    }

    pub fn aria_cell_column_span(&self) -> Option<usize> {
        self.aria_cell_column_span
    }
    pub fn set_aria_cell_column_span(&mut self, value: usize) {
        self.aria_cell_column_span = Some(value);
    }
    pub fn clear_aria_cell_column_span(&mut self) {
        self.aria_cell_column_span = None;
    }

    pub fn aria_row_count(&self) -> Option<usize> {
        self.aria_row_count
    }
    pub fn set_aria_row_count(&mut self, value: usize) {
        self.aria_row_count = Some(value);
    }
    pub fn clear_aria_row_count(&mut self) {
        self.aria_row_count = None;
    }

    pub fn aria_cell_row_index(&self) -> Option<usize> {
        self.aria_cell_row_index
    }
    pub fn set_aria_cell_row_index(&mut self, value: usize) {
        self.aria_cell_row_index = Some(value);
    }
    pub fn clear_aria_cell_row_index(&mut self) {
        self.aria_cell_row_index = None;
    }

    pub fn aria_cell_row_span(&self) -> Option<usize> {
        self.aria_cell_row_span
    }
    pub fn set_aria_cell_row_span(&mut self, value: usize) {
        self.aria_cell_row_span = Some(value);
    }
    pub fn clear_aria_cell_row_span(&mut self) {
        self.aria_cell_row_span = None;
    }

    // Table attributes.

    pub fn table_row_count(&self) -> Option<usize> {
        self.table_row_count
    }
    pub fn set_table_row_count(&mut self, value: usize) {
        self.table_row_count = Some(value);
    }
    pub fn clear_table_row_count(&mut self) {
        self.table_row_count = None;
    }

    pub fn table_column_count(&self) -> Option<usize> {
        self.table_column_count
    }
    pub fn set_table_column_count(&mut self, value: usize) {
        self.table_column_count = Some(value);
    }
    pub fn clear_table_column_count(&mut self) {
        self.table_column_count = None;
    }

    pub fn table_header(&self) -> Option<NodeId> {
        self.table_header
    }
    pub fn set_table_header(&mut self, value: NodeId) {
        self.table_header = Some(value);
    }
    pub fn clear_table_header(&mut self) {
        self.table_header = None;
    }

    // Table row attributes.

    pub fn table_row_index(&self) -> Option<usize> {
        self.table_row_index
    }
    pub fn set_table_row_index(&mut self, value: usize) {
        self.table_row_index = Some(value);
    }
    pub fn clear_table_row_index(&mut self) {
        self.table_row_index = None;
    }

    pub fn table_row_header(&self) -> Option<NodeId> {
        self.table_row_header
    }
    pub fn set_table_row_header(&mut self, value: NodeId) {
        self.table_row_header = Some(value);
    }
    pub fn clear_table_row_header(&mut self) {
        self.table_row_header = None;
    }

    // Table column attributes.

    pub fn table_column_index(&self) -> Option<usize> {
        self.table_column_index
    }
    pub fn set_table_column_index(&mut self, value: usize) {
        self.table_column_index = Some(value);
    }
    pub fn clear_table_column_index(&mut self) {
        self.table_column_index = None;
    }

    pub fn table_column_header(&self) -> Option<NodeId> {
        self.table_column_header
    }
    pub fn set_table_column_header(&mut self, value: NodeId) {
        self.table_column_header = Some(value);
    }
    pub fn clear_table_column_header(&mut self) {
        self.table_column_header = None;
    }

    // Table cell attributes.

    pub fn table_cell_column_index(&self) -> Option<usize> {
        self.table_cell_column_index
    }
    pub fn set_table_cell_column_index(&mut self, value: usize) {
        self.table_cell_column_index = Some(value);
    }
    pub fn clear_table_cell_column_index(&mut self) {
        self.table_cell_column_index = None;
    }

    pub fn table_cell_column_span(&self) -> Option<usize> {
        self.table_cell_column_span
    }
    pub fn set_table_cell_column_span(&mut self, value: usize) {
        self.table_cell_column_span = Some(value);
    }
    pub fn clear_table_cell_column_span(&mut self) {
        self.table_cell_column_span = None;
    }

    pub fn table_cell_row_index(&self) -> Option<usize> {
        self.table_cell_row_index
    }
    pub fn set_table_cell_row_index(&mut self, value: usize) {
        self.table_cell_row_index = Some(value);
    }
    pub fn clear_table_cell_row_index(&mut self) {
        self.table_cell_row_index = None;
    }

    pub fn table_cell_row_span(&self) -> Option<usize> {
        self.table_cell_row_span
    }
    pub fn set_table_cell_row_span(&mut self, value: usize) {
        self.table_cell_row_span = Some(value);
    }
    pub fn clear_table_cell_row_span(&mut self) {
        self.table_cell_row_span = None;
    }

    pub fn sort_direction(&self) -> Option<SortDirection> {
        self.sort_direction
    }
    pub fn set_sort_direction(&mut self, value: SortDirection) {
        self.sort_direction = Some(value);
    }
    pub fn clear_sort_direction(&mut self) {
        self.sort_direction = None;
    }

    /// Tree control attributes.
    pub fn hierarchical_level(&self) -> Option<usize> {
        self.hierarchical_level
    }
    pub fn set_hierarchical_level(&mut self, value: usize) {
        self.hierarchical_level = Some(value);
    }
    pub fn clear_hierarchical_level(&mut self) {
        self.hierarchical_level = None;
    }

    /// Use for a textbox that allows focus/selection but not input.
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
    pub fn set_read_only(&mut self, value: bool) {
        self.read_only = value;
    }

    /// Use for a control or group of controls that disallows input.
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
    pub fn set_disabled(&mut self, value: bool) {
        self.disabled = value;
    }

    // Position or Number of items in current set of listitems or treeitems

    pub fn size_of_set(&self) -> Option<usize> {
        self.size_of_set
    }
    pub fn set_size_of_set(&mut self, value: usize) {
        self.size_of_set = Some(value);
    }
    pub fn clear_size_of_set(&mut self) {
        self.size_of_set = None;
    }

    pub fn position_in_set(&self) -> Option<usize> {
        self.position_in_set
    }
    pub fn set_position_in_set(&mut self, value: usize) {
        self.position_in_set = Some(value);
    }
    pub fn clear_position_in_set(&mut self) {
        self.position_in_set = None;
    }

    /// For [`Role::ColorWell`], specifies the selected color in RGBA.
    pub fn color_value(&self) -> Option<u32> {
        self.color_value
    }
    pub fn set_color_value(&mut self, value: u32) {
        self.color_value = Some(value);
    }
    pub fn clear_color_value(&mut self) {
        self.color_value = None;
    }

    pub fn aria_current(&self) -> Option<AriaCurrent> {
        self.aria_current
    }
    pub fn set_aria_current(&mut self, value: AriaCurrent) {
        self.aria_current = Some(value);
    }
    pub fn clear_aria_current(&mut self) {
        self.aria_current = None;
    }

    /// Background color in RGBA.
    pub fn background_color(&self) -> Option<u32> {
        self.background_color
    }
    pub fn set_background_color(&mut self, value: u32) {
        self.background_color = Some(value);
    }
    pub fn clear_background_color(&mut self) {
        self.background_color = None;
    }

    /// Foreground color in RGBA.
    pub fn foreground_color(&self) -> Option<u32> {
        self.foreground_color
    }
    pub fn set_foreground_color(&mut self, value: u32) {
        self.foreground_color = Some(value);
    }
    pub fn clear_foreground_color(&mut self) {
        self.foreground_color = None;
    }

    pub fn has_popup(&self) -> Option<HasPopup> {
        self.has_popup
    }

    /// The list style type. Only available on list items.
    pub fn list_style(&self) -> Option<ListStyle> {
        self.list_style
    }
    pub fn set_list_style(&mut self, value: ListStyle) {
        self.list_style = Some(value);
    }
    pub fn clear_list_style(&mut self) {
        self.list_style = None;
    }

    pub fn text_align(&self) -> Option<TextAlign> {
        self.text_align
    }
    pub fn set_text_align(&mut self, value: TextAlign) {
        self.text_align = Some(value);
    }
    pub fn clear_text_align(&mut self) {
        self.text_align = None;
    }

    pub fn vertical_offset(&self) -> Option<VerticalOffset> {
        self.vertical_offset
    }
    pub fn set_vertical_offset(&mut self, value: VerticalOffset) {
        self.vertical_offset = Some(value);
    }
    pub fn clear_vertical_offset(&mut self) {
        self.vertical_offset = None;
    }

    pub fn is_bold(&self) -> bool {
        self.bold
    }
    pub fn set_bold(&mut self, value: bool) {
        self.bold = value;
    }

    pub fn is_italic(&self) -> bool {
        self.italic
    }
    pub fn set_italic(&mut self, value: bool) {
        self.italic = value;
    }

    pub fn overline(&self) -> Option<TextDecoration> {
        self.overline
    }
    pub fn set_overline(&mut self, value: TextDecoration) {
        self.overline = Some(value);
    }
    pub fn clear_overline(&mut self) {
        self.overline = None;
    }

    pub fn strikethrough(&self) -> Option<TextDecoration> {
        self.strikethrough
    }
    pub fn set_strikethrough(&mut self, value: TextDecoration) {
        self.strikethrough = Some(value);
    }
    pub fn clear_strikethrough(&mut self) {
        self.strikethrough = None;
    }

    pub fn underline(&self) -> Option<TextDecoration> {
        self.underline
    }
    pub fn set_underline(&mut self, value: TextDecoration) {
        self.underline = Some(value);
    }
    pub fn clear_underline(&mut self) {
        self.underline = None;
    }

    // Focus traversal order.

    pub fn previous_focus(&self) -> Option<NodeId> {
        self.previous_focus
    }
    pub fn set_previous_focus(&mut self, value: NodeId) {
        self.previous_focus = Some(value);
    }
    pub fn clear_previous_focus(&mut self) {
        self.previous_focus = None;
    }

    pub fn next_focus(&self) -> Option<NodeId> {
        self.next_focus
    }
    pub fn set_next_focus(&mut self, value: NodeId) {
        self.next_focus = Some(value);
    }
    pub fn clear_next_focus(&mut self) {
        self.next_focus = None;
    }

    // Numeric value attributes.

    pub fn numeric_value(&self) -> Option<f64> {
        self.numeric_value
    }
    pub fn set_numeric_value(&mut self, value: f64) {
        self.numeric_value = Some(value);
    }
    pub fn clear_numeric_value(&mut self) {
        self.numeric_value = None;
    }

    pub fn min_numeric_value(&self) -> Option<f64> {
        self.min_numeric_value
    }
    pub fn set_min_numeric_value(&mut self, value: f64) {
        self.min_numeric_value = Some(value);
    }
    pub fn clear_min_numeric_value(&mut self) {
        self.min_numeric_value = None;
    }

    pub fn max_numeric_value(&self) -> Option<f64> {
        self.max_numeric_value
    }
    pub fn set_max_numeric_value(&mut self, value: f64) {
        self.max_numeric_value = Some(value);
    }
    pub fn clear_max_numeric_value(&mut self) {
        self.max_numeric_value = None;
    }

    pub fn numeric_value_step(&self) -> Option<f64> {
        self.numeric_value_step
    }
    pub fn set_numeric_value_step(&mut self, value: f64) {
        self.numeric_value_step = Some(value);
    }
    pub fn clear_numeric_value_step(&mut self) {
        self.numeric_value_step = None;
    }

    pub fn numeric_value_jump(&self) -> Option<f64> {
        self.numeric_value_jump
    }
    pub fn set_numeric_value_jump(&mut self, value: f64) {
        self.numeric_value_jump = Some(value);
    }
    pub fn clear_numeric_value_jump(&mut self) {
        self.numeric_value_jump = None;
    }

    /// Font size is in pixels.
    pub fn font_size(&self) -> Option<f32> {
        self.font_size
    }
    pub fn set_font_size(&mut self, value: f32) {
        self.font_size = Some(value);
    }
    pub fn clear_font_size(&mut self) {
        self.font_size = None;
    }

    /// Font weight can take on any arbitrary numeric value. Increments of 100 in
    /// range `[0, 900]` represent keywords such as light, normal, bold, etc.
    pub fn font_weight(&self) -> Option<f32> {
        self.font_weight
    }
    pub fn set_font_weight(&mut self, value: f32) {
        self.font_weight = Some(value);
    }
    pub fn clear_font_weight(&mut self) {
        self.font_weight = None;
    }

    /// The text indent of the text, in mm.
    pub fn text_indent(&self) -> Option<f32> {
        self.text_indent
    }
    pub fn set_text_indent(&mut self, value: f32) {
        self.text_indent = Some(value);
    }
    pub fn clear_text_indent(&mut self) {
        self.text_indent = None;
    }
}

/// The data associated with an accessibility tree that's global to the
/// tree and not associated with any particular node.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Tree {
    pub root: NodeId,

    /// The node that's used as the root scroller, if any. On some platforms
    /// like Android we need to ignore accessibility scroll offsets for
    /// that node and get them from the viewport instead.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub root_scroller: Option<NodeId>,
}

impl Tree {
    pub fn new(root: NodeId) -> Tree {
        Tree {
            root,
            root_scroller: None,
        }
    }
}

/// A serializable representation of an atomic change to a [`Tree`].
///
/// The sender and receiver must be in sync; the update is only meant
/// to bring the tree from a specific previous state into its next state.
/// Trying to apply it to the wrong tree should immediately panic.
///
/// Note that for performance, an update should only include nodes that are
/// new or changed. AccessKit platform adapters will avoid raising extraneous
/// events for nodes that have not changed since the previous update,
/// but there is still a cost in processing these nodes and replacing
/// the previous instances.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TreeUpdate {
    /// Zero or more new or updated nodes. Order doesn't matter.
    ///
    /// Each node in this list will overwrite any existing node with the same ID.
    /// This means that when updating a node, fields that are unchanged
    /// from the previous version must still be set to the same values
    /// as before.
    ///
    /// It is an error for any node in this list to not be either the root
    /// or a child of another node. For nodes other than the root, the parent
    /// must be either an unchanged node already in the tree, or another node
    /// in this list.
    ///
    /// To add a child to the tree, the list must include both the child
    /// and an updated version of the parent with the child's ID added to
    /// [`Node::children`].
    ///
    /// To remove a child and all of its descendants, this list must include
    /// an updated version of the parent node with the child's ID removed
    /// from [`Node::children`]. Neither the child nor any of its descendants
    /// may be included in this list.
    pub nodes: Vec<(NodeId, Arc<Node>)>,

    /// Rarely updated information about the tree as a whole. This may be omitted
    /// if it has not changed since the previous update, but providing the same
    /// information again is also allowed. This is required when initializing
    /// a tree.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub tree: Option<Tree>,

    /// The node with keyboard focus within this tree, if any.
    /// The most recent focus, if any,must be provided with every tree update.
    ///
    /// This field must contain a value if and only if the native host
    /// (e.g. window) currently has the keyboard focus. This implies
    /// that the AccessKit provider must track the native focus state
    /// and send matching tree updates. Rationale: A robust GUI toolkit
    /// must do this native focus tracking anyway in order to correctly
    /// render widgets (e.g. to draw or not draw a focus rectangle),
    /// so this focus tracking should not be duplicated between the toolkit
    /// and the AccessKit platform adapters.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub focus: Option<NodeId>,
}

impl<T: FnOnce() -> TreeUpdate> From<T> for TreeUpdate {
    fn from(factory: T) -> Self {
        factory()
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum ActionData {
    CustomAction(i32),
    Value(Box<str>),
    NumericValue(f64),
    /// Optional target rectangle for [`Action::ScrollIntoView`], in
    /// the coordinate space of the action's target node.
    ScrollTargetRect(Rect),
    /// Target for [`Action::ScrollToPoint`], in platform-native coordinates
    /// relative to the origin of the tree's container (e.g. window).
    ScrollToPoint(Point),
    /// Target for [`Action::SetScrollOffset`], in the coordinate space
    /// of the action's target node.
    SetScrollOffset(Point),
    SetTextSelection(TextSelection),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ActionRequest {
    pub action: Action,
    pub target: NodeId,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub data: Option<ActionData>,
}

/// Handles requests from assistive technologies or other clients.
pub trait ActionHandler: Send + Sync {
    /// Perform the requested action. If the requested action is not supported,
    /// this method must do nothing.
    ///
    /// This method may be called on any thread. In particular, on platforms
    /// with a designated UI thread, this method may or may not be called
    /// on that thread. Implementations must correctly handle both cases.
    ///
    /// This method may queue the request and handle it asynchronously.
    /// This behavior is preferred over blocking, e.g. when dispatching
    /// the request to another thread.
    fn do_action(&self, request: ActionRequest);
}
