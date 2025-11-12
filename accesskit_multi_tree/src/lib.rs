use accesskit::TreeUpdate;

pub struct Adapter {
    // TODO: servoshell on Android and OpenHarmony do not use winit
    inner: accesskit_winit::Adapter,
}

impl Adapter {
    pub fn new(inner: accesskit_winit::Adapter) -> Self {
        Self { inner }
    }

    pub fn update_if_active(&mut self, updater: impl FnOnce() -> TreeUpdate) {
        self.inner.update_if_active(updater);
    }
}
