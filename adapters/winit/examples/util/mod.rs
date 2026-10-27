use accesskit::{Size, Vec2};
use example_common::{Key, KeyEvent, KeyState, Modifiers, UiState};
use winit::{
    event::{ElementState, KeyEvent as WinitKeyEvent, Modifiers as WinitModifiers},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key as WinitKey, NamedKey},
    window::Window,
};

#[cfg(target_os = "ios")]
fn safe_area_inset(window: &Window) -> Vec2 {
    let Ok(outer) = window.outer_position() else {
        return Vec2::ZERO;
    };
    let Ok(inner) = window.inner_position() else {
        return Vec2::ZERO;
    };
    Vec2::new((inner.x - outer.x) as f64, (inner.y - outer.y) as f64)
}

#[cfg(not(target_os = "ios"))]
fn safe_area_inset(_: &Window) -> Vec2 {
    Vec2::ZERO
}

/// Tells the UI the window's scale factor, and the area it can use.
pub fn set_viewport(ui: &mut UiState, window: &Window) {
    let scale_factor = window.scale_factor();
    let size = window.inner_size().to_logical::<f64>(scale_factor);
    ui.set_viewport(
        scale_factor,
        safe_area_inset(window),
        Size::new(size.width, size.height),
    );
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

/// Handles the delayed messages that are due, and schedules the next ones.
pub fn process_due_messages(event_loop: &ActiveEventLoop, ui: &mut UiState) -> bool {
    let processed = ui.process_due_messages();
    let control_flow = match ui.time_until_next_message() {
        Some(remaining) => ControlFlow::wait_duration(remaining),
        None => ControlFlow::Wait,
    };
    event_loop.set_control_flow(control_flow);
    processed
}
