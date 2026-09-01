mod util;

use accesskit::{ActivationHandler, TreeUpdate};
use accesskit_winit::{Adapter, Event as AccessKitEvent, WindowEvent as AccessKitWindowEvent};
use example_common::{Modifiers, Renderer, UiState, WINDOW_TITLE};
use std::{
    error::Error,
    sync::{Arc, Mutex},
};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent as WinitKeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

struct TearoffActivationHandler {
    ui: Arc<Mutex<UiState>>,
}

impl ActivationHandler for TearoffActivationHandler {
    fn request_initial_tree(&mut self) -> Option<TreeUpdate> {
        Some(self.ui.lock().unwrap().build_tree_update())
    }
}

struct WindowState {
    // Declared first so that the renderer is dropped before the window.
    renderer: Renderer<Arc<Window>>,
    window: Arc<Window>,
    adapter: Adapter,
    ui: Arc<Mutex<UiState>>,
    modifiers: Modifiers,
}

impl WindowState {
    fn new(window: Arc<Window>, adapter: Adapter, ui: Arc<Mutex<UiState>>) -> Self {
        Self {
            renderer: Renderer::new(Arc::clone(&window)),
            window,
            adapter,
            ui,
            modifiers: Modifiers::default(),
        }
    }

    fn update_accessibility_tree(&mut self) {
        let mut ui = self.ui.lock().unwrap();
        self.adapter.update_if_active(|| ui.build_tree_update());
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
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_visible(false);

        let window = Arc::new(event_loop.create_window(window_attributes)?);
        let mut ui = UiState::new();
        ui.set_viewport(window.scale_factor(), util::safe_area_inset(&window));
        let ui = Arc::new(Mutex::new(ui));
        let activation_handler = TearoffActivationHandler {
            ui: Arc::clone(&ui),
        };
        let adapter = Adapter::with_mixed_handlers(
            event_loop,
            &window,
            activation_handler,
            self.event_loop_proxy.clone(),
        );
        window.set_visible(true);

        self.window = Some(WindowState::new(window, adapter, ui));
        Ok(())
    }
}

impl ApplicationHandler<AccessKitEvent> for Application {
    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let Some(window) = &mut self.window else {
            return;
        };

        window.adapter.process_event(&window.window, &event);
        match event {
            WindowEvent::CloseRequested => {
                self.window = None;
            }
            WindowEvent::Resized(_) => {
                let scale_factor = window.window.scale_factor();
                let inset = util::safe_area_inset(&window.window);
                window.ui.lock().unwrap().set_viewport(scale_factor, inset);
                window.update_accessibility_tree();
                window.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let size = window.window.inner_size();
                window.renderer.draw(size.width, size.height);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                window.modifiers = util::modifiers(&modifiers);
            }
            WindowEvent::KeyboardInput {
                event: WinitKeyEvent {
                    logical_key, state, ..
                },
                ..
            } => {
                if let Some(event) = util::key_event(&logical_key, state, window.modifiers) {
                    window.ui.lock().unwrap().handle_key(event);
                    window.update_accessibility_tree();
                    window.window.request_redraw();
                }
            }
            _ => (),
        }
    }

    fn user_event(&mut self, _: &ActiveEventLoop, user_event: AccessKitEvent) {
        let Some(window) = &mut self.window else {
            return;
        };

        match user_event.window_event {
            // The tearoff activation handler takes care of this.
            AccessKitWindowEvent::InitialTreeRequested => unreachable!(),
            AccessKitWindowEvent::ActionRequested(request) => {
                window.ui.lock().unwrap().do_action(&request);
                window.update_accessibility_tree();
                window.window.request_redraw();
            }
            AccessKitWindowEvent::AccessibilityDeactivated => {
                window.ui.lock().unwrap().deactivated();
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.create_window(event_loop)
                .expect("failed to create initial window");
        }
        if let Some(window) = self.window.as_ref() {
            window.window.request_redraw();
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = &mut self.window else {
            event_loop.exit();
            return;
        };

        let flushed = util::flush_announcement(event_loop, &mut window.ui.lock().unwrap());
        if flushed {
            window.update_accessibility_tree();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    example_common::print_instructions();

    let event_loop = EventLoop::with_user_event().build()?;
    let mut state = Application::new(event_loop.create_proxy());
    event_loop.run_app(&mut state).map_err(Into::into)
}
