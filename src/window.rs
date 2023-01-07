use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

use crate::{event::Event, platform::WindowImpl};

pub struct Window {
    window_impl: WindowImpl,
}

impl Window {
    pub fn new(width: i32, height: i32, title: &str) -> Self {
        let window_impl = WindowImpl::new(width, height, title);

        Self { window_impl }
    }

    pub async fn next_events(&mut self) -> impl Iterator<Item = Event> {
        self.window_impl.next_events().await
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window_impl.raw_window_handle()
    }
}
