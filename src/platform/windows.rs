use std::{default::Default, iter, thread};

use futures::{channel::mpsc, StreamExt};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, GetWindowLongPtrW, LoadCursorW, PostQuitMessage, RegisterClassW,
            SetWindowLongPtrW, TranslateMessage, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, GWLP_USERDATA, IDC_ARROW, WM_CREATE,
            WM_DESTROY, WM_NCCREATE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

use crate::Event;

enum EventTransport {
    Created(HWND),
    Event(Event),
}

impl EventTransport {
    pub fn created(self) -> HWND {
        if let Self::Created(x) = self {
            x
        } else {
            panic!("Expected Created event")
        }
    }

    pub fn event(self) -> Event {
        if let Self::Event(x) = self {
            x
        } else {
            panic!("Expected Event event")
        }
    }
}

pub struct WindowImpl {
    hwnd: HWND,
    hinstance: HINSTANCE,
    rx: mpsc::UnboundedReceiver<EventTransport>,
}

impl WindowImpl {
    pub async fn new(width: i32, height: i32, title: &str) -> Self {
        let hinstance = unsafe { GetModuleHandleW(None).unwrap() };

        let (tx, mut rx) = mpsc::unbounded();

        let inner = WindowImplInner::new(tx);
        let title = title.encode_utf16().chain(iter::once(0)).collect::<Box<[u16]>>();

        thread::spawn(move || unsafe {
            let cursor = LoadCursorW(None, IDC_ARROW).unwrap();

            let wnd_class = WNDCLASSW {
                hCursor: cursor,
                hInstance: hinstance,
                lpszClassName: w!("windowing"),

                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(WindowImplInner::wndproc),
                ..Default::default()
            };

            RegisterClassW(&wnd_class);

            CreateWindowExW(
                Default::default(),
                wnd_class.lpszClassName,
                PCWSTR::from_raw(title.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width,
                height,
                None,
                None,
                hinstance,
                Some(&inner as *const _ as *const _),
            );

            inner.run();
        });

        let hwnd = rx.next().await.unwrap().created();

        Self { hwnd, hinstance, rx }
    }

    pub async fn next_events(&mut self, wait: bool) -> Option<impl Iterator<Item = Event>> {
        if wait {
            let event = self.rx.next().await.unwrap().event();

            Some(iter::once(event))
        } else {
            let event = self.rx.try_next().unwrap();

            event.map(|x| iter::once(x.event()))
        }
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let mut window_handle = Win32WindowHandle::empty();
        window_handle.hwnd = self.hwnd.0 as *mut _;
        window_handle.hinstance = self.hinstance.0 as *mut _;

        RawWindowHandle::Win32(window_handle)
    }

    pub fn raw_display_handle(&self) -> RawDisplayHandle {
        let display_handle = WindowsDisplayHandle::empty();

        RawDisplayHandle::Windows(display_handle)
    }
}

struct WindowImplInner {
    tx: mpsc::UnboundedSender<EventTransport>,
}

impl WindowImplInner {
    pub fn new(tx: mpsc::UnboundedSender<EventTransport>) -> Self {
        Self { tx }
    }

    pub fn run(self) {
        let mut msg = Default::default();

        unsafe {
            while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let mut userdata = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if userdata == 0 && (msg == WM_NCCREATE || msg == WM_CREATE) {
            let create_struct = lparam.0 as *const CREATESTRUCTW;
            let inner = (*create_struct).lpCreateParams as *const WindowImplInner;

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, inner as _);

            (*inner).init(hwnd);

            userdata = inner as isize;
        }

        if userdata == 0 {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        } else {
            let inner = userdata as *mut WindowImplInner;
            (*inner).handle_message(hwnd, msg, wparam, lparam)
        }
    }

    fn handle_message(&self, hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_CREATE => {
                self.send_event(Event::Created);

                LRESULT(0)
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };

                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    }

    fn init(&self, hwnd: HWND) {
        self.tx.unbounded_send(EventTransport::Created(hwnd)).unwrap();
    }

    fn send_event(&self, event: Event) {
        self.tx.unbounded_send(EventTransport::Event(event)).unwrap();
    }
}
