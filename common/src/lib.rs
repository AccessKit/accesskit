// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

// Derived from Chromium's accessibility abstraction.
// Copyright 2018 The Chromium Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE.chromium file.

#[cfg(feature = "pyo3")]
use pyo3::pyclass;
#[cfg(feature = "schemars")]
use schemars::{
    gen::SchemaGenerator,
    schema::{ArrayValidation, InstanceType, ObjectValidation, Schema, SchemaObject},
    JsonSchema, Map as SchemaMap,
};
#[cfg(feature = "serde")]
use serde::{
    de::{Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq, Serializer},
    Deserialize, Serialize,
};
use std::{collections::BTreeSet, ops::DerefMut, sync::Arc};
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum Role {
    #[default]
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

    /// A generic container that should be ignored by assistive technologies
    /// and filtered out of platform accessibility trees. Equivalent to the ARIA
    /// `none` or `presentation` role, or to an HTML `div` with no role.
    GenericContainer,

    CheckBox,
    RadioButton,
    TextInput,
    Button,
    DefaultButton,
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

    MultilineTextInput,
    SearchInput,
    DateInput,
    DateTimeInput,
    WeekInput,
    MonthInput,
    TimeInput,
    EmailInput,
    NumberInput,
    PasswordInput,
    PhoneNumberInput,
    UrlInput,

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
    Code,
    ColorWell,
    ComboBox,
    EditableComboBox,
    Complementary,
    Comment,
    ContentDeletion,
    ContentInsertion,
    ContentInfo,
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

    /// This is just like a multi-line document, but signals that assistive
    /// technologies should implement behavior specific to a VT-100-style
    /// terminal.
    Terminal,
}

/// An action to be taken on an accessibility node.
///
/// In contrast to [`DefaultActionVerb`], these describe what happens to the
/// object, e.g. "focus".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
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

    /// Delete any selected text in the control's text value and
    /// insert the specified value in its place, like when typing or pasting.
    /// Requires [`ActionRequest::data`] to be set to [`ActionData::Value`].
    ReplaceSelectedText,

    // Scrolls by approximately one screen in a specific direction.
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
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

    #[inline]
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

#[cfg(feature = "schemars")]
impl JsonSchema for Actions {
    #[inline]
    fn schema_name() -> String {
        "Actions".into()
    }

    #[inline]
    fn is_referenceable() -> bool {
        false
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(
                ArrayValidation {
                    unique_items: Some(true),
                    items: Some(gen.subschema_for::<Action>().into()),
                    ..Default::default()
                }
                .into(),
            ),
            ..Default::default()
        }
        .into()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum Orientation {
    /// E.g. most toolbars and separators.
    Horizontal,
    /// E.g. menu or combo box.
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
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
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum Invalid {
    True,
    Grammar,
    Spelling,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum Checked {
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
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
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
    Unselect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum SortDirection {
    Unsorted,
    Ascending,
    Descending,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
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
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum AutoComplete {
    Inline,
    List,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum Live {
    Off,
    Polite,
    Assertive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum HasPopup {
    True,
    Menu,
    Listbox,
    Tree,
    Grid,
    Dialog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
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
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum TextAlign {
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum VerticalOffset {
    Subscript,
    Superscript,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "enumn", derive(enumn::N))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(
    feature = "pyo3",
    pyclass(module = "accesskit", rename_all = "SCREAMING_SNAKE_CASE")
)]
#[repr(u8)]
pub enum TextDecoration {
    Solid,
    Dotted,
    Dashed,
    Double,
    Wavy,
}

pub type NodeIdContent = u64;

/// The stable identity of a [`Node`], unique within the node's tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[repr(transparent)]
pub struct NodeId(pub NodeIdContent);

impl From<NodeIdContent> for NodeId {
    #[inline]
    fn from(inner: NodeIdContent) -> Self {
        Self(inner)
    }
}

impl From<NodeId> for NodeIdContent {
    #[inline]
    fn from(outer: NodeId) -> Self {
        outer.0
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
    /// but only a caret, this must be equal to the value of [`TextSelection::focus`].
    /// This is also known as a degenerate selection.
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
    Hovered,
    Hidden,
    Linked,
    Multiselectable,
    Required,
    Visited,
    Busy,
    LiveAtomic,
    Modal,
    TouchTransparent,
    ReadOnly,
    Disabled,
    Bold,
    Italic,
    ClipsChildren,
    IsLineBreakingObject,
    IsPageBreakingObject,
    IsSpellingError,
    IsGrammarError,
    IsSearchMatch,
    IsSuggestion,
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
    TextDecoration(TextDecoration),
    LengthSlice(Box<[u8]>),
    CoordSlice(Box<[f32]>),
    Bool(bool),
    Invalid(Invalid),
    Checked(Checked),
    Live(Live),
    DefaultActionVerb(DefaultActionVerb),
    TextDirection(TextDirection),
    Orientation(Orientation),
    SortDirection(SortDirection),
    AriaCurrent(AriaCurrent),
    AutoComplete(AutoComplete),
    HasPopup(HasPopup),
    ListStyle(ListStyle),
    TextAlign(TextAlign),
    VerticalOffset(VerticalOffset),
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

    // String
    Name,
    Description,
    Value,
    AccessKey,
    ClassName,
    FontFamily,
    HtmlTag,
    InnerHtml,
    KeyboardShortcut,
    Language,
    Placeholder,
    RoleDescription,
    StateDescription,
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

    // TextDecoration
    Overline,
    Strikethrough,
    Underline,

    // LengthSlice
    CharacterLengths,
    WordLengths,

    // CoordSlice
    CharacterPositions,
    CharacterWidths,

    // bool
    Expanded,
    Selected,

    // Unique enums
    Invalid,
    Checked,
    Live,
    DefaultActionVerb,
    TextDirection,
    Orientation,
    SortDirection,
    AriaCurrent,
    AutoComplete,
    HasPopup,
    ListStyle,
    TextAlign,
    VerticalOffset,

    // Other
    Transform,
    Bounds,
    TextSelection,
    CustomActions,

    // This MUST be last.
    Unset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct PropertyIndices([u8; PropertyId::Unset as usize]);

impl Default for PropertyIndices {
    fn default() -> Self {
        Self([PropertyId::Unset as u8; PropertyId::Unset as usize])
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
struct NodeClass {
    role: Role,
    actions: Actions,
    indices: PropertyIndices,
}

/// Allows nodes that have the same role, actions, and set of defined properties
/// to share metadata. Each node has a class which is created by [`NodeBuilder`],
/// and when [`NodeBuilder::build`] is called, the node's class is added
/// to the provided instance of this struct if an identical class isn't
/// in that set already. Once a class is added to a class set, it currently
/// remains in that set for the life of that set, whether or not any nodes
/// are still using the class.
///
/// It's not an error for different nodes in the same tree, or even subsequent
/// versions of the same node, to be built from different class sets;
/// it's merely suboptimal.
///
/// Note: This struct's `Default` implementation doesn't provide access to
/// a shared set, as one might assume; it creates a new set. For a shared set,
/// use [`NodeClassSet::lock_global`].
#[derive(Clone, Default)]
#[repr(transparent)]
pub struct NodeClassSet(BTreeSet<Arc<NodeClass>>);

impl NodeClassSet {
    #[inline]
    pub const fn new() -> Self {
        Self(BTreeSet::new())
    }

    /// Accesses a shared class set guarded by a mutex.
    pub fn lock_global() -> impl DerefMut<Target = Self> {
        use std::sync::Mutex;

        static INSTANCE: Mutex<NodeClassSet> = Mutex::new(NodeClassSet::new());

        INSTANCE.lock().unwrap()
    }
}

/// A single accessible object. A complete UI is represented as a tree of these.
///
/// For brevity, and to make more of the documentation usable in bindings
/// to other languages, documentation of getter methods is written as if
/// documenting fields in a struct, and such methods are referred to
/// as properties.
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    class: Arc<NodeClass>,
    flags: u32,
    props: Arc<[PropertyValue]>,
}

/// Builds a [`Node`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NodeBuilder {
    class: NodeClass,
    flags: u32,
    props: Vec<PropertyValue>,
}

impl NodeClass {
    fn get_property<'a>(&self, props: &'a [PropertyValue], id: PropertyId) -> &'a PropertyValue {
        let index = self.indices.0[id as usize];
        if index == PropertyId::Unset as u8 {
            &PropertyValue::None
        } else {
            &props[index as usize]
        }
    }
}

fn unexpected_property_type() -> ! {
    panic!();
}

impl NodeBuilder {
    fn get_property_mut(&mut self, id: PropertyId, default: PropertyValue) -> &mut PropertyValue {
        let index = self.class.indices.0[id as usize] as usize;
        if index == PropertyId::Unset as usize {
            self.props.push(default);
            let index = self.props.len() - 1;
            self.class.indices.0[id as usize] = index as u8;
            &mut self.props[index]
        } else {
            if matches!(self.props[index], PropertyValue::None) {
                self.props[index] = default;
            }
            &mut self.props[index]
        }
    }

    fn set_property(&mut self, id: PropertyId, value: PropertyValue) {
        let index = self.class.indices.0[id as usize];
        if index == PropertyId::Unset as u8 {
            self.props.push(value);
            self.class.indices.0[id as usize] = (self.props.len() - 1) as u8;
        } else {
            self.props[index as usize] = value;
        }
    }

    fn clear_property(&mut self, id: PropertyId) {
        let index = self.class.indices.0[id as usize];
        if index != PropertyId::Unset as u8 {
            self.props[index as usize] = PropertyValue::None;
        }
    }
}

macro_rules! flag_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        impl Node {
            $($(#[$doc])*
            #[inline]
            pub fn $getter(&self) -> bool {
                (self.flags & (Flag::$id).mask()) != 0
            })*
        }
        impl NodeBuilder {
            $($(#[$doc])*
            #[inline]
            pub fn $getter(&self) -> bool {
                (self.flags & (Flag::$id).mask()) != 0
            }
            #[inline]
            pub fn $setter(&mut self) {
                self.flags |= (Flag::$id).mask();
            }
            #[inline]
            pub fn $clearer(&mut self) {
                self.flags &= !((Flag::$id).mask());
            })*
        }
    }
}

macro_rules! option_ref_type_getters {
    ($(($method:ident, $type:ty, $variant:ident)),+) => {
        impl NodeClass {
            $(fn $method<'a>(&self, props: &'a [PropertyValue], id: PropertyId) -> Option<&'a $type> {
                match self.get_property(props, id) {
                    PropertyValue::None => None,
                    PropertyValue::$variant(value) => Some(value),
                    _ => unexpected_property_type(),
                }
            })*
        }
    }
}

macro_rules! slice_type_getters {
    ($(($method:ident, $type:ty, $variant:ident)),+) => {
        impl NodeClass {
            $(fn $method<'a>(&self, props: &'a [PropertyValue], id: PropertyId) -> &'a [$type] {
                match self.get_property(props, id) {
                    PropertyValue::None => &[],
                    PropertyValue::$variant(value) => value,
                    _ => unexpected_property_type(),
                }
            })*
        }
    }
}

macro_rules! copy_type_getters {
    ($(($method:ident, $type:ty, $variant:ident)),+) => {
        impl NodeClass {
            $(fn $method(&self, props: &[PropertyValue], id: PropertyId) -> Option<$type> {
                match self.get_property(props, id) {
                    PropertyValue::None => None,
                    PropertyValue::$variant(value) => Some(*value),
                    _ => unexpected_property_type(),
                }
            })*
        }
    }
}

macro_rules! box_type_setters {
    ($(($method:ident, $type:ty, $variant:ident)),+) => {
        impl NodeBuilder {
            $(fn $method(&mut self, id: PropertyId, value: impl Into<Box<$type>>) {
                self.set_property(id, PropertyValue::$variant(value.into()));
            })*
        }
    }
}

macro_rules! copy_type_setters {
    ($(($method:ident, $type:ty, $variant:ident)),+) => {
        impl NodeBuilder {
            $(fn $method(&mut self, id: PropertyId, value: $type) {
                self.set_property(id, PropertyValue::$variant(value));
            })*
        }
    }
}

macro_rules! vec_type_methods {
    ($(($type:ty, $variant:ident, $getter:ident, $setter:ident, $pusher:ident)),+) => {
        $(slice_type_getters! {
            ($getter, $type, $variant)
        })*
        impl NodeBuilder {
            $(fn $setter(&mut self, id: PropertyId, value: impl Into<Vec<$type>>) {
                self.set_property(id, PropertyValue::$variant(value.into()));
            }
            fn $pusher(&mut self, id: PropertyId, item: $type) {
                match self.get_property_mut(id, PropertyValue::$variant(Vec::new())) {
                    PropertyValue::$variant(v) => {
                        v.push(item);
                    }
                    _ => unexpected_property_type(),
                }
            })*
        }
    }
}

macro_rules! property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $type_getter:ident, $getter_result:ty, $setter:ident, $type_setter:ident, $setter_param:ty, $clearer:ident)),+) => {
        impl Node {
            $($(#[$doc])*
            #[inline]
            pub fn $getter(&self) -> $getter_result {
                self.class.$type_getter(&self.props, PropertyId::$id)
            })*
        }
        impl NodeBuilder {
            $($(#[$doc])*
            #[inline]
            pub fn $getter(&self) -> $getter_result {
                self.class.$type_getter(&self.props, PropertyId::$id)
            }
            #[inline]
            pub fn $setter(&mut self, value: $setter_param) {
                self.$type_setter(PropertyId::$id, value);
            }
            #[inline]
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
        impl NodeBuilder {
            #[inline]
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
            ($id, $getter, get_node_id_property, Option<NodeId>, $setter, set_node_id_property, NodeId, $clearer)
        })*
    }
}

macro_rules! string_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_string_property, Option<&str>, $setter, set_string_property, impl Into<Box<str>>, $clearer)
        })*
    }
}

macro_rules! f64_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_f64_property, Option<f64>, $setter, set_f64_property, f64, $clearer)
        })*
    }
}

macro_rules! usize_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_usize_property, Option<usize>, $setter, set_usize_property, usize, $clearer)
        })*
    }
}

macro_rules! color_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_color_property, Option<u32>, $setter, set_color_property, u32, $clearer)
        })*
    }
}

macro_rules! text_decoration_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_text_decoration_property, Option<TextDecoration>, $setter, set_text_decoration_property, TextDecoration, $clearer)
        })*
    }
}

macro_rules! length_slice_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_length_slice_property, &[u8], $setter, set_length_slice_property, impl Into<Box<[u8]>>, $clearer)
        })*
    }
}

macro_rules! coord_slice_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_coord_slice_property, Option<&[f32]>, $setter, set_coord_slice_property, impl Into<Box<[f32]>>, $clearer)
        })*
    }
}

macro_rules! bool_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        $(property_methods! {
            $(#[$doc])*
            ($id, $getter, get_bool_property, Option<bool>, $setter, set_bool_property, bool, $clearer)
        })*
    }
}

macro_rules! unique_enum_property_methods {
    ($($(#[$doc:meta])* ($id:ident, $getter:ident, $setter:ident, $clearer:ident)),+) => {
        impl Node {
            $($(#[$doc])*
            #[inline]
            pub fn $getter(&self) -> Option<$id> {
                match self.class.get_property(&self.props, PropertyId::$id) {
                    PropertyValue::None => None,
                    PropertyValue::$id(value) => Some(*value),
                    _ => unexpected_property_type(),
                }
            })*
        }
        impl NodeBuilder {
            $($(#[$doc])*
            #[inline]
            pub fn $getter(&self) -> Option<$id> {
                match self.class.get_property(&self.props, PropertyId::$id) {
                    PropertyValue::None => None,
                    PropertyValue::$id(value) => Some(*value),
                    _ => unexpected_property_type(),
                }
            }
            #[inline]
            pub fn $setter(&mut self, value: $id) {
                self.set_property(PropertyId::$id, PropertyValue::$id(value));
            }
            #[inline]
            pub fn $clearer(&mut self) {
                self.clear_property(PropertyId::$id);
            })*
        }
    }
}

impl NodeBuilder {
    #[inline]
    pub fn new(role: Role) -> Self {
        Self {
            class: NodeClass {
                role,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    pub fn build(self, classes: &mut NodeClassSet) -> Node {
        let class = if let Some(class) = classes.0.get(&self.class) {
            Arc::clone(class)
        } else {
            let class = Arc::new(self.class);
            classes.0.insert(Arc::clone(&class));
            class
        };
        Node {
            class,
            flags: self.flags,
            props: self.props.into(),
        }
    }
}

impl Node {
    #[inline]
    pub fn role(&self) -> Role {
        self.class.role
    }
}

impl NodeBuilder {
    #[inline]
    pub fn role(&self) -> Role {
        self.class.role
    }
    #[inline]
    pub fn set_role(&mut self, value: Role) {
        self.class.role = value;
    }
}

impl Node {
    #[inline]
    pub fn supports_action(&self, action: Action) -> bool {
        (self.class.actions.0 & action.mask()) != 0
    }
}

impl NodeBuilder {
    #[inline]
    pub fn supports_action(&self, action: Action) -> bool {
        (self.class.actions.0 & action.mask()) != 0
    }
    #[inline]
    pub fn add_action(&mut self, action: Action) {
        self.class.actions.0 |= action.mask();
    }
    #[inline]
    pub fn remove_action(&mut self, action: Action) {
        self.class.actions.0 &= !(action.mask());
    }
    #[inline]
    pub fn clear_actions(&mut self) {
        self.class.actions.0 = 0;
    }
}

flag_methods! {
    (Hovered, is_hovered, set_hovered, clear_hovered),
    /// Exclude this node and its descendants from the tree presented to
    /// assistive technologies, and from hit testing.
    (Hidden, is_hidden, set_hidden, clear_hidden),
    (Linked, is_linked, set_linked, clear_linked),
    (Multiselectable, is_multiselectable, set_multiselectable, clear_multiselectable),
    (Required, is_required, set_required, clear_required),
    (Visited, is_visited, set_visited, clear_visited),
    (Busy, is_busy, set_busy, clear_busy),
    (LiveAtomic, is_live_atomic, set_live_atomic, clear_live_atomic),
    /// If a dialog box is marked as explicitly modal.
    (Modal, is_modal, set_modal, clear_modal),
    /// This element allows touches to be passed through when a screen reader
    /// is in touch exploration mode, e.g. a virtual keyboard normally
    /// behaves this way.
    (TouchTransparent, is_touch_transparent, set_touch_transparent, clear_touch_transparent),
    /// Use for a textbox that allows focus/selection but not input.
    (ReadOnly, is_read_only, set_read_only, clear_read_only),
    /// Use for a control or group of controls that disallows input.
    (Disabled, is_disabled, set_disabled, clear_disabled),
    (Bold, is_bold, set_bold, clear_bold),
    (Italic, is_italic, set_italic, clear_italic),
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
    (IsSuggestion, is_suggestion, set_is_suggestion, clear_is_suggestion)
}

option_ref_type_getters! {
    (get_affine_property, Affine, Affine),
    (get_string_property, str, String),
    (get_coord_slice_property, [f32], CoordSlice),
    (get_text_selection_property, TextSelection, TextSelection)
}

slice_type_getters! {
    (get_length_slice_property, u8, LengthSlice)
}

copy_type_getters! {
    (get_rect_property, Rect, Rect),
    (get_node_id_property, NodeId, NodeId),
    (get_f64_property, f64, F64),
    (get_usize_property, usize, Usize),
    (get_color_property, u32, Color),
    (get_text_decoration_property, TextDecoration, TextDecoration),
    (get_bool_property, bool, Bool)
}

box_type_setters! {
    (set_affine_property, Affine, Affine),
    (set_string_property, str, String),
    (set_length_slice_property, [u8], LengthSlice),
    (set_coord_slice_property, [f32], CoordSlice),
    (set_text_selection_property, TextSelection, TextSelection)
}

copy_type_setters! {
    (set_rect_property, Rect, Rect),
    (set_node_id_property, NodeId, NodeId),
    (set_f64_property, f64, F64),
    (set_usize_property, usize, Usize),
    (set_color_property, u32, Color),
    (set_text_decoration_property, TextDecoration, TextDecoration),
    (set_bool_property, bool, Bool)
}

vec_type_methods! {
    (NodeId, NodeIdVec, get_node_id_vec, set_node_id_vec, push_to_node_id_vec),
    (CustomAction, CustomActionVec, get_custom_action_vec, set_custom_action_vec, push_to_custom_action_vec)
}

node_id_vec_property_methods! {
    (Children, children, set_children, push_child, clear_children),
    (Controls, controls, set_controls, push_controlled, clear_controls),
    (Details, details, set_details, push_detail, clear_details),
    (DescribedBy, described_by, set_described_by, push_described_by, clear_described_by),
    (FlowTo, flow_to, set_flow_to, push_flow_to, clear_flow_to),
    (LabelledBy, labelled_by, set_labelled_by, push_labelled_by, clear_labelled_by),
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
    (TableColumnHeader, table_column_header, set_table_column_header, clear_table_column_header)
}

string_property_methods! {
    (Name, name, set_name, clear_name),
    (Description, description, set_description, clear_description),
    (Value, value, set_value, clear_value),
    /// A single character, usually part of this node's name, that can be pressed,
    /// possibly along with a platform-specific modifier, to perform
    /// this node's default action. For menu items, the access key is only active
    /// while the menu is active, in contrast with [`keyboard_shortcut`];
    /// a single menu item may in fact have both properties.
    ///
    /// [`keyboard_shortcut`]: Node::keyboard_shortcut
    (AccessKey, access_key, set_access_key, clear_access_key),
    (ClassName, class_name, set_class_name, clear_class_name),
    /// Only present when different from parent.
    (FontFamily, font_family, set_font_family, clear_font_family),
    (HtmlTag, html_tag, set_html_tag, clear_html_tag),
    /// Inner HTML of an element. Only used for a top-level math element,
    /// to support third-party math accessibility products that parse MathML.
    (InnerHtml, inner_html, set_inner_html, clear_inner_html),
    /// A keystroke or sequence of keystrokes, complete with any required
    /// modifiers(s), that will perform this node's default action.
    /// The value of this property should be in a human-friendly format.
    (KeyboardShortcut, keyboard_shortcut, set_keyboard_shortcut, clear_keyboard_shortcut),
    /// Only present when different from parent.
    (Language, language, set_language, clear_language),
    /// If a text input has placeholder text, it should be exposed
    /// through this property rather than [`name`].
    ///
    /// [`name`]: Node::name
    (Placeholder, placeholder, set_placeholder, clear_placeholder),
    /// An optional string that may override an assistive technology's
    /// description of the node's role. Only provide this for custom control types.
    /// The value of this property should be in a human-friendly, localized format.
    (RoleDescription, role_description, set_role_description, clear_role_description),
    /// An optional string that may override an assistive technology's
    /// description of the node's state, replacing default strings such as
    /// "checked" or "selected". Note that most platform accessibility APIs
    /// and assistive technologies do not support this feature.
    (StateDescription, state_description, set_state_description, clear_state_description),
    /// If a node's only accessible name comes from a tooltip, it should be
    /// exposed through this property rather than [`name`].
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
    (FontWeight, font_weight, set_font_weight, clear_font_weight)
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

text_decoration_property_methods! {
    (Overline, overline, set_overline, clear_overline),
    (Strikethrough, strikethrough, set_strikethrough, clear_strikethrough),
    (Underline, underline, set_underline, clear_underline)
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

bool_property_methods! {
    /// Whether this node is expanded, collapsed, or neither.
    ///
    /// Setting this to `false` means the node is collapsed; omitting it means this state
    /// isn't applicable.
    (Expanded, is_expanded, set_expanded, clear_expanded),

    /// Indicates whether this node is selected or unselected.
    ///
    /// The absence of this flag (as opposed to a `false` setting)
    /// means that the concept of "selected" doesn't apply.
    /// When deciding whether to set the flag to false or omit it,
    /// consider whether it would be appropriate for a screen reader
    /// to announce "not selected". The ambiguity of this flag
    /// in platform accessibility APIs has made extraneous
    /// "not selected" announcements a common annoyance.
    (Selected, is_selected, set_selected, clear_selected)
}

unique_enum_property_methods! {
    (Invalid, invalid, set_invalid, clear_invalid),
    (Checked, checked, set_checked, clear_checked),
    (Live, live, set_live, clear_live),
    (DefaultActionVerb, default_action_verb, set_default_action_verb, clear_default_action_verb),
    (TextDirection, text_direction, set_text_direction, clear_text_direction),
    (Orientation, orientation, set_orientation, clear_orientation),
    (SortDirection, sort_direction, set_sort_direction, clear_sort_direction),
    (AriaCurrent, aria_current, set_aria_current, clear_aria_current),
    (AutoComplete, auto_complete, set_auto_complete, clear_auto_complete),
    (HasPopup, has_popup, set_has_popup, clear_has_popup),
    /// The list style type. Only available on list items.
    (ListStyle, list_style, set_list_style, clear_list_style),
    (TextAlign, text_align, set_text_align, clear_text_align),
    (VerticalOffset, vertical_offset, set_vertical_offset, clear_vertical_offset)
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
    (Transform, transform, get_affine_property, Option<&Affine>, set_transform, set_affine_property, impl Into<Box<Affine>>, clear_transform),

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
    (Bounds, bounds, get_rect_property, Option<Rect>, set_bounds, set_rect_property, Rect, clear_bounds),

    (TextSelection, text_selection, get_text_selection_property, Option<&TextSelection>, set_text_selection, set_text_selection_property, impl Into<Box<TextSelection>>, clear_text_selection)
}

vec_property_methods! {
    (CustomActions, CustomAction, custom_actions, get_custom_action_vec, set_custom_actions, set_custom_action_vec, push_custom_action, push_to_custom_action_vec, clear_custom_actions)
}

#[cfg(feature = "serde")]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
enum ClassFieldId {
    Role,
    Actions,
}

#[cfg(feature = "serde")]
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
enum DeserializeKey {
    ClassField(ClassFieldId),
    Flag(Flag),
    Property(PropertyId),
    Unknown(String),
}

#[cfg(feature = "serde")]
macro_rules! serialize_class_fields {
    ($self:ident, $map:ident, { $(($name:ident, $id:ident)),+ }) => {
        $($map.serialize_entry(&ClassFieldId::$id, &$self.class.$name)?;)*
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
macro_rules! deserialize_class_field {
    ($builder:ident, $map:ident, $key:ident, { $(($name:ident, $id:ident)),+ }) => {
        match $key {
            $(ClassFieldId::$id => {
                $builder.class.$name = $map.next_value()?;
            })*
        }
    }
}

#[cfg(feature = "serde")]
macro_rules! deserialize_property {
    ($builder:ident, $map:ident, $key:ident, { $($type:ident { $($id:ident),+ }),+ }) => {
        match $key {
            $($(PropertyId::$id => {
                if let Some(value) = $map.next_value()? {
                    $builder.set_property(PropertyId::$id, PropertyValue::$type(value));
                } else {
                    $builder.clear_property(PropertyId::$id);
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
        serialize_class_fields!(self, map, {
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
        for (id, index) in self.class.indices.0.iter().copied().enumerate() {
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
                TextDecoration,
                LengthSlice,
                CoordSlice,
                Bool,
                Invalid,
                Checked,
                Live,
                DefaultActionVerb,
                TextDirection,
                Orientation,
                SortDirection,
                AriaCurrent,
                AutoComplete,
                HasPopup,
                ListStyle,
                TextAlign,
                VerticalOffset,
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

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Node")
    }

    fn visit_map<V>(self, mut map: V) -> Result<Node, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut builder = NodeBuilder::default();
        while let Some(key) = map.next_key()? {
            match key {
                DeserializeKey::ClassField(id) => {
                    deserialize_class_field!(builder, map, id, {
                       (role, Role),
                       (actions, Actions)
                    });
                }
                DeserializeKey::Flag(flag) => {
                    if map.next_value()? {
                        builder.flags |= flag.mask();
                    } else {
                        builder.flags &= !(flag.mask());
                    }
                }
                DeserializeKey::Property(id) => {
                    deserialize_property!(builder, map, id, {
                        NodeIdVec {
                            Children,
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
                            TableColumnHeader
                        },
                        String {
                            Name,
                            Description,
                            Value,
                            AccessKey,
                            ClassName,
                            FontFamily,
                            HtmlTag,
                            InnerHtml,
                            KeyboardShortcut,
                            Language,
                            Placeholder,
                            RoleDescription,
                            StateDescription,
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
                            FontWeight
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
                        TextDecoration {
                            Overline,
                            Strikethrough,
                            Underline
                        },
                        LengthSlice {
                            CharacterLengths,
                            WordLengths
                        },
                        CoordSlice {
                            CharacterPositions,
                            CharacterWidths
                        },
                        Bool {
                            Expanded,
                            Selected
                        },
                        Invalid { Invalid },
                        Checked { Checked },
                        Live { Live },
                        DefaultActionVerb { DefaultActionVerb },
                        TextDirection { TextDirection },
                        Orientation { Orientation },
                        SortDirection { SortDirection },
                        AriaCurrent { AriaCurrent },
                        AutoComplete { AutoComplete },
                        HasPopup { HasPopup },
                        ListStyle { ListStyle },
                        TextAlign { TextAlign },
                        VerticalOffset { VerticalOffset },
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

        Ok(builder.build(&mut NodeClassSet::lock_global()))
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

#[cfg(feature = "schemars")]
macro_rules! add_schema_property {
    ($gen:ident, $properties:ident, $enum_value:expr, $type:ty) => {{
        let name = format!("{:?}", $enum_value);
        let name = name[..1].to_ascii_lowercase() + &name[1..];
        let subschema = $gen.subschema_for::<$type>();
        $properties.insert(name, subschema);
    }};
}

#[cfg(feature = "schemars")]
macro_rules! add_flags_to_schema {
    ($gen:ident, $properties:ident, { $($variant:ident),+ }) => {
        $(add_schema_property!($gen, $properties, Flag::$variant, bool);)*
    }
}

#[cfg(feature = "schemars")]
macro_rules! add_properties_to_schema {
    ($gen:ident, $properties:ident, { $($type:ty { $($id:ident),+ }),+ }) => {
        $($(add_schema_property!($gen, $properties, PropertyId::$id, Option<$type>);)*)*
    }
}

#[cfg(feature = "schemars")]
impl JsonSchema for Node {
    #[inline]
    fn schema_name() -> String {
        "Node".into()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut properties = SchemaMap::<String, Schema>::new();
        add_schema_property!(gen, properties, ClassFieldId::Role, Role);
        add_schema_property!(gen, properties, ClassFieldId::Actions, Actions);
        add_flags_to_schema!(gen, properties, {
            Hovered,
            Hidden,
            Linked,
            Multiselectable,
            Required,
            Visited,
            Busy,
            LiveAtomic,
            Modal,
            TouchTransparent,
            ReadOnly,
            Disabled,
            Bold,
            Italic,
            ClipsChildren,
            IsLineBreakingObject,
            IsPageBreakingObject,
            IsSpellingError,
            IsGrammarError,
            IsSearchMatch,
            IsSuggestion
        });
        add_properties_to_schema!(gen, properties, {
            Vec<NodeId> {
                Children,
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
                TableColumnHeader
            },
            Box<str> {
                Name,
                Description,
                Value,
                AccessKey,
                ClassName,
                FontFamily,
                HtmlTag,
                InnerHtml,
                KeyboardShortcut,
                Language,
                Placeholder,
                RoleDescription,
                StateDescription,
                Tooltip,
                Url
            },
            f64 {
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
                FontWeight
            },
            usize {
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
            u32 {
                ColorValue,
                BackgroundColor,
                ForegroundColor
            },
            TextDecoration {
                Overline,
                Strikethrough,
                Underline
            },
            Box<[u8]> {
                CharacterLengths,
                WordLengths
            },
            Box<[f32]> {
                CharacterPositions,
                CharacterWidths
            },
            bool {
                Expanded,
                Selected
            },
            Invalid { Invalid },
            Checked { Checked },
            Live { Live },
            DefaultActionVerb { DefaultActionVerb },
            TextDirection { TextDirection },
            Orientation { Orientation },
            SortDirection { SortDirection },
            AriaCurrent { AriaCurrent },
            AutoComplete { AutoComplete },
            HasPopup { HasPopup },
            ListStyle { ListStyle },
            TextAlign { TextAlign },
            VerticalOffset { VerticalOffset },
            Affine { Transform },
            Rect { Bounds },
            TextSelection { TextSelection },
            Vec<CustomAction> { CustomActions }
        });
        SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            object: Some(
                ObjectValidation {
                    properties,
                    ..Default::default()
                }
                .into(),
            ),
            ..Default::default()
        }
        .into()
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
    /// The identifier of the tree's root node.
    pub root: NodeId,
    /// The name of the application this tree belongs to.
    pub app_name: Option<String>,
    /// The name of the UI toolkit in use.
    pub toolkit_name: Option<String>,
    /// The version of the UI toolkit.
    pub toolkit_version: Option<String>,
}

impl Tree {
    #[inline]
    pub fn new(root: NodeId) -> Tree {
        Tree {
            root,
            app_name: None,
            toolkit_name: None,
            toolkit_version: None,
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
#[derive(Clone, Debug, PartialEq)]
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

    /// The node within this tree that has keyboard focus when the native
    /// host (e.g. window) has focus. If no specific node within the tree
    /// has keyboard focus, this must be set to the root. The latest focus state
    /// must be provided with every tree update, even if the focus state
    /// didn't change in a given update.
    pub focus: NodeId,
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
#[repr(C)]
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
pub trait ActionHandler {
    /// Perform the requested action. If the requested action is not supported,
    /// this method must do nothing.
    ///
    /// The thread on which this method is called is platform-dependent.
    /// Refer to the platform adapter documentation for more details.
    ///
    /// This method may queue the request and handle it asynchronously.
    /// This behavior is preferred over blocking, e.g. when dispatching
    /// the request to another thread.
    fn do_action(&mut self, request: ActionRequest);
}
