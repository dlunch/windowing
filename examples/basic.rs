use windowing::Window;

#[tokio::main]
pub async fn main() {
    let w = Window::new(640, 480, "test");
    w.run().await;
}
