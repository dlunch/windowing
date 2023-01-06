use std::future::Future;

use crate::Event;

#[derive(Default)]
pub struct WindowImpl {}

impl WindowImpl {
    pub fn new(_width: i32, _height: i32, _title: &str) -> Self {
        Self {}
    }

    pub async fn run<F, Fut>(mut self, _: F)
    where
        F: Fn(Event) -> Fut,
        Fut: Future<Output = ()>,
    {
    }
}
