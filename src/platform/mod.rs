#[cfg(target_os = "linux")]
mod wayland;
#[cfg(target_os = "linux")]
pub use self::wayland::WindowImpl;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use self::web::WindowImpl;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::WindowImpl;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use self::macos::WindowImpl;
