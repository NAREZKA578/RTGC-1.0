//! DirectX 11 Context - Stub

use tracing::info;
use windows::Win32::Foundation::HWND;

pub struct Dx11Context {
    pub hwnd: HWND,
    pub width: u32,
    pub height: u32,
}

impl Dx11Context {
    pub fn new(hwnd: isize, width: u32, height: u32) -> Result<Self, String> {
        info!(target: "dx11", "=== Dx11Context::new START ===");
        Ok(Self {
            hwnd: HWND(hwnd),
            width,
            height,
        })
    }

    pub fn begin_frame(&self) {
        info!(target: "dx11", "begin_frame (stub)");
    }

    pub fn end_frame(&self) {
        info!(target: "dx11", "end_frame (stub)");
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.width = width;
        self.height = height;
        Ok(())
    }

    pub fn set_viewport(&self, x: i32, y: i32, width: u32, height: u32) {
        info!(target: "dx11", "set_viewport: {}x{} at {}x{}", width, height, x, y);
        let _ = (x, y);
    }
}
