use crate::capturable::{Capturable, Recorder};
use captrs::Capturer;
use std::any::Any;
use std::boxed::Box;
use std::error::Error;
use std::mem;
use std::ptr;
use winapi::shared::windef::RECT;
use winapi::um::wingdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits,
    SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CAPTUREBLT, DIB_RGB_COLORS, SRCCOPY,
};
use winapi::um::winuser::{GetDC, ReleaseDC};

use super::Geometry;

#[derive(Clone)]
pub struct CaptrsCapturable {
    id: u8,
    name: String,
    screen: RECT,
    virtual_screen: RECT,
}

impl CaptrsCapturable {
    pub fn new(id: u8, name: String, screen: RECT, virtual_screen: RECT) -> CaptrsCapturable {
        CaptrsCapturable {
            id,
            name,
            screen,
            virtual_screen,
        }
    }
}

impl Capturable for CaptrsCapturable {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn name(&self) -> String {
        format!("Desktop {} (captrs)", self.name).into()
    }
    fn before_input(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
    fn recorder(&self, _capture_cursor: bool) -> Result<Box<dyn Recorder>, Box<dyn Error>> {
        match CaptrsRecorder::new(self.id) {
            Ok(recorder) => Ok(Box::new(recorder)),
            Err(err) => {
                tracing::warn!(
                    "DXGI screen capture failed for {}, falling back to GDI: {}",
                    self.name,
                    err
                );
                Ok(Box::new(GdiRecorder::new(self.screen)?))
            }
        }
    }
    fn geometry(&self) -> Result<Geometry, Box<dyn Error>> {
        Ok(Geometry::VirtualScreen(
            self.screen.left - self.virtual_screen.left,
            self.screen.top - self.virtual_screen.top,
            (self.screen.right - self.screen.left) as u32,
            (self.screen.bottom - self.screen.top) as u32,
            self.screen.left,
            self.screen.top,
        ))
    }
}
#[derive(Debug)]
pub struct CaptrsError(String);

impl std::fmt::Display for CaptrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(s) = self;
        write!(f, "{}", s)
    }
}

impl Error for CaptrsError {}
pub struct CaptrsRecorder {
    capturer: Capturer,
}

impl CaptrsRecorder {
    pub fn new(id: u8) -> Result<CaptrsRecorder, Box<dyn Error>> {
        Ok(CaptrsRecorder {
            capturer: Capturer::new(id.into())?,
        })
    }
}

pub struct GdiRecorder {
    rect: RECT,
    width: usize,
    height: usize,
    buffer: Vec<u8>,
}

impl GdiRecorder {
    pub fn new(rect: RECT) -> Result<Self, Box<dyn Error>> {
        let width = (rect.right - rect.left).max(0) as usize;
        let height = (rect.bottom - rect.top).max(0) as usize;
        if width == 0 || height == 0 {
            return Err(Box::new(CaptrsError(
                "Invalid GDI capture rectangle".into(),
            )));
        }
        Ok(Self {
            rect,
            width,
            height,
            buffer: vec![0; width * height * 4],
        })
    }
}

impl Recorder for GdiRecorder {
    fn backend_name(&self) -> &'static str {
        "GDI BitBlt"
    }

    fn capture(&mut self) -> Result<crate::video::PixelProvider, Box<dyn Error>> {
        unsafe {
            let screen_dc = GetDC(ptr::null_mut());
            if screen_dc.is_null() {
                return Err(Box::new(CaptrsError("GDI GetDC failed".into())));
            }
            let mem_dc = CreateCompatibleDC(screen_dc);
            if mem_dc.is_null() {
                ReleaseDC(ptr::null_mut(), screen_dc);
                return Err(Box::new(CaptrsError(
                    "GDI CreateCompatibleDC failed".into(),
                )));
            }
            let bitmap = CreateCompatibleBitmap(screen_dc, self.width as i32, self.height as i32);
            if bitmap.is_null() {
                DeleteDC(mem_dc);
                ReleaseDC(ptr::null_mut(), screen_dc);
                return Err(Box::new(CaptrsError(
                    "GDI CreateCompatibleBitmap failed".into(),
                )));
            }
            let old = SelectObject(mem_dc, bitmap.cast());
            let copied = BitBlt(
                mem_dc,
                0,
                0,
                self.width as i32,
                self.height as i32,
                screen_dc,
                self.rect.left,
                self.rect.top,
                SRCCOPY | CAPTUREBLT,
            );
            if copied == 0 {
                SelectObject(mem_dc, old);
                DeleteObject(bitmap.cast());
                DeleteDC(mem_dc);
                ReleaseDC(ptr::null_mut(), screen_dc);
                return Err(Box::new(CaptrsError("GDI BitBlt failed".into())));
            }

            let mut info: BITMAPINFO = mem::zeroed();
            info.bmiHeader = BITMAPINFOHEADER {
                biSize: mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: self.width as i32,
                biHeight: -(self.height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: (self.buffer.len()) as u32,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            };
            let lines = GetDIBits(
                mem_dc,
                bitmap,
                0,
                self.height as u32,
                self.buffer.as_mut_ptr().cast(),
                &mut info,
                DIB_RGB_COLORS,
            );
            SelectObject(mem_dc, old);
            DeleteObject(bitmap.cast());
            DeleteDC(mem_dc);
            ReleaseDC(ptr::null_mut(), screen_dc);

            if lines == 0 {
                return Err(Box::new(CaptrsError("GDI GetDIBits failed".into())));
            }
        }
        Ok(crate::video::PixelProvider::BGR0(
            self.width,
            self.height,
            &self.buffer,
        ))
    }
}

impl Recorder for CaptrsRecorder {
    fn backend_name(&self) -> &'static str {
        "DXGI Desktop Duplication"
    }

    fn capture(&mut self) -> Result<crate::video::PixelProvider, Box<dyn Error>> {
        self.capturer
            .capture_store_frame()
            .map_err(|_e| CaptrsError("Captrs failed to capture frame".into()))?;
        let (w, h) = self.capturer.geometry();
        Ok(crate::video::PixelProvider::BGR0(
            w as usize,
            h as usize,
            unsafe { std::mem::transmute(self.capturer.get_stored_frame().unwrap()) },
        ))
    }
}
