use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    ClientToScreen, MONITOR_DEFAULTTONULL, MonitorFromRect, ScreenToClient,
};
use windows::Win32::UI::Input::KeyboardAndMouse::BlockInput;
use windows::Win32::UI::WindowsAndMessaging::{
    GA_ROOTOWNER, GetAncestor, GetCursorPos, GetLastActivePopup, GetWindowRect, IsWindow,
    IsWindowVisible, PostMessageW, SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SendMessageW,
    SetCursorPos, SetWindowPos,
};

use crate::{MAKELONG, MAKELPARAM};
// use crate::include::winuser::MAKELPARAM;
use crate::input_utils::{Contact, send_activate_message};

#[derive(PartialEq)]
pub enum Mode {
    SendMessage,
    PostMessage,
}

pub struct Config {
    pub mode: Mode,
    pub with_cursor_pos: bool,
    pub with_window_pos: bool,
    pub block_input: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::SendMessage,
            with_cursor_pos: false,
            with_window_pos: false,
            block_input: false,
        }
    }
}

#[derive(Default)]
pub struct MessageInput {
    hwnd: HWND,
    config: Config,

    last_pos: Option<(i32, i32)>,
    saved_cursor_pos: Option<POINT>,
    saved_window_rect: Option<RECT>,
}

impl MessageInput {
    pub fn new(hwnd: HWND, config: Config) -> Self {
        Self {
            hwnd,
            config,
            ..Default::default()
        }
    }

    pub fn touch_down(&mut self, contact: Contact, x: i32, y: i32) -> windows::core::Result<()> {
        let move_info = contact.to_mouse_move_message();
        let down_info = contact.to_mouse_down_message();
        let target = self.send_activate();
        self.check_and_block_input();
        self.save_pos();

        let l_param = self.prepare_mouse_position(x, y);
        self.send_or_post_w(target, move_info.message, move_info.w_param, l_param)?;
        thread::sleep(Duration::from_millis(10));
        self.send_or_post_w(target, down_info.message, down_info.w_param, l_param)?;
        self.last_pos = Some((x, y));
        Ok(())
    }

    fn send_activate(&self) -> HWND {
        let target = self.get_active_hwnd();
        let use_post = self.config.mode == Mode::PostMessage;
        send_activate_message(self.hwnd, use_post);
        target
    }

    fn send_or_post_w(
        &self,
        target: HWND,
        message: u32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> windows::core::Result<()> {
        if target.is_invalid() || !unsafe { IsWindow(target).as_bool() } {
            return Err(windows::core::Error::new(
                windows::core::HRESULT(0),
                "Invalid target window".to_string(),
            ));
        }

        match self.config.mode {
            Mode::SendMessage => {
                unsafe { SendMessageW(target, message, w_param, l_param) };
                Ok(())
            }
            Mode::PostMessage => unsafe { PostMessageW(target, message, w_param, l_param) },
        }
    }

    fn get_active_hwnd(&self) -> HWND {
        let root = unsafe { GetAncestor(self.hwnd, GA_ROOTOWNER) };
        if root.is_invalid() {
            return self.hwnd;
        }
        let popup = unsafe { GetLastActivePopup(root) };
        if !popup.is_invalid() && popup != self.hwnd && unsafe { IsWindowVisible(popup).as_bool() }
        {
            return popup;
        }
        self.hwnd
    }

    fn make_mouse_lparam(&self, target: HWND, x: i32, y: i32) -> LPARAM {
        if target == self.hwnd {
            return MAKELPARAM!(x, y);
        }
        let mut pt = POINT { x, y };
        if !unsafe { ClientToScreen(self.hwnd, &mut pt).as_bool() } {
            return MAKELPARAM!(x, y);
        }
        if !unsafe { ScreenToClient(target, &mut pt).as_bool() } {
            return MAKELPARAM!(x, y);
        }
        MAKELPARAM!(pt.x, pt.y)
    }

    fn client_to_screen(&self, x: i32, y: i32) -> POINT {
        let mut pt = POINT { x, y };
        if !self.hwnd.is_invalid() {
            unsafe { ClientToScreen(self.hwnd, &mut pt) };
        }
        pt
    }

    fn save_cursor_pos(&mut self) -> windows::core::Result<()> {
        let mut pos = POINT::default();
        unsafe { GetCursorPos(&mut pos) }?;
        self.saved_cursor_pos = Some(pos);
        Ok(())
    }

    fn restore_cursor_pos(&mut self) -> windows::core::Result<()> {
        match self.saved_cursor_pos {
            Some(POINT { x, y }) => {
                thread::sleep(Duration::from_millis(10));
                self.saved_cursor_pos = None;
                unsafe { SetCursorPos(x, y) }?;
                Ok(())
            }
            None => Ok(()),
        }
    }

    fn save_window_pos(&mut self) -> windows::core::Result<()> {
        // 保留首次进入 WithWindowPos 会话前的位置，依赖 inactive/析构路径统一恢复并清空标记。
        if self.hwnd.is_invalid() {
            return Ok(());
        }

        match self.saved_window_rect {
            Some(_) => Ok(()),
            None => {
                let mut rect = RECT::default();
                unsafe { GetWindowRect(self.hwnd, &mut rect) }?;
                self.saved_window_rect = Some(rect);
                Ok(())
            }
        }
    }

    fn restore_window_pos(&mut self) -> windows::core::Result<()> {
        if self.hwnd.is_invalid() {
            return Ok(());
        }

        match self.saved_window_rect {
            Some(rect) => {
                thread::sleep(Duration::from_millis(10));

                let mut left = rect.left;
                let mut top = rect.top;

                if unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTONULL) }.is_invalid() {
                    left = 0;
                    top = 0;
                }

                self.saved_window_rect = None;
                unsafe {
                    SetWindowPos(
                        self.hwnd,
                        HWND::default(),
                        left,
                        top,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                    )
                }?;

                Ok(())
            }
            None => Ok(()),
        }
    }

    fn save_pos(&mut self) -> windows::core::Result<()> {
        if self.config.with_cursor_pos {
            self.save_cursor_pos()?;
        }
        if self.config.with_window_pos {
            self.save_window_pos()?;
        }
        Ok(())
    }

    fn check_and_block_input(&self) -> windows::core::Result<()> {
        if !self.config.block_input {
            return Ok(());
        }
        unsafe { BlockInput(true) }
    }

    fn unblock_input(&self) -> windows::core::Result<()> {
        if !self.config.block_input {
            return Ok(());
        }
        unsafe { BlockInput(false) }
    }

    fn prepare_mouse_position(&mut self, x: i32, y: i32) -> LPARAM {
        if self.config.with_cursor_pos {
            let screen_pos = self.client_to_screen(x, y);
            unsafe { SetCursorPos(screen_pos.x, screen_pos.y) };
            thread::sleep(Duration::from_millis(1));
        } else if self.config.with_window_pos {
            todo!();
        }
        MAKELPARAM!(x, y)
    }
}

impl Drop for MessageInput {
    fn drop(&mut self) {}
}
