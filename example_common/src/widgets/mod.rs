mod button;
mod image;
mod label;
mod list_box;
mod scroll_bar;
mod spinner;
mod text_input;
mod toggle_button;
mod window;

pub use button::Button;
pub use image::Image;
pub use label::Label;
pub use list_box::{ListBox, ListItem};
pub use scroll_bar::{ScrollBar, ScrollPosition};
pub use spinner::Spinner;
pub use text_input::{ImeEdit, InputType, TextInput, TextInputInfo, TextInputState};
pub use toggle_button::ToggleButton;
pub use window::Window;

use accesskit::{ActionRequest, Node, NodeId};

use crate::KeyEvent;

/// How a widget reacted to an event.
pub enum Response<M> {
    /// Not handled, so the parent widget gets a chance to handle it.
    Ignored,
    /// Handled, with nothing else to do.
    Handled,
    /// Handled, producing a message for the application.
    Message(M),
    /// Handled, producing messages for the application.
    Messages(Vec<M>),
    /// Handled, moving focus to the given widget.
    Focus(NodeId),
    /// Handled, moving focus to the given widget and producing a message.
    FocusAndMessage(NodeId, M),
}

/// A part of the user interface, exposed as an AccessKit node.
pub trait Widget<M> {
    fn id(&self) -> NodeId;

    fn build(&self) -> Node;

    /// The widgets whose nodes are children of this one.
    fn children(&self) -> Vec<&dyn Widget<M>> {
        Vec::new()
    }

    /// Whether the widget can receive keyboard focus.
    fn is_focusable(&self) -> bool {
        false
    }

    /// Handles a key event when the widget or one of its descendants has
    /// focus.
    fn on_key(&self, _event: &KeyEvent, _focus: NodeId) -> Response<M> {
        Response::Ignored
    }

    /// Handles an action request targeting the widget or one of its
    /// descendants.
    fn on_action(&self, _request: &ActionRequest) -> Response<M> {
        Response::Ignored
    }

    /// The state of the widget, if it is a text input.
    fn text_input(&self) -> Option<TextInputInfo> {
        None
    }

    /// Handles an edit from the platform's input method when the widget has
    /// focus.
    fn on_ime_edit(&self, _edit: &ImeEdit) -> Response<M> {
        Response::Ignored
    }
}

/// Calls `f` with a widget and all of its descendants, in tree order.
pub fn visit<'a, M>(widget: &'a dyn Widget<M>, f: &mut dyn FnMut(&'a dyn Widget<M>)) {
    f(widget);
    for child in widget.children() {
        visit(child, f);
    }
}

/// Offers an event to the target widget, then to its ancestors until one of
/// them handles it.
pub fn bubble<M>(
    widget: &dyn Widget<M>,
    target: NodeId,
    handle: &mut dyn FnMut(&dyn Widget<M>) -> Response<M>,
) -> Response<M> {
    bubble_from(widget, target, handle).unwrap_or(Response::Ignored)
}

fn bubble_from<M>(
    widget: &dyn Widget<M>,
    target: NodeId,
    handle: &mut dyn FnMut(&dyn Widget<M>) -> Response<M>,
) -> Option<Response<M>> {
    if widget.id() == target {
        return Some(handle(widget));
    }
    let response = widget
        .children()
        .into_iter()
        .find_map(|child| bubble_from(child, target, handle))?;
    Some(match response {
        Response::Ignored => handle(widget),
        response => response,
    })
}
