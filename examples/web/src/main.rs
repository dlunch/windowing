use wasm_bindgen_futures::spawn_local;

use windowing::Window;

pub fn main() {
    spawn_local(async move {
        let mut w = Window::new(640, 480, "test");
        loop {
            let events = w.next_events().await;
            for event in events {
                println!("{:?}", event);
            }
        }
    });
}
