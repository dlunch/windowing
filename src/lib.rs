#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

mod event;
mod platform;
mod window;

pub use event::Event;
pub use window::Window;
