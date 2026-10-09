use accesskit::Vec2;
use example_common::{Key, KeyEvent, KeyState, Modifiers, UiState};
use winit::{
    event::{ElementState, KeyEvent as WinitKeyEvent, Modifiers as WinitModifiers},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key as WinitKey, NamedKey},
    window::Window,
};

#[cfg(target_os = "ios")]
pub fn safe_area_inset(window: &Window) -> Vec2 {
    let Ok(outer) = window.outer_position() else {
        return Vec2::ZERO;
    };
    let Ok(inner) = window.inner_position() else {
        return Vec2::ZERO;
    };
    Vec2::new((inner.x - outer.x) as f64, (inner.y - outer.y) as f64)
}

#[cfg(not(target_os = "ios"))]
pub fn safe_area_inset(_: &Window) -> Vec2 {
    Vec2::ZERO
}

/// Translates a winit key event into the example's key events.
pub fn key_events(event: &WinitKeyEvent, modifiers: Modifiers) -> Vec<KeyEvent> {
    let state = match event.state {
        ElementState::Pressed => KeyState::Pressed,
        ElementState::Released => KeyState::Released,
    };
    let key_event = |key| KeyEvent {
        key,
        state,
        modifiers,
    };
    let key = match &event.logical_key {
        WinitKey::Named(NamedKey::Enter) => Key::Enter,
        WinitKey::Named(NamedKey::Space) => Key::Space,
        WinitKey::Named(NamedKey::Tab) => Key::Tab,
        WinitKey::Named(NamedKey::Backspace) => Key::Backspace,
        WinitKey::Named(NamedKey::Delete) => Key::Delete,
        WinitKey::Named(NamedKey::ArrowLeft) => Key::Left,
        WinitKey::Named(NamedKey::ArrowRight) => Key::Right,
        WinitKey::Named(NamedKey::ArrowUp) => Key::Up,
        WinitKey::Named(NamedKey::ArrowDown) => Key::Down,
        WinitKey::Named(NamedKey::Home) => Key::Home,
        WinitKey::Named(NamedKey::End) => Key::End,
        _ if state == KeyState::Pressed && !modifiers.control => {
            let text = event.text.as_deref().unwrap_or_default();
            return text
                .chars()
                .filter_map(Key::character)
                .map(key_event)
                .collect();
        }
        _ => return Vec::new(),
    };
    vec![key_event(key)]
}

pub fn modifiers(modifiers: &WinitModifiers) -> Modifiers {
    Modifiers {
        shift: modifiers.state().shift_key(),
        control: modifiers.state().control_key(),
    }
}

pub fn flush_announcement(event_loop: &ActiveEventLoop, ui: &mut UiState) -> bool {
    match ui.time_until_announcement() {
        Some(remaining) if !remaining.is_zero() => {
            event_loop.set_control_flow(ControlFlow::wait_duration(remaining));
            false
        }
        Some(_) => {
            let flushed = ui.flush_announcement();
            event_loop.set_control_flow(ControlFlow::Wait);
            flushed
        }
        None => false,
    }
}
