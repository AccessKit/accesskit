use accesskit::{ActivationHandler, TreeUpdate};
use accesskit_demo_lib::{Key, WindowState as AccessKitWindowState};
use accesskit_winit::{Adapter, Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    keyboard::NamedKey,
    window::{Window, WindowId},
};

struct TearoffActivationHandler {
    state: Arc<Mutex<AccessKitWindowState>>,
}

impl ActivationHandler for TearoffActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.state.lock().unwrap().build_initial_tree())
    }
}

struct WindowState {
    window: Window,
    adapter: Adapter,
    inner: Arc<Mutex<AccessKitWindowState>>,
}

impl WindowState {
    fn new(window: Window, adapter: Adapter, inner: Arc<Mutex<AccessKitWindowState>>) -> Self {
        Self {
            window,
            adapter,
            inner,
        }
    }
}

struct Application {
    event_loop_proxy: EventLoopProxy<AccessKitEvent>,
    window: Option<WindowState>,
}

impl Application {
    fn new(event_loop_proxy: EventLoopProxy<AccessKitEvent>) -> Self {
        Self {
            event_loop_proxy,
            window: None,
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<(), Box<dyn Error>> {
        let state = Arc::new(Mutex::new(AccessKitWindowState::default()));
        let window_attributes = Window::default_attributes()
            .with_title(state.lock().unwrap().title())
            .with_visible(false);

        let window = event_loop.create_window(window_attributes)?;
        let activation_handler = TearoffActivationHandler {
            state: Arc::clone(&state),
        };
        let adapter = Adapter::with_mixed_handlers(
            event_loop,
            &window,
            activation_handler,
            self.event_loop_proxy.clone(),
        );
        window.set_visible(true);

        self.window = Some(WindowState::new(window, adapter, state));
        Ok(())
    }
}

impl ApplicationHandler<AccessKitEvent> for Application {
    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let window = match &mut self.window {
            Some(window) => window,
            None => return,
        };
        let adapter = &mut window.adapter;
        let state = &mut window.inner;

        adapter.process_event(&window.window, &event);
        match event {
            WindowEvent::CloseRequested => {
                self.window = None;
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: virtual_code,
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                let key = match virtual_code {
                    winit::keyboard::Key::Named(NamedKey::ArrowDown) => Some(Key::Down),
                    winit::keyboard::Key::Named(NamedKey::ArrowLeft) => Some(Key::Left),
                    winit::keyboard::Key::Named(NamedKey::ArrowRight) => Some(Key::Right),
                    winit::keyboard::Key::Named(NamedKey::Space) => Some(Key::Space),
                    winit::keyboard::Key::Named(NamedKey::Tab) => Some(Key::Tab),
                    winit::keyboard::Key::Named(NamedKey::ArrowUp) => Some(Key::Up),
                    _ => None,
                };
                if let Some(key) = key {
                    let mut state = state.lock().unwrap();
                    state.key_pressed(key);
                    adapter.update_if_active(|| state.build_tree());
                }
            }
            _ => (),
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, user_event: AccessKitEvent) {
        let window = match &mut self.window {
            Some(window) => window,
            None => return,
        };
        let adapter = &mut window.adapter;
        let state = &mut window.inner.lock().unwrap();

        match user_event.window_event {
            AccessKitWindowEvent::InitialTreeRequested => unreachable!(),
            AccessKitWindowEvent::ActionRequested(request) => {
                state.do_action(&request);
                adapter.update_if_active(|| state.build_tree());
            }
            AccessKitWindowEvent::AccessibilityDeactivated => {
                state.deactivate_accessibility();
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.create_window(event_loop)
            .expect("failed to create initial window");
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            event_loop.exit();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("This example has no visible GUI, and a keyboard interface:");
    println!("- [Tab] switches focus between two logical buttons.");
    println!("- [Space] 'presses' the button, adding static text in a live region announcing that it was pressed.");
    #[cfg(target_os = "windows")]
    println!("Enable Narrator with [Win]+[Ctrl]+[Enter] (or [Win]+[Enter] on older versions of Windows).");
    #[cfg(all(
        feature = "accesskit_unix",
        any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    ))]
    println!("Enable Orca with [Super]+[Alt]+[S].");

    let event_loop = EventLoop::with_user_event().build()?;
    let mut state = Application::new(event_loop.create_proxy());
    event_loop.run_app(&mut state).map_err(Into::into)
}
