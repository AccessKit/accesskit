use accesskit::{NodeId, Rect, Size};
use std::time::Duration;

use crate::{
    Command, Message, rect,
    widgets::{
        Button, Image, InputType, Label, Spinner, TextInput, TextInputState, ToggleButton, Window,
    },
};

const ILLUSTRATION_ID: NodeId = NodeId(1);
const WELCOME_ID: NodeId = NodeId(2);
const EMAIL_LABEL_ID: NodeId = NodeId(3);
const EMAIL_INPUT_ID: NodeId = NodeId(4);
const PASSWORD_LABEL_ID: NodeId = NodeId(6);
const PASSWORD_INPUT_ID: NodeId = NodeId(7);
const SHOW_PASSWORD_ID: NodeId = NodeId(9);
/// The text runs of each input use consecutive IDs from these.
const EMAIL_FIRST_TEXT_RUN_ID: NodeId = NodeId(10_000);
const PASSWORD_FIRST_TEXT_RUN_ID: NodeId = NodeId(20_000);
const LOGIN_BUTTON_ID: NodeId = NodeId(10);
const SPINNER_ID: NodeId = NodeId(11);
/// The widget that has focus when the screen appears.
pub(crate) const INITIAL_FOCUS: NodeId = EMAIL_INPUT_ID;

const MARGIN: f64 = 24.0;
const COLUMN_MAX_WIDTH: f64 = 345.0;

/// The centered column the controls are stacked in.
struct Layout {
    x0: f64,
    x1: f64,
}

impl Layout {
    fn new(size: Size) -> Self {
        let width = (size.width - 2.0 * MARGIN).min(COLUMN_MAX_WIDTH);
        let x0 = (size.width - width) / 2.0;
        Self { x0, x1: x0 + width }
    }

    fn row(&self, y0: f64, y1: f64) -> Rect {
        rect(self.x0, y0, self.x1, y1)
    }

    fn square(&self, y0: f64, side: f64) -> Rect {
        let x0 = (self.x0 + self.x1 - side) / 2.0;
        rect(x0, y0, x0 + side, y0 + side)
    }
}

/// How long the simulated login takes.
const LOGIN_DURATION: Duration = Duration::from_millis(500);

/// The screen for logging in, with any email and password.
#[derive(Default)]
pub(crate) struct LoginScreen {
    email: TextInputState,
    password: TextInputState,
    is_password_visible: bool,
    is_logging_in: bool,
}

impl LoginScreen {
    pub(crate) fn update(&mut self, message: Message) -> Command {
        match message {
            Message::EmailChanged(state) => self.email = state,
            Message::PasswordChanged(state) => self.password = state,
            Message::PasswordVisibilityChanged(is_visible) => {
                self.is_password_visible = is_visible;
            }
            Message::LogIn => {
                self.is_logging_in = true;
                return Command::PostAfter(LOGIN_DURATION, Message::LoginFinished);
            }
            _ => (),
        }
        Command::None
    }

    pub(crate) fn view(&self, root: Window<Message>, size: Size) -> Window<Message> {
        let layout = Layout::new(size);
        let password_type = if self.is_password_visible {
            InputType::Text
        } else {
            InputType::Password
        };
        let root = root
            .with_child(Image::new(
                ILLUSTRATION_ID,
                "Diagram of three people linked by arrows, exchanging messages",
                layout.row(24.0, 200.0),
            ))
            .with_child(Label::new(
                WELCOME_ID,
                "Welcome back! Log in to your workspace.",
                layout.row(216.0, 244.0),
            ))
            .with_child(Label::new(
                EMAIL_LABEL_ID,
                "Email",
                layout.row(276.0, 296.0),
            ))
            .with_child(
                TextInput::new(
                    EMAIL_INPUT_ID,
                    EMAIL_FIRST_TEXT_RUN_ID,
                    InputType::Email,
                    &self.email,
                    layout.row(300.0, 344.0),
                    Message::EmailChanged,
                )
                .with_labelled_by(EMAIL_LABEL_ID)
                .with_placeholder("name@company.com")
                .with_required(true)
                .with_disabled(self.is_logging_in),
            )
            .with_child(Label::new(
                PASSWORD_LABEL_ID,
                "Password",
                layout.row(360.0, 380.0),
            ))
            .with_child(
                TextInput::new(
                    PASSWORD_INPUT_ID,
                    PASSWORD_FIRST_TEXT_RUN_ID,
                    password_type,
                    &self.password,
                    layout.row(384.0, 428.0),
                    Message::PasswordChanged,
                )
                .with_labelled_by(PASSWORD_LABEL_ID)
                .with_required(true)
                .with_disabled(self.is_logging_in),
            )
            .with_child(
                ToggleButton::new(
                    SHOW_PASSWORD_ID,
                    "Show password",
                    layout.row(444.0, 476.0),
                    self.is_password_visible,
                    Message::PasswordVisibilityChanged,
                )
                .with_controls(PASSWORD_INPUT_ID)
                .with_disabled(self.is_logging_in),
            )
            .with_child(
                Button::new(
                    LOGIN_BUTTON_ID,
                    "Log in",
                    layout.row(500.0, 548.0),
                    Message::LogIn,
                )
                .with_disabled(self.is_logging_in),
            );
        if self.is_logging_in {
            root.with_child(Spinner::new(
                SPINNER_ID,
                "Logging in",
                layout.square(564.0, 24.0),
            ))
        } else {
            root
        }
    }
}
