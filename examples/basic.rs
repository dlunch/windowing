use core::time::Duration;

use tokio::time;

use windowing::Window;

#[tokio::main]
pub async fn main() {
    let mut w = Window::new(640, 480, "test");
    loop {
        let events = w.next_events().await;
        for event in events {
            println!("{:?}", event);
        }

        time::sleep(Duration::from_millis(16)).await;
    }
}
