use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};

use crate::{event::Event, platform::WindowImpl};

pub struct Window {
    window_impl: WindowImpl,
}

impl Window {
    pub async fn new(width: i32, height: i32, title: &str) -> Self {
        let window_impl = WindowImpl::new(width, height, title).await;

        Self { window_impl }
    }

    pub async fn next_events(&mut self, wait: bool) -> Option<impl Iterator<Item = Event>> {
        self.window_impl.next_events(wait).await
    }
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window_impl.raw_window_handle()
    }
}

unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.window_impl.raw_display_handle()
    }
}
