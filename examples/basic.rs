use windowing::Window;

#[tokio::main]
pub async fn main() {
    let mut w = Window::new(640, 480, "test");
    loop {
        let events = w.next_events(true).await;
        for event in events {
            println!("{:?}", event);
        }
    }
}
