//! OpenGL Context Management

use glow::Context;
use std::sync::Arc;
use std::cell::RefCell;
use winit::window::Window;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use crate::graphics::rhi::gl::{GlDevice, GlCommandQueue, GlSwapChainInternal};
use crate::graphics::rhi::device::{IDevice, ICommandQueue, ISwapChain};
use crate::graphics::rhi::types::{TextureFormat, RhiResult};

pub struct GlContext {
    window: Arc<Window>,
    gl_context: Arc<Context>,
    device: Arc<GlDevice>,
    command_queue: Arc<GlCommandQueue>,
    swapchain: RefCell<Option<Arc<GlSwapChainInternal>>>,
    width: u32,
    height: u32,
    initialized: bool,
}

#[cfg(windows)]
unsafe fn win32_loader(name: &str) -> *mut std::ffi::c_void { unsafe {
    use std::ffi::CString;
    use std::os::raw::c_char;
    
    unsafe extern "system" {
        fn wglGetProcAddress(lpProcName: *const c_char) -> *mut std::ffi::c_void;
    }
    
    let name_c = CString::new(name).unwrap();
    wglGetProcAddress(name_c.as_ptr())
}}

#[cfg(not(windows))]
unsafe fn win32_loader(name: &str) -> *mut std::ffi::c_void {
    std::ptr::null_mut()
}

impl GlContext {
    pub fn new(window: Window) -> RhiResult<Self> {
        let (width, height) = window.inner_size().into();
        
        let _raw_handle = window.window_handle()
            .map_err(|e| crate::graphics::rhi::types::RhiError::InitializationFailed(
                format!("Failed to get window handle: {:?}", e)
            ))?
            .as_raw();
        
        let gl_context = Arc::new(unsafe { 
            Context::from_loader_function(|name| {
                win32_loader(name)
            })
        });
        
        let device = Arc::new(GlDevice::new(gl_context.clone()));
        let command_queue = Arc::new(GlCommandQueue::new(
            gl_context.clone(), 
            crate::graphics::rhi::types::CommandListType::Direct
        ));
        
        let window = Arc::new(window);
        
        Ok(Self {
            window,
            gl_context,
            device,
            command_queue,
            swapchain: RefCell::new(None),
            width,
            height,
            initialized: false,
        })
    }
    
    pub fn window(&self) -> &Window {
        &self.window
    }
    
    pub fn device(&self) -> Arc<dyn IDevice> {
        self.device.clone()
    }
    
    pub fn command_queue(&self) -> Arc<dyn ICommandQueue> {
        self.command_queue.clone()
    }
    
    pub fn create_swapchain(&mut self, vsync: bool) -> RhiResult<()> {
        let raw_handle = self.window.window_handle()
            .map_err(|e| crate::graphics::rhi::types::RhiError::InitializationFailed(
                format!("Failed to get window handle: {:?}", e)
            ))?
            .as_raw();
        
        let ptr = match raw_handle {
            RawWindowHandle::Win32(h) => h.hwnd.get() as *mut _,
            _ => std::ptr::null_mut(),
        };
        
        let swapchain = self.device.create_swap_chain(
            ptr,
            self.width,
            self.height,
            TextureFormat::Bgra8Unorm,
            vsync,
        )?;
        
        let gl_swapchain = swapchain
            .as_any()
            .downcast_ref::<GlSwapChainInternal>()
            .ok_or_else(|| crate::graphics::rhi::types::RhiError::InitializationFailed(
                "Failed to downcast swapchain".to_string()
            ))?
            .owned_clone();
        
        self.swapchain.replace(Some(Arc::new(gl_swapchain)));
        
        Ok(())
    }
    
    pub fn swapchain(&self) -> Option<Arc<dyn ISwapChain>> {
        self.swapchain.borrow().clone().map(|s| s as Arc<dyn ISwapChain>)
    }
    
    pub fn gl_swapchain(&self) -> Option<Arc<GlSwapChainInternal>> {
        self.swapchain.borrow().clone()
    }
    
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    
    pub fn on_resize(&mut self, width: u32, height: u32) -> RhiResult<()> {
        self.width = width;
        self.height = height;
        self.swapchain.replace(None);
        Ok(())
    }
    
    pub fn present(&self) -> RhiResult<()> {
        Ok(())
    }
    
    pub fn swap_buffers(&self) -> RhiResult<()> {
        self.present()
    }
}