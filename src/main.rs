use rdev::{Event, EventType, Key, listen};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_FILENAME, SND_MEMORY};
use windows::Win32::System::SystemServices::MK_LBUTTON;
use windows::Win32::UI::HiDpi::{
    DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetThreadDpiAwarenessContext,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, mouse_event,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, PostMessageW,
    SendMessageW, SetCursorPos, WA_ACTIVE, WM_ACTIVATE, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MOUSEMOVE,
};
use windows::core::PCWSTR;

// use crate::base::ScreencapBase;
// use crate::{MAKELONG, MAKELPARAM};

// mod base;
mod include;
// mod input_utils;
// mod message_input;
// mod print_window;

const SKIP_CONVERSATION_CLICKS: (i32, i32) = (1407, 802);
const REFERENCE_RESOLUTION: (i32, i32) = (1920, 1080);
const SUPPORTED_TITLES: &[&str] = &["原神", "崩坏：星穹铁道", "Genshin Impact"];
const CLICK_INTERVAL: Duration = Duration::from_millis(100);
const SOUND_ENABLE: &[u8] = include_bytes!("../sound/enable.wav");
const SOUND_DISABLE: &[u8] = include_bytes!("../sound/disable.wav");

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    SendMessage,
    PostMessage,
}

fn get_active_window() -> HWND {
    unsafe { GetForegroundWindow() }
}

fn get_window_title(hwnd: HWND) -> Option<String> {
    let length = unsafe { GetWindowTextLengthW(hwnd) };
    if length == 0 {
        return None;
    }

    let mut buffer = vec![0u16; (length + 1) as usize];
    let read_length = unsafe { GetWindowTextW(hwnd, &mut buffer) };

    let title = String::from_utf16_lossy(&buffer[..read_length as usize]);
    Some(title)
}

#[allow(dead_code)]
fn get_client_rect(hwnd: HWND) -> Result<RECT, windows::core::Error> {
    let mut rect = RECT::default();
    unsafe { GetClientRect(hwnd, &mut rect) }?;
    Ok(rect)
}

fn client_to_screen(hwnd: HWND, x: i32, y: i32) -> Result<POINT, windows::core::Error> {
    let mut point = POINT { x, y };
    unsafe { ClientToScreen(hwnd, &mut point) }.ok()?;
    Ok(point)
}

fn scale_coordinate(
    base_coordinate: (i32, i32),
    base_resolution: (i32, i32),
    actual_resolution: (i32, i32),
) -> (i32, i32) {
    if actual_resolution.0 <= 0
        || actual_resolution.1 <= 0
        || base_resolution.0 <= 0
        || base_resolution.1 <= 0
    {
        return base_coordinate;
    }

    let x =
        (base_coordinate.0 as f64 * actual_resolution.0 as f64 / base_resolution.0 as f64) as i32;
    let y =
        (base_coordinate.1 as f64 * actual_resolution.1 as f64 / base_resolution.1 as f64) as i32;
    (x, y)
}

fn prepare_mouse_position(hwnd: HWND, x: i32, y: i32) -> Result<LPARAM, windows::core::Error> {
    let screen_pos = client_to_screen(hwnd, x, y)?;
    unsafe { SetCursorPos(screen_pos.x, screen_pos.y) }?;
    Ok(MAKELPARAM!(x, y))
}

fn send_or_post_message(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    mode: Mode,
) -> Result<(), windows::core::Error> {
    match mode {
        Mode::SendMessage => {
            unsafe { SendMessageW(hwnd, msg, Some(wparam), Some(lparam)) };
            Ok(())
        }
        Mode::PostMessage => unsafe { PostMessageW(Some(hwnd), msg, wparam, lparam) },
    }
}

/// 发送 WM_ACTIVATE 消息激活窗口（用于后台消息发送方式）
/// 让目标窗口认为自己被激活，但不实际改变前台窗口
pub fn send_activate_message(hwnd: HWND, mode: Mode) {
    if hwnd.is_invalid() {
        return;
    }
    // WM_ACTIVATE + WA_ACTIVE，lParam 为 0 表示没有前一个窗口
    let _ = send_or_post_message(
        hwnd,
        WM_ACTIVATE,
        WPARAM(WA_ACTIVE as usize),
        LPARAM(0),
        mode,
    );

    thread::sleep(Duration::from_millis(10));
}

fn mouse_left_click(hwnd: HWND, x: i32, y: i32, mode: Mode) -> Result<(), windows::core::Error> {
    send_activate_message(hwnd, mode);
    let lparam = prepare_mouse_position(hwnd, x, y)?;
    send_or_post_message(
        hwnd,
        WM_MOUSEMOVE,
        WPARAM(MK_LBUTTON.0 as usize),
        lparam,
        mode,
    )?;
    thread::sleep(Duration::from_millis(10));
    send_or_post_message(
        hwnd,
        WM_LBUTTONDOWN,
        WPARAM(MK_LBUTTON.0 as usize),
        lparam,
        mode,
    )?;
    thread::sleep(Duration::from_millis(10));
    send_or_post_message(hwnd, WM_LBUTTONUP, WPARAM(0), lparam, mode)?;
    Ok(())
}

enum PlaySoundMode<'a, 'b> {
    File(&'a str),
    Memory(&'b [u8]),
}

fn play_sound(mode: PlaySoundMode) {
    match mode {
        PlaySoundMode::File(path) => {
            let path_u16: Vec<u16> = path.encode_utf16().chain([0]).collect();
            let _ =
                unsafe { PlaySoundW(PCWSTR(path_u16.as_ptr()), None, SND_FILENAME | SND_ASYNC) };
        }
        PlaySoundMode::Memory(data) => {
            let _ = unsafe {
                PlaySoundW(
                    PCWSTR(data.as_ptr() as *const u16),
                    None,
                    SND_MEMORY | SND_ASYNC,
                )
            };
        }
    }
}

#[allow(dead_code)]
fn left_click(x: i32, y: i32) -> Result<(), windows::core::Error> {
    // Rust 这里的实现使用了简单的 windows API，注意：SetCursorPos 需要将绝对坐标转为屏幕坐标
    // 为了简化，直接在循环中计算并调用 mouse_event
    // 注意：mouse_event 实际上操作的是当前鼠标位置，所以通常需要先 SetCursorPos
    unsafe { SetCursorPos(x, y) }?;
    unsafe { mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0) };
    unsafe { mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0) };
    Ok(())
}

fn main() {
    unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };

    let running = Arc::new(AtomicBool::new(false));
    let running_clone = Arc::clone(&running);

    // 监听线程
    thread::spawn(move || {
        if let Err(error) = listen(move |event: Event| {
            if let EventType::KeyPress(Key::KeyP) = event.event_type {
                let hwnd = get_active_window();
                if let Some(title) = get_window_title(hwnd) {
                    if SUPPORTED_TITLES.contains(&title.as_str()) {
                        let previous = running_clone.fetch_not(Ordering::SeqCst);
                        if !previous {
                            println!("Skip Conversation Enabled");
                            play_sound(PlaySoundMode::Memory(SOUND_ENABLE));
                        } else {
                            println!("Skip Conversation Disabled");
                            play_sound(PlaySoundMode::Memory(SOUND_DISABLE));
                        }
                    }
                }
            }
        }) {
            eprintln!("Error: {:?}", error);
        }
    });

    println!("Genshin Assistant Started. Press 'P' in game to toggle.");

    // 主循环：执行点击逻辑
    loop {
        if running.load(Ordering::SeqCst) {
            let hwnd = get_active_window();
            if let Some(title) = get_window_title(hwnd) {
                if SUPPORTED_TITLES.contains(&title.as_str()) {
                    if let Ok(rect) = get_client_rect(hwnd) {
                        let actual_resolution = (rect.right - rect.left, rect.bottom - rect.top);
                        let (x, y) = scale_coordinate(
                            SKIP_CONVERSATION_CLICKS,
                            REFERENCE_RESOLUTION,
                            actual_resolution,
                        );

                        mouse_left_click(hwnd, x, y, Mode::SendMessage)
                            .unwrap_or_else(|e| eprintln!("Failed to click: {:?}", e));
                    }
                }
            }
        }
        thread::sleep(CLICK_INTERVAL);
    }
}
