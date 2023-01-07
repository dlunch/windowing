use std::future::Future;

use js_sys::Promise;
use raw_window_handle::{RawWindowHandle, WebWindowHandle};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, HtmlCanvasElement};

use crate::Event;

pub struct WindowImpl {
    id: u32,
}

impl WindowImpl {
    pub fn new(_width: i32, _height: i32, _title: &str) -> Self {
        let window = web_sys::window().unwrap();

        let canvas = Document::create_element(&window.document().unwrap(), "canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        let id = 0; // TODO
        canvas.set_attribute("data-raw-handle", &id.to_string()).unwrap();

        Self { id }
    }

    pub async fn run<F, Fut>(self, handler: F)
    where
        F: Fn(Event) -> Fut + 'static,
        Fut: Future<Output = ()>,
    {
        loop {
            handler(Event::Paint).await;

            let future = JsFuture::from(Promise::new(&mut |resolve, _| {
                let closure = Closure::once_into_js(move || {
                    resolve.call0(&JsValue::NULL).unwrap();
                });

                web_sys::window()
                    .unwrap()
                    .request_animation_frame(closure.as_ref().unchecked_ref())
                    .unwrap();
            }));
            future.await.unwrap();
        }
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let mut window_handle = WebWindowHandle::empty();
        window_handle.id = self.id;

        RawWindowHandle::Web(window_handle)
    }
}
