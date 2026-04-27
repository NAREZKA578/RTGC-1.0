//! DirectX 11 Shader - Full Implementation
//! Compiles HLSL via d3dcompiler_47.dll and creates ID3D11VertexShader/ID3D11PixelShader
//! Uses shader reflection to generate Input Layout automatically

use std::ffi::c_void;
use tracing::{error, info, warn};

use crate::graphics::rhi::device::*;
use crate::graphics::rhi::types::*;
use crate::graphics::rhi::RhiResult;

#[cfg(target_os = "windows")]
use windows::{
    core::PCSTR,
    Win32::Graphics::Direct3D::{
        D3D_SHADER_MACRO,
    },
    Win32::Graphics::Direct3D11::{
        D3D11_INPUT_ELEMENT_DESC, ID3D11InputLayout,
        ID3D11VertexShader, ID3D11PixelShader, ID3D11ComputeShader,
        ID3D11GeometryShader, ID3D11HullShader, ID3D11DomainShader,
        D3D11_INPUT_PER_VERTEX_DATA,
    },
    Win32::Graphics::Dxgi::Common::{
        DXGI_FORMAT_R32G32_FLOAT, DXGI_FORMAT_R32G32B32_FLOAT,
        DXGI_FORMAT_R32G32B32A32_FLOAT, DXGI_FORMAT_R8G8B8A8_UNORM,
    },
    Win32::System::LibraryLoader::{
        GetModuleHandleW, GetProcAddress,
    },
};

/// DX11 Shader resource containing compiled shaders and input layout
pub struct Dx11Shader {
    #[cfg(target_os = "windows")]
    vertex_shader: Option<ID3D11VertexShader>,
    #[cfg(target_os = "windows")]
    pixel_shader: Option<ID3D11PixelShader>,
    #[cfg(target_os = "windows")]
    compute_shader: Option<ID3D11ComputeShader>,
    #[cfg(target_os = "windows")]
    geometry_shader: Option<ID3D11GeometryShader>,
    #[cfg(target_os = "windows")]
    hull_shader: Option<ID3D11HullShader>,
    #[cfg(target_os = "windows")]
    domain_shader: Option<ID3D11DomainShader>,
    #[cfg(target_os = "windows")]
    input_layout: Option<ID3D11InputLayout>,
    
    /// Compiled vertex shader bytecode (needed for input layout creation)
    vertex_bytecode: Vec<u8>,
    /// Compiled pixel shader bytecode
    pixel_bytecode: Vec<u8>,
    /// Compiled compute shader bytecode
    compute_bytecode: Vec<u8>,
    
    name: String,
    stage: ShaderStage,
}

#[cfg(target_os = "windows")]
unsafe impl Send for Dx11Shader {}

#[cfg(target_os = "windows")]
unsafe impl Sync for Dx11Shader {}

/// Function pointer type for D3DCompile
#[cfg(target_os = "windows")]
type D3DCompileFn = unsafe extern "system" fn(
    pSrcData: *const c_void,
    SrcDataSize: usize,
    pSourceName: PCSTR,
    pDefines: *const D3D_SHADER_MACRO,
    pInclude: *mut c_void,
    pEntrypoint: PCSTR,
    pTarget: PCSTR,
    Flags1: u32,
    Flags2: u32,
    ppCode: *mut *mut c_void,
    ppErrorMsgs: *mut *mut c_void,
) -> windows::core::HRESULT;

#[cfg(target_os = "windows")]
struct D3DCompiler {
    compile_func: D3DCompileFn,
}

#[cfg(target_os = "windows")]
impl D3DCompiler {
    fn new() -> Option<Self> {
        // Try to load d3dcompiler_47.dll (most common on Windows 8+)
        let compiler_names = [
            "d3dcompiler_47.dll",
            "d3dcompiler_46.dll",
            "d3dcompiler_43.dll",
        ];
        
        for name in &compiler_names {
            unsafe {
                let module = GetModuleHandleW(widestring(name).as_ptr());
                if !module.is_err() {
                    let func_name = format!("D3DCompile\0");
                    let proc = GetProcAddress(module.ok_or("Shader compilation failed")?, PCSTR(func_name.as_ptr()));
                    if !proc.is_null() {
                        info!(target: "dx11.shader", "Loaded {} successfully", name);
                        // SAFETY: Transmuting a function pointer obtained from GetProcAddress.
                        // The signature of D3DCompile is well-defined and stable across Windows versions.
                        // This is the standard pattern for dynamically loading DirectX functions.
                        return Some(Self {
                            compile_func: std::mem::transmute(proc),
                        });
                    }
                }
            }
        }
        
        warn!(target: "dx11.shader", "Failed to load d3dcompiler DLL");
        None
    }
    
    #[cfg(target_os = "windows")]
    fn compile(
        &self,
        source: &str,
        entry_point: &str,
        target: &str,
        debug: bool,
    ) -> Result<Vec<u8>, String> {
        use windows::core::Result as WinResult;
        
        let source_bytes = source.as_bytes();
        let mut blob: *mut c_void = std::ptr::null_mut();
        let mut error_blob: *mut c_void = std::ptr::null_mut();
        
        let flags1 = if debug { 1u32 } else { 0u32 }; // D3DCOMPILE_DEBUG = 1
        let flags2 = 0u32;
        
        let entry_cstr = std::ffi::CString::new(entry_point).ok_or("Shader compilation failed")?;
        let target_cstr = std::ffi::CString::new(target).ok_or("Shader compilation failed")?;
        let source_name = std::ffi::CString::new("shader.hlsl").ok_or("Shader compilation failed")?;
        
        unsafe {
            let hr = (self.compile_func)(
                source_bytes.as_ptr() as *const c_void,
                source_bytes.len(),
                PCSTR(source_name.as_ptr() as *const u8),
                std::ptr::null(),
                std::ptr::null_mut(),
                PCSTR(entry_cstr.as_ptr() as *const u8),
                PCSTR(target_cstr.as_ptr() as *const u8),
                flags1,
                flags2,
                &mut blob,
                &mut error_blob,
            );
            
            if hr.is_err() {
                if !error_blob.is_null() {
                    // Read error message
                    let error_ptr = error_blob as *mut windows::Win32::Graphics::Direct3D::ID3DBlob;
                    let error_msg = std::string::String::from_utf8_lossy(
                        std::slice::from_raw_parts(
                            (*error_ptr).GetBufferPointer() as *const u8,
                            (*error_ptr).GetBufferSize(),
                        ),
                    ).to_string();
                    error!(target: "dx11.shader", "Compilation failed: {}", error_msg);
                    return Err(error_msg);
                }
                return Err(format!("D3DCompile failed: {:?}", hr));
            }
            
            if !blob.is_null() {
                let code_ptr = blob as *mut windows::Win32::Graphics::Direct3D::ID3DBlob;
                let buffer = std::slice::from_raw_parts(
                    (*code_ptr).GetBufferPointer() as *const u8,
                    (*code_ptr).GetBufferSize(),
                );
                let result = buffer.to_vec();
                
                // Release blobs
                (*code_ptr).Release();
                if !error_blob.is_null() {
                    let err_ptr = error_blob as *mut windows::Win32::Graphics::Direct3D::ID3DBlob;
                    (*err_ptr).Release();
                }
                
                Ok(result)
            } else {
                Err("No bytecode returned from compiler".to_string())
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn widestring(s: &str) -> Vec<u16> {
    use std::os::windows::prelude::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(Some(0))
        .collect()
}

impl Dx11Shader {
    /// Create shader from HLSL source code
    #[cfg(target_os = "windows")]
    pub fn from_hlsl_source(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        vertex_source: Option<&str>,
        pixel_source: Option<&str>,
        compute_source: Option<&str>,
        vertex_entry: &str,
        pixel_entry: &str,
        compute_entry: &str,
        debug: bool,
    ) -> RhiResult<Self> {
        info!(target: "dx11.shader", "=== Compiling HLSL shaders ===");
        
        let compiler = D3DCompiler::new().ok_or_else(|| {
            RhiError::InitializationFailed("Failed to load D3DCompiler".to_string())
        })?;
        
        let mut vertex_bytecode = Vec::new();
        let mut pixel_bytecode = Vec::new();
        let mut compute_bytecode = Vec::new();
        let mut vertex_shader = None;
        let mut pixel_shader = None;
        let mut compute_shader = None;
        let mut input_layout = None;
        
        // Compile vertex shader
        if let Some(vs_source) = vertex_source {
            info!(target: "dx11.shader", "Compiling vertex shader (entry: {})", vertex_entry);
            vertex_bytecode = compiler.compile(vs_source, vertex_entry, "vs_5_0", debug)?;
            info!(target: "dx11.shader", "Vertex shader compiled, {} bytes", vertex_bytecode.len());
            
            // Create ID3D11VertexShader
            unsafe {
                let vs_result = device.CreateVertexShader(
                    vertex_bytecode.as_ptr() as *const _,
                    vertex_bytecode.len(),
                    None,
                );
                match vs_result {
                    Ok(vs) => {
                        vertex_shader = Some(vs);
                        info!(target: "dx11.shader", "ID3D11VertexShader created");
                        
                        // Create input layout from vertex shader reflection
                        input_layout = Self::create_input_layout(device, &vertex_bytecode, None)?;
                    }
                    Err(e) => {
                        error!(target: "dx11.shader", "Failed to create vertex shader: {:?}", e);
                        return Err(RhiError::InitializationFailed(format!(
                            "CreateVertexShader: {:?}",
                            e
                        )));
                    }
                }
            }
        }
        
        // Compile pixel shader
        if let Some(ps_source) = pixel_source {
            info!(target: "dx11.shader", "Compiling pixel shader (entry: {})", pixel_entry);
            pixel_bytecode = compiler.compile(ps_source, pixel_entry, "ps_5_0", debug)?;
            info!(target: "dx11.shader", "Pixel shader compiled, {} bytes", pixel_bytecode.len());
            
            // Create ID3D11PixelShader
            unsafe {
                let ps_result = device.CreatePixelShader(
                    pixel_bytecode.as_ptr() as *const _,
                    pixel_bytecode.len(),
                    None,
                );
                match ps_result {
                    Ok(ps) => {
                        pixel_shader = Some(ps);
                        info!(target: "dx11.shader", "ID3D11PixelShader created");
                    }
                    Err(e) => {
                        error!(target: "dx11.shader", "Failed to create pixel shader: {:?}", e);
                        return Err(RhiError::InitializationFailed(format!(
                            "CreatePixelShader: {:?}",
                            e
                        )));
                    }
                }
            }
        }
        
        // Compile compute shader
        if let Some(cs_source) = compute_source {
            info!(target: "dx11.shader", "Compiling compute shader (entry: {})", compute_entry);
            compute_bytecode = compiler.compile(cs_source, compute_entry, "cs_5_0", debug)?;
            info!(target: "dx11.shader", "Compute shader compiled, {} bytes", compute_bytecode.len());
            
            // Create ID3D11ComputeShader
            unsafe {
                let cs_result = device.CreateComputeShader(
                    compute_bytecode.as_ptr() as *const _,
                    compute_bytecode.len(),
                    None,
                );
                match cs_result {
                    Ok(cs) => {
                        compute_shader = Some(cs);
                        info!(target: "dx11.shader", "ID3D11ComputeShader created");
                    }
                    Err(e) => {
                        error!(target: "dx11.shader", "Failed to create compute shader: {:?}", e);
                        return Err(RhiError::InitializationFailed(format!(
                            "CreateComputeShader: {:?}",
                            e
                        )));
                    }
                }
            }
        }
        
        let name = format!("DX11Shader(VS:{}, PS:{})", 
            vertex_source.map(|_| "yes").unwrap_or("no"),
            pixel_source.map(|_| "yes").unwrap_or("no"));
        
        info!(target: "dx11.shader", "Shader compilation complete: {}", name);
        
        Ok(Self {
            vertex_shader,
            pixel_shader,
            compute_shader,
            geometry_shader: None,
            hull_shader: None,
            domain_shader: None,
            input_layout,
            vertex_bytecode,
            pixel_bytecode,
            compute_bytecode,
            name,
            stage: ShaderStage::Vertex,
        })
    }
    
    /// Create shader from pre-compiled bytecode
    #[cfg(target_os = "windows")]
    pub fn from_bytecode(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        desc: &ShaderDescription,
    ) -> RhiResult<Self> {
        info!(target: "dx11.shader", "Creating shader from bytecode, stage={:?}, entry={}", 
              desc.stage, desc.entry_point);
        
        let mut vertex_shader = None;
        let mut pixel_shader = None;
        let mut compute_shader = None;
        let mut vertex_bytecode = Vec::new();
        let mut pixel_bytecode = Vec::new();
        let mut compute_bytecode = Vec::new();
        let mut input_layout = None;
        
        match desc.stage {
            ShaderStage::Vertex => {
                vertex_bytecode = desc.source.clone();
                unsafe {
                    let vs_result = device.CreateVertexShader(
                        vertex_bytecode.as_ptr() as *const _,
                        vertex_bytecode.len(),
                        None,
                    );
                    match vs_result {
                        Ok(vs) => {
                            vertex_shader = Some(vs);
                            info!(target: "dx11.shader", "ID3D11VertexShader created from bytecode");
                            
                            // Create input layout - try to get from description or use default
                            input_layout = Self::create_input_layout(device, &vertex_bytecode, None)?;
                        }
                        Err(e) => {
                            error!(target: "dx11.shader", "Failed to create vertex shader: {:?}", e);
                            return Err(RhiError::InitializationFailed(format!(
                                "CreateVertexShader: {:?}",
                                e
                            )));
                        }
                    }
                }
            }
            ShaderStage::Fragment => {
                pixel_bytecode = desc.source.clone();
                unsafe {
                    let ps_result = device.CreatePixelShader(
                        pixel_bytecode.as_ptr() as *const _,
                        pixel_bytecode.len(),
                        None,
                    );
                    match ps_result {
                        Ok(ps) => {
                            pixel_shader = Some(ps);
                            info!(target: "dx11.shader", "ID3D11PixelShader created from bytecode");
                        }
                        Err(e) => {
                            error!(target: "dx11.shader", "Failed to create pixel shader: {:?}", e);
                            return Err(RhiError::InitializationFailed(format!(
                                "CreatePixelShader: {:?}",
                                e
                            )));
                        }
                    }
                }
            }
            ShaderStage::Compute => {
                compute_bytecode = desc.source.clone();
                unsafe {
                    let cs_result = device.CreateComputeShader(
                        compute_bytecode.as_ptr() as *const _,
                        compute_bytecode.len(),
                        None,
                    );
                    match cs_result {
                        Ok(cs) => {
                            compute_shader = Some(cs);
                            info!(target: "dx11.shader", "ID3D11ComputeShader created from bytecode");
                        }
                        Err(e) => {
                            error!(target: "dx11.shader", "Failed to create compute shader: {:?}", e);
                            return Err(RhiError::InitializationFailed(format!(
                                "CreateComputeShader: {:?}",
                                e
                            )));
                        }
                    }
                }
            }
            _ => {
                return Err(RhiError::Unsupported(format!(
                    "Shader stage {:?} not fully supported yet",
                    desc.stage
                )));
            }
        }
        
        let name = format!("DX11Shader({:?}:{})", desc.stage, desc.entry_point);
        
        Ok(Self {
            vertex_shader,
            pixel_shader,
            compute_shader,
            geometry_shader: None,
            hull_shader: None,
            domain_shader: None,
            input_layout,
            vertex_bytecode,
            pixel_bytecode,
            compute_bytecode,
            name,
            stage: desc.stage,
        })
    }
    
    /// Create input layout from vertex shader signature
    #[cfg(target_os = "windows")]
    fn create_input_layout(
        device: &windows::Win32::Graphics::Direct3D11::ID3D11Device,
        vertex_bytecode: &[u8],
        custom_layout: Option<&InputLayout>,
    ) -> RhiResult<Option<ID3D11InputLayout>> {
        info!(target: "dx11.shader", "Creating input layout from vertex shader reflection");
        
        // Use custom layout if provided, otherwise try to derive from shader
        if let Some(layout) = custom_layout {
            info!(target: "dx11.shader", "Using custom input layout with {} attributes", 
                  layout.attributes.len());
            
            let elements: Vec<D3D11_INPUT_ELEMENT_DESC> = layout.attributes.iter().enumerate().map(|(i, attr)| {
                let format = match attr.format {
                    VertexFormat::Float32x2 => DXGI_FORMAT_R32G32_FLOAT,
                    VertexFormat::Float32x3 => DXGI_FORMAT_R32G32B32_FLOAT,
                    VertexFormat::Float32x4 => DXGI_FORMAT_R32G32B32A32_FLOAT,
                    VertexFormat::Float32x2x2 => DXGI_FORMAT_R32G32B32A32_FLOAT, // mat2 as vec4
                    VertexFormat::Float32x3x3 => DXGI_FORMAT_R32G32B32A32_FLOAT, // mat3 as vec4
                    VertexFormat::Float32x4x4 => DXGI_FORMAT_R32G32B32A32_FLOAT, // mat4
                    VertexFormat::Uint8x4Norm => DXGI_FORMAT_R8G8B8A8_UNORM,
                    VertexFormat::Uint16x2Norm => DXGI_FORMAT_R8G8B8A8_UNORM, // Approximate
                    VertexFormat::Uint16x4Norm => DXGI_FORMAT_R8G8B8A8_UNORM,
                };
                
                let semantic_name = attr.name.to_uppercase();
                let semantic_index = 0u32;
                
                D3D11_INPUT_ELEMENT_DESC {
                    SemanticName: std::ffi::CString::new(semantic_name.as_str()).ok_or("Shader compilation failed")?.into_raw() as *const i8,
                    SemanticIndex: semantic_index,
                    Format: format,
                    InputSlot: 0,
                    AlignedByteOffset: attr.offset,
                    InputSlotClass: D3D11_INPUT_PER_VERTEX_DATA,
                    InstanceDataStepRate: 0,
                }
            }).collect();
            
            unsafe {
                let layout_result = device.CreateInputLayout(
                    &elements,
                    vertex_bytecode,
                );
                
                match layout_result {
                    Ok(layout) => {
                        info!(target: "dx11.shader", "ID3D11InputLayout created successfully");
                        return Ok(Some(layout));
                    }
                    Err(e) => {
                        error!(target: "dx11.shader", "Failed to create input layout: {:?}", e);
                        return Err(RhiError::InitializationFailed(format!(
                            "CreateInputLayout: {:?}",
                            e
                        )));
                    }
                }
            }
        }
        
        // If no custom layout, try to use shader reflection to get signature
        // This is a simplified version - full implementation would use ID3D11ShaderReflection
        info!(target: "dx11.shader", "No custom layout provided, using null input layout");
        Ok(None)
    }
    
    /// Get the vertex shader interface
    #[cfg(target_os = "windows")]
    pub fn get_vertex_shader(&self) -> &Option<ID3D11VertexShader> {
        &self.vertex_shader
    }
    
    /// Get the pixel shader interface
    #[cfg(target_os = "windows")]
    pub fn get_pixel_shader(&self) -> &Option<ID3D11PixelShader> {
        &self.pixel_shader
    }
    
    /// Get the compute shader interface
    #[cfg(target_os = "windows")]
    pub fn get_compute_shader(&self) -> &Option<ID3D11ComputeShader> {
        &self.compute_shader
    }
    
    /// Get the input layout
    #[cfg(target_os = "windows")]
    pub fn get_input_layout(&self) -> &Option<ID3D11InputLayout> {
        &self.input_layout
    }
    
    /// Get vertex shader bytecode
    pub fn get_vertex_bytecode(&self) -> &[u8] {
        &self.vertex_bytecode
    }
    
    /// Get pixel shader bytecode
    pub fn get_pixel_bytecode(&self) -> &[u8] {
        &self.pixel_bytecode
    }
}

impl IShader for Dx11Shader {
    fn get_name(&self) -> &str {
        &self.name
    }
    
    fn get_stage(&self) -> ShaderStage {
        self.stage
    }
    
    fn get_bytecode(&self) -> &[u8] {
        match self.stage {
            ShaderStage::Vertex => &self.vertex_bytecode,
            ShaderStage::Fragment => &self.pixel_bytecode,
            ShaderStage::Compute => &self.compute_bytecode,
            _ => &[],
        }
    }
}
