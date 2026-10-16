use accesskit::Vec2;
use example_common::{Key, KeyEvent, KeyState, Modifiers, UiState};
use winit::{
    event::{ElementState, Modifiers as WinitModifiers},
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

pub fn key_event(key: &WinitKey, state: ElementState, modifiers: Modifiers) -> Option<KeyEvent> {
    let key = match key {
        WinitKey::Named(NamedKey::Enter) => Key::Enter,
        WinitKey::Named(NamedKey::Space) => Key::Space,
        WinitKey::Named(NamedKey::Tab) => Key::Tab,
        _ => return None,
    };
    let state = match state {
        ElementState::Pressed => KeyState::Pressed,
        ElementState::Released => KeyState::Released,
    };
    Some(KeyEvent {
        key,
        state,
        modifiers,
    })
}

pub fn modifiers(modifiers: &WinitModifiers) -> Modifiers {
    Modifiers {
        shift: modifiers.state().shift_key(),
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
