mod platform;

use platform::WindowImpl;

pub struct Window {
    window_impl: WindowImpl,
}

impl Window {
    pub fn new(width: i32, height: i32, title: &str) -> Self {
        let window_impl = WindowImpl::new(width, height, title);

        Self { window_impl }
    }

    pub fn run(self) {
        self.window_impl.run()
    }
}
