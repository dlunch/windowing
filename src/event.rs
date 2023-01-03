pub enum Event {
    Created,
    Resize((i32, i32)),
    Paint,
    Close,
}
