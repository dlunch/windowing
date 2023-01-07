use alloc::vec::Vec;
use core::{cmp::min, iter};
use std::io;

use raw_window_handle::{RawWindowHandle, WaylandWindowHandle};
use smithay_client_toolkit::{
    environment::Environment,
    new_default_environment,
    reexports::client::{
        protocol::{wl_shm, wl_surface},
        EventQueue,
    },
    shm::AutoMemPool,
    window::{Event as WEvent, FallbackFrame, Window},
};

use crate::Event;

smithay_client_toolkit::default_environment!(Wayland, desktop);

pub struct WindowImpl {
    env: Environment<Wayland>,
    window: Window<FallbackFrame>,
    queue: EventQueue,
    dimensions: (u32, u32),
}

impl WindowImpl {
    pub fn new(width: i32, height: i32, title: &str) -> Self {
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

        Self {
            env,
            window,
            queue,
            dimensions,
        }
    }

    pub async fn next_events(&mut self) -> impl Iterator<Item = Event> {
        let mut pool = self.env.create_auto_pool().expect("Failed to create a memory pool !");

        let mut events = Vec::<WEvent>::new();
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
                WEvent::Close => return iter::once(Event::Close),
                WEvent::Configure { new_size, states } => {
                    if let Some((w, h)) = new_size {
                        self.window.resize(w, h);
                        self.dimensions = (w, h);
                    }
                    println!("Window states: {states:?}");

                    self.window.refresh();
                    redraw(&mut pool, self.window.surface(), self.dimensions).expect("Failed to draw");
                }
            }
        }
        self.queue.display().flush().unwrap();

        iter::once(Event::Paint)
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let mut window_handle = WaylandWindowHandle::empty();
        window_handle.surface = self.window.surface().as_ref().c_ptr() as *mut _;

        RawWindowHandle::Wayland(window_handle)
    }
}

#[allow(clippy::many_single_char_names)]
fn redraw(pool: &mut AutoMemPool, surface: &wl_surface::WlSurface, (buf_x, buf_y): (u32, u32)) -> Result<(), ::std::io::Error> {
    let (canvas, new_buffer) = pool.buffer(buf_x as i32, buf_y as i32, 4 * buf_x as i32, wl_shm::Format::Argb8888)?;
    for (i, dst_pixel) in canvas.chunks_exact_mut(4).enumerate() {
        let x = i as u32 % buf_x;
        let y = i as u32 / buf_x;
        let r: u32 = min(((buf_x - x) * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
        let g: u32 = min((x * 0xFF) / buf_x, ((buf_y - y) * 0xFF) / buf_y);
        let b: u32 = min(((buf_x - x) * 0xFF) / buf_x, (y * 0xFF) / buf_y);
        let pixel: [u8; 4] = ((0xFF << 24) + (r << 16) + (g << 8) + b).to_ne_bytes();
        dst_pixel[0] = pixel[0];
        dst_pixel[1] = pixel[1];
        dst_pixel[2] = pixel[2];
        dst_pixel[3] = pixel[3];
    }
    surface.attach(Some(&new_buffer), 0, 0);
    if surface.as_ref().version() >= 4 {
        surface.damage_buffer(0, 0, buf_x as i32, buf_y as i32);
    } else {
        surface.damage(0, 0, buf_x as i32, buf_y as i32);
    }
    surface.commit();
    Ok(())
}
