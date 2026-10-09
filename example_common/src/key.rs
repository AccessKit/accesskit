/// A key the example handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Enter,
    Space,
    Tab,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    /// A printable character other than a space, only reported when pressed.
    Character(char),
}

impl Key {
    /// The key typing `c`, if it's a printable character other than a space.
    pub fn character(c: char) -> Option<Self> {
        (!c.is_control() && c != ' ').then_some(Self::Character(c))
    }
}

/// Whether a key was pressed or released.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

/// The modifier keys held during a key event.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
}

/// A key being pressed or released.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub key: Key,
    pub state: KeyState,
    pub modifiers: Modifiers,
}
