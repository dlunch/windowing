use std::{default::Default, future::Future, iter};

use raw_window_handle::{RawWindowHandle, Win32WindowHandle};
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

pub struct WindowImpl {
    inner: Box<WindowImplInner>,
}

impl WindowImpl {
    pub fn new(width: i32, height: i32, title: &str) -> Self {
        unsafe {
            let hinstance = GetModuleHandleW(None).unwrap();
            let cursor = LoadCursorW(None, IDC_ARROW).unwrap();

            let wnd_class = WNDCLASSW {
                hCursor: cursor,
                hInstance: hinstance,
                lpszClassName: w!("windowing"),

                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                ..Default::default()
            };

            RegisterClassW(&wnd_class);

            let title = title.encode_utf16().chain(iter::once(0)).collect::<Box<[u16]>>();

            let inner = Box::new(WindowImplInner {
                hwnd: HWND(0),
                hinstance,
                events: Vec::new(),
            });
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
                Some(inner.as_ref() as *const _ as *const _),
            );

            Self { inner }
        }
    }

    pub async fn run<F, Fut>(mut self, callback: F)
    where
        F: Fn(Event) -> Fut,
        Fut: Future<Output = ()>,
    {
        let mut msg = Default::default();

        unsafe {
            while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);

                for event in self.inner.events.drain(..) {
                    callback(event).await
                }
            }
        }
    }

    pub fn raw_window_handle(&self) -> RawWindowHandle {
        let mut window_handle = Win32WindowHandle::empty();
        window_handle.hwnd = self.inner.hwnd.0 as *mut _;
        window_handle.hinstance = self.inner.hinstance.0 as *mut _;

        RawWindowHandle::Win32(window_handle)
    }

    unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        let mut userdata = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if userdata == 0 && (msg == WM_NCCREATE || msg == WM_CREATE) {
            let create_struct = lparam.0 as *const CREATESTRUCTW;
            let inner = (*create_struct).lpCreateParams as *mut WindowImplInner;

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, inner as _);

            (*inner).hwnd = hwnd;

            userdata = inner as isize;
        }

        if userdata == 0 {
            DefWindowProcW(hwnd, msg, wparam, lparam)
        } else {
            let inner = userdata as *mut WindowImplInner;
            (*inner).handle_message(msg, wparam, lparam)
        }
    }
}

struct WindowImplInner {
    hwnd: HWND,
    hinstance: HINSTANCE,
    events: Vec<Event>,
}

impl WindowImplInner {
    pub fn handle_message(&mut self, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match msg {
            WM_CREATE => {
                self.events.push(Event::Created);

                LRESULT(0)
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };

                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(self.hwnd, msg, wparam, lparam) },
        }
    }
}
