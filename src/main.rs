// use opencv::core;
// use opencv::imgcodecs;
// use opencv::prelude::*;
use rdev::{Event, EventType, Key, listen};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetThreadDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, mouse_event,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowRect, GetWindowTextW};

// use crate::base::ScreencapBase;

// mod base;
mod include;
mod input_utils;
mod message_input;
// mod print_window;

const SKIP_CONVERSATION_CLICKS: (f32, f32) = (0.7160, 0.7444);
const SUPPORTED_TITLES: &[&str] = &["原神", "崩坏：星穹铁道", "Genshin Impact"];
const CLICK_INTERVAL: Duration = Duration::from_millis(100);

fn get_active_window_title() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut text: [u16; 512] = [0; 512];
        let len = GetWindowTextW(hwnd, &mut text);
        if len > 0 {
            Some(String::from_utf16_lossy(&text[..len as usize]))
        } else {
            None
        }
    }
}

fn get_window_rect(hwnd: HWND) -> Option<RECT> {
    unsafe {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_ok() {
            Some(rect)
        } else {
            None
        }
    }
}

fn left_click(x: i32, y: i32) {
    unsafe {
        // Rust 这里的实现使用了简单的 windows API，注意：SetCursorPos 需要将绝对坐标转为屏幕坐标
        // 为了简化，直接在循环中计算并调用 mouse_event
        // 注意：mouse_event 实际上操作的是当前鼠标位置，所以通常需要先 SetCursorPos
        windows::Win32::UI::WindowsAndMessaging::SetCursorPos(x, y).ok();
        mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0);
        mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0);
    }
}

fn main() {
    env_logger::init();

    unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };

    let running = Arc::new(AtomicBool::new(false));
    let click_thread_active = Arc::new(AtomicBool::new(false));

    let running_for_listener = Arc::clone(&running);
    let click_thread_active_for_listener = Arc::clone(&click_thread_active);

    println!("Genshin Assistant (Rust) Started. Press 'P' in game to toggle.");

    if let Err(error) = listen(move |event: Event| {
        if let EventType::KeyPress(Key::KeyP) = event.event_type {
            if let Some(title) = get_active_window_title() {
                if SUPPORTED_TITLES.contains(&title.as_str()) {
                    let current = running_for_listener.load(Ordering::SeqCst);
                    let next = !current;
                    running_for_listener.store(next, Ordering::SeqCst);

                    if next {
                        println!("Skip Conversation Enabled");

                        if click_thread_active_for_listener
                            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                            .is_ok()
                        {
                            let running_for_click = Arc::clone(&running_for_listener);
                            let click_thread_active_for_click =
                                Arc::clone(&click_thread_active_for_listener);

                            thread::spawn(move || {
                                while running_for_click.load(Ordering::SeqCst) {
                                    unsafe {
                                        let hwnd = GetForegroundWindow();
                                        if !hwnd.0.is_null() {
                                            if let Some(title) = get_active_window_title() {
                                                if SUPPORTED_TITLES.contains(&title.as_str()) {
                                                    if let Some(rect) = get_window_rect(hwnd) {
                                                        let width = rect.right - rect.left;
                                                        let height = rect.bottom - rect.top;
                                                        let x = rect.left
                                                            + (width as f32
                                                                * SKIP_CONVERSATION_CLICKS.0)
                                                                as i32;
                                                        let y = rect.top
                                                            + (height as f32
                                                                * SKIP_CONVERSATION_CLICKS.1)
                                                                as i32;
                                                        left_click(x, y);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    thread::sleep(CLICK_INTERVAL);
                                }

                                click_thread_active_for_click.store(false, Ordering::SeqCst);
                            });
                        }
                    } else {
                        println!("Skip Conversation Disabled");
                    }
                }
            }
        }
    }) {
        eprintln!("Error: {:?}", error);
    }
}

// fn main() {
//     let mut print_window_screencap =
//         print_window::PrintWindowScreencap::new(unsafe { GetForegroundWindow() });
//     match print_window_screencap.screencap() {
//         Ok(mat) => {
//             imgcodecs::imwrite("screencap.png", &mat, &core::Vector::new())
//                 .expect("Failed to save screenshot");
//             println!("Screenshot saved as screencap.png");
//         }
//         Err(e) => {
//             eprintln!("Failed to capture screen: {:?}", e);
//         }
//     }
// }
