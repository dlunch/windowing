use core::iter;

use raw_window_handle::{AppKitDisplayHandle, AppKitWindowHandle, RawDisplayHandle, RawWindowHandle};

use crate::Event;

#[derive(Default)]
pub struct WindowImpl {}

impl WindowImpl {
    pub async fn new(_width: i32, _height: i32, _title: &str) -> Self {
        Self {}
    }

    pub async fn next_events(&self, _: bool) -> impl Iterator<Item = Event> {
        iter::once(Event::Paint)
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let window_handle = AppKitWindowHandle::empty();
        // TODO

        RawWindowHandle::AppKit(window_handle)
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        let display_handle = AppKitDisplayHandle::empty();

        RawDisplayHandle::AppKit(display_handle)
    }
}
