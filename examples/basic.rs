use windowing::Window;

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run());

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(run());
}

pub async fn run() {
    let mut w = Window::new(640, 480, "test").await;
    loop {
        let events = w.next_events(true).await;
        for event in events {
            println!("{:?}", event);
        }
    }
}
