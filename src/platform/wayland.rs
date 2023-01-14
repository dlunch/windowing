use alloc::vec::Vec;
use core::iter;
use std::io;

use raw_window_handle::{RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle};
use smithay_client_toolkit::{
    new_default_environment,
    reexports::client::EventQueue,
    window::{Event as WEvent, FallbackFrame, Window},
};

use crate::Event;

smithay_client_toolkit::default_environment!(Wayland, desktop);

pub struct WindowImpl {
    window: Window<FallbackFrame>,
    queue: EventQueue,
    dimensions: (u32, u32),
}

impl WindowImpl {
    pub async fn new(width: i32, height: i32, title: &str) -> Self {
        let (env, _, queue) = new_default_environment!(Wayland, desktop).expect("Unable to connect to a Wayland compositor");

        let surface = env.create_surface().detach();
        let dimensions = (width as u32, height as u32);

        let window = env
            .create_window::<FallbackFrame, _>(surface, None, dimensions, |e, mut d| {
                let events = d.get::<Vec<WEvent>>().unwrap();

                events.push(e);
            })
            .unwrap();

        window.set_title(title.into());

        Self { window, queue, dimensions }
    }

    pub async fn next_events(&mut self, wait: bool) -> Option<impl Iterator<Item = Event>> {
        let mut events = Vec::<WEvent>::new();

        if wait {
            self.queue.dispatch(&mut events, |e, o, _| println!("Unhandled {e:?} {o:?}")).unwrap();
        } else {
            self.queue
                .dispatch_pending(&mut events, |e, o, _| println!("Unhandled {e:?} {o:?}"))
                .unwrap();
            {
                let read_guard = self.queue.prepare_read().unwrap();
                let r = read_guard.read_events();
                if let Err(err) = r {
                    if err.kind() != io::ErrorKind::WouldBlock {
                        panic!("{}", err);
                    }
                }
            }
        }

        for event in events {
            match event {
                WEvent::Refresh => {
                    self.window.refresh();
                    self.window.surface().commit();
                }
                WEvent::Close => return iter::once(Event::Close),
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
        self.queue.display().flush().unwrap();

        Some(ter::once(Event::Paint))
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
