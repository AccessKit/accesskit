// Copyright 2021 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use serde::{Deserialize, Serialize};
use zvariant::{
    derive::Type
};

mod bus;
pub mod interfaces;
mod object_address;
mod object_id;
mod object_ref;
pub mod proxies;
mod state;

/// Enumeration used by interface #AtspiAccessible to specify the role
/// of an #AtspiAccessible object.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, Type)]
pub enum Role {
    /// A role indicating an error condition, such as
    /// uninitialized Role data.
    Invalid,
    /// Object is a label indicating the keyboard
    /// accelerators for the parent.
    AcceleratorLabel,
    /// Object is used to alert the user about something.
    Alert,
    /// Object contains a dynamic or moving image of some kind.
    Animation,
    /// Object is a 2d directional indicator.
    Arrow,
    /// Object contains one or more dates, usually arranged
    /// into a 2d list.
    Calendar,
    /// Object that can be drawn into and is used to trap events.
    Canvas,
    /// A choice that can be checked or unchecked and
    /// provides a separate indicator for the current state.
    CheckBox,
    /// A menu item that behaves like a check box. See @CHECK_BOX.
    CheckMenuItem,
    /// A specialized dialog that lets the user choose a color.
    ColorChooser,
    /// The header for a column of data.
    ColumnHeader,
    /// A list of choices the user can select from.
    ComboBox,
    /// An object which allows entry of a date.
    DateEditor,
    /// An inconifed internal frame within a DESKTOP_PANE.
    DesktopIcon,
    /// A pane that supports internal frames and
    /// iconified versions of those internal frames.
    DesktopFrame,
    /// An object that allows a value to be changed via rotating a
    /// visual element, or which displays a value via such a rotating element.
    Dial,
    /// A top level window with title bar and a border.
    Dialog,
    /// A pane that allows the user to navigate through
    /// and select the contents of a directory.
    DirectoryPane,
    /// A specialized dialog that displays the files in
    /// the directory and lets the user select a file, browse a different
    /// directory, or specify a filename.
    DrawingArea,
    /// An object used for drawing custom user interface elements.
    FileChooser,
    /// A object that fills up space in a user interface.
    Filler,
    /// Don't use, reserved for future use.
    FocusTraversable,
    /// Allows selection of a display font.
    FontChooser,
    /// A top level window with a title bar, border, menubar, etc.
    Frame,
    /// A pane that is guaranteed to be painted on top of
    /// all panes beneath it.
    GlassPane,
    /// A document container for HTML, whose children   
    /// represent the document content.
    HtmlContainer,
    /// A small fixed size picture, typically used to decorate
    /// components.
    Icon,
    /// An image, typically static.
    Image,
    /// A frame-like object that is clipped by a desktop pane.
    InternalFrame,
    /// An object used to present an icon or short string in an interface.
    Label,
    /// A specialized pane that allows its children to be
    /// drawn in layers, providing a form of stacking order.
    LayeredPane,
    /// An object that presents a list of objects to the user and
    /// allows the user to select one or more of them.
    List,
    /// An object that represents an element of a list.
    ListItem,
    /// An object usually found inside a menu bar that contains a
    /// list of actions the user can choose from.
    Menu,
    /// An object usually drawn at the top of the primary
    /// dialog box of an application that contains a list of menus the user can
    /// choose from.
    MenuBar,
    /// An object usually contained in a menu that presents
    /// an action the user can choose.
    MenuItem,
    /// A specialized pane whose primary use is inside a dialog.
    OptionPane,
    /// An object that is a child of a page tab list.
    PageTab,
    /// An object that presents a series of panels (or page tabs),
    /// one at a time,through some mechanism provided by the object.
    PageTabList,
    /// A generic container that is often used to group objects.
    Panel,
    /// A text object uses for passwords, or other places
    /// where the text content is not shown visibly to the user.
    PasswordText,
    /// A temporary window that is usually used to offer the
    /// user a list of choices, and then hides when the user selects one of those
    /// choices.
    PopupMenu,
    /// An object used to indicate how much of a task has been completed.
    ProgressBar,
    /// An object the user can manipulate to tell the
    /// application to do something.
    PushButton,
    /// A specialized check box that will cause other radio buttons
    /// in the same group to become unchecked when this one is checked.
    RadioButton,
    /// Object is both a menu item and a 'radio button'
    /// See @RADIO_BUTTON.
    RadioMenuItem,
    /// A specialized pane that has a glass pane and a
    /// layered pane as its children.
    RootPane,
    /// The header for a row of data.
    RowHeader,
    /// An object usually used to allow a user to
    /// incrementally view a large amount of data by moving the bounds of a
    /// viewport along a one-dimensional axis.
    ScrollBar,
    /// An object that allows a user to incrementally view
    /// a large amount of information. @SCROLL_PANE objects are usually
    /// accompanied by @SCROLL_BAR controllers, on which the
    /// @RELATION_CONTROLLER_FOR and @RELATION_CONTROLLED_BY 
    /// reciprocal relations are set. See  #get_relation_set.
    ScrollPane,
    /// An object usually contained in a menu to provide a
    /// visible and logical separation of the contents in a menu.
    Separator,
    /// An object that allows the user to select from a bounded range.
    Slider,
    /// An object which allows one of a set of choices to
    /// be selected, and which displays the current choice.  Unlike
    /// @SCROLL_BAR, @SLIDER objects need not control 
    /// 'viewport'-like objects.
    SpinButton,
    /// A specialized panel that presents two other panels
    /// at the same time.
    SplitPane,
    /// Object displays non-quantitative status information
    /// (c.f. @PROGRESS_BAR)
    StatusBar,
    /// An object used to repesent information in terms of rows and columns.
    Table,
    /// A 'cell' or discrete child within a Table. Note:
    /// Table cells need not have @TABLE_CELL, other 
    /// #AtspiRoleType values are valid as well.
    TableCell,
    /// An object which labels a particular column
    /// in an #AtspiTable.
    TableColumnHeader,
    /// An object which labels a particular row in a
    /// #AtspiTable. #AtspiTable rows and columns may also be labelled via the
    /// @RELATION_LABEL_FOR/@RELATION_LABELLED_BY relationships.
    /// See #get_relation_set.
    TableRowHeader,
    /// Object allows menu to be removed from menubar
    /// and shown in its own window.
    TearoffMenuItem,
    /// An object that emulates a terminal.
    Terminal,
    /// An interactive widget that supports multiple lines of text
    /// and optionally accepts user input, but whose purpose is not to solicit user
    /// input. Thus @TEXT is appropriate for the text view in a plain text
    /// editor but inappropriate for an input field in a dialog box or web form. For
    /// widgets whose purpose is to solicit input from the user, see @ENTRY
    /// and @PASSWORD_TEXT. For generic objects which display a brief amount
    /// of textual information, see @STATIC.
    Text,
    /// A specialized push button that can be checked or
    /// unchecked, but does not procide a separate indicator for the current
    /// state.
    ToggleButton,
    /// A bar or palette usually composed of push buttons or
    /// toggle buttons.
    ToolBar,
    /// An object that provides information about another object.
    ToolTip,
    /// An object used to repsent hierarchical information to the user.
    Tree,
    /// An object that presents both tabular and
    /// hierarchical info to the user.
    TreeTable,
    /// The object contains some #AtspiAccessible information, 
    /// but its role is not known.
    Unknown,
    /// An object usually used in a scroll pane, or to
    /// otherwise clip a larger object or content renderer to a specific
    /// onscreen viewport.
    Viewport,
    /// A top level window with no title or border.
    Window,
    /// Means that the role for this item is known, but not
    /// included in the core enumeration. Deprecated since 2.24.
    Extended,
    /// An object that serves as a document header.
    Header,
    /// An object that serves as a document footer.
    Footer,
    /// An object which is contains a single paragraph of
    /// text content. See also @TEXT.
    Paragraph,
    /// An object which describes margins and tab stops, etc.    
    /// for text objects which it controls (should have 
    /// @RELATION_CONTROLLER_FOR relation to such).
    Ruler,
    /// An object corresponding to the toplevel accessible
    /// of an application, which may contain @FRAME objects or other      
    /// accessible objects. Children of #AccessibleDesktop objects  are generally
    /// @APPLICATION objects.
    Application,
    /// The object is a dialog or list containing items
    /// for insertion into an entry widget, for instance a list of words for
    /// completion of a text entry.
    Autocomplete,
    /// The object is an editable text object in a toolbar.
    Editbar,
    /// The object is an embedded component container.  This
    /// role is a "grouping" hint that the contained objects share a context
    /// which is different from the container in which this accessible is
    /// embedded. In particular, it is used for some kinds of document embedding,
    /// and for embedding of out-of-process component, "panel applets", etc.
    Embedded,
    /// The object is a component whose textual content may be
    /// entered or modified by the user, provided @STATE_EDITABLE is present.
    /// A readonly @ENTRY object (i.e. where @STATE_EDITABLE is 
    /// not present) implies a read-only 'text field' in a form, as opposed to a 
    /// title, label, or caption.
    Entry,
    /// The object is a graphical depiction of quantitative data.
    /// It may contain multiple subelements whose attributes and/or description
    /// may be queried to obtain both the  quantitative data and information about
    /// how the data is being presented. The @LABELLED_BY relation is 
    /// particularly important in interpreting objects of this type, as is the
    /// accessible description property. See @CAPTION.
    Chart,
    /// The object contains descriptive information, usually
    /// textual, about another user interface element such as a table, chart, or
    /// image.
    Caption,
    /// The object is a visual frame or container which
    /// contains a view of document content. #AtspiDocument frames may occur within
    /// another #AtspiDocument instance, in which case the second  document may be
    /// said to be embedded in the containing instance.  HTML frames are often
    /// DOCUMENT_FRAME:  Either this object, or a singleton descendant, 
    /// should implement the #AtspiDocument interface.
    DocumentFrame,
    /// The object serves as a heading for content which
    /// follows it in a document. The 'heading level' of the heading, if
    /// availabe,  may be obtained by querying the object's attributes.
    Heading,
    /// The object is a containing instance which encapsulates a
    /// page of information. @PAGE is used in documents and content which
    /// support a paginated navigation model.
    Page,
    /// The object is a containing instance of document content
    /// which constitutes a particular 'logical' section of the document.  The
    /// type of content within a section, and the nature of the section division
    /// itself, may be obtained by querying the object's attributes.  Sections
    /// may be nested.
    Section,
    /// The object is redundant with another object in
    /// the hierarchy, and is exposed for purely technical reasons.  Objects of
    /// this role should be ignored by clients, if they are encountered at all.
    RedundantObject,
    /// The object is a containing instance of document content
    /// which has within it components with which the user can interact in order
    /// to input information; i.e. the object is a container for pushbuttons,    
    /// comboboxes, text input fields, and other 'GUI' components. @FORM
    /// should not, in general, be used for toplevel GUI containers or dialogs,
    /// but should be reserved for 'GUI' containers which occur within document
    /// content, for instance within Web documents, presentations, or text
    /// documents.  Unlike other GUI containers and dialogs which occur inside      
    /// application instances, @FORM containers' components are
    /// associated with the current document, rather than the current foreground 
    /// application or viewer instance.
    Form,
    /// The object is a hypertext anchor, i.e. a "link" in a      
    /// hypertext document.  Such objects are distinct from 'inline'       content
    /// which may also use the #AtspiHypertext/#AtspiHyperlink interfacesto indicate
    /// the range/location within a text object where an inline or embedded object
    /// lies.
    Link,
    /// The object is a window or similar viewport
    /// which is used to allow composition or input of a 'complex character',    
    /// in other words it is an "input method window".
    InputMethodWindow,
    /// A row in a table.
    TableRow,
    /// An object that represents an element of a tree.
    TreeItem,
    /// A document frame which contains a spreadsheet.
    DocumentSpreadsheet,
    /// A document frame which contains a
    /// presentation or slide content.
    DocumentPresentation,
    /// A document frame which contains textual content,
    /// such as found in a word processing application.
    DocumentText,
    /// A document frame which contains HTML or other
    /// markup suitable for display in a web browser.
    DocumentWeb,
    /// A document frame which contains email content
    /// to be displayed or composed either in plain text or HTML.
    DocumentEmail,
    /// An object found within a document and designed to
    /// present a comment, note, or other annotation. In some cases, this object
    /// might not be visible until activated.
    Comment,
    /// A non-collapsible list of choices the user can select from.
    ListBox,
    /// A group of related widgets. This group typically has a label.
    Grouping,
    /// An image map object. Usually a graphic with multiple
    /// hotspots, where each hotspot can be activated resulting in the loading of
    /// another document or section of a document.
    ImageMap,
    /// A transitory object designed to present a
    /// message to the user, typically at the desktop level rather than inside a
    /// particular application.
    Notification,
    /// An object designed to present a message to the user
    /// within an existing window.
    InfoBar,
    /// A bar that serves as a level indicator to, for
    /// instance, show the strength of a password or the state of a battery. @Since: 2.8
    LevelBar,
    /// A bar that serves as the title of a window or a
    /// dialog. @Since: 2.12
    TitleBar,
    /// An object which contains a text section
    /// that is quoted from another source.  @Since: 2.12
    BlockQuote,
    /// An object which represents an audio
    /// element. @Since: 2.12
    Audio,
    /// An object which represents a video element. @Since: 2.12
    Video,
    /// A definition of a term or concept. @Since: 2.12
    Definition,
    /// A section of a page that consists of a
    /// composition that forms an independent part of a document, page, or
    /// site. Examples: A blog entry, a news story, a forum post.
    /// @Since: 2.12
    Article,
    /// A region of a web page intended as a
    /// navigational landmark. This is designed to allow Assistive
    /// Technologies to provide quick navigation among key regions within a
    /// document. @Since: 2.12
    Landmark,
    /// A text widget or container holding log content, such
    /// as chat history and error logs. In this role there is a
    /// relationship between the arrival of new items in the log and the
    /// reading order. The log contains a meaningful sequence and new
    /// information is added only to the end of the log, not at arbitrary
    /// points. @Since: 2.12
    Log,
    /// A container where non-essential information
    /// changes frequently. Common usages of marquee include stock tickers
    /// and ad banners. The primary difference between a marquee and a log
    /// is that logs usually have a meaningful order or sequence of
    /// important content changes. @Since: 2.12
    Marquee,
    /// A text widget or container that holds a mathematical
    /// expression. @Since: 2.12
    Math,
    /// A widget whose purpose is to display a rating,
    /// such as the number of stars associated with a song in a media
    /// player. Objects of this role should also implement
    /// AtspiValue. @Since: 2.12
    Rating,
    /// An object containing a numerical counter which
    /// indicates an amount of elapsed time from a start point, or the time
    /// remaining until an end point. @Since: 2.12
    Timer,
    /// A generic non-container object whose purpose is to display
    /// a brief amount of information to the user and whose role is known by the
    /// implementor but lacks semantic value for the user. Examples in which
    /// @STATIC is appropriate include the message displayed in a message
    /// box and an image used as an alternative means to display text.
    /// @STATIC should not be applied to widgets which are traditionally
    /// interactive, objects which display a significant amount of content, or any
    /// object which has an accessible relation pointing to another object. The
    /// displayed information, as a general rule, should be exposed through the
    /// accessible name of the object. For labels which describe another widget, see
    /// @LABEL. For text views, see @TEXT. For generic
    /// containers, see @PANEL. For objects whose role is not known by the
    /// implementor, see @UNKNOWN. @Since: 2.16.
    Static,
    /// An object that represents a mathematical fraction. @Since: 2.16.
    MathFraction,
    /// An object that represents a mathematical expression
    /// displayed with a radical. @Since: 2.16.
    MathRoot,
    /// An object that contains text that is displayed as a
    /// subscript. @Since: 2.16.
    Subscript,
    /// An object that contains text that is displayed as a
    /// superscript. @Since: 2.16.
    Superscript,
    /// An object that represents a list of term-value
    /// groups. A term-value group represents an individual description and consist
    /// of one or more names (@DESCRIPTION_TERM) followed by one or more
    /// values (@DESCRIPTION_VALUE). For each list, there should not be
    /// more than one group with the same term name. @Since: 2.26.
    DescriptionList,
    /// An object that represents a term or phrase
    /// with a corresponding definition. @Since: 2.26.
    DescriptionTerm,
    /// An object that represents the description,
    /// definition, or value of a term. @Since: 2.26.
    DescriptionValue,
    /// An object that contains the text of a footnote. @Since: 2.26.
    Footnote,
    /// Content previously deleted or proposed to be
    /// deleted, e.g. in revision history or a content view providing suggestions
    /// from reviewers. @Since: 2.34.
    ContentDeletion,
    /// Content previously inserted or proposed to be
    /// inserted, e.g. in revision history or a content view providing suggestions
    /// from reviewers. @Since: 2.34.
    ContentInsertion,
    /// A run of content that is marked or highlighted, such as for
    /// reference purposes, or to call it out as having a special purpose. If the
    /// marked content has an associated section in the document elaborating on the
    /// reason for the mark, then %RELATION_DETAILS should be used on the mark
    /// to point to that associated section. In addition, the reciprocal relation
    /// %RELATION_DETAILS_FOR should be used on the associated content section
    /// to point back to the mark. @Since: 2.36.
    Mark,
    /// A container for content that is called out as a proposed
    /// change from the current version of the document, such as by a reviewer of the
    /// content. This role should include either %CONTENT_DELETION and/or
    /// %CONTENT_INSERTION children, in any order, to indicate what the
    /// actual change is. @Since: 2.36
    Suggestion,
    /// Not a valid role, used for finding end of enumeration.
    LastDefined,
}

pub use bus::Bus;
pub use object_address::*;
pub use object_id::*;
pub use object_ref::*;
pub use state::*;
