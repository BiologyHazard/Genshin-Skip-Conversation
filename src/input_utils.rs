use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::SystemServices::{
    MK_LBUTTON, MK_MBUTTON, MK_RBUTTON, MK_XBUTTON1, MK_XBUTTON2,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    MAPVK_VK_TO_VSC, MOUSE_EVENT_FLAGS, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP,
    MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MapVirtualKeyW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, HWND_TOP, PostMessageW, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    SendMessageW, SetForegroundWindow, SetWindowPos, WA_ACTIVE, WM_ACTIVATE, WM_LBUTTONDOWN,
    WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_RBUTTONDOWN, WM_RBUTTONUP,
    WM_XBUTTONDOWN, WM_XBUTTONUP, XBUTTON1, XBUTTON2,
};

// use crate::include::winuser::MAKEWPARAM;
use crate::{MAKELONG, MAKEWPARAM};

// 发送 WM_ACTIVATE 消息激活窗口（用于后台消息发送方式）
// 让目标窗口认为自己被激活，但不实际改变前台窗口
pub fn send_activate_message(hwnd: HWND, use_post: bool) {
    if hwnd.is_invalid() {
        return;
    }
    // WM_ACTIVATE + WA_ACTIVE，lParam 为 0 表示没有前一个窗口
    if use_post {
        unsafe { PostMessageW(hwnd, WM_ACTIVATE, WPARAM(WA_ACTIVE as usize), LPARAM(0)) };
    } else {
        unsafe { SendMessageW(hwnd, WM_ACTIVATE, WPARAM(WA_ACTIVE as usize), LPARAM(0)) };
    }

    thread::sleep(Duration::from_millis(10));
}

// 窗口激活并置顶工具函数（强化版本，用于需要前台的物理输入方式）
// 用于 LegacyEventInput 和 SeizeInput，因为它们使用 SendInput/mouse_event 等物理输入 API
pub fn ensure_foreground_and_topmost(hwnd: HWND) {
    if hwnd.is_invalid() {
        return;
    }

    // 如果窗口不在前台，先将其置顶
    if hwnd != unsafe { GetForegroundWindow() } {
        // 将窗口移到 Z 序顶部
        unsafe {
            SetWindowPos(
                hwnd,
                HWND_TOP,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_SHOWWINDOW,
            )
        };
        thread::sleep(Duration::from_millis(5));

        // 尝试设置为前台窗口
        unsafe { SetForegroundWindow(hwnd) };
        thread::sleep(Duration::from_millis(10));

        // 再次检查，如果仍然不在前台，再次置顶
        if hwnd != unsafe { GetForegroundWindow() } {
            unsafe { SetWindowPos(hwnd, HWND_TOP, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE) };
            thread::sleep(Duration::from_millis(5));
        }
    }
}

pub enum Contact {
    Left = 0,
    Right = 1,
    Middle = 2,
    X1 = 3,
    X2 = 4,
}

// Contact 到 WM_* 消息的转换结果
pub struct MouseMessageInfo {
    pub message: u32,
    pub w_param: WPARAM,
}

// 将 contact ID 转换为鼠标按下消息
pub fn contact_to_mouse_down_message(contact: &Contact) -> MouseMessageInfo {
    match contact {
        Contact::Left => MouseMessageInfo {
            message: WM_LBUTTONDOWN,
            w_param: WPARAM(MK_LBUTTON.0 as usize),
        },
        Contact::Right => MouseMessageInfo {
            message: WM_RBUTTONDOWN,
            w_param: WPARAM(MK_RBUTTON.0 as usize),
        },
        Contact::Middle => MouseMessageInfo {
            message: WM_MBUTTONDOWN,
            w_param: WPARAM(MK_MBUTTON.0 as usize),
        },
        Contact::X1 => MouseMessageInfo {
            message: WM_XBUTTONDOWN,
            w_param: MAKEWPARAM!(MK_XBUTTON1.0, XBUTTON1),
        },
        Contact::X2 => MouseMessageInfo {
            message: WM_XBUTTONDOWN,
            w_param: MAKEWPARAM!(MK_XBUTTON2.0, XBUTTON2),
        },
    }
}

// 将 contact ID 转换为鼠标移动消息
pub fn contact_to_mouse_move_message(contact: &Contact) -> MouseMessageInfo {
    match contact {
        Contact::Left => MouseMessageInfo {
            message: WM_MOUSEMOVE,
            w_param: WPARAM(MK_LBUTTON.0 as usize),
        },
        Contact::Right => MouseMessageInfo {
            message: WM_MOUSEMOVE,
            w_param: WPARAM(MK_RBUTTON.0 as usize),
        },
        Contact::Middle => MouseMessageInfo {
            message: WM_MOUSEMOVE,
            w_param: WPARAM(MK_MBUTTON.0 as usize),
        },
        Contact::X1 => MouseMessageInfo {
            message: WM_MOUSEMOVE,
            w_param: WPARAM(MK_XBUTTON1.0 as usize),
        },
        Contact::X2 => MouseMessageInfo {
            message: WM_MOUSEMOVE,
            w_param: WPARAM(MK_XBUTTON2.0 as usize),
        },
    }
}

// 将 contact ID 转换为鼠标抬起消息
pub fn contact_to_mouse_up_message(contact: &Contact) -> MouseMessageInfo {
    match contact {
        Contact::Left => MouseMessageInfo {
            message: WM_LBUTTONUP,
            w_param: WPARAM(0),
        },
        Contact::Right => MouseMessageInfo {
            message: WM_RBUTTONUP,
            w_param: WPARAM(0),
        },
        Contact::Middle => MouseMessageInfo {
            message: WM_MBUTTONUP,
            w_param: WPARAM(0),
        },
        Contact::X1 => MouseMessageInfo {
            message: WM_XBUTTONUP,
            w_param: MAKEWPARAM!(0u32, XBUTTON1),
        },
        Contact::X2 => MouseMessageInfo {
            message: WM_XBUTTONUP,
            w_param: MAKEWPARAM!(0u32, XBUTTON2),
        },
    }
}

// MOUSEEVENTF 标志和按钮数据
pub struct MouseEventFlags {
    pub flags: MOUSE_EVENT_FLAGS,
    pub button_data: u32,
}

// 将 contact ID 转换为 MOUSEEVENTF 按下标志（用于 SendInput/mouse_event）
pub fn contact_to_mouse_down_flags(contact: &Contact) -> MouseEventFlags {
    match contact {
        Contact::Left => MouseEventFlags {
            flags: MOUSEEVENTF_LEFTDOWN,
            button_data: 0,
        },
        Contact::Right => MouseEventFlags {
            flags: MOUSEEVENTF_RIGHTDOWN,
            button_data: 0,
        },
        Contact::Middle => MouseEventFlags {
            flags: MOUSEEVENTF_MIDDLEDOWN,
            button_data: 0,
        },
        Contact::X1 => MouseEventFlags {
            flags: MOUSEEVENTF_XDOWN,
            button_data: XBUTTON1 as u32,
        },
        Contact::X2 => MouseEventFlags {
            flags: MOUSEEVENTF_XDOWN,
            button_data: XBUTTON2 as u32,
        },
    }
}

// 将 contact ID 转换为 MOUSEEVENTF 抬起标志（用于 SendInput/mouse_event）
pub fn contact_to_mouse_up_flags(contact: &Contact) -> MouseEventFlags {
    match contact {
        Contact::Left => MouseEventFlags {
            flags: MOUSEEVENTF_LEFTUP,
            button_data: 0,
        },
        Contact::Right => MouseEventFlags {
            flags: MOUSEEVENTF_RIGHTUP,
            button_data: 0,
        },
        Contact::Middle => MouseEventFlags {
            flags: MOUSEEVENTF_MIDDLEUP,
            button_data: 0,
        },
        Contact::X1 => MouseEventFlags {
            flags: MOUSEEVENTF_XUP,
            button_data: XBUTTON1 as u32,
        },
        Contact::X2 => MouseEventFlags {
            flags: MOUSEEVENTF_XUP,
            button_data: XBUTTON2 as u32,
        },
    }
}

impl Contact {
    pub fn to_mouse_down_message(&self) -> MouseMessageInfo {
        contact_to_mouse_down_message(self)
    }

    pub fn to_mouse_move_message(&self) -> MouseMessageInfo {
        contact_to_mouse_move_message(self)
    }

    pub fn to_mouse_up_message(&self) -> MouseMessageInfo {
        contact_to_mouse_up_message(self)
    }

    pub fn to_mouse_down_flags(&self) -> MouseEventFlags {
        contact_to_mouse_down_flags(self)
    }

    pub fn to_mouse_up_flags(&self) -> MouseEventFlags {
        contact_to_mouse_up_flags(self)
    }
}

// 构造 WM_KEYDOWN 的 lParam
pub fn make_keydown_lparam(key: i32) -> LPARAM {
    let sc = unsafe { MapVirtualKeyW(key as u32, MAPVK_VK_TO_VSC) };
    LPARAM((1 | (sc << 16)) as isize)
}

// 构造 WM_KEYUP 的 lParam
pub fn make_keyup_lparam(key: i32) -> LPARAM {
    let sc = unsafe { MapVirtualKeyW(key as u32, MAPVK_VK_TO_VSC) };
    // 置位先前状态与转换状态位
    LPARAM((1 | (sc << 16) | (1 << 30) | (1 << 31)) as isize)
}
