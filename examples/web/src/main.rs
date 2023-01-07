use wasm_bindgen_futures::spawn_local;

use windowing::Window;

pub fn main() {
    spawn_local(async move {
        let w = Window::new(640, 480, "test");
        w.run().await;
    });
}
