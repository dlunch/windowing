use std::future::Future;

use raw_window_handle::{RawWindowHandle, WebWindowHandle};

use crate::Event;

#[derive(Default)]
pub struct WindowImpl {}

impl WindowImpl {
    pub fn new(_width: i32, _height: i32, _title: &str) -> Self {
        Self {}
    }

    pub async fn run<F, Fut>(self, _: F)
    where
        F: Fn(Event) -> Fut,
        Fut: Future<Output = ()>,
    {
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let window_handle = WebWindowHandle::empty();
        // TODO

        RawWindowHandle::Web(window_handle)
    }
}
