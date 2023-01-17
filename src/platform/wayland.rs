use alloc::vec::Vec;
use core::iter;
use std::{
    io,
    os::unix::io::{AsRawFd, RawFd},
};

use raw_window_handle::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};
use smithay_client_toolkit::{
    new_default_environment,
    reexports::client::EventQueue,
    window::{Event as WEvent, FallbackFrame, Window},
};
use tokio::io::unix::AsyncFd;

use crate::Event;

smithay_client_toolkit::default_environment!(Wayland, desktop);

struct WaylandConnectionFd {
    connection_fd: RawFd,
}

impl AsRawFd for WaylandConnectionFd {
    fn as_raw_fd(&self) -> RawFd {
        self.connection_fd
    }
}

pub struct WindowImpl {
    window: Window<FallbackFrame>,
    queue: EventQueue,
    dimensions: (u32, u32),
    connection_fd: AsyncFd<WaylandConnectionFd>,
}

impl WindowImpl {
    pub async fn new(width: i32, height: i32, title: &str) -> Self {
        let (env, display, queue) = new_default_environment!(Wayland, desktop).expect("Unable to connect to a Wayland compositor");

        let surface = env.create_surface().detach();
        let dimensions = (width as u32, height as u32);

        let window = env
            .create_window::<FallbackFrame, _>(surface, None, dimensions, |e, mut d| {
                let events = d.get::<Vec<WEvent>>().unwrap();

                events.push(e);
            })
            .unwrap();

        window.set_title(title.into());

        let connection_fd = display.get_connection_fd();

        Self {
            window,
            queue,
            dimensions,
            connection_fd: AsyncFd::new(WaylandConnectionFd { connection_fd }).unwrap(),
        }
    }

    pub async fn next_events(&mut self, wait: bool) -> Option<impl Iterator<Item = Event>> {
        let mut events = Vec::<WEvent>::new();
        self.queue.display().flush().unwrap();

        if wait {
            self.connection_fd.readable().await.unwrap().clear_ready();
        }

        {
            let read_guard = self.queue.prepare_read().unwrap();
            let r = read_guard.read_events();
            if let Err(err) = r {
                if err.kind() != io::ErrorKind::WouldBlock {
                    panic!("{}", err);
                }
            }
        }

        self.queue
            .dispatch_pending(&mut events, |e, o, _| println!("Unhandled {e:?} {o:?}"))
            .unwrap();

        for event in events {
            match event {
                WEvent::Refresh => {
                    self.window.refresh();
                    self.window.surface().commit();
                }
                WEvent::Close => return Some(iter::once(Event::Close)),
                WEvent::Configure { new_size, states } => {
                    if let Some((w, h)) = new_size {
                        self.window.resize(w, h);
                        self.dimensions = (w, h);
                    }
                    println!("Window states: {states:?}");

                    self.window.refresh();
                }
            }
        }

        Some(iter::once(Event::Paint))
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let mut window_handle = WaylandWindowHandle::empty();
        window_handle.surface = self.window.surface().as_ref().c_ptr() as *mut _;

        RawWindowHandle::Wayland(window_handle)
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        let mut display_handle = WaylandDisplayHandle::empty();
        display_handle.display = self.queue.display().get_display_ptr() as *mut _;

        RawDisplayHandle::Wayland(display_handle)
    }
}
