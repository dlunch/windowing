use core::iter;

use raw_window_handle::{AppKitWindowHandle, RawWindowHandle};

use crate::Event;

#[derive(Default)]
pub struct WindowImpl {}

impl WindowImpl {
    pub fn new(_width: i32, _height: i32, _title: &str) -> Self {
        Self {}
    }

    pub async fn next_events(&self) -> impl Iterator<Item = Event> {
        iter::once(Event::Paint)
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let window_handle = AppKitWindowHandle::empty();
        // TODO

        RawWindowHandle::AppKit(window_handle)
    }
}
