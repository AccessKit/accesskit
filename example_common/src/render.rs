//! Adapted from winit's `examples/util/fill.rs`.

pub use platform::Renderer;

#[cfg(not(target_os = "android"))]
mod platform {
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
    use softbuffer::{Context, Surface};
    use std::num::NonZeroU32;

    const DARK_GRAY: u32 = 0xff181818;

    pub struct Renderer<W> {
        // Declared first so that it's dropped before the context it came from.
        surface: Surface<W, W>,
        // Kept alive for as long as the surface that was made from it.
        _context: Context<W>,
    }

    impl<W: HasDisplayHandle + HasWindowHandle + Clone> Renderer<W> {
        /// Create a renderer for a window.
        ///
        /// Drop this before the window itself goes away.
        pub fn new(window: W) -> Self {
            let context =
                Context::new(window.clone()).expect("failed to create a softbuffer context");
            let surface =
                Surface::new(&context, window).expect("failed to create a softbuffer surface");
            Self {
                surface,
                _context: context,
            }
        }

        /// Fill the window with a solid color.
        ///
        /// The size is in physical pixels. Nothing is drawn if either
        /// dimension is zero, as happens when a window is minimized.
        pub fn draw(&mut self, width: u32, height: u32) {
            let (Some(width), Some(height)) = (NonZeroU32::new(width), NonZeroU32::new(height))
            else {
                return;
            };

            self.surface
                .resize(width, height)
                .expect("failed to resize the softbuffer surface");

            let mut buffer = self
                .surface
                .buffer_mut()
                .expect("failed to get the softbuffer buffer");
            buffer.fill(DARK_GRAY);
            buffer
                .present()
                .expect("failed to present the softbuffer buffer");
        }
    }
}

#[cfg(target_os = "android")]
mod platform {
    use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
    use std::marker::PhantomData;

    /// Drawing is a no-op on Android, which softbuffer doesn't support.
    pub struct Renderer<W>(PhantomData<W>);

    impl<W: HasDisplayHandle + HasWindowHandle + Clone> Renderer<W> {
        pub fn new(_window: W) -> Self {
            Self(PhantomData)
        }

        pub fn draw(&mut self, _width: u32, _height: u32) {}
    }
}
