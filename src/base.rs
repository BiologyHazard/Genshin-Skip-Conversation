pub trait ScreencapBase<T> {
    fn new(hwnd: windows::Win32::Foundation::HWND) -> Self
    where
        Self: Sized;
    fn screencap(&mut self) -> anyhow::Result<T>;
}

pub trait InputBase {
    fn new(hwnd: windows::Win32::Foundation::HWND) -> Self
    where
        Self: Sized;
    fn touch_down(&mut self, x: i32, y: i32) -> anyhow::Result<()>;
    fn touch_up(&mut self, x: i32, y: i32) -> anyhow::Result<()>;
}
