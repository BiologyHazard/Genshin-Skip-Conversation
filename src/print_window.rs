use anyhow::{Result, anyhow};
use opencv::prelude::*;
use windows::Win32::Foundation::{GetLastError, HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, HBITMAP, HDC, HGDIOBJ, ReleaseDC,
    SelectObject,
};
use windows::Win32::Storage::Xps::{PRINT_WINDOW_FLAGS, PW_CLIENTONLY, PrintWindow};
use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

use crate::base::ScreencapBase;

// PW_RENDERFULLCONTENT (0x2): 捕获非最小化后台窗口
const PW_RENDERFULLCONTENT: PRINT_WINDOW_FLAGS = PRINT_WINDOW_FLAGS(0x2_u32);

pub struct PrintWindowScreencap {
    hwnd: HWND,
}
impl ScreencapBase<Mat> for PrintWindowScreencap {
    fn new(hwnd: HWND) -> Self {
        Self::new(hwnd)
    }

    fn screencap(&mut self) -> Result<Mat> {
        self.screencap()
    }
}

impl PrintWindowScreencap {
    pub fn new(hwnd: HWND) -> Self {
        Self { hwnd }
    }

    pub fn screencap(&mut self) -> Result<Mat> {
        if self.hwnd.is_invalid() {
            return Err(anyhow!("hwnd_ is nullptr"));
        }

        // 确定要捕获的区域大小
        // 使用 PW_CLIENTONLY 标志，只获取客户端区域（不含窗口边框）
        let mut rect = RECT::default();
        unsafe {
            GetClientRect(self.hwnd, &mut rect)?;
        }

        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;

        if width <= 0 || height <= 0 {
            return Err(anyhow!(
                "Invalid window size width={} height={}",
                width,
                height
            ));
        }

        // 创建与窗口兼容的 DC
        let hdc: HDC = unsafe { GetDC(self.hwnd) };
        if hdc.is_invalid() {
            return Err(anyhow!("GetDC failed, error code: {:?}", unsafe {
                GetLastError()
            }));
        }
        let _hdc_guard = DcGuard(self.hwnd, hdc);

        let mem_dc: HDC = unsafe { CreateCompatibleDC(hdc) };
        if mem_dc.is_invalid() {
            return Err(anyhow!(
                "CreateCompatibleDC failed, error code: {:?}",
                unsafe { GetLastError() }
            ));
        }
        let _mem_dc_guard = MemDcGuard(mem_dc);

        let bitmap: HBITMAP = unsafe { CreateCompatibleBitmap(hdc, width, height) };
        if bitmap.is_invalid() {
            return Err(anyhow!(
                "CreateCompatibleBitmap failed, error code: {:?}",
                unsafe { GetLastError() }
            ));
        }
        let _bitmap_guard = BitmapGuard(bitmap);

        let old_obj: HGDIOBJ = unsafe { SelectObject(mem_dc, bitmap) };
        if old_obj.is_invalid() {
            return Err(anyhow!("SelectObject failed, error code: {:?}", unsafe {
                GetLastError()
            }));
        }
        let _select_guard = SelectGuard(mem_dc, old_obj);

        // 使用PrintWindow捕获窗口内容
        // 使用PW_CLIENTONLY | PW_RENDERFULLCONTENT标志:
        // - PW_CLIENTONLY (0x1): 只获取客户端区域
        // - PW_RENDERFULLCONTENT (0x2): 捕获非最小化后台窗口
        let n_flags = PRINT_WINDOW_FLAGS(PW_CLIENTONLY.0 | PW_RENDERFULLCONTENT.0);
        if !unsafe { PrintWindow(self.hwnd, mem_dc, n_flags) }.as_bool() {
            return Err(anyhow!("PrintWindow failed, error code: {:?}", unsafe {
                GetLastError()
            }));
        }

        // 使用 GetDIBits 将位图一致转换为 32bpp BGRA
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0 as u32,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut mat = Mat::zeros(height as i32, width as i32, opencv::core::CV_8UC4)?.to_mat()?;

        if unsafe {
            GetDIBits(
                mem_dc,
                bitmap,
                0,
                height as u32,
                Some(mat.data_mut() as *mut _),
                &mut bmi,
                DIB_RGB_COLORS,
            )
        } == 0
        {
            return Err(anyhow!("GetDIBits failed, error code: {:?}", unsafe {
                GetLastError()
            }));
        }

        Ok(mat)
    }
}

struct DcGuard(HWND, HDC);
impl Drop for DcGuard {
    fn drop(&mut self) {
        unsafe {
            ReleaseDC(self.0, self.1);
        }
    }
}

struct MemDcGuard(HDC);
impl Drop for MemDcGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteDC(self.0);
        }
    }
}

struct BitmapGuard(HBITMAP);
impl Drop for BitmapGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = DeleteObject(self.0);
        }
    }
}

struct SelectGuard(HDC, HGDIOBJ);
impl Drop for SelectGuard {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.0, self.1);
        }
    }
}
