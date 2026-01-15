// Adapted from winit's examples/util/fill.rs.

//! Fill the window buffer with a solid color.
//!
//! Launching a window without drawing to it has unpredictable results varying from platform to
//! platform. In order to have well-defined examples, this module provides an easy way to
//! fill the window buffer with a solid color.
//!
//! The `softbuffer` crate is used, largely because of its ease of use. `glutin` or `wgpu` could
//! also be used to fill the window buffer, but they are more complicated to use.

pub use platform::cleanup_window;
pub use platform::fill_window;
pub use platform::draw_button;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod platform {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::mem;
    use std::mem::ManuallyDrop;
    use std::num::NonZeroU32;

    use accesskit::Rect;
    use softbuffer::{Context, Surface};
    use winit::window::{Window, WindowId};

    thread_local! {
        // NOTE: You should never do things like that, create context and drop it before
        // you drop the event loop. We do this for brevity to not blow up examples. We use
        // ManuallyDrop to prevent destructors from running.
        //
        // A static, thread-local map of graphics contexts to open windows.
        static GC: ManuallyDrop<RefCell<Option<GraphicsContext>>> = const { ManuallyDrop::new(RefCell::new(None)) };
    }

    /// The graphics context used to draw to a window.
    struct GraphicsContext {
        /// The global softbuffer context.
        context: RefCell<Context<&'static Window>>,

        /// The hash map of window IDs to surfaces.
        surfaces: HashMap<WindowId, Surface<&'static Window, &'static Window>>,
    }

    impl GraphicsContext {
        fn new(w: &Window) -> Self {
            Self {
                context: RefCell::new(
                    Context::new(unsafe { mem::transmute::<&'_ Window, &'static Window>(w) })
                        .expect("Failed to create a softbuffer context"),
                ),
                surfaces: HashMap::new(),
            }
        }

        fn create_surface(
            &mut self,
            window: &Window,
        ) -> &mut Surface<&'static Window, &'static Window> {
            self.surfaces.entry(window.id()).or_insert_with(|| {
                Surface::new(&self.context.borrow(), unsafe {
                    mem::transmute::<&'_ Window, &'static Window>(window)
                })
                .expect("Failed to create a softbuffer surface")
            })
        }

        fn destroy_surface(&mut self, window: &Window) {
            self.surfaces.remove(&window.id());
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct FrameSize {
        pub width: u32,
        pub height: u32
    }
    pub fn draw_button(
        buffer: &mut [u32],
        size: FrameSize,
        rect: &Rect,
        focused: bool,
    ) {
        // Colors (ARGB format: 0xAARRGGBB)
        const BUTTON_COLOR: u32 = 0xff3366aa;       // Blue button
        const BUTTON_FOCUSED: u32 = 0xff66aaff;     // Lighter blue when focused
        const BORDER_COLOR: u32 = 0xffffffff;       // White border
        const FOCUS_BORDER: u32 = 0xffffcc00;       // Yellow focus ring

        let x0 = rect.x0 as usize;
        let y0 = rect.y0 as usize;
        let x1 = rect.x1 as usize;
        let y1 = rect.y1 as usize;

        let fill_color = if focused { BUTTON_FOCUSED } else { BUTTON_COLOR };
        let border_color = if focused { FOCUS_BORDER } else { BORDER_COLOR };

        // Uses FrameSize.width for jumping to start of scan lines
        let stride: usize = size.width as usize;

        // Draw the button fill
        for y in y0..y1 {
            for x in x0..x1 {
                let idx = y * stride + x;
                if idx < buffer.len() {
                    // Check if we're on the border (2px border)
                    let on_border = x < x0 + 2 || x >= x1 - 2 || y < y0 + 2 || y >= y1 - 2;
                    buffer[idx] = if on_border { border_color } else { fill_color };
                }
            }
        }

        // Draw focus ring (additional 2px outside border if focused)
        if focused {
            let ring_x0 = x0.saturating_sub(2);
            let ring_y0 = y0.saturating_sub(2);
            let ring_x1 = x1 + 2;
            let ring_y1 = y1 + 2;

            // Top edge
            for y in ring_y0..y0 {
                for x in ring_x0..ring_x1 {
                    let idx = y * stride + x;
                    if idx < buffer.len() {
                        buffer[idx] = FOCUS_BORDER;
                    }
                }
            }
            // Bottom edge
            for y in y1..ring_y1 {
                for x in ring_x0..ring_x1 {
                    let idx = y * stride + x;
                    if idx < buffer.len() {
                        buffer[idx] = FOCUS_BORDER;
                    }
                }
            }
            // Left edge
            for y in y0..y1 {
                for x in ring_x0..x0 {
                    let idx = y * stride + x;
                    if idx < buffer.len() {
                        buffer[idx] = FOCUS_BORDER;
                    }
                }
            }
            // Right edge
            for y in y0..y1 {
                for x in x1..ring_x1 {
                    let idx = y * stride + x;
                    if idx < buffer.len() {
                        buffer[idx] = FOCUS_BORDER;
                    }
                }
            }
        }
    }

    pub fn fill_window<F>(window: &Window, udf_draw: F)
    where
        F: FnOnce(&mut [u32], FrameSize),
    {
        GC.with(|gc| {
            let size = window.inner_size();
            let (Some(width), Some(height)) =
                (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
            else {
                return;
            };

            // Either get the last context used or create a new one.
            let mut gc = gc.borrow_mut();
            let surface = gc
                .get_or_insert_with(|| GraphicsContext::new(window))
                .create_surface(window);

            surface
                .resize(width, height)
                .expect("Failed to resize the softbuffer surface");

            let mut buffer = surface
                .buffer_mut()
                .expect("Failed to get the softbuffer buffer");

            const DARK_GRAY: u32 = 0xff181818;
            buffer.fill(DARK_GRAY);

            udf_draw(&mut buffer, FrameSize { width: size.width, height: size.height });

            buffer
                .present()
                .expect("Failed to present the softbuffer buffer");
        })
    }

    pub fn cleanup_window(window: &Window) {
        GC.with(|gc| {
            let mut gc = gc.borrow_mut();
            if let Some(context) = gc.as_mut() {
                context.destroy_surface(window);
            }
        });
    }
}

#[cfg(any(target_os = "android", target_os = "ios"))]
mod platform {
    pub fn fill_window(_window: &winit::window::Window) {
        // No-op on mobile platforms.
    }

    pub fn cleanup_window(_window: &winit::window::Window) {
        // No-op on mobile platforms.
    }
}
