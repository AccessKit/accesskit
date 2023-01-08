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
use paste::paste;
#[cfg(feature = "schemars")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
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

#[derive(EnumSetType, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "serde", enumset(serialize_as_list))]
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
    NotUserSelectableStyle,
    SelectedFromFocus,
    TouchPassThrough,
    ReadOnly,
    Disabled,
    Bold,
    Italic,
    CanvasHasFallback,
    ClipsChildren,
    HasAriaAttribute,
    IsLineBreakingObject,
    IsPageBreakingObject,
    IsSpellingError,
    IsGrammarError,
    IsSearchMatch,
    IsSuggestion,
    IsNonatomicTextFieldRoot,
}

// The following is based on the technique described here:
// https://viruta.org/reducing-memory-consumption-in-librsvg-2.html

#[derive(Clone, Debug, PartialEq)]
enum Property {
    None,
    NodeIdVec(Vec<NodeId>),
    NodeId(NodeId),
    String(Box<str>),
    F64(f64),
    Usize(usize),
    Color(u32),
    LengthSlice(Box<[u8]>),
    CoordSlice(Box<[f32]>),
    Affine(Affine),
    Rect(Rect),
    TextSelection(TextSelection),
    CustomActionVec(Vec<CustomAction>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    AriaColumnCount,
    AriaCellColumnIndex,
    AriaCellColumnSpan,
    AriaRowCount,
    AriaCellRowIndex,
    AriaCellRowSpan,
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
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident))+) => {
        paste! {
            impl Node {
                $($(#[$doc])*
                pub fn [< is_ $base_name >](&self) -> bool {
                    self.flags.contains(Flag::$id)
                }
                pub fn [< set_ $base_name >](&mut self) {
                    self.flags.insert(Flag::$id);
                }
                pub fn [< clear_ $base_name >](&mut self) {
                    self.flags.remove(Flag::$id);
                })*
            }
        }
    }
}

macro_rules! irregular_flag_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident))+) => {
        paste! {
            impl Node {
                $($(#[$doc])*
                pub fn $base_name(&self) -> bool {
                    self.flags.contains(Flag::$id)
                }
                pub fn [< set_ $base_name >](&mut self) {
                    self.flags.insert(Flag::$id);
                }
                pub fn [< clear_ $base_name >](&mut self) {
                    self.flags.remove(Flag::$id);
                })*
            }
        }
    }
}

macro_rules! optional_bool_methods {
    ($($(#[$doc:meta])* ($base_name:ident))+) => {
        paste! {
            impl Node {
                $($(#[$doc])*
                pub fn [< is_ $base_name >](&self) -> Option<bool> {
                    self.$base_name
                }
                pub fn [< set_ $base_name >](&mut self, value: bool) {
                    self.$base_name = Some(value);
                }
                pub fn [< clear_ $base_name >](&mut self) {
                    self.$base_name = None;
                })*
            }
        }
    }
}

macro_rules! optional_enum_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $type:ty))+) => {
        paste! {
            impl Node {
                $($(#[$doc])*
                pub fn $base_name(&self) -> Option<$type> {
                    self.$base_name
                }
                pub fn [< set_ $base_name >](&mut self, value: $type) {
                    self.$base_name = Some(value);
                }
                pub fn [< clear_ $base_name >](&mut self) {
                    self.$base_name = None;
                })*
            }
        }
    }
}

macro_rules! property_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident, $type_method_base:ident, $getter_result:ty, $setter_param:ty))+) => {
        paste! {
            impl Node {
                $($(#[$doc])*
                pub fn $base_name(&self) -> $getter_result {
                    self.[< get_ $type_method_base >](PropertyId::$id)
                }
                pub fn [< set_ $base_name >](&mut self, value: $setter_param) {
                    self.[< set_ $type_method_base >](PropertyId::$id, value);
                }
                pub fn [< clear_ $base_name >](&mut self) {
                    self.clear_property(PropertyId::$id);
                })*
            }
        }
    }
}

macro_rules! vec_property_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident, $type_method_base:ident, $item_type:ty))+) => {
        paste! {
            impl Node {
                $($(#[$doc])*
                pub fn $base_name(&self) -> &[$item_type] {
                    self.[< get_ $type_method_base >](PropertyId::$id)
                }
                pub fn [< set_ $base_name >](&mut self, value: impl Into<Vec<$item_type>>) {
                    self.[< set_ $type_method_base >](PropertyId::$id, value);
                }
                pub fn [< push_to_ $base_name >](&mut self, item: $item_type) {
                    self.[< push_to_ $type_method_base >](PropertyId::$id, item);
                }
                pub fn [< clear_ $base_name >](&mut self) {
                    self.clear_property(PropertyId::$id);
                })*
            }
        }
    }
}

macro_rules! node_id_vec_property_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident))+) => {
        vec_property_methods! {
            $($(#[$doc])*
            ($base_name, $id, node_id_vec, NodeId))*
        }
    }
}

macro_rules! node_id_property_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident))+) => {
        property_methods! {
            $($(#[$doc])*
            ($base_name, $id, node_id, Option<NodeId>, NodeId))*
        }
    }
}

macro_rules! string_property_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident))+) => {
        property_methods! {
            $($(#[$doc])*
            ($base_name, $id, string, Option<&str>, impl Into<Box<str>>))*
        }
    }
}

macro_rules! f64_property_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident))+) => {
        property_methods! {
            $($(#[$doc])*
            ($base_name, $id, f64, Option<f64>, f64))*
        }
    }
}

macro_rules! usize_property_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident))+) => {
        property_methods! {
            $($(#[$doc])*
            ($base_name, $id, usize, Option<usize>, usize))*
        }
    }
}

macro_rules! color_property_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident))+) => {
        property_methods! {
            $($(#[$doc])*
            ($base_name, $id, color, Option<u32>, u32))*
        }
    }
}

macro_rules! length_slice_property_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident))+) => {
        property_methods! {
            $($(#[$doc])*
            ($base_name, $id, length_slice, &[u8], impl Into<Box<[u8]>>))*
        }
    }
}

macro_rules! coord_slice_property_methods {
    ($($(#[$doc:meta])* ($base_name:ident, $id:ident))+) => {
        property_methods! {
            $($(#[$doc])*
            ($base_name, $id, coord_slice, Option<&[f32]>, impl Into<Box<[f32]>>))*
        }
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
    actions: EnumSet<Action>,
    indices: PropertyIndices,
    props: Vec<Property>,
    flags: EnumSet<Flag>,
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
    fn get_property(&self, id: PropertyId) -> &Property {
        let index = self.indices.0[id as usize];
        if index == PropertyId::Unset as u8 {
            &Property::None
        } else {
            &self.props[index as usize]
        }
    }

    fn get_property_mut(&mut self, id: PropertyId, default: Property) -> &mut Property {
        let index = self.indices.0[id as usize] as usize;
        if index == PropertyId::Unset as usize {
            self.props.push(default);
            let index = self.props.len() - 1;
            self.indices.0[id as usize] = index as u8;
            &mut self.props[index]
        } else {
            if matches!(self.props[index], Property::None) {
                self.props[index] = default;
            }
            &mut self.props[index]
        }
    }

    fn set_property(&mut self, id: PropertyId, value: Property) {
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
            self.props[index as usize] = Property::None;
        }
    }

    fn get_affine(&self, id: PropertyId) -> Option<Affine> {
        match self.get_property(id) {
            Property::None => None,
            Property::Affine(value) => Some(*value),
            _ => panic!(),
        }
    }

    fn set_affine(&mut self, id: PropertyId, value: Affine) {
        self.set_property(id, Property::Affine(value));
    }

    fn get_rect(&self, id: PropertyId) -> Option<Rect> {
        match self.get_property(id) {
            Property::None => None,
            Property::Rect(value) => Some(*value),
            _ => panic!(),
        }
    }

    fn set_rect(&mut self, id: PropertyId, value: Rect) {
        self.set_property(id, Property::Rect(value));
    }

    fn get_node_id_vec(&self, id: PropertyId) -> &[NodeId] {
        match self.get_property(id) {
            Property::None => &[],
            Property::NodeIdVec(value) => value,
            _ => panic!(),
        }
    }

    fn push_to_node_id_vec(&mut self, property_id: PropertyId, node_id: NodeId) {
        match self.get_property_mut(property_id, Property::NodeIdVec(Vec::new())) {
            Property::NodeIdVec(v) => {
                v.push(node_id);
            }
            _ => panic!(),
        }
    }

    fn set_node_id_vec(&mut self, id: PropertyId, value: impl Into<Vec<NodeId>>) {
        self.set_property(id, Property::NodeIdVec(value.into()));
    }

    fn get_node_id(&self, id: PropertyId) -> Option<NodeId> {
        match self.get_property(id) {
            Property::None => None,
            Property::NodeId(value) => Some(*value),
            _ => panic!(),
        }
    }

    fn set_node_id(&mut self, id: PropertyId, value: NodeId) {
        self.set_property(id, Property::NodeId(value));
    }

    fn get_string(&self, id: PropertyId) -> Option<&str> {
        match self.get_property(id) {
            Property::None => None,
            Property::String(value) => Some(value),
            _ => panic!(),
        }
    }

    fn set_string(&mut self, id: PropertyId, value: impl Into<Box<str>>) {
        self.set_property(id, Property::String(value.into()));
    }

    fn get_f64(&self, id: PropertyId) -> Option<f64> {
        match self.get_property(id) {
            Property::None => None,
            Property::F64(value) => Some(*value),
            _ => panic!(),
        }
    }

    fn set_f64(&mut self, id: PropertyId, value: f64) {
        self.set_property(id, Property::F64(value));
    }

    fn get_usize(&self, id: PropertyId) -> Option<usize> {
        match self.get_property(id) {
            Property::None => None,
            Property::Usize(value) => Some(*value),
            _ => panic!(),
        }
    }

    fn set_usize(&mut self, id: PropertyId, value: usize) {
        self.set_property(id, Property::Usize(value));
    }

    fn get_color(&self, id: PropertyId) -> Option<u32> {
        match self.get_property(id) {
            Property::None => None,
            Property::Color(value) => Some(*value),
            _ => panic!(),
        }
    }

    fn set_color(&mut self, id: PropertyId, value: u32) {
        self.set_property(id, Property::Color(value));
    }

    fn get_length_slice(&self, id: PropertyId) -> &[u8] {
        match self.get_property(id) {
            Property::None => &[],
            Property::LengthSlice(value) => value,
            _ => panic!(),
        }
    }

    fn set_length_slice(&mut self, id: PropertyId, value: impl Into<Box<[u8]>>) {
        self.set_property(id, Property::LengthSlice(value.into()));
    }

    fn get_coord_slice(&self, id: PropertyId) -> Option<&[f32]> {
        match self.get_property(id) {
            Property::None => None,
            Property::CoordSlice(value) => Some(value),
            _ => panic!(),
        }
    }

    fn set_coord_slice(&mut self, id: PropertyId, value: impl Into<Box<[f32]>>) {
        self.set_property(id, Property::CoordSlice(value.into()));
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
}

flag_methods! {
    (autofill_available, AutofillAvailable)
    (default, Default)
    (editable, Editable)
    (hovered, Hovered)
    /// Exclude this node and its descendants from the tree presented to
    /// assistive technologies, and from hit testing.
    (hidden, Hidden)
    (linked, Linked)
    (multiline, Multiline)
    (multiselectable, Multiselectable)
    (protected, Protected)
    (required, Required)
    (visited, Visited)
    (busy, Busy)
    (live_atomic, LiveAtomic)
    /// If a dialog box is marked as explicitly modal.
    (modal, Modal)
    /// Indicates this node is user-scrollable, e.g. `overflow: scroll|auto`, as
    /// opposed to only programmatically scrollable, like `overflow: hidden`, or
    /// not scrollable at all, e.g. `overflow: visible`.
    (scrollable, Scrollable)
    /// Indicates that this node is not selectable because the style has
    /// `user-select: none`. Note that there may be other reasons why a node is
    /// not selectable - for example, bullets in a list. However, this attribute
    /// is only set on `user-select: none`.
    (not_user_selectable_style, NotUserSelectableStyle)
    /// Indicates whether this node is selected due to selection follows focus.
    (selected_from_focus, SelectedFromFocus)
    /// This element allows touches to be passed through when a screen reader
    /// is in touch exploration mode, e.g. a virtual keyboard normally
    /// behaves this way.
    (touch_pass_through, TouchPassThrough)
    /// Use for a textbox that allows focus/selection but not input.
    (read_only, ReadOnly)
    /// Use for a control or group of controls that disallows input.
    (disabled, Disabled)
    (bold, Bold)
    (italic, Italic)
}

irregular_flag_methods! {
    /// Set on a canvas element if it has fallback content.
    (canvas_has_fallback, CanvasHasFallback)
    /// Indicates that this node clips its children, i.e. may have
    /// `overflow: hidden` or clip children by default.
    (clips_children, ClipsChildren)
    /// True if the node has any ARIA attributes set.
    (has_aria_attribute, HasAriaAttribute)
    /// Indicates whether this node causes a hard line-break
    /// (e.g. block level elements, or `<br>`).
    (is_line_breaking_object, IsLineBreakingObject)
    /// Indicates whether this node causes a page break.
    (is_page_breaking_object, IsPageBreakingObject)
    (is_spelling_error, IsSpellingError)
    (is_grammar_error, IsGrammarError)
    (is_search_match, IsSearchMatch)
    (is_suggestion, IsSuggestion)
    /// The object functions as a text field which exposes its descendants.
    ///
    /// Use cases include the root of a content-editable region, an ARIA
    /// textbox which isn't currently editable and which has interactive
    /// descendants, and a `<body>` element that has "design-mode" set to "on".
    (is_nonatomic_text_field_root, IsNonatomicTextFieldRoot)
}

optional_bool_methods! {
    /// Whether this node is expanded, collapsed, or neither.
    ///
    /// Setting this to `false` means the node is collapsed; omitting it means this state
    /// isn't applicable.
    (expanded)

    /// Indicates whether this node is selected or unselected.
    ///
    /// The absence of this flag (as opposed to a `false` setting)
    /// means that the concept of "selected" doesn't apply.
    /// When deciding whether to set the flag to false or omit it,
    /// consider whether it would be appropriate for a screen reader
    /// to announce "not selected". The ambiguity of this flag
    /// in platform accessibility APIs has made extraneous
    /// "not selected" announcements a common annoyance.
    (selected)
}

optional_enum_methods! {
    /// What information was used to compute the object's name.
    (name_from, NameFrom)
    /// What information was used to compute the object's description.
    (description_from, DescriptionFrom)
    (invalid, Invalid)
    (checked_state, CheckedState)
    (live, Live)
    (default_action_verb, DefaultActionVerb)
    (text_direction, TextDirection)
    (orientation, Orientation)
    (sort_direction, SortDirection)
    (aria_current, AriaCurrent)
    (has_popup, HasPopup)
    /// The list style type. Only available on list items.
    (list_style, ListStyle)
    (text_align, TextAlign)
    (vertical_offset, VerticalOffset)
    (overline, TextDecoration)
    (strikethrough, TextDecoration)
    (underline, TextDecoration)
}

node_id_vec_property_methods! {
    (children, Children)
    /// Ids of nodes that are children of this node logically, but are
    /// not children of this node in the tree structure. As an example,
    /// a table cell is a child of a row, and an 'indirect' child of a
    /// column.
    (indirect_children, IndirectChildren)
    (controls, Controls)
    (details, Details)
    (described_by, DescribedBy)
    (flow_to, FlowTo)
    (labelled_by, LabelledBy)
    /// On radio buttons this should be set to a list of all of the buttons
    /// in the same group as this one, including this radio button itself.
    (radio_group, RadioGroup)
}

node_id_property_methods! {
    (active_descendant, ActiveDescendant)
    (error_message, ErrorMessage)
    (in_page_link_target, InPageLinkTarget)
    (member_of, MemberOf)
    (next_on_line, NextOnLine)
    (previous_on_line, PreviousOnLine)
    (popup_for, PopupFor)
    (table_header, TableHeader)
    (table_row_header, TableRowHeader)
    (table_column_header, TableColumnHeader)
    (next_focus, NextFocus)
    (previous_focus, PreviousFocus)
}

string_property_methods! {
    (name, Name)
    (description, Description)
    (value, Value)
    (access_key, AccessKey)
    (auto_complete, AutoComplete)
    (checked_state_description, CheckedStateDescription)
    (class_name, ClassName)
    (css_display, CssDisplay)
    /// Only present when different from parent.
    (font_family, FontFamily)
    (html_tag, HtmlTag)
    /// Inner HTML of an element. Only used for a top-level math element,
    /// to support third-party math accessibility products that parse MathML.
    (inner_html, InnerHtml)
    (input_type, InputType)
    (key_shortcuts, KeyShortcuts)
    /// Only present when different from parent.
    (language, Language)
    (live_relevant, LiveRelevant)
    /// Only if not already exposed in [`name`] ([`NameFrom::Placeholder`]).
    ///
    /// [`name`]: Node::name
    (placeholder, Placeholder)
    (aria_role, AriaRole)
    (role_description, RoleDescription)
    /// Only if not already exposed in [`name`] ([`NameFrom::Title`]).
    ///
    /// [`name`]: Node::name
    (tooltip, Tooltip)
    (url, Url)
}

f64_property_methods! {
    (scroll_x, ScrollX)
    (scroll_x_min, ScrollXMin)
    (scroll_x_max, ScrollXMax)
    (scroll_y, ScrollY)
    (scroll_y_min, ScrollYMin)
    (scroll_y_max, ScrollYMax)
    (numeric_value, NumericValue)
    (min_numeric_value, MinNumericValue)
    (max_numeric_value, MaxNumericValue)
    (numeric_value_step, NumericValueStep)
    (numeric_value_jump, NumericValueJump)
    /// Font size is in pixels.
    (font_size, FontSize)
    /// Font weight can take on any arbitrary numeric value. Increments of 100 in
    /// range `[0, 900]` represent keywords such as light, normal, bold, etc.
    (font_weight, FontWeight)
    /// The indentation of the text, in mm.
    (text_indent, TextIndent)
}

usize_property_methods! {
    (aria_column_count, AriaColumnCount)
    (aria_cell_column_index, AriaCellColumnIndex)
    (aria_cell_column_span, AriaCellColumnSpan)
    (aria_row_count, AriaRowCount)
    (aria_cell_row_index, AriaCellRowIndex)
    (aria_cell_row_span, AriaCellRowSpan)
    (table_row_count, TableRowCount)
    (table_column_count, TableColumnCount)
    (table_row_index, TableRowIndex)
    (table_column_index, TableColumnIndex)
    (table_cell_column_index, TableCellColumnIndex)
    (table_cell_column_span, TableCellColumnSpan)
    (table_cell_row_index, TableCellRowIndex)
    (table_cell_row_span, TableCellRowSpan)
    (hierarchical_level, HierarchicalLevel)
    (size_of_set, SizeOfSet)
    (position_in_set, PositionInSet)
}

color_property_methods! {
    /// For [`Role::ColorWell`], specifies the selected color in RGBA.
    (color_value, ColorValue)
    /// Background color in RGBA.
    (background_color, BackgroundColor)
    /// Foreground color in RGBA.
    (foreground_color, ForegroundColor)
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
    (character_lengths, CharacterLengths)

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
    (word_lengths, WordLengths)
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
    (character_positions, CharacterPositions)

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
    (character_widths, CharacterWidths)
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
    (transform, Transform, affine, Option<Affine>, Affine)

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
    (bounds, Bounds, rect, Option<Rect>, Rect)
}

impl Node {
    pub fn text_selection(&self) -> Option<TextSelection> {
        match self.get_property(PropertyId::TextSelection) {
            Property::None => None,
            Property::TextSelection(value) => Some(*value),
            _ => panic!(),
        }
    }
    pub fn set_text_selection(&mut self, value: TextSelection) {
        self.set_property(PropertyId::TextSelection, Property::TextSelection(value));
    }
    pub fn clear_text_selection(&mut self) {
        self.clear_property(PropertyId::TextSelection);
    }

    pub fn custom_actions(&self) -> &[CustomAction] {
        match self.get_property(PropertyId::CustomActions) {
            Property::None => &[],
            Property::CustomActionVec(value) => value,
            _ => panic!(),
        }
    }
    pub fn set_custom_actions(&mut self, value: impl Into<Vec<CustomAction>>) {
        self.set_property(
            PropertyId::CustomActions,
            Property::CustomActionVec(value.into()),
        );
    }
    pub fn push_to_custom_actions(&mut self, action: CustomAction) {
        match self.get_property_mut(
            PropertyId::CustomActions,
            Property::CustomActionVec(Vec::new()),
        ) {
            Property::CustomActionVec(v) => {
                v.push(action);
            }
            _ => panic!(),
        }
    }
    pub fn clear_custom_actions(&mut self) {
        self.clear_property(PropertyId::CustomActions);
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
