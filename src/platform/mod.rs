#[cfg(target_os = "linux")]
mod wayland;
#[cfg(target_os = "linux")]
pub use self::wayland::WindowImpl;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::WindowImpl;
