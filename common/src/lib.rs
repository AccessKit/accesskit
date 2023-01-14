// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{
    de::{Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq, Serializer},
    Deserialize, Serialize,
};
use std::num::{NonZeroU128, NonZeroU64};
#[cfg(feature = "serde")]
use std::{fmt, mem::size_of_val};

mod geometry;
pub use geometry::{Affine, Point, Rect, Size, Vec2};

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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, enumn::N))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[repr(u8)]
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

impl Action {
    fn mask(self) -> u32 {
        1 << (self as u8)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
struct Actions(u32);

#[cfg(feature = "serde")]
impl Serialize for Actions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(None)?;
        for i in 0..((size_of_val(&self.0) as u8) * 8) {
            if let Some(action) = Action::n(i) {
                if (self.0 & action.mask()) != 0 {
                    seq.serialize_element(&action)?;
                }
            }
        }
        seq.end()
    }
}

#[cfg(feature = "serde")]
struct ActionsVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for ActionsVisitor {
    type Value = Actions;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("action set")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Actions, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut actions = Actions::default();
        while let Some(action) = seq.next_element::<Action>()? {
            actions.0 |= action.mask();
        }
        Ok(actions)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Actions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ActionsVisitor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Invalid {
    True,
    Grammar,
    Spelling,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum Live {
    Off,
    Polite,
    Assertive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub enum VerticalOffset {
    Subscript,
    Superscript,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct CustomAction {
    pub id: i32,
    pub description: Box<str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, enumn::N))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[repr(u8)]
enum Flag {
    AutofillAvailable,
    Default,
    Editable,
    Hovered,
    Hidden,
    Linked,
    Multiline,
    Multiselectable,
    Protected,
    Required,
    Visited,
    Busy,
    LiveAtomic,
    Modal,
    Scrollable,
    SelectedFromFocus,
    TouchPassThrough,
    ReadOnly,
    Disabled,
    Bold,
    Italic,
    CanvasHasFallback,
    ClipsChildren,
    IsLineBreakingObject,
    IsPageBreakingObject,
    IsSpellingError,
    IsGrammarError,
    IsSearchMatch,
    IsSuggestion,
    IsNonatomicTextFieldRoot,
}

impl Flag {
    fn mask(self) -> u32 {
        1 << (self as u8)
    }
}

// The following is based on the technique described here:
// https://viruta.org/reducing-memory-consumption-in-librsvg-2.html

#[derive(Clone, Debug, PartialEq)]
enum PropertyValue {
    None,
    NodeIdVec(Vec<NodeId>),
    NodeId(NodeId),
    String(Box<str>),
    F64(f64),
    Usize(usize),
    Color(u32),
    LengthSlice(Box<[u8]>),
    CoordSlice(Box<[f32]>),
    Affine(Box<Affine>),
    Rect(Rect),
    TextSelection(Box<TextSelection>),
    CustomActionVec(Vec<CustomAction>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize, enumn::N))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[repr(u8)]
enum PropertyId {
    // NodeIdVec
    Children,
    IndirectChildren,
    Controls,
    Details,
    DescribedBy,
    FlowTo,
    LabelledBy,
    RadioGroup,

    // NodeId
    ActiveDescendant,
    ErrorMessage,
    InPageLinkTarget,
    MemberOf,
    NextOnLine,
    PreviousOnLine,
    PopupFor,
    TableHeader,
    TableRowHeader,
    TableColumnHeader,
    NextFocus,
    PreviousFocus,

    // String
    Name,
    Description,
    Value,
    AccessKey,
    AutoComplete,
    CheckedStateDescription,
    ClassName,
    CssDisplay,
    FontFamily,
    HtmlTag,
    InnerHtml,
    InputType,
    KeyShortcuts,
    Language,
    LiveRelevant,
    Placeholder,
    AriaRole,
    RoleDescription,
    Tooltip,
    Url,

    // f64
    ScrollX,
    ScrollXMin,
    ScrollXMax,
    ScrollY,
    ScrollYMin,
    ScrollYMax,
    NumericValue,
    MinNumericValue,
    MaxNumericValue,
    NumericValueStep,
    NumericValueJump,
    FontSize,
    FontWeight,
    TextIndent,

    // usize
    TableRowCount,
    TableColumnCount,
    TableRowIndex,
    TableColumnIndex,
    TableCellColumnIndex,
    TableCellColumnSpan,
    TableCellRowIndex,
    TableCellRowSpan,
    HierarchicalLevel,
    SizeOfSet,
    PositionInSet,

    // Color
    ColorValue,
    BackgroundColor,
    ForegroundColor,

    // LengthSlice
    CharacterLengths,
    WordLengths,

    // CoordSlice
    CharacterPositions,
    CharacterWidths,

    // Other
    Transform,
    Bounds,
    TextSelection,
    CustomActions,

    // This MUST be last.
    Unset,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
struct PropertyIndices([u8; PropertyId::Unset as usize]);

impl Default for PropertyIndices {
    fn default() -> Self {
        Self([PropertyId::Unset as u8; PropertyId::Unset as usize])
    }
}

macro_rules! flag_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        impl Node {
            $($(#[$doc])*
            pub fn $getter(&self) -> bool {
                (self.flags & (Flag::$id).mask()) != 0
            }
            pub fn $setter(&mut self) {
                self.flags |= (Flag::$id).mask();
            }
            pub fn $clearer(&mut self) {
                self.flags &= !((Flag::$id).mask());
            })*
        }
    }
}

macro_rules! optional_bool_methods {
    ($($(#[$doc:meta])* ($field:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        impl Node {
            $($(#[$doc])*
            pub fn $getter(&self) -> Option<bool> {
                self.$field
            }
            pub fn $setter(&mut self, value: bool) {
                self.$field = Some(value);
            }
            pub fn $clearer(&mut self) {
                self.$field = None;
            })*
        }
    }
}

macro_rules! optional_enum_methods {
    ($($(#[$doc:meta])* ($field:ident, $type:ty, $setter:ident, $clearer:ident)),+) => {
        impl Node {
            $($(#[$doc])*
            pub fn $field(&self) -> Option<$type> {
                self.$field
            }
            pub fn $setter(&mut self, value: $type) {
                self.$field = Some(value);
            }
            pub fn $clearer(&mut self) {
                self.$field = None;
            })*
        }
    }
}

macro_rules! property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $type_getter:ident, $getter_result:ty, $setter:ident, $type_setter:ident, $setter_param:ty, $clearer:ident)),+) => {
        impl Node {
            $($(#[$doc])*
            pub fn $getter(&self) -> $getter_result {
                self.$type_getter(PropertyId::$id)
            }
            pub fn $setter(&mut self, value: $setter_param) {
                self.$type_setter(PropertyId::$id, value);
            }
            pub fn $clearer(&mut self) {
                self.clear_property(PropertyId::$id);
            })*
        }
    }
}

macro_rules! vec_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $item_type:ty, $getter:ident, $type_getter:ident, $setter:ident, $type_setter:ident, $pusher:ident, $type_pusher:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, $type_getter, &[$item_type], $setter, $type_setter, impl Into<Vec<$item_type>>, $clearer)
        }
        impl Node {
            pub fn $pusher(&mut self, item: $item_type) {
                self.$type_pusher(PropertyId::$id, item);
            }
        })*
    }
}

macro_rules! node_id_vec_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $pusher:ident, $clearer:ident)),+) => {
        $(vec_property_methods! {
            $(#[$doc])*
            ($id, NodeId, $getter, get_node_id_vec, $setter, set_node_id_vec, $pusher, push_to_node_id_vec, $clearer)
        })*
    }
}

macro_rules! node_id_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_node_id, Option<NodeId>, $setter, set_node_id, NodeId, $clearer)
        })*
    }
}

macro_rules! string_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_string, Option<&str>, $setter, set_string, impl Into<Box<str>>, $clearer)
        })*
    }
}

macro_rules! f64_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_f64, Option<f64>, $setter, set_f64, f64, $clearer)
        })*
    }
}

macro_rules! usize_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_usize, Option<usize>, $setter, set_usize, usize, $clearer)
        })*
    }
}

macro_rules! color_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_color, Option<u32>, $setter, set_color, u32, $clearer)
        })*
    }
}

macro_rules! length_slice_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_length_slice, &[u8], $setter, set_length_slice, impl Into<Box<[u8]>>, $clearer)
        })*
    }
}

macro_rules! coord_slice_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_coord_slice, Option<&[f32]>, $setter, set_coord_slice, impl Into<Box<[f32]>>, $clearer)
        })*
    }
}

/// A single accessible object. A complete UI is represented as a tree of these.
///
/// For brevity, and to make more of the documentation usable in bindings
/// to other languages, documentation of getter methods is written as if
/// documenting fields in a struct, and such methods are referred to
/// as properties.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Node {
    role: Role,
    actions: Actions,
    indices: PropertyIndices,
    props: Vec<PropertyValue>,
    flags: u32,
    expanded: Option<bool>,
    selected: Option<bool>,
    name_from: Option<NameFrom>,
    description_from: Option<DescriptionFrom>,
    invalid: Option<Invalid>,
    checked_state: Option<CheckedState>,
    live: Option<Live>,
    default_action_verb: Option<DefaultActionVerb>,
    text_direction: Option<TextDirection>,
    orientation: Option<Orientation>,
    sort_direction: Option<SortDirection>,
    aria_current: Option<AriaCurrent>,
    has_popup: Option<HasPopup>,
    list_style: Option<ListStyle>,
    text_align: Option<TextAlign>,
    vertical_offset: Option<VerticalOffset>,
    overline: Option<TextDecoration>,
    strikethrough: Option<TextDecoration>,
    underline: Option<TextDecoration>,
}

impl Node {
    fn get_property(&self, id: PropertyId) -> &PropertyValue {
        let index = self.indices.0[id as usize];
        if index == PropertyId::Unset as u8 {
            &PropertyValue::None
        } else {
            &self.props[index as usize]
        }
    }

    fn get_property_mut(&mut self, id: PropertyId, default: PropertyValue) -> &mut PropertyValue {
        let index = self.indices.0[id as usize] as usize;
        if index == PropertyId::Unset as usize {
            self.props.push(default);
            let index = self.props.len() - 1;
            self.indices.0[id as usize] = index as u8;
            &mut self.props[index]
        } else {
            if matches!(self.props[index], PropertyValue::None) {
                self.props[index] = default;
            }
            &mut self.props[index]
        }
    }

    fn set_property(&mut self, id: PropertyId, value: PropertyValue) {
        let index = self.indices.0[id as usize];
        if index == PropertyId::Unset as u8 {
            self.props.push(value);
            self.indices.0[id as usize] = (self.props.len() - 1) as u8;
        } else {
            self.props[index as usize] = value;
        }
    }

    fn clear_property(&mut self, id: PropertyId) {
        let index = self.indices.0[id as usize];
        if index != PropertyId::Unset as u8 {
            self.props[index as usize] = PropertyValue::None;
        }
    }

    fn get_affine(&self, id: PropertyId) -> Option<&Affine> {
        match self.get_property(id) {
            PropertyValue::None => None,
            PropertyValue::Affine(value) => Some(value),
            _ => panic!(),
        }
    }

    fn set_affine(&mut self, id: PropertyId, value: impl Into<Box<Affine>>) {
        self.set_property(id, PropertyValue::Affine(value.into()));
    }

    fn get_rect(&self, id: PropertyId) -> Option<Rect> {
        match self.get_property(id) {
            PropertyValue::None => None,
            PropertyValue::Rect(value) => Some(*value),
            _ => panic!(),
        }
    }

    fn set_rect(&mut self, id: PropertyId, value: Rect) {
        self.set_property(id, PropertyValue::Rect(value));
    }

    fn get_node_id_vec(&self, id: PropertyId) -> &[NodeId] {
        match self.get_property(id) {
            PropertyValue::None => &[],
            PropertyValue::NodeIdVec(value) => value,
            _ => panic!(),
        }
    }

    fn push_to_node_id_vec(&mut self, property_id: PropertyId, node_id: NodeId) {
        match self.get_property_mut(property_id, PropertyValue::NodeIdVec(Vec::new())) {
            PropertyValue::NodeIdVec(v) => {
                v.push(node_id);
            }
            _ => panic!(),
        }
    }

    fn set_node_id_vec(&mut self, id: PropertyId, value: impl Into<Vec<NodeId>>) {
        self.set_property(id, PropertyValue::NodeIdVec(value.into()));
    }

    fn get_node_id(&self, id: PropertyId) -> Option<NodeId> {
        match self.get_property(id) {
            PropertyValue::None => None,
            PropertyValue::NodeId(value) => Some(*value),
            _ => panic!(),
        }
    }

    fn set_node_id(&mut self, id: PropertyId, value: NodeId) {
        self.set_property(id, PropertyValue::NodeId(value));
    }

    fn get_string(&self, id: PropertyId) -> Option<&str> {
        match self.get_property(id) {
            PropertyValue::None => None,
            PropertyValue::String(value) => Some(value),
            _ => panic!(),
        }
    }

    fn set_string(&mut self, id: PropertyId, value: impl Into<Box<str>>) {
        self.set_property(id, PropertyValue::String(value.into()));
    }

    fn get_f64(&self, id: PropertyId) -> Option<f64> {
        match self.get_property(id) {
            PropertyValue::None => None,
            PropertyValue::F64(value) => Some(*value),
            _ => panic!(),
        }
    }

    fn set_f64(&mut self, id: PropertyId, value: f64) {
        self.set_property(id, PropertyValue::F64(value));
    }

    fn get_usize(&self, id: PropertyId) -> Option<usize> {
        match self.get_property(id) {
            PropertyValue::None => None,
            PropertyValue::Usize(value) => Some(*value),
            _ => panic!(),
        }
    }

    fn set_usize(&mut self, id: PropertyId, value: usize) {
        self.set_property(id, PropertyValue::Usize(value));
    }

    fn get_color(&self, id: PropertyId) -> Option<u32> {
        match self.get_property(id) {
            PropertyValue::None => None,
            PropertyValue::Color(value) => Some(*value),
            _ => panic!(),
        }
    }

    fn set_color(&mut self, id: PropertyId, value: u32) {
        self.set_property(id, PropertyValue::Color(value));
    }

    fn get_length_slice(&self, id: PropertyId) -> &[u8] {
        match self.get_property(id) {
            PropertyValue::None => &[],
            PropertyValue::LengthSlice(value) => value,
            _ => panic!(),
        }
    }

    fn set_length_slice(&mut self, id: PropertyId, value: impl Into<Box<[u8]>>) {
        self.set_property(id, PropertyValue::LengthSlice(value.into()));
    }

    fn get_coord_slice(&self, id: PropertyId) -> Option<&[f32]> {
        match self.get_property(id) {
            PropertyValue::None => None,
            PropertyValue::CoordSlice(value) => Some(value),
            _ => panic!(),
        }
    }

    fn set_coord_slice(&mut self, id: PropertyId, value: impl Into<Box<[f32]>>) {
        self.set_property(id, PropertyValue::CoordSlice(value.into()));
    }

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

    pub fn supports_action(&self, action: Action) -> bool {
        (self.actions.0 & action.mask()) != 0
    }
    pub fn add_action(&mut self, action: Action) {
        self.actions.0 |= action.mask();
    }
    pub fn remove_action(&mut self, action: Action) {
        self.actions.0 &= !(action.mask());
    }
    pub fn clear_actions(&mut self) {
        self.actions.0 = 0;
    }
}

flag_methods! {
    (AutofillAvailable, is_autofill_available, set_autofill_available, clear_autofill_available),
    (Default, is_default, set_default, clear_default),
    (Editable, is_editable, set_editable, clear_editable),
    (Hovered, is_hovered, set_hovered, clear_hovered),
    /// Exclude this node and its descendants from the tree presented to
    /// assistive technologies, and from hit testing.
    (Hidden, is_hidden, set_hidden, clear_hidden),
    (Linked, is_linked, set_linked, clear_linked),
    (Multiline, is_multiline, set_multiline, clear_multiline),
    (Multiselectable, is_multiselectable, set_multiselectable, clear_multiselectable),
    (Protected, is_protected, set_protected, clear_protected),
    (Required, is_required, set_required, clear_required),
    (Visited, is_visited, set_visited, clear_visited),
    (Busy, is_busy, set_busy, clear_busy),
    (LiveAtomic, is_live_atomic, set_live_atomic, clear_live_atomic),
    /// If a dialog box is marked as explicitly modal.
    (Modal, is_modal, set_modal, clear_modal),
    /// Indicates this node is user-scrollable, e.g. `overflow: scroll|auto`, as
    /// opposed to only programmatically scrollable, like `overflow: hidden`, or
    /// not scrollable at all, e.g. `overflow: visible`.
    (Scrollable, is_scrollable, set_scrollable, clear_scrollable),
    /// Indicates whether this node is selected due to selection follows focus.
    (SelectedFromFocus, is_selected_from_focus, set_selected_from_focus, clear_selected_from_focus),
    /// This element allows touches to be passed through when a screen reader
    /// is in touch exploration mode, e.g. a virtual keyboard normally
    /// behaves this way.
    (TouchPassThrough, is_touch_pass_through, set_touch_pass_through, clear_touch_pass_through),
    /// Use for a textbox that allows focus/selection but not input.
    (ReadOnly, is_read_only, set_read_only, clear_read_only),
    /// Use for a control or group of controls that disallows input.
    (Disabled, is_disabled, set_disabled, clear_disabled),
    (Bold, is_bold, set_bold, clear_bold),
    (Italic, is_italic, set_italic, clear_italic),
    /// Set on a canvas element if it has fallback content.
    (CanvasHasFallback, canvas_has_fallback, set_canvas_has_fallback, clear_canvas_has_fallback),
    /// Indicates that this node clips its children, i.e. may have
    /// `overflow: hidden` or clip children by default.
    (ClipsChildren, clips_children, set_clips_children, clear_clips_children),
    /// Indicates whether this node causes a hard line-break
    /// (e.g. block level elements, or `<br>`).
    (IsLineBreakingObject, is_line_breaking_object, set_is_line_breaking_object, clear_is_line_breaking_object),
    /// Indicates whether this node causes a page break.
    (IsPageBreakingObject, is_page_breaking_object, set_is_page_breaking_object, clear_is_page_breaking_object),
    (IsSpellingError, is_spelling_error, set_is_spelling_error, clear_is_spelling_error),
    (IsGrammarError, is_grammar_error, set_is_grammar_error, clear_is_grammar_error),
    (IsSearchMatch, is_search_match, set_is_search_match, clear_is_search_match),
    (IsSuggestion, is_suggestion, set_is_suggestion, clear_is_suggestion),
    /// The object functions as a text field which exposes its descendants.
    ///
    /// Use cases include the root of a content-editable region, an ARIA
    /// textbox which isn't currently editable and which has interactive
    /// descendants, and a `<body>` element that has "design-mode" set to "on".
    (IsNonatomicTextFieldRoot, is_nonatomic_text_field_root, set_is_nonatomic_text_field_root, clear_is_nonatomic_text_field_root)
}

optional_bool_methods! {
    /// Whether this node is expanded, collapsed, or neither.
    ///
    /// Setting this to `false` means the node is collapsed; omitting it means this state
    /// isn't applicable.
    (expanded, is_expanded, set_expanded, clear_expanded),

    /// Indicates whether this node is selected or unselected.
    ///
    /// The absence of this flag (as opposed to a `false` setting)
    /// means that the concept of "selected" doesn't apply.
    /// When deciding whether to set the flag to false or omit it,
    /// consider whether it would be appropriate for a screen reader
    /// to announce "not selected". The ambiguity of this flag
    /// in platform accessibility APIs has made extraneous
    /// "not selected" announcements a common annoyance.
    (selected, is_selected, set_selected, clear_selected)
}

optional_enum_methods! {
    /// What information was used to compute the object's name.
    (name_from, NameFrom, set_name_from, clear_name_from),
    /// What information was used to compute the object's description.
    (description_from, DescriptionFrom, set_description_from, clear_description_from),
    (invalid, Invalid, set_invalid, clear_invalid),
    (checked_state, CheckedState, set_checked_state, clear_checked_state),
    (live, Live, set_live, clear_live),
    (default_action_verb, DefaultActionVerb, set_default_action_verb, clear_default_action_verb),
    (text_direction, TextDirection, set_text_direction, clear_text_direction),
    (orientation, Orientation, set_orientation, clear_orientation),
    (sort_direction, SortDirection, set_sort_direction, clear_sort_direction),
    (aria_current, AriaCurrent, set_aria_current, clear_aria_current),
    (has_popup, HasPopup, set_has_popup, clear_has_popup),
    /// The list style type. Only available on list items.
    (list_style, ListStyle, set_list_style, clear_list_style),
    (text_align, TextAlign, set_text_align, clear_text_align),
    (vertical_offset, VerticalOffset, set_vertical_offset, clear_vertical_offset),
    (overline, TextDecoration, set_overline, clear_overline),
    (strikethrough, TextDecoration, set_strikethrough, clear_strikethrough),
    (underline, TextDecoration, set_underline, clear_underline)
}

node_id_vec_property_methods! {
    (Children, children, set_children, push_to_children, clear_children),
    /// Ids of nodes that are children of this node logically, but are
    /// not children of this node in the tree structure. As an example,
    /// a table cell is a child of a row, and an 'indirect' child of a
    /// column.
    (IndirectChildren, indirect_children, set_indirect_children, push_to_indirect_children, clear_indirect_children),
    (Controls, controls, set_controls, push_to_controls, clear_controls),
    (Details, details, set_details, push_to_details, clear_details),
    (DescribedBy, described_by, set_described_by, push_to_described_by, clear_described_by),
    (FlowTo, flow_to, set_flow_to, push_to_flow_to, clear_flow_to),
    (LabelledBy, labelled_by, set_labelled_by, push_to_labelled_by, clear_labelled_by),
    /// On radio buttons this should be set to a list of all of the buttons
    /// in the same group as this one, including this radio button itself.
    (RadioGroup, radio_group, set_radio_group, push_to_radio_group, clear_radio_group)
}

node_id_property_methods! {
    (ActiveDescendant, active_descendant, set_active_descendant, clear_active_descendant),
    (ErrorMessage, error_message, set_error_message, clear_error_message),
    (InPageLinkTarget, in_page_link_target, set_in_page_link_target, clear_in_page_link_target),
    (MemberOf, member_of, set_member_of, clear_member_of),
    (NextOnLine, next_on_line, set_next_on_line, clear_next_on_line),
    (PreviousOnLine, previous_on_line, set_previous_on_line, clear_previous_on_line),
    (PopupFor, popup_for, set_popup_for, clear_popup_for),
    (TableHeader, table_header, set_table_header, clear_table_header),
    (TableRowHeader, table_row_header, set_table_row_header, clear_table_row_header),
    (TableColumnHeader, table_column_header, set_table_column_header, clear_table_column_header),
    (NextFocus, next_focus, set_next_focus, clear_next_focus),
    (PreviousFocus, previous_focus, set_previous_focus, clear_previous_focus)
}

string_property_methods! {
    (Name, name, set_name, clear_name),
    (Description, description, set_description, clear_description),
    (Value, value, set_value, clear_value),
    (AccessKey, access_key, set_access_key, clear_access_key),
    (AutoComplete, auto_complete, set_auto_complete, clear_auto_complete),
    (CheckedStateDescription, checked_state_description, set_checked_state_description, clear_checked_state_description),
    (ClassName, class_name, set_class_name, clear_class_name),
    (CssDisplay, css_display, set_css_display, clear_css_display),
    /// Only present when different from parent.
    (FontFamily, font_family, set_font_family, clear_font_family),
    (HtmlTag, html_tag, set_html_tag, clear_html_tag),
    /// Inner HTML of an element. Only used for a top-level math element,
    /// to support third-party math accessibility products that parse MathML.
    (InnerHtml, inner_html, set_inner_html, clear_inner_html),
    (InputType, input_type, set_input_type, clear_input_type),
    (KeyShortcuts, key_shortcuts, set_key_shortcuts, clear_key_shortcuts),
    /// Only present when different from parent.
    (Language, language, set_language, clear_language),
    (LiveRelevant, live_relevant, set_live_relevant, clear_live_relevant),
    /// Only if not already exposed in [`name`] ([`NameFrom::Placeholder`]).
    ///
    /// [`name`]: Node::name
    (Placeholder, placeholder, set_placeholder, clear_placeholder),
    (AriaRole, aria_role, set_aria_role, clear_aria_role),
    (RoleDescription, role_description, set_role_description, clear_role_description),
    /// Only if not already exposed in [`name`] ([`NameFrom::Title`]).
    ///
    /// [`name`]: Node::name
    (Tooltip, tooltip, set_tooltip, clear_tooltip),
    (Url, url, set_url, clear_url)
}

f64_property_methods! {
    (ScrollX, scroll_x, set_scroll_x, clear_scroll_x),
    (ScrollXMin, scroll_x_min, set_scroll_x_min, clear_scroll_x_min),
    (ScrollXMax, scroll_x_max, set_scroll_x_max, clear_scroll_x_max),
    (ScrollY, scroll_y, set_scroll_y, clear_scroll_y),
    (ScrollYMin, scroll_y_min, set_scroll_y_min, clear_scroll_y_min),
    (ScrollYMax, scroll_y_max, set_scroll_y_max, clear_scroll_y_max),
    (NumericValue, numeric_value, set_numeric_value, clear_numeric_value),
    (MinNumericValue, min_numeric_value, set_min_numeric_value, clear_min_numeric_value),
    (MaxNumericValue, max_numeric_value, set_max_numeric_value, clear_max_numeric_value),
    (NumericValueStep, numeric_value_step, set_numeric_value_step, clear_numeric_value_step),
    (NumericValueJump, numeric_value_jump, set_numeric_value_jump, clear_numeric_value_jump),
    /// Font size is in pixels.
    (FontSize, font_size, set_font_size, clear_font_size),
    /// Font weight can take on any arbitrary numeric value. Increments of 100 in
    /// range `[0, 900]` represent keywords such as light, normal, bold, etc.
    (FontWeight, font_weight, set_font_weight, clear_font_weight),
    /// The indentation of the text, in mm.
    (TextIndent, text_indent, set_text_indent, clear_text_indent)
}

usize_property_methods! {
    (TableRowCount, table_row_count, set_table_row_count, clear_table_row_count),
    (TableColumnCount, table_column_count, set_table_column_count, clear_table_column_count),
    (TableRowIndex, table_row_index, set_table_row_index, clear_table_row_index),
    (TableColumnIndex, table_column_index, set_table_column_index, clear_table_column_index),
    (TableCellColumnIndex, table_cell_column_index, set_table_cell_column_index, clear_table_cell_column_index),
    (TableCellColumnSpan, table_cell_column_span, set_table_cell_column_span, clear_table_cell_column_span),
    (TableCellRowIndex, table_cell_row_index, set_table_cell_row_index, clear_table_cell_row_index),
    (TableCellRowSpan, table_cell_row_span, set_table_cell_row_span, clear_table_cell_row_span),
    (HierarchicalLevel, hierarchical_level, set_hierarchical_level, clear_hierarchical_level),
    (SizeOfSet, size_of_set, set_size_of_set, clear_size_of_set),
    (PositionInSet, position_in_set, set_position_in_set, clear_position_in_set)
}

color_property_methods! {
    /// For [`Role::ColorWell`], specifies the selected color in RGBA.
    (ColorValue, color_value, set_color_value, clear_color_value),
    /// Background color in RGBA.
    (BackgroundColor, background_color, set_background_color, clear_background_color),
    /// Foreground color in RGBA.
    (ForegroundColor, foreground_color, set_foreground_color, clear_foreground_color)
}

length_slice_property_methods! {
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
    /// [`value`]: Node::value
    (CharacterLengths, character_lengths, set_character_lengths, clear_character_lengths),

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
    /// [`character_lengths`]: Node::character_lengths
    (WordLengths, word_lengths, set_word_lengths, clear_word_lengths)
}

coord_slice_property_methods! {
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
    /// [`text_direction`]: Node::text_direction
    /// [`character_lengths`]: Node::character_lengths
    (CharacterPositions, character_positions, set_character_positions, clear_character_positions),

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
    /// [`text_direction`]: Node::text_direction
    /// [`character_lengths`]: Node::character_lengths
    (CharacterWidths, character_widths, set_character_widths, clear_character_widths)
}

property_methods! {
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
    /// [`bounds`]: Node::bounds
    (Transform, transform, get_affine, Option<&Affine>, set_transform, set_affine, impl Into<Box<Affine>>, clear_transform),

    /// The bounding box of this node, in the node's coordinate space.
    /// This property does not affect the coordinate space of either this node
    /// or its descendants; only the [`transform`] property affects that.
    /// This, along with the recommendation that most nodes should have
    /// a [`transform`] of `None`, implies that the `bounds` property
    /// of most nodes should be in the coordinate space of the nearest ancestor
    /// with a non-`None` [`transform`], or if there is no such ancestor,
    /// the tree's container (e.g. window).
    ///
    /// [`transform`]: Node::transform
    (Bounds, bounds, get_rect, Option<Rect>, set_bounds, set_rect, Rect, clear_bounds)
}

impl Node {
    pub fn text_selection(&self) -> Option<&TextSelection> {
        match self.get_property(PropertyId::TextSelection) {
            PropertyValue::None => None,
            PropertyValue::TextSelection(value) => Some(value),
            _ => panic!(),
        }
    }
    pub fn set_text_selection(&mut self, value: impl Into<Box<TextSelection>>) {
        self.set_property(
            PropertyId::TextSelection,
            PropertyValue::TextSelection(value.into()),
        );
    }
    pub fn clear_text_selection(&mut self) {
        self.clear_property(PropertyId::TextSelection);
    }

    pub fn custom_actions(&self) -> &[CustomAction] {
        match self.get_property(PropertyId::CustomActions) {
            PropertyValue::None => &[],
            PropertyValue::CustomActionVec(value) => value,
            _ => panic!(),
        }
    }
    pub fn set_custom_actions(&mut self, value: impl Into<Vec<CustomAction>>) {
        self.set_property(
            PropertyId::CustomActions,
            PropertyValue::CustomActionVec(value.into()),
        );
    }
    pub fn push_to_custom_actions(&mut self, action: CustomAction) {
        match self.get_property_mut(
            PropertyId::CustomActions,
            PropertyValue::CustomActionVec(Vec::new()),
        ) {
            PropertyValue::CustomActionVec(v) => {
                v.push(action);
            }
            _ => panic!(),
        }
    }
    pub fn clear_custom_actions(&mut self) {
        self.clear_property(PropertyId::CustomActions);
    }
}

#[cfg(feature = "serde")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
enum FieldId {
    Role,
    Actions,
    Expanded,
    Selected,
    NameFrom,
    DescriptionFrom,
    Invalid,
    CheckedState,
    Live,
    DefaultActionVerb,
    TextDirection,
    Orientation,
    SortDirection,
    AriaCurrent,
    HasPopup,
    ListStyle,
    TextAlign,
    VerticalOffset,
    Overline,
    Strikethrough,
    Underline,
}

#[cfg(feature = "serde")]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
enum DeserializeKey {
    Field(FieldId),
    Flag(Flag),
    Property(PropertyId),
    Unknown(String),
}

#[cfg(feature = "serde")]
macro_rules! serialize_simple_fields {
    ($self:ident, $map:ident, { $(($name:ident, $id:ident)),+ }) => {
        $($map.serialize_entry(&FieldId::$id, &$self.$name)?;)*
    }
}

#[cfg(feature = "serde")]
macro_rules! serialize_optional_fields {
    ($self:ident, $map:ident, { $(($name:ident, $id:ident)),+ }) => {
        $(if $self.$name.is_some() {
            $map.serialize_entry(&FieldId::$id, &$self.$name)?;
        })*
    }
}

#[cfg(feature = "serde")]
macro_rules! serialize_property {
    ($self:ident, $map:ident, $index:ident, $id:ident, { $($variant:ident),+ }) => {
        match &$self.props[$index as usize] {
            PropertyValue::None => (),
            $(PropertyValue::$variant(value) => {
                $map.serialize_entry(&$id, &Some(value))?;
            })*
        }
    }
}

#[cfg(feature = "serde")]
macro_rules! deserialize_field {
    ($node:ident, $map:ident, $key:ident, { $(($name:ident, $id:ident)),+ }) => {
        match $key {
            $(FieldId::$id => {
                $node.$name = $map.next_value()?;
            })*
        }
    }
}

#[cfg(feature = "serde")]
macro_rules! deserialize_property {
    ($node:ident, $map:ident, $key:ident, { $($type:ident { $($id:ident),+ }),+ }) => {
        match $key {
            $($(PropertyId::$id => {
                if let Some(value) = $map.next_value()? {
                    $node.set_property(PropertyId::$id, PropertyValue::$type(value));
                } else {
                    $node.clear_property(PropertyId::$id);
                }
            })*)*
            PropertyId::Unset => {
                let _ = $map.next_value::<IgnoredAny>()?;
            }
        }
    }
}

#[cfg(feature = "serde")]
impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        serialize_simple_fields!(self, map, {
            (role, Role),
            (actions, Actions)
        });
        for i in 0..((size_of_val(&self.flags) as u8) * 8) {
            if let Some(flag) = Flag::n(i) {
                if (self.flags & flag.mask()) != 0 {
                    map.serialize_entry(&flag, &true)?;
                }
            }
        }
        serialize_optional_fields!(self, map, {
            (expanded, Expanded),
            (selected, Selected),
            (name_from, NameFrom),
            (description_from, DescriptionFrom),
            (invalid, Invalid),
            (checked_state, CheckedState),
            (live, Live),
            (default_action_verb, DefaultActionVerb),
            (text_direction, TextDirection),
            (orientation, Orientation),
            (sort_direction, SortDirection),
            (aria_current, AriaCurrent),
            (has_popup, HasPopup),
            (list_style, ListStyle),
            (text_align, TextAlign),
            (vertical_offset, VerticalOffset),
            (overline, Overline),
            (strikethrough, Strikethrough),
            (underline, Underline)
        });
        for (id, index) in self.indices.0.iter().copied().enumerate() {
            if index == PropertyId::Unset as u8 {
                continue;
            }
            let id = PropertyId::n(id as _).unwrap();
            serialize_property!(self, map, index, id, {
                NodeIdVec,
                NodeId,
                String,
                F64,
                Usize,
                Color,
                LengthSlice,
                CoordSlice,
                Affine,
                Rect,
                TextSelection,
                CustomActionVec
            });
        }
        map.end()
    }
}

#[cfg(feature = "serde")]
struct NodeVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for NodeVisitor {
    type Value = Node;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Node")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Node, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut node = Node::default();
        while let Some(key) = map.next_key()? {
            match key {
                DeserializeKey::Field(id) => {
                    deserialize_field!(node, map, id, {
                       (role, Role),
                       (actions, Actions),
                       (expanded, Expanded),
                       (selected, Selected),
                       (name_from, NameFrom),
                       (description_from, DescriptionFrom),
                       (invalid, Invalid),
                       (checked_state, CheckedState),
                       (live, Live),
                       (default_action_verb, DefaultActionVerb),
                       (text_direction, TextDirection),
                       (orientation, Orientation),
                       (sort_direction, SortDirection),
                       (aria_current, AriaCurrent),
                       (has_popup, HasPopup),
                       (list_style, ListStyle),
                       (text_align, TextAlign),
                       (vertical_offset, VerticalOffset),
                       (overline, Overline),
                       (strikethrough, Strikethrough),
                       (underline, Underline)
                    });
                }
                DeserializeKey::Flag(flag) => {
                    if map.next_value()? {
                        node.flags |= flag.mask();
                    } else {
                        node.flags &= !(flag.mask());
                    }
                }
                DeserializeKey::Property(id) => {
                    deserialize_property!(node, map, id, {
                        NodeIdVec {
                            Children,
                            IndirectChildren,
                            Controls,
                            Details,
                            DescribedBy,
                            FlowTo,
                            LabelledBy,
                            RadioGroup
                        },
                        NodeId {
                            ActiveDescendant,
                            ErrorMessage,
                            InPageLinkTarget,
                            MemberOf,
                            NextOnLine,
                            PreviousOnLine,
                            PopupFor,
                            TableHeader,
                            TableRowHeader,
                            TableColumnHeader,
                            NextFocus,
                            PreviousFocus
                        },
                        String {
                            Name,
                            Description,
                            Value,
                            AccessKey,
                            AutoComplete,
                            CheckedStateDescription,
                            ClassName,
                            CssDisplay,
                            FontFamily,
                            HtmlTag,
                            InnerHtml,
                            InputType,
                            KeyShortcuts,
                            Language,
                            LiveRelevant,
                            Placeholder,
                            AriaRole,
                            RoleDescription,
                            Tooltip,
                            Url
                        },
                        F64 {
                            ScrollX,
                            ScrollXMin,
                            ScrollXMax,
                            ScrollY,
                            ScrollYMin,
                            ScrollYMax,
                            NumericValue,
                            MinNumericValue,
                            MaxNumericValue,
                            NumericValueStep,
                            NumericValueJump,
                            FontSize,
                            FontWeight,
                            TextIndent
                        },
                        Usize {
                            TableRowCount,
                            TableColumnCount,
                            TableRowIndex,
                            TableColumnIndex,
                            TableCellColumnIndex,
                            TableCellColumnSpan,
                            TableCellRowIndex,
                            TableCellRowSpan,
                            HierarchicalLevel,
                            SizeOfSet,
                            PositionInSet
                        },
                        Color {
                            ColorValue,
                            BackgroundColor,
                            ForegroundColor
                        },
                        LengthSlice {
                            CharacterLengths,
                            WordLengths
                        },
                        CoordSlice {
                            CharacterPositions,
                            CharacterWidths
                        },
                        Affine { Transform },
                        Rect { Bounds },
                        TextSelection { TextSelection },
                        CustomActionVec { CustomActions }
                    });
                }
                DeserializeKey::Unknown(_) => {
                    let _ = map.next_value::<IgnoredAny>()?;
                }
            }
        }

        Ok(node)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Node {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(NodeVisitor)
    }
}

/// The data associated with an accessibility tree that's global to the
/// tree and not associated with any particular node.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct Tree {
    pub root: NodeId,

    /// The node that's used as the root scroller, if any. On some platforms
    /// like Android we need to ignore accessibility scroll offsets for
    /// that node and get them from the viewport instead.
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
    pub nodes: Vec<(NodeId, Node)>,

    /// Rarely updated information about the tree as a whole. This may be omitted
    /// if it has not changed since the previous update, but providing the same
    /// information again is also allowed. This is required when initializing
    /// a tree.
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
