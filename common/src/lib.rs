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
use std::ops::Range;

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

/// An action to be taken on an accessibility node.
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

    /// Decrement a slider or range control by one step value.
    Decrement,
    /// Increment a slider or range control by one step value.
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
    /// (e.g. window).  /// Requires [`ActionRequest::data`] to be set to
    /// [`ActionData::ScrollToPoint`].
    ScrollToPoint,

    /// Requires [`ActionRequest::data`] to be set to [`ActionData::ScrollOffset`].
    SetScrollOffset,

    /// Requires [`ActionRequest::data`] to be set to [`ActionData::TextSelection`].
    SetTextSelection,

    /// Don't focus this node, but set it as the sequential focus navigation
    /// starting point, so that pressing Tab moves to the next element
    /// following this one, for example.
    SetSequentialFocusNavigationStartingPoint,

    /// Replace the value of the control with the specified value and
    /// reset the selection, if applicable. Requires [`ActionRequest::data`]
    /// to be set to [`ActionData::Value`].
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
    /// E.g. `aria-label`.
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

/// Function that can be performed when a dragged object is released
/// on a drop target.
/// Note: aria-dropeffect is deprecated in WAI-ARIA 1.1.
#[derive(EnumSetType, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "serde", enumset(serialize_as_list))]
pub enum DropEffect {
    Copy,
    Execute,
    Link,
    Move,
    Popup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum MarkerType {
    SpellingError,
    GrammarError,
    SearchMatch,
    ActiveSuggestion,
    Suggestion,
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

/// Indicates if a form control has invalid input or
/// if a web DOM element has an aria-invalid attribute.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum InvalidState {
    False,
    True,
    Other(Box<str>),
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
/// In contrast to [`Action`], these describe what the user can do on the
/// object, e.g. "press", not what happens to the object as a result.
/// Only one verb can be used at a time to describe the default action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum DefaultActionVerb {
    Activate,
    Check,
    Uncheck,
    Click,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum StringEncoding {
    Utf8,
    Utf16,
}

// This is NonZeroU64 because we regularly store Option<NodeId>.
pub type NodeIdContent = std::num::NonZeroU64;

/// The stable identity of a node, unique within the node's tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
pub struct NodeId(pub NodeIdContent);

/// The globally unique ID of a tree. The format of this ID
/// is up to the implementer. A UUID v4 is a safe choice.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
pub struct TreeId(pub Box<str>);

/// A marker spanning a range within text.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TextMarker {
    pub marker_type: MarkerType,
    /// Indices are in code units for the encoding specified in
    /// [`Tree::source_string_encoding`].
    pub range: Range<usize>,
}

/// Defines a custom action for a UI element. For example, a list UI
/// can allow a user to reorder items in the list by dragging the items.
#[derive(Clone, Debug, PartialEq)]
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

/// Offsets are in code units for the encoding specified in
/// [`Tree::source_string_encoding`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TextSelection {
    anchor_node: NodeId,
    anchor_offset: usize,
    focus_node: NodeId,
    focus_offset: usize,
}

/// A single accessible object. A complete UI is represented as a tree of these.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Node {
    pub id: NodeId,
    pub role: Role,
    /// An affine transform to apply to any coordinates within this node
    /// and its descendants, including the [`bounds`] field of this node.
    /// The combined transforms of this node and its ancestors define
    /// the coordinate space of this node. This field should be `None`
    /// if it would be set to the identity transform, which should be
    /// the case for most nodes.
    ///
    /// AccessKit is not opinionated about the resolution dependence
    /// or y direction of any coordinates; these requirements are
    /// ultimately determined by the platform accessibility API.
    /// This implies that for a tree generated by cross-platform code,
    /// a platform-specific layer between that cross-platform code
    /// and the AccessKit platform adapter must add an appropriate transform to
    /// the root node. However, AccessKit expects the final transformed
    /// coordinates to be relative to the origin of the tree's container
    /// (e.g. window).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub transform: Option<Box<Affine>>,
    /// The bounding box of this node, in the node's coordinate space.
    /// This field does not affect the coordinate space of either this node
    /// or its descendants; only the [`transform`] field affects that.
    /// This, along with the recommendation that most nodes should have `None`
    /// in their [`transform`] field, implies that the [`bounds`] field
    /// of most nodes should be in the coordinate space of the nearest ancestor
    /// with a non-`None` [`Transform`] field, or if there is no such ancestor,
    /// the tree's container (e.g. window).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bounds: Option<Rect>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub children: Vec<NodeId>,

    /// Unordered set of actions supported by this node.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "EnumSet::is_empty"))]
    pub actions: EnumSet<Action>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub name: Option<Box<str>>,
    /// What information was used to compute the object's name.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub name_from: Option<NameFrom>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub description: Option<Box<str>>,
    /// What information was used to compute the object's description.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub description_from: Option<DescriptionFrom>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub value: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub autofill_available: bool,
    /// Whether this node is expanded, collapsed, or neither. Setting this
    /// to false means the node is collapsed; omitting it means this state
    /// isn't applicable.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub expanded: Option<bool>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub default: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub editable: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub focusable: bool,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub orientation: Option<Orientation>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub hovered: bool,
    /// Skip over this node in the accessibility tree, but keep its subtree.
    /// This flag also indicates that this node, but not its descendants,
    /// should be skipped when hit testing.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub ignored: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub invisible: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub linked: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub multiline: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub multiselectable: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub protected: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub required: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub visited: bool,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub busy: bool,

    /// The object functions as a text field which exposes its descendants.
    /// Use cases include the root of a content-editable region, an ARIA
    /// textbox which isn't currently editable and which has interactive
    /// descendants, and a <body> element that has "design-mode" set to "on".
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub nonatomic_text_field_root: bool,

    // Live region attributes.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub container_live_atomic: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub container_live_busy: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub live_atomic: bool,

    /// If a dialog box is marked as explicitly modal
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub modal: bool,

    /// Set on a canvas element if it has fallback content.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub canvas_has_fallback: bool,

    /// Indicates this node is user-scrollable, e.g. overflow:scroll|auto, as
    /// opposed to only programmatically scrollable, like overflow:hidden, or
    /// not scrollable at all, e.g. overflow:visible.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub scrollable: bool,

    /// A hint to clients that the node is clickable.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub clickable: bool,

    /// Indicates that this node clips its children, i.e. may have
    /// overflow: hidden or clip children by default.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub clips_children: bool,

    /// Indicates that this node is not selectable because the style has
    /// user-select: none. Note that there may be other reasons why a node is
    /// not selectable - for example, bullets in a list. However, this attribute
    /// is only set on user-select: none.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub not_user_selectable_style: bool,

    /// Indicates whether this node is selected or unselected.
    /// The absence of this flag (as opposed to a false setting)
    /// means that the concept of "selected" doesn't apply.
    /// When deciding whether to set the flag to false or omit it,
    /// consider whether it would be appropriate for a screen reader
    /// to announce "not selected". The ambiguity of this flag
    /// in platform accessibility APIs has made extraneous
    /// "not selected" announcements a common annoyance.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub selected: Option<bool>,
    /// Indicates whether this node is selected due to selection follows focus.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub selected_from_focus: bool,

    /// Indicates whether this node can be grabbed for drag-and-drop operation.
    /// Setting this flag to false rather than omitting it means that
    /// this node is not currently grabbed but it can be.
    /// Note: aria-grabbed is deprecated in WAI-ARIA 1.1.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub grabbed: Option<bool>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "EnumSet::is_empty"))]
    pub drop_effects: EnumSet<DropEffect>,

    /// Indicates whether this node causes a hard line-break
    /// (e.g. block level elements, or <br>)
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub is_line_breaking_object: bool,
    /// Indicates whether this node causes a page break
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub is_page_breaking_object: bool,

    /// True if the node has any ARIA attributes set.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub has_aria_attribute: bool,

    /// This element allows touches to be passed through when a screen reader
    /// is in touch exploration mode, e.g. a virtual keyboard normally
    /// behaves this way.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub touch_pass_through: bool,

    /// Ids of nodes that are children of this node logically, but are
    /// not children of this node in the tree structure. As an example,
    /// a table cell is a child of a row, and an 'indirect' child of a
    /// column.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub indirect_children: Vec<NodeId>,

    // Relationships between this node and other nodes.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub active_descendant: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub error_message: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub in_page_link_target: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub member_of: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub next_on_line: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub previous_on_line: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub popup_for: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub controls: Vec<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub details: Vec<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub described_by: Vec<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub flow_to: Vec<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub labelled_by: Vec<NodeId>,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub radio_groups: Vec<NodeId>,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub markers: Box<[TextMarker]>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub text_direction: Option<TextDirection>,
    /// For inline text. This is the pixel position of the end of each
    /// character within the bounding rectangle of this object, in the
    /// direction given by [`Node::text_direction`]. For example, for left-to-right
    /// text, the first offset is the right coordinate of the first
    /// character within the object's bounds, the second offset
    /// is the right coordinate of the second character, and so on.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub character_offsets: Box<[f32]>,

    /// For inline text. The indices of each word, in code units for
    /// the encoding specified in [`Tree::source_string_encoding`].
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub words: Box<[Range<usize>]>,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_empty"))]
    pub custom_actions: Box<[CustomAction]>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub access_key: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub invalid_state: Option<InvalidState>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub auto_complete: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub checked_state: Option<CheckedState>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub checked_state_description: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub child_tree: Option<TreeId>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub class_name: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub container_live_relevant: Option<Box<str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub container_live_status: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub css_display: Option<Box<str>>,

    /// Only present when different from parent.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub font_family: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub html_tag: Option<Box<str>>,

    /// Inner HTML of an element. Only used for a top-level math element,
    /// to support third-party math accessibility products that parse MathML.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub inner_html: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub input_type: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub key_shortcuts: Option<Box<str>>,

    /// Only present when different from parent.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub language: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub live_relevant: Option<Box<str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub live_status: Option<Box<str>>,

    /// Only if not already exposed in [`Node::name`] ([`NameFrom::Placeholder`]).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub placeholder: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub aria_role: Option<Box<str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub role_description: Option<Box<str>>,

    /// Only if not already exposed in [`Node::name`] ([`NameFrom::Title`]).
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub tooltip: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub url: Option<Box<str>>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub default_action_verb: Option<DefaultActionVerb>,

    // Scrollable container attributes.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub scroll_x: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub scroll_x_min: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub scroll_x_max: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub scroll_y: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub scroll_y_min: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub scroll_y_max: Option<f32>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub text_selection: Option<TextSelection>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub aria_column_count: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub aria_cell_column_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub aria_cell_column_span: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub aria_row_count: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub aria_cell_row_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub aria_cell_row_span: Option<usize>,

    // Table attributes.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub table_row_count: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub table_column_count: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub table_header: Option<NodeId>,

    // Table row attributes.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub table_row_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub table_row_header: Option<NodeId>,

    // Table column attributes.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub table_column_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub table_column_header: Option<NodeId>,

    // Table cell attributes.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub table_cell_column_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub table_cell_column_span: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub table_cell_row_index: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub table_cell_row_span: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub sort_direction: Option<SortDirection>,

    /// Tree control attributes.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub hierarchical_level: Option<usize>,

    /// Use for a textbox that allows focus/selection but not input.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub read_only: bool,
    /// Use for a control or group of controls that disallows input.
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub disabled: bool,

    // Position or Number of items in current set of listitems or treeitems
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub set_size: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub pos_in_set: Option<usize>,

    /// For [`Role::ColorWell`], specifies the selected color in RGBA.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub color_value: Option<u32>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub aria_current: Option<AriaCurrent>,

    /// Background color in RGBA.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub background_color: Option<u32>,
    /// Foreground color in RGBA.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub foreground_color: Option<u32>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub has_popup: Option<HasPopup>,

    /// The list style type. Only available on list items.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub list_style: Option<ListStyle>,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub text_align: Option<TextAlign>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub vertical_offset: Option<VerticalOffset>,

    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub bold: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "is_false"))]
    pub italic: bool,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub overline: Option<TextDecoration>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub strikethrough: Option<TextDecoration>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub underline: Option<TextDecoration>,

    // Focus traversal order.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub previous_focus: Option<NodeId>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub next_focus: Option<NodeId>,

    // Range attributes.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub value_for_range: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub min_value_for_range: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub max_value_for_range: Option<f32>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub step_value_for_range: Option<f32>,

    // Text attributes.
    /// Font size is in pixels.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub font_size: Option<f32>,
    /// Font weight can take on any arbitrary numeric value. Increments of 100 in
    /// range [0, 900] represent keywords such as light, normal, bold, etc.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub font_weight: Option<f32>,
    /// The text indent of the text, in mm.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub text_indent: Option<f32>,
}

impl Node {
    pub fn new(id: NodeId, role: Role) -> Node {
        Node {
            id,
            role,
            transform: None,
            bounds: None,
            children: Default::default(),
            actions: EnumSet::new(),
            name: None,
            name_from: None,
            description: None,
            description_from: None,
            value: None,
            autofill_available: false,
            expanded: None,
            default: false,
            editable: false,
            focusable: false,
            orientation: None,
            hovered: false,
            ignored: false,
            invisible: false,
            linked: false,
            multiline: false,
            multiselectable: false,
            protected: false,
            required: false,
            visited: false,
            busy: false,
            nonatomic_text_field_root: false,
            container_live_atomic: false,
            container_live_busy: false,
            live_atomic: false,
            modal: false,
            canvas_has_fallback: false,
            scrollable: false,
            clickable: false,
            clips_children: false,
            not_user_selectable_style: false,
            selected: None,
            selected_from_focus: false,
            grabbed: None,
            drop_effects: EnumSet::new(),
            is_line_breaking_object: false,
            is_page_breaking_object: false,
            has_aria_attribute: false,
            touch_pass_through: false,
            indirect_children: Default::default(),
            active_descendant: None,
            error_message: None,
            in_page_link_target: None,
            member_of: None,
            next_on_line: None,
            previous_on_line: None,
            popup_for: None,
            controls: Default::default(),
            details: Default::default(),
            described_by: Default::default(),
            flow_to: Default::default(),
            labelled_by: Default::default(),
            radio_groups: Default::default(),
            markers: Default::default(),
            text_direction: None,
            character_offsets: Default::default(),
            words: Default::default(),
            custom_actions: Default::default(),
            access_key: None,
            invalid_state: None,
            auto_complete: None,
            checked_state: None,
            checked_state_description: None,
            child_tree: None,
            class_name: None,
            container_live_relevant: None,
            container_live_status: None,
            css_display: None,
            font_family: None,
            html_tag: None,
            inner_html: None,
            input_type: None,
            key_shortcuts: None,
            language: None,
            live_relevant: None,
            live_status: None,
            placeholder: None,
            aria_role: None,
            role_description: None,
            tooltip: None,
            url: None,
            default_action_verb: None,
            scroll_x: None,
            scroll_x_min: None,
            scroll_x_max: None,
            scroll_y: None,
            scroll_y_min: None,
            scroll_y_max: None,
            text_selection: None,
            aria_column_count: None,
            aria_cell_column_index: None,
            aria_cell_column_span: None,
            aria_row_count: None,
            aria_cell_row_index: None,
            aria_cell_row_span: None,
            table_row_count: None,
            table_column_count: None,
            table_header: None,
            table_row_index: None,
            table_row_header: None,
            table_column_index: None,
            table_column_header: None,
            table_cell_column_index: None,
            table_cell_column_span: None,
            table_cell_row_index: None,
            table_cell_row_span: None,
            sort_direction: None,
            hierarchical_level: None,
            read_only: false,
            disabled: false,
            set_size: None,
            pos_in_set: None,
            color_value: None,
            aria_current: None,
            background_color: None,
            foreground_color: None,
            has_popup: None,
            list_style: None,
            text_align: None,
            vertical_offset: None,
            bold: false,
            italic: false,
            overline: None,
            strikethrough: None,
            underline: None,
            previous_focus: None,
            next_focus: None,
            value_for_range: None,
            min_value_for_range: None,
            max_value_for_range: None,
            step_value_for_range: None,
            font_size: None,
            font_weight: None,
            text_indent: None,
        }
    }
}

/// The data associated with an accessibility tree that's global to the
/// tree and not associated with any particular node.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Tree {
    pub id: TreeId,
    pub root: NodeId,

    /// The string encoding used by the tree source. This is required
    /// to disambiguate string indices, e.g. in [`Node::words`].
    /// When the tree is serialized, the encoding specified by
    /// the target serialization format is always used. But this way,
    /// the tree source doesn't have to convert string indices;
    /// the platform adapter will do this if needed.
    pub source_string_encoding: StringEncoding,

    /// The ID of the tree that this tree is contained in, if any.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub parent: Option<TreeId>,

    /// The node that's used as the root scroller, if any. On some platforms
    /// like Android we need to ignore accessibility scroll offsets for
    /// that node and get them from the viewport instead.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub root_scroller: Option<NodeId>,
}

impl Tree {
    pub fn new(id: TreeId, root: NodeId, source_string_encoding: StringEncoding) -> Tree {
        Tree {
            id,
            root,
            source_string_encoding,
            parent: None,
            root_scroller: None,
        }
    }
}

/// A serializable representation of an atomic change to a tree.
/// The sender and receiver must be in sync; the update is only meant
/// to bring the tree from a specific previous state into its next state.
/// Trying to apply it to the wrong tree should immediately panic.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(crate = "serde"))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TreeUpdate {
    /// The optional ID of a node to clear, before applying any updates.
    /// Clearing a node means deleting all of its children and their descendants,
    /// but leaving that node in the tree. It's an error to clear a node but not
    /// subsequently update it as part of the same `TreeUpdate`.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub clear: Option<NodeId>,

    /// An ordered list of zero or more node updates to apply to the tree.
    ///
    /// Suppose that the next [`Node`] to be applied is `node`. The following
    /// invariants must hold:
    ///
    /// * Either:
    ///     1. `node.id` is already in the tree, or
    ///     2. the tree is empty, and `node` is the new root of the tree.
    /// * Every child ID in `node.children` must either be already a child
    ///   of this node, or a new ID not previously in the tree. It is not
    ///   allowed to "reparent" a child to this node without first removing
    ///   that child from its previous parent.
    /// * When a new ID appears in `node.children`, the tree should create a
    ///   new uninitialized placeholder node for it immediately. That
    ///   placeholder must be updated within the same `TreeUpdate`, otherwise
    ///   it's a fatal error. This guarantees the tree is always complete
    ///   before or after a `TreeUpdate`.
    pub nodes: Vec<Node>,

    /// Rarely updated information about the tree as a whole. This may be omitted
    /// if it has not changed since the previous update, but providing the same
    /// information again is also allowed. This is required when initializing
    /// a tree.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub tree: Option<Tree>,

    /// The node with keyboard focus within this tree, if any.
    /// If the focus is in a descendant tree, set this to the node
    /// to which that tree is anchored. The most recent focus, if any,
    /// must be provided with every tree update.
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
