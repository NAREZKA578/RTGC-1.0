// DirectX 12 Backend - Shader Implementation
// Implements shader compilation and management for DX12

use crate::graphics::rhi::types::*;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*,
    Win32::Graphics::Direct3D12::*,
    Win32::Graphics::Dxc::*,
};

/// DX12 Shader
pub struct Dx12Shader {
    #[cfg(target_os = "windows")]
    bytecode: Vec<u8>,
    
    handle: ResourceHandle,
    description: ShaderDescription,
    entry_point: String,
}

unsafe impl Send for Dx12Shader {}
unsafe impl Sync for Dx12Shader {}

impl Dx12Shader {
    /// Create a new DX12 shader from SPIR-V or DXIL bytecode
    #[cfg(target_os = "windows")]
    pub fn new(desc: &ShaderDescription, handle: ResourceHandle) -> RhiResult<Self> {
        // If source is already DXIL bytecode, use it directly
        // Otherwise, compile HLSL source to DXIL
        
        let bytecode = if Self::is_dxil(&desc.source) {
            desc.source.clone()
        } else if Self::is_spirv(&desc.source) {
            // SPIR-V не поддерживается напрямую в DX12 без конвертации
            // Для кросс-платформенной совместимости рекомендуется использовать HLSL или DXIL
            return Err(RhiError::Unsupported(
                "SPIR-V shaders require conversion to DXIL. Please use HLSL source or pre-compiled DXIL for DX12 backend.".to_string()
            ));
        } else {
            // Assume HLSL source, compile it
            Self::compile_hlsl(&desc.source, &desc.entry_point, desc.stage)?
        };
        
        Ok(Self {
            bytecode,
            handle,
            description: desc.clone(),
            entry_point: desc.entry_point.clone(),
        })
    }
    
    #[cfg(target_os = "windows")]
    fn is_dxil(source: &[u8]) -> bool {
        // DXIL magic number: 'DXIL'
        source.len() >= 4 && &source[0..4] == b"DXIL"
    }
    
    #[cfg(target_os = "windows")]
    fn is_spirv(source: &[u8]) -> bool {
        // SPIR-V magic number: 0x07230203
        source.len() >= 4 && u32::from_le_bytes([source[0], source[1], source[2], source[3]]) == 0x07230203
    }
    
    #[cfg(target_os = "windows")]
    fn compile_hlsl(source: &[u8], entry_point: &str, stage: ShaderStage) -> RhiResult<Vec<u8>> {
        use windows::Win32::Graphics::Dxc::*;
        use windows::core::{IUnknown, Interface};
        
        // Initialize DxcCompiler
        let compiler: IDxcCompiler3 = unsafe {
            CoCreateInstance(&DxcCompiler)
                .map_err(|e| RhiError::ShaderCompilationFailed(format!("Failed to create DXC compiler: {:?}", e)))?
        };
        
        // Create blob from source
        let library: IDxcBlobEncoding = unsafe {
            DxcCreateBlobFromEncoding(
                source.as_ptr(),
                source.len() as u32,
                0, // CP_ACP
            )
            .map_err(|e| RhiError::ShaderCompilationFailed(format!("Failed to create blob: {:?}", e)))?
        };
        
        // Create buffer for blob
        let mut result_blob: Option<IDxcBlob> = None;
        
        // Build arguments
        let entry_point_wide: Vec<u16> = format!("-E{}\0", entry_point).encode_utf16().collect();
        let target_profile = Self::get_target_profile(stage);
        let target_wide: Vec<u16> = format!("-T{}\0", target_profile).encode_utf16().collect();
        
        let args: Vec<&[u16]> = vec![
            &entry_point_wide,
            &target_wide,
        ];
        
        let arg_ptrs: Vec<*const u16> = args.iter().map(|s| s.as_ptr()).collect();
        
        // Compile
        unsafe {
            compiler.Compile(
                &library,
                None, // No file name
                arg_ptrs.as_ptr(),
                arg_ptrs.len() as u32,
                None, // No include handler
                &mut result_blob,
            )
            .map_err(|e| RhiError::ShaderCompilationFailed(format!("Compilation failed: {:?}", e)))?;
        }
        
        let result_blob = result_blob
            .ok_or_else(|| RhiError::ShaderCompilationFailed("No compilation result".to_string()))?;
        
        // Get result bytes
        let result_bytes = unsafe {
            std::slice::from_raw_parts(
                result_blob.GetBufferPointer() as *const u8,
                result_blob.GetBufferSize(),
            )
        };
        
        Ok(result_bytes.to_vec())
    }
    
    #[cfg(target_os = "windows")]
    fn get_target_profile(stage: ShaderStage) -> &'static str {
        match stage {
            ShaderStage::Vertex => "vs_6_0",
            ShaderStage::Fragment => "ps_6_0",
            ShaderStage::Compute => "cs_6_0",
            ShaderStage::Geometry => "gs_6_0",
            ShaderStage::TessellationControl => "hs_6_0",
            ShaderStage::TessellationEvaluation => "ds_6_0",
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    pub fn new(desc: &ShaderDescription, handle: ResourceHandle) -> RhiResult<Self> {
        Err(RhiError::Unsupported("DirectX 12 is only available on Windows".to_string()))
    }
    
    #[cfg(target_os = "windows")]
    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }
    
    pub fn handle(&self) -> ResourceHandle {
        self.handle
    }
    
    pub fn description(&self) -> &ShaderDescription {
        &self.description
    }
    
    pub fn entry_point(&self) -> &str {
        &self.entry_point
    }
}

/// Helper function to create DX12 shader
pub fn create_dx12_shader(desc: &ShaderDescription) -> RhiResult<Dx12Shader> {
    static HANDLE_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let handle = ResourceHandle(HANDLE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed));
    
    Dx12Shader::new(desc, handle)
}
