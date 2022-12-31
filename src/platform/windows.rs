use std::{default::Default, iter};

use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, LoadCursorW, PostQuitMessage, RegisterClassW, TranslateMessage,
            CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, IDC_ARROW, WM_DESTROY, WM_PAINT, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};

pub struct WindowImpl {
    #[allow(dead_code)]
    hwnd: HWND,
}

impl WindowImpl {
    pub fn new(width: i32, height: i32, title: &str) -> Self {
        unsafe {
            let instance = GetModuleHandleW(None).unwrap();
            let cursor = LoadCursorW(None, IDC_ARROW).unwrap();

            let wnd_class = WNDCLASSW {
                hCursor: cursor,
                hInstance: instance,
                lpszClassName: w!("windowing"),

                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                ..Default::default()
            };

            RegisterClassW(&wnd_class);

            let title = title.encode_utf16().chain(iter::once(0)).collect::<Box<[u16]>>();

            let hwnd = CreateWindowExW(
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
                instance,
                None,
            );

            Self { hwnd }
        }
    }

    pub fn run(self) {
        let mut message = Default::default();

        unsafe {
            while GetMessageW(&mut message, HWND(0), 0, 0).into() {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    }

    extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_PAINT => LRESULT(0),
                WM_DESTROY => {
                    println!("WM_DESTROY");
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcW(window, message, wparam, lparam),
            }
        }
    }
}
