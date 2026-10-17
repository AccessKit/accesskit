use accesskit::{Live, NodeId, Rect, Size};
use std::time::Duration;

use crate::{
    Command, Message, rect,
    widgets::{
        Button, InputType, ListBox, ListItem, ScrollPosition, TextInput, TextInputState, Window,
    },
};

const CONTACT_LIST_ID: NodeId = NodeId(100);
const FIRST_CONTACT_ID: u64 = 101;
const MESSAGE_LIST_ID: NodeId = NodeId(110);
const MESSAGE_SCROLL_BAR_ID: NodeId = NodeId(111);
const COMPOSER_ID: NodeId = NodeId(120);
const SEND_BUTTON_ID: NodeId = NodeId(121);
/// The text runs of the composer use consecutive IDs from this one.
const COMPOSER_FIRST_TEXT_RUN_ID: NodeId = NodeId(10_000);
/// The messages use consecutive IDs from this one.
const FIRST_MESSAGE_ID: u64 = 1_000_000;
/// The widget that has focus when the screen appears.
pub(crate) const INITIAL_FOCUS: NodeId = CONTACT_LIST_ID;

const CONTACT_LIST_WIDTH: f64 = 120.0;
const CONTACT_HEIGHT: f64 = 48.0;
const MESSAGE_HEIGHT: f64 = 56.0;
const COMPOSER_HEIGHT: f64 = 104.0;
const SEND_BUTTON_WIDTH: f64 = 52.0;
const SPACING: f64 = 8.0;
/// Mobile platforms don't expose scrollbars to assistive technologies,
/// which scroll the list itself.
const HAS_SCROLL_BARS: bool = !cfg!(any(target_os = "android", target_os = "ios"));
const SCROLL_BAR_WIDTH: f64 = if HAS_SCROLL_BARS { 12.0 } else { 0.0 };

/// Contacts on the left, and the conversation above the composer on the
/// right.
struct Layout {
    contact_list: Rect,
    message_list: Rect,
    message_scroll_bar: Rect,
    composer: Rect,
    send_button: Rect,
}

impl Layout {
    fn new(size: Size) -> Self {
        let contact_list = rect(0.0, 0.0, CONTACT_LIST_WIDTH, size.height);
        let bottom = size.height - SPACING;
        let top = bottom - COMPOSER_HEIGHT;
        let right = size.width - SPACING;
        let send_button = rect(right - SEND_BUTTON_WIDTH, top, right, bottom);
        let composer = rect(
            CONTACT_LIST_WIDTH + SPACING,
            top,
            send_button.x0 - SPACING,
            bottom,
        );
        let message_list = rect(
            CONTACT_LIST_WIDTH,
            0.0,
            size.width - SCROLL_BAR_WIDTH,
            top - SPACING,
        );
        let message_scroll_bar = Rect {
            x0: message_list.x1,
            x1: size.width,
            ..message_list
        };
        Self {
            contact_list,
            message_list,
            message_scroll_bar,
            composer,
            send_button,
        }
    }

    fn contact(&self, index: usize) -> Rect {
        let y0 = index as f64 * CONTACT_HEIGHT;
        Rect {
            y0,
            y1: y0 + CONTACT_HEIGHT,
            ..self.contact_list
        }
    }

    fn message(&self, index: usize) -> Rect {
        let y0 = self.message_list.y0 + SPACING + index as f64 * MESSAGE_HEIGHT;
        rect(
            self.message_list.x0 + SPACING,
            y0,
            self.message_list.x1 - SPACING,
            y0 + MESSAGE_HEIGHT - SPACING,
        )
    }
}

const CONTACTS: [&str; 3] = ["Alice", "Bruce", "Lingfei"];
const REPLIES: [&str; 3] = ["Hello. How are you doing?", "Hey there!", "Welcome back!"];
const REPLY_DELAY: Duration = Duration::from_secs(2);

struct ChatMessage {
    id: u64,
    /// The contact who sent the message, or `None` for the user.
    sender: Option<usize>,
    text: String,
}

/// A conversation with a contact, which starts scrolled to the end.
struct Conversation {
    messages: Vec<ChatMessage>,
    selected_message: Option<usize>,
    scroll_position: ScrollPosition,
    draft: TextInputState,
}

impl Default for Conversation {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            selected_message: None,
            scroll_position: ScrollPosition::End,
            draft: TextInputState::default(),
        }
    }
}

/// The screen for exchanging messages with contacts.
pub(crate) struct MainScreen {
    selected_contact: usize,
    conversations: Vec<Conversation>,
    next_message_id: u64,
    /// Replies from this one on are announced, so that switching to a
    /// conversation doesn't announce the replies it already had.
    first_announced_message_id: u64,
    next_reply_index: usize,
}

impl MainScreen {
    /// Creates the screen, with empty conversations.
    pub(crate) fn new() -> Self {
        Self {
            selected_contact: 0,
            conversations: CONTACTS.iter().map(|_| Conversation::default()).collect(),
            next_message_id: 0,
            first_announced_message_id: 0,
            next_reply_index: 0,
        }
    }

    fn add_message(&mut self, contact: usize, sender: Option<usize>, text: String) {
        let id = self.next_message_id;
        self.next_message_id += 1;
        self.conversations[contact]
            .messages
            .push(ChatMessage { id, sender, text });
    }

    pub(crate) fn update(&mut self, message: Message) -> Command {
        let conversation = &mut self.conversations[self.selected_contact];
        match message {
            Message::ContactSelected(contact) => {
                self.selected_contact = contact;
                self.first_announced_message_id = self.next_message_id;
            }
            Message::ChatMessageSelected(index) => conversation.selected_message = Some(index),
            Message::MessagesScrolled(position) => conversation.scroll_position = position,
            Message::DraftChanged(state) => conversation.draft = state,
            Message::SendDraft => {
                let text = conversation.draft.text().to_string();
                if text.trim().is_empty() {
                    return Command::None;
                }
                conversation.draft = TextInputState::default();
                let contact = self.selected_contact;
                self.add_message(contact, None, text);
                return Command::PostAfter(REPLY_DELAY, Message::ReplyReceived(contact));
            }
            Message::ReplyReceived(contact) => {
                self.add_message(
                    contact,
                    Some(contact),
                    REPLIES[self.next_reply_index].to_string(),
                );
                self.next_reply_index = (self.next_reply_index + 1) % 3;
            }
            _ => (),
        }
        Command::None
    }

    pub(crate) fn view(&self, root: Window<Message>, size: Size) -> Window<Message> {
        let layout = Layout::new(size);
        let contact_name = CONTACTS[self.selected_contact];
        let conversation = &self.conversations[self.selected_contact];
        let contacts = CONTACTS
            .iter()
            .enumerate()
            .map(|(index, name)| ListItem {
                id: NodeId(FIRST_CONTACT_ID + index as u64),
                label: name.to_string(),
                bounds: layout.contact(index),
                live: None,
            })
            .collect();
        let messages = conversation
            .messages
            .iter()
            .enumerate()
            .map(|(index, message)| {
                let sender = message.sender.map_or("You", |contact| CONTACTS[contact]);
                let is_announced =
                    message.sender.is_some() && message.id >= self.first_announced_message_id;
                ListItem {
                    id: NodeId(FIRST_MESSAGE_ID + message.id),
                    label: format!("{sender}: {}", message.text),
                    bounds: layout.message(index),
                    live: is_announced.then_some(Live::Polite),
                }
            })
            .collect();
        let message_list = ListBox::new(
            MESSAGE_LIST_ID,
            format!("Conversation with {contact_name}"),
            layout.message_list,
            messages,
            conversation.selected_message,
            Message::ChatMessageSelected,
        )
        .with_scrolling(conversation.scroll_position, Message::MessagesScrolled);
        let message_scroll_bar = HAS_SCROLL_BARS
            .then(|| {
                message_list.vertical_scroll_bar(MESSAGE_SCROLL_BAR_ID, layout.message_scroll_bar)
            })
            .flatten();
        let mut root = root
            .with_child(
                ListBox::new(
                    CONTACT_LIST_ID,
                    "Contacts",
                    layout.contact_list,
                    contacts,
                    Some(self.selected_contact),
                    Message::ContactSelected,
                )
                .with_set_info(true),
            )
            .with_child(message_list);
        if let Some(scroll_bar) = message_scroll_bar {
            root = root.with_child(scroll_bar);
        }
        root.with_child(
            TextInput::new(
                COMPOSER_ID,
                COMPOSER_FIRST_TEXT_RUN_ID,
                InputType::Multiline,
                &conversation.draft,
                layout.composer,
                Message::DraftChanged,
            )
            .with_label(format!("Message to {contact_name}"))
            .with_on_submit(Message::SendDraft),
        )
        .with_child(Button::new(
            SEND_BUTTON_ID,
            "Send",
            layout.send_button,
            Message::SendDraft,
        ))
    }
}
