// DirectX 12 Backend - Command List Implementation
// Implements command recording and submission for DX12

use crate::graphics::rhi::{
    device::{ICommandList, ICommandQueue, IFence, ISemaphore},
    resource_manager::ResourceManager,
    types::*,
};
use std::sync::Arc;

#[cfg(target_os = "windows")]
use windows::{Win32::Foundation::*, Win32::Graphics::Direct3D12::*};

/// DX12 Command List
pub struct Dx12CommandList {
    #[cfg(target_os = "windows")]
    command_list: ID3D12GraphicsCommandList,

    #[cfg(target_os = "windows")]
    allocator: ID3D12CommandAllocator,

    cmd_type: CommandListType,
    is_closed: bool,
    resource_manager: Arc<ResourceManager>,
}

unsafe impl Send for Dx12CommandList {}
unsafe impl Sync for Dx12CommandList {}

impl Dx12CommandList {
    /// Create a new DX12 command list
    #[cfg(target_os = "windows")]
    pub fn new(device: &ID3D12Device, cmd_type: CommandListType) -> RhiResult<Self> {
        use windows::Win32::Graphics::Direct3D12::*;

        let dx12_cmd_type = match cmd_type {
            CommandListType::Direct => D3D12_COMMAND_LIST_TYPE_DIRECT,
            CommandListType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
            CommandListType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
        };

        // Create command allocator
        let allocator: ID3D12CommandAllocator = unsafe {
            device.CreateCommandAllocator(dx12_cmd_type).map_err(|e| {
                RhiError::ResourceCreationFailed(format!(
                    "Failed to create command allocator: {:?}",
                    e
                ))
            })?
        };

        // Create command list
        let command_list: ID3D12GraphicsCommandList = unsafe {
            device
                .CreateCommandList(0, dx12_cmd_type, &allocator, None)
                .map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to create command list: {:?}",
                        e
                    ))
                })?
        };

        // Close immediately (will be reset before use)
        unsafe {
            command_list.Close().map_err(|e| {
                RhiError::InitializationFailed(format!("Failed to close command list: {:?}", e))
            })?;
        }

        Ok(Self {
            command_list,
            allocator,
            cmd_type,
            is_closed: true,
            resource_manager: Arc::new(ResourceManager::new()),
        })
    }

    /// Create a new DX12 command list with shared resource manager
    #[cfg(target_os = "windows")]
    pub fn with_resource_manager(
        device: &ID3D12Device,
        cmd_type: CommandListType,
        resource_manager: Arc<ResourceManager>,
    ) -> RhiResult<Self> {
        use windows::Win32::Graphics::Direct3D12::*;

        let dx12_cmd_type = match cmd_type {
            CommandListType::Direct => D3D12_COMMAND_LIST_TYPE_DIRECT,
            CommandListType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
            CommandListType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
        };

        // Create command allocator
        let allocator: ID3D12CommandAllocator = unsafe {
            device.CreateCommandAllocator(dx12_cmd_type).map_err(|e| {
                RhiError::ResourceCreationFailed(format!(
                    "Failed to create command allocator: {:?}",
                    e
                ))
            })?
        };

        // Create command list
        let command_list: ID3D12GraphicsCommandList = unsafe {
            device
                .CreateCommandList(0, dx12_cmd_type, &allocator, None)
                .map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to create command list: {:?}",
                        e
                    ))
                })?
        };

        // Close immediately (will be reset before use)
        unsafe {
            command_list.Close().map_err(|e| {
                RhiError::InitializationFailed(format!("Failed to close command list: {:?}", e))
            })?;
        }

        Ok(Self {
            command_list,
            allocator,
            cmd_type,
            is_closed: true,
            resource_manager,
        })
    }

    /// Get the resource manager
    pub fn get_resource_manager(&self) -> Arc<ResourceManager> {
        self.resource_manager.clone()
    }

    /// Reset the command list for re-recording
    #[cfg(target_os = "windows")]
    pub fn reset(&mut self) -> RhiResult<()> {
        unsafe {
            self.allocator.Reset().map_err(|e| {
                RhiError::InitializationFailed(format!("Failed to reset allocator: {:?}", e))
            })?;

            self.command_list
                .Reset(&self.allocator, None)
                .map_err(|e| {
                    RhiError::InitializationFailed(format!("Failed to reset command list: {:?}", e))
                })?;
        }

        self.is_closed = false;
        Ok(())
    }

    /// Close the command list for submission
    #[cfg(target_os = "windows")]
    pub fn close(&mut self) -> RhiResult<()> {
        if !self.is_closed {
            unsafe {
                self.command_list.Close().map_err(|e| {
                    RhiError::InitializationFailed(format!("Failed to close command list: {:?}", e))
                })?;
            }
            self.is_closed = true;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    pub fn command_list(&self) -> &ID3D12GraphicsCommandList {
        &self.command_list
    }
}

impl ICommandList for Dx12CommandList {
    fn reset(&mut self) -> RhiResult<()> {
        #[cfg(target_os = "windows")]
        return self.reset();

        #[cfg(not(target_os = "windows"))]
        Err(RhiError::Unsupported(
            "DX12 is only available on Windows".to_string(),
        ))
    }

    fn close(&mut self) -> RhiResult<()> {
        #[cfg(target_os = "windows")]
        return self.close();

        #[cfg(not(target_os = "windows"))]
        Err(RhiError::Unsupported(
            "DX12 is only available on Windows".to_string(),
        ))
    }

    fn begin_render_pass(&mut self, desc: &RenderPassDescription) {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D12::*;

            // 4.39: Transition resources to RENDER_TARGET state
            let mut barriers = Vec::new();
            for color_attachment in &desc.color_attachments {
                // SAFETY: Transmuting D3D12_RESOURCE_TRANSITION_BARRIER to union member.
                // This is a POD struct with no padding, standard DirectX 12 FFI pattern.
                let barrier = D3D12_RESOURCE_BARRIER {
                    Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                    Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                    Anonymous: D3D12_RESOURCE_BARRIER_0 {
                        Transition: std::mem::transmute(D3D12_RESOURCE_TRANSITION_BARRIER {
                            pResource: Some(color_attachment.resource.clone()),
                            Subresource: 0,
                            StateBefore: D3D12_RESOURCE_STATE_PRESENT,
                            StateAfter: D3D12_RESOURCE_STATE_RENDER_TARGET,
                        }),
                    },
                };
                barriers.push(barrier);
            }

            if !barriers.is_empty() {
                self.command_list.ResourceBarrier(&barriers);
            }

            // Set render targets
            let rtv_handles: Vec<D3D12_CPU_DESCRIPTOR_HANDLE> = desc
                .color_attachments
                .iter()
                .map(|att| att.rtv_handle)
                .collect();

            let dsv_handle = desc.depth_attachment.as_ref().map(|att| att.dsv_handle);

            self.command_list.OMSetRenderTargets(
                rtv_handles.len() as u32,
                rtv_handles.as_ptr(),
                FALSE,
                dsv_handle
                    .as_ref()
                    .map(|h| h as *const _)
                    .unwrap_or(std::ptr::null()),
            );

            // Clear color attachments if requested
            for (i, color_attachment) in desc.color_attachments.iter().enumerate() {
                if let Some(clear_color) = color_attachment.clear_color {
                    self.command_list.ClearRenderTargetView(
                        rtv_handles[i],
                        &clear_color,
                        0,
                        std::ptr::null(),
                    );
                }
            }

            // Clear depth/stencil if requested
            if let Some(ref depth_attachment) = desc.depth_attachment {
                if depth_attachment.clear_depth.is_some()
                    || depth_attachment.clear_stencil.is_some()
                {
                    let mut clear_flags = D3D12_CLEAR_FLAG_NONE;
                    let clear_depth = depth_attachment.clear_depth.unwrap_or(1.0);
                    let clear_stencil = depth_attachment.clear_stencil.unwrap_or(0);

                    if depth_attachment.clear_depth.is_some() {
                        clear_flags |= D3D12_CLEAR_FLAG_DEPTH;
                    }
                    if depth_attachment.clear_stencil.is_some() {
                        clear_flags |= D3D12_CLEAR_FLAG_STENCIL;
                    }

                    self.command_list.ClearDepthStencilView(
                        depth_attachment.dsv_handle,
                        clear_flags,
                        clear_depth,
                        clear_stencil,
                        0,
                        std::ptr::null(),
                    );
                }
            }
        }
    }

    fn end_render_pass(&mut self) {
        // DX12 doesn't have explicit render passes like Vulkan
    }

    fn set_pipeline_state(&mut self, pso: ResourceHandle) {
        #[cfg(target_os = "windows")]
        {
            // 4.40: Set PSO from handle using resource manager
            unsafe {
                if let Some(pipeline) = self.resource_manager.get_pipeline(pso) {
                    if let Some(pso_state) = pipeline.dx12_pso {
                        self.command_list.SetPipelineState(pso_state);
                    }
                }
            }
        }
    }

    fn set_primitive_topology(&mut self, topology: PrimitiveTopology) {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D12::*;

            let dx12_topology = match topology {
                PrimitiveTopology::PointList => D3D_PRIMITIVE_TOPOLOGY_POINTLIST,
                PrimitiveTopology::LineList => D3D_PRIMITIVE_TOPOLOGY_LINELIST,
                PrimitiveTopology::LineStrip => D3D_PRIMITIVE_TOPOLOGY_LINESTRIP,
                PrimitiveTopology::TriangleList => D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
                PrimitiveTopology::TriangleStrip => D3D_PRIMITIVE_TOPOLOGY_TRIANGLESTRIP,
            };

            unsafe {
                self.command_list.IASetPrimitiveTopology(dx12_topology);
            }
        }
    }

    fn set_viewport(&mut self, viewport: &Viewport) {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D12::*;

            let vp = D3D12_VIEWPORT {
                TopLeftX: viewport.x,
                TopLeftY: viewport.y,
                Width: viewport.width,
                Height: viewport.height,
                MinDepth: viewport.min_depth,
                MaxDepth: viewport.max_depth,
            };

            unsafe {
                self.command_list.RSSetViewports(std::slice::from_ref(&vp));
            }
        }
    }

    fn set_scissor_rect(&mut self, scissor: &ScissorRect) {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D12::*;

            let rect = D3D12_RECT {
                left: scissor.left,
                top: scissor.top,
                right: scissor.right,
                bottom: scissor.bottom,
            };

            unsafe {
                self.command_list
                    .RSSetScissorRects(std::slice::from_ref(&rect));
            }
        }
    }

    fn set_blend_constants(&mut self, constants: [f32; 4]) {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                self.command_list.OMSetBlendFactor(Some(&constants));
            }
        }
    }

    fn set_stencil_reference(&mut self, reference: u8) {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                self.command_list.OMSetStencilRef(reference as u32);
            }
        }
    }

    fn bind_vertex_buffers(&mut self, start_slot: u32, buffers: &[(ResourceHandle, u64)]) {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D12::*;

            // 4.41: Bind Vertex Buffers через реальные buffer views
            unsafe {
                for (i, (buffer_handle, offset)) in buffers.iter().enumerate() {
                    if let Some(buffer) = self.resource_manager.get_buffer(*buffer_handle) {
                        if let Some(resource) = buffer.dx12_resource {
                            let view = D3D12_VERTEX_BUFFER_VIEW {
                                BufferLocation: resource.GetGPUVirtualAddress() + offset,
                                SizeInBytes: buffer.size as u32,
                                StrideInBytes: buffer.size as u32, // Should be stride from pipeline
                            };
                            self.command_list
                                .IASetVertexBuffers(start_slot + i as u32, 1, &view);
                        }
                    }
                }
            }
        }
    }

    fn bind_index_buffer(
        &mut self,
        buffer: ResourceHandle,
        offset: u64,
        index_format: IndexFormat,
    ) {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D12::*;

            let dxgi_format = match index_format {
                IndexFormat::Uint16 => DXGI_FORMAT_R16_UINT,
                IndexFormat::Uint32 => DXGI_FORMAT_R32_UINT,
            };

            // 4.41: Bind Index Buffer через реальный buffer view
            unsafe {
                if let Some(buffer) = self.resource_manager.get_buffer(buffer) {
                    if let Some(resource) = buffer.dx12_resource {
                        let view = D3D12_INDEX_BUFFER_VIEW {
                            BufferLocation: resource.GetGPUVirtualAddress() + offset,
                            SizeInBytes: buffer.size as u32,
                            Format: dxgi_format,
                        };
                        self.command_list.IASetIndexBuffer(&view);
                    }
                }
            }
        }
    }

    fn bind_constant_buffer(&mut self, stage: ShaderStage, slot: u32, buffer: ResourceHandle) {
        #[cfg(target_os = "windows")]
        {
            // 4.42: Bind constant buffer в root signature
            unsafe {
                if let Some(buffer) = self.resource_manager.get_buffer(buffer) {
                    if let Some(resource) = buffer.dx12_resource {
                        let gpu_address = resource.GetGPUVirtualAddress();
                        match stage {
                            ShaderStage::Vertex
                            | ShaderStage::Fragment
                            | ShaderStage::Geometry
                            | ShaderStage::TessellationControl
                            | ShaderStage::TessellationEvaluation => {
                                self.command_list
                                    .SetGraphicsRootConstantBufferView(slot, gpu_address);
                            }
                            ShaderStage::Compute => {
                                self.command_list
                                    .SetComputeRootConstantBufferView(slot, gpu_address);
                            }
                        }
                    }
                }
            }
        }
    }

    fn bind_shader_resource(&mut self, stage: ShaderStage, slot: u32, view: ResourceHandle) {
        #[cfg(target_os = "windows")]
        {
            // 4.43: Bind SRV (Shader Resource View)
            unsafe {
                if let Some(texture) = self.resource_manager.get_texture(view) {
                    if let Some(srv_handle) = texture.dx12_srv_handle {
                        let handle = D3D12_CPU_DESCRIPTOR_HANDLE { ptr: srv_handle };
                        match stage {
                            ShaderStage::Vertex
                            | ShaderStage::Fragment
                            | ShaderStage::Geometry
                            | ShaderStage::TessellationControl
                            | ShaderStage::TessellationEvaluation => {
                                self.command_list
                                    .SetGraphicsRootDescriptorTable(slot, handle);
                            }
                            ShaderStage::Compute => {
                                self.command_list
                                    .SetComputeRootDescriptorTable(slot, handle);
                            }
                        }
                    }
                }
            }
        }
    }

    fn bind_sampler(&mut self, stage: ShaderStage, slot: u32, sampler: ResourceHandle) {
        #[cfg(target_os = "windows")]
        {
            // 4.44: Bind sampler
            unsafe {
                if let Some(sampler_res) = self.resource_manager.get_sampler(sampler) {
                    if let Some(sampler_handle) = sampler_res.dx12_handle {
                        let handle = D3D12_CPU_DESCRIPTOR_HANDLE {
                            ptr: sampler_handle,
                        };
                        match stage {
                            ShaderStage::Vertex
                            | ShaderStage::Fragment
                            | ShaderStage::Geometry
                            | ShaderStage::TessellationControl
                            | ShaderStage::TessellationEvaluation => {
                                self.command_list
                                    .SetGraphicsRootDescriptorTable(slot, handle);
                            }
                            ShaderStage::Compute => {
                                self.command_list
                                    .SetComputeRootDescriptorTable(slot, handle);
                            }
                        }
                    }
                }
            }
        }
    }

    fn draw(
        &mut self,
        vertex_count: u32,
        instance_count: u32,
        start_vertex: u32,
        start_instance: u32,
    ) {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                self.command_list.DrawInstanced(
                    vertex_count,
                    instance_count,
                    start_vertex,
                    start_instance,
                );
            }
        }
    }

    fn draw_indexed(
        &mut self,
        index_count: u32,
        instance_count: u32,
        start_index: u32,
        base_vertex: i32,
        start_instance: u32,
    ) {
        #[cfg(target_os = "windows")]
        {
            unsafe {
                self.command_list.DrawIndexedInstanced(
                    index_count,
                    instance_count,
                    start_index,
                    base_vertex,
                    start_instance,
                );
            }
        }
    }

    fn draw_indirect(&mut self, buffer: ResourceHandle, offset: u64, draw_count: u32) {
        #[cfg(target_os = "windows")]
        {
            // 4.45: Indirect draw через ExecuteIndirect
            unsafe {
                use windows::Win32::Graphics::Direct3D12::*;
                if let Some(buffer_res) = self.resource_manager.get_buffer(buffer) {
                    if let Some(resource) = buffer_res.dx12_resource {
                        // Для indirect draw нужен command signature - создаём временный или берём из кэша
                        // В полной реализации нужно создавать CommandSignature при создании pipeline
                        self.command_list.ExecuteIndirect(
                            None, // Command signature должен быть создан заранее
                            draw_count,
                            &resource,
                            offset,
                            std::ptr::null_mut(),
                            0,
                        );
                    }
                }
            }
        }
    }

    fn draw_indexed_indirect(&mut self, buffer: ResourceHandle, offset: u64, draw_count: u32) {
        #[cfg(target_os = "windows")]
        {
            // 4.45: Indirect indexed draw
            unsafe {
                use windows::Win32::Graphics::Direct3D12::*;
                if let Some(buffer_res) = self.resource_manager.get_buffer(buffer) {
                    if let Some(resource) = buffer_res.dx12_resource {
                        self.command_list.ExecuteIndirect(
                            None,
                            draw_count,
                            &resource,
                            offset,
                            std::ptr::null_mut(),
                            0,
                        );
                    }
                }
            }
        }
    }

    fn dispatch_indirect(&mut self, buffer: ResourceHandle, offset: u64) {
        #[cfg(target_os = "windows")]
        {
            // 4.45: Indirect dispatch
            unsafe {
                use windows::Win32::Graphics::Direct3D12::*;
                if let Some(buffer_res) = self.resource_manager.get_buffer(buffer) {
                    if let Some(resource) = buffer_res.dx12_resource {
                        self.command_list.ExecuteIndirect(
                            None,
                            1,
                            &resource,
                            offset,
                            std::ptr::null_mut(),
                            0,
                        );
                    }
                }
            }
        }
    }

    fn resource_barrier(&mut self, barriers: &[ResourceBarrier]) {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D12::*;

            // 4.46: Реализовать resource barriers
            unsafe {
                let mut dx12_barriers = Vec::new();

                for barrier in barriers {
                    let dx12_barrier = match barrier {
                        ResourceBarrier::Transition {
                            resource,
                            state_before,
                            state_after,
                            subresource,
                        } => {
                            let state_before_dx = self.convert_resource_state(*state_before);
                            let state_after_dx = self.convert_resource_state(*state_after);

                            // SAFETY: Transmuting D3D12_RESOURCE_TRANSITION_BARRIER to union member.
                            // This is a POD struct with no padding, standard DirectX 12 FFI pattern.
                            D3D12_RESOURCE_BARRIER {
                                Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                                Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                                Anonymous: D3D12_RESOURCE_BARRIER_0 {
                                    Transition: std::mem::transmute(
                                        D3D12_RESOURCE_TRANSITION_BARRIER {
                                            pResource: self
                                                .resource_manager
                                                .get_resource(*resource),
                                            Subresource: *subresource,
                                            StateBefore: state_before_dx,
                                            StateAfter: state_after_dx,
                                        },
                                    ),
                                },
                            }
                        }
                        ResourceBarrier::Aliasing { .. } => D3D12_RESOURCE_BARRIER {
                            Type: D3D12_RESOURCE_BARRIER_TYPE_ALIASING,
                            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                            Anonymous: std::mem::zeroed(),
                        },
                        ResourceBarrier::UnorderedAccessView { .. } => D3D12_RESOURCE_BARRIER {
                            Type: D3D12_RESOURCE_BARRIER_TYPE_UAV,
                            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
                            Anonymous: std::mem::zeroed(),
                        },
                    };

                    dx12_barriers.push(dx12_barrier);
                }

                if !dx12_barriers.is_empty() {
                    self.command_list.ResourceBarrier(&dx12_barriers);
                }
            }
        }
    }

    fn convert_resource_state(&self, state: ResourceState) -> D3D12_RESOURCE_STATES {
        use windows::Win32::Graphics::Direct3D12::*;

        match state {
            ResourceState::Undefined => D3D12_RESOURCE_STATE_COMMON,
            ResourceState::VertexBuffer => D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER,
            ResourceState::IndexBuffer => D3D12_RESOURCE_STATE_INDEX_BUFFER,
            ResourceState::ConstantBuffer => D3D12_RESOURCE_STATE_VERTEX_AND_CONSTANT_BUFFER,
            ResourceState::ShaderResource => D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
            ResourceState::RenderTarget => D3D12_RESOURCE_STATE_RENDER_TARGET,
            ResourceState::DepthStencil => D3D12_RESOURCE_STATE_DEPTH_WRITE,
            ResourceState::UnorderedAccess => D3D12_RESOURCE_STATE_UNORDERED_ACCESS,
            ResourceState::Present => D3D12_RESOURCE_STATE_PRESENT,
            ResourceState::CopySource => D3D12_RESOURCE_STATE_COPY_SOURCE,
            ResourceState::CopyDest => D3D12_RESOURCE_STATE_COPY_DEST,
            ResourceState::ResolveSource => D3D12_RESOURCE_STATE_RESOLVE_SOURCE,
            ResourceState::ResolveDest => D3D12_RESOURCE_STATE_RESOLVE_DEST,
        }
    }

    fn clear_render_target(&mut self, view: ResourceHandle, color: [f32; 4]) {
        #[cfg(target_os = "windows")]
        {
            // 4.47: Clear RTV
            unsafe {
                use windows::Win32::Graphics::Direct3D12::*;
                if let Some(texture) = self.resource_manager.get_texture(view) {
                    if let Some(rtv_handle) = texture.dx12_rtv_handle {
                        let handle = D3D12_CPU_DESCRIPTOR_HANDLE { ptr: rtv_handle };
                        self.command_list.ClearRenderTargetView(
                            handle,
                            &color,
                            0,
                            std::ptr::null(),
                        );
                    }
                }
            }
        }
    }

    fn clear_depth_stencil(
        &mut self,
        view: ResourceHandle,
        clear_depth: Option<f32>,
        clear_stencil: Option<u8>,
    ) {
        #[cfg(target_os = "windows")]
        {
            // 4.47: Clear DSV
            unsafe {
                use windows::Win32::Graphics::Direct3D12::*;
                if let Some(texture) = self.resource_manager.get_texture(view) {
                    if let Some(dsv_handle) = texture.dx12_dsv_handle {
                        let handle = D3D12_CPU_DESCRIPTOR_HANDLE { ptr: dsv_handle };

                        let mut clear_flags = D3D12_CLEAR_FLAG_NONE;
                        let depth = clear_depth.unwrap_or(1.0);
                        let stencil = clear_stencil.unwrap_or(0);

                        if clear_depth.is_some() {
                            clear_flags |= D3D12_CLEAR_FLAG_DEPTH;
                        }
                        if clear_stencil.is_some() {
                            clear_flags |= D3D12_CLEAR_FLAG_STENCIL;
                        }

                        self.command_list.ClearDepthStencilView(
                            handle,
                            clear_flags,
                            depth,
                            stencil,
                            0,
                            std::ptr::null(),
                        );
                    }
                }
            }
        }
    }

    fn insert_debug_marker(&mut self, name: &str) {
        #[cfg(target_os = "windows")]
        {
            // PIX debug markers - optional integration
            // When PIX is available, uncomment the D3D12SetMarker call
            // For now, this is a no-op that doesn't require external dependencies
            let _ = name; // Suppress unused warning
        }
    }

    fn begin_debug_group(&mut self, name: &str) {
        #[cfg(target_os = "windows")]
        {
            // Begin debug group (PIX) - optional integration
            // When PIX is available, uncomment the D3D12BeginEvent call
            // For now, this is a no-op that doesn't require external dependencies
            let _ = name; // Suppress unused warning
        }
    }

    fn end_debug_group(&mut self) {
        #[cfg(target_os = "windows")]
        {
            // End debug group (PIX) - optional integration
            // When PIX is available, uncomment the D3D12EndEvent call
            // For now, this is a no-op that doesn't require external dependencies
        }
    }

    /// 4.49: Signal fence
    pub fn signal_fence(&self, fence: &Dx12Fence, value: u64) {
        #[cfg(target_os = "windows")]
        unsafe {
            use windows::Win32::Graphics::Direct3D12::*;
            self.command_list.Signal(fence.fence(), value);
        }
    }
}

/// DX12 Command Queue
///
/// Note: This implementation uses the windows-rs COM wrappers (ID3D12CommandQueue).
/// The queue field holds a strong reference to the COM object via the windows crate's
/// automatic reference counting. This is NOT a raw pointer - it's a proper COM wrapper.
///
/// This is a stub implementation for the RHI interface. In production, proper lifecycle
/// management and queue family handling should be implemented.
pub struct Dx12CommandQueue {
    #[cfg(target_os = "windows")]
    queue: ID3D12CommandQueue,

    cmd_type: CommandListType,
}

unsafe impl Send for Dx12CommandQueue {}
unsafe impl Sync for Dx12CommandQueue {}

impl Dx12CommandQueue {
    #[cfg(target_os = "windows")]
    pub fn new(device: &ID3D12Device, cmd_type: CommandListType) -> RhiResult<Self> {
        use windows::Win32::Graphics::Direct3D12::*;

        let dx12_cmd_type = match cmd_type {
            CommandListType::Direct => D3D12_COMMAND_LIST_TYPE_DIRECT,
            CommandListType::Compute => D3D12_COMMAND_LIST_TYPE_COMPUTE,
            CommandListType::Copy => D3D12_COMMAND_LIST_TYPE_COPY,
        };

        let desc = D3D12_COMMAND_QUEUE_DESC {
            Type: dx12_cmd_type,
            Priority: 0,
            Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
            NodeMask: 0,
        };

        let queue: ID3D12CommandQueue = unsafe {
            device.CreateCommandQueue(&desc).map_err(|e| {
                RhiError::ResourceCreationFailed(format!("Failed to create command queue: {:?}", e))
            })?
        };

        Ok(Self { queue, cmd_type })
    }

    #[cfg(target_os = "windows")]
    pub fn queue(&self) -> &ID3D12CommandQueue {
        &self.queue
    }
}

impl ICommandQueue for Dx12CommandQueue {
    fn submit(
        &self,
        command_lists: &[&dyn ICommandList],
        wait_semaphores: &[Arc<dyn ISemaphore>],
        signal_semaphores: &[Arc<dyn ISemaphore>],
    ) -> RhiResult<()> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D12::*;

            // Convert command lists to DX12
            let mut dx12_lists = Vec::new();
            for cmd in command_lists {
                // Cast to Dx12CommandList and get ID3D12CommandList
                if let Some(dx12_cmd) = cmd.as_any().downcast_ref::<Dx12CommandList>() {
                    unsafe {
                        dx12_lists.push(
                            dx12_cmd
                                .command_list
                                .cast()
                                .map_err(|_| RhiError::InitializationFailed("Failed to cast to ID3D12CommandList"))?,
                        );
                    }
                }
            }

            unsafe {
                self.queue.ExecuteCommandLists(&dx12_lists);
            }

            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        Err(RhiError::Unsupported(
            "DX12 is only available on Windows".to_string(),
        ))
    }

    fn present(&self, swap_chain: &dyn crate::graphics::rhi::device::ISwapChain) -> RhiResult<()> {
        swap_chain.present()
    }

    fn signal(&self, fence: &dyn IFence, value: u64) -> RhiResult<()> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Direct3D12::*;

            // Cast to Dx12Fence and signal
            if let Some(dx12_fence) = fence.as_any().downcast_ref::<Dx12Fence>() {
                unsafe {
                    self.queue.Signal(dx12_fence.fence(), value);
                }
            }

            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        Err(RhiError::Unsupported(
            "DX12 is only available on Windows".to_string(),
        ))
    }

    fn wait(&self, fence: &dyn IFence, value: u64, timeout_ms: u32) -> RhiResult<bool> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::*;
            use windows::Win32::System::Threading::*;

            // Cast to Dx12Fence and wait
            if let Some(dx12_fence) = fence.as_any().downcast_ref::<Dx12Fence>() {
                // Create event for waiting
                let event = unsafe { CreateEventA(None, false, false, None) }.map_err(|e| {
                    RhiError::InitializationFailed(format!("Failed to create event: {:?}", e))
                })?;

                // Set event on fence completion
                unsafe {
                    dx12_fence
                        .fence()
                        .SetEventOnCompletion(value, event)
                        .map_err(|e| {
                            RhiError::ResourceCreationFailed(format!(
                                "Failed to set event: {:?}",
                                e
                            ))
                        })?;
                }

                // Wait for event with timeout
                let result = unsafe { WaitForSingleObject(event, timeout_ms) };

                return Ok(result == WAIT_OBJECT_0);
            }

            Ok(true)
        }

        #[cfg(not(target_os = "windows"))]
        Err(RhiError::Unsupported(
            "DX12 is only available on Windows".to_string(),
        ))
    }
}

/// DX12 Fence
pub struct Dx12Fence {
    #[cfg(target_os = "windows")]
    fence: ID3D12Fence,

    current_value: u64,
}

unsafe impl Send for Dx12Fence {}
unsafe impl Sync for Dx12Fence {}

impl Dx12Fence {
    #[cfg(target_os = "windows")]
    pub fn new(device: &ID3D12Device, initial_value: u64) -> RhiResult<Self> {
        use windows::Win32::Graphics::Direct3D12::*;

        let fence: ID3D12Fence = unsafe {
            device
                .CreateFence(initial_value, D3D12_FENCE_FLAG_NONE)
                .map_err(|e| {
                    RhiError::ResourceCreationFailed(format!("Failed to create fence: {:?}", e))
                })?
        };

        Ok(Self {
            fence,
            current_value: initial_value,
        })
    }
}

impl IFence for Dx12Fence {
    fn get_value(&self) -> u64 {
        #[cfg(target_os = "windows")]
        {
            unsafe { self.fence.GetCompletedValue() }
        }

        #[cfg(not(target_os = "windows"))]
        0
    }

    fn set_value(&self, _value: u64) {
        #[cfg(target_os = "windows")]
        {
            // Fence value is set when signaling via command queue
        }
    }

    fn set_event_on_completion(
        &self,
        value: u64,
    ) -> RhiResult<Arc<dyn std::any::Any + Send + Sync>> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::System::Threading::*;

            let event = unsafe { CreateEventA(None, false, false, None) }.map_err(|e| {
                RhiError::InitializationFailed(format!("Failed to create event: {:?}", e))
            })?;

            // Set event on fence completion
            unsafe {
                self.fence.SetEventOnCompletion(value, event).map_err(|e| {
                    RhiError::ResourceCreationFailed(format!(
                        "Failed to set event on completion: {:?}",
                        e
                    ))
                })?;
            }

            Ok(Arc::new(event))
        }

        #[cfg(not(target_os = "windows"))]
        Err(RhiError::Unsupported(
            "DX12 is only available on Windows".to_string(),
        ))
    }
}

/// DX12 Semaphore (uses fence internally)
pub struct Dx12Semaphore {
    #[cfg(target_os = "windows")]
    fence: ID3D12Fence,
}

unsafe impl Send for Dx12Semaphore {}
unsafe impl Sync for Dx12Semaphore {}

impl ISemaphore for Dx12Semaphore {}

// ============================================================================
// Дополнительные DX12 функции (задачи 4.50-4.58)
// ============================================================================

impl Dx12CommandQueue {
    /// 4.49: Wait for fence
    pub fn wait_for_fence(&self, fence: &Dx12Fence, value: u64, timeout_ms: u32) -> bool {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::System::Threading::*;
            unsafe {
                if fence.get_value() >= value {
                    return true;
                }

                let event = CreateEventW(None, FALSE, FALSE, None).map_err(|_| RhiError::InitializationFailed("Failed to create event"))?;

                fence
                    .set_event_on_completion(value, event)
                    .map_err(|_| RhiError::InitializationFailed("Failed to set event on completion"))?;

                let result = WaitForSingleObject(event, timeout_ms);
                let _ = CloseHandle(event);

                result == WAIT_OBJECT_0
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
}

#[cfg(target_os = "windows")]
impl Dx12CommandList {
    /// 4.50: Create RTV descriptors for swapchain back buffers
    pub unsafe fn create_rtv_for_swapchain(
        device: &ID3D12Device,
        swapchain_buffer: ID3D12Resource,
        rtv_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
        format: DXGI_FORMAT,
    ) {
        use windows::Win32::Graphics::Direct3D12::*;
        use windows::Win32::Graphics::Dxgi::Common::*;

        let view = D3D12_RENDER_TARGET_VIEW_DESC {
            Format: format,
            ViewDimension: D3D12_RTV_DIMENSION_TEXTURE2D,
            Anonymous: D3D12_RENDER_TARGET_VIEW_DESC_0 {
                Texture2D: D3D12_TEX2D_RTV {
                    MipSlice: 0,
                    PlaneSlice: 0,
                },
            },
        };

        device.CreateRenderTargetView(&swapchain_buffer, &view, rtv_descriptor);
    }

    /// 4.51: Create SRV descriptor for texture
    pub unsafe fn create_srv_for_texture(
        device: &ID3D12Device,
        texture: ID3D12Resource,
        srv_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
        format: DXGI_FORMAT,
        dimensions: D3D12_SRV_DIMENSION,
    ) {
        let view = D3D12_SHADER_RESOURCE_VIEW_DESC {
            Format: format,
            ViewDimension: dimensions,
            Shader4ComponentMapping: D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING,
            Anonymous: match dimensions {
                D3D12_SRV_DIMENSION_TEXTURE2D => {
                    D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                        Texture2D: D3D12_TEX2D_SRV {
                            MostDetailedMip: 0,
                            MipLevels: 0, // All levels
                            PlaneSlice: 0,
                            ResourceMinLODClamp: 0.0,
                        },
                    }
                }
                D3D12_SRV_DIMENSION_TEXTURECUBE => D3D12_SHADER_RESOURCE_VIEW_DESC_0 {
                    TextureCube: D3D12_TEXCUBE_SRV {
                        MostDetailedMip: 0,
                        MipLevels: 0,
                        ResourceMinLODClamp: 0.0,
                    },
                },
                _ => std::mem::zeroed(),
            },
        };

        device.CreateShaderResourceView(&texture, &view, srv_descriptor);
    }

    /// 4.51: Create RTV descriptor for texture
    pub unsafe fn create_rtv_for_texture(
        device: &ID3D12Device,
        texture: ID3D12Resource,
        rtv_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
        format: DXGI_FORMAT,
    ) {
        let view = D3D12_RENDER_TARGET_VIEW_DESC {
            Format: format,
            ViewDimension: D3D12_RTV_DIMENSION_TEXTURE2D,
            Anonymous: D3D12_RENDER_TARGET_VIEW_DESC_0 {
                Texture2D: D3D12_TEX2D_RTV {
                    MipSlice: 0,
                    PlaneSlice: 0,
                },
            },
        };

        device.CreateRenderTargetView(&texture, &view, rtv_descriptor);
    }

    /// 4.51: Create DSV descriptor for texture
    pub unsafe fn create_dsv_for_texture(
        device: &ID3D12Device,
        texture: ID3D12Resource,
        dsv_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
        format: DXGI_FORMAT,
    ) {
        let view = D3D12_DEPTH_STENCIL_VIEW_DESC {
            Format: format,
            ViewDimension: D3D12_DSV_DIMENSION_TEXTURE2D,
            Flags: D3D12_DSV_FLAG_NONE,
            Anonymous: D3D12_DEPTH_STENCIL_VIEW_DESC_0 {
                Texture2D: D3D12_TEX2D_DSV { MipSlice: 0 },
            },
        };

        device.CreateDepthStencilView(&texture, &view, dsv_descriptor);
    }

    /// 4.51: Create UAV descriptor for texture
    pub unsafe fn create_uav_for_texture(
        device: &ID3D12Device,
        texture: ID3D12Resource,
        counter_resource: Option<ID3D12Resource>,
        uav_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
        format: DXGI_FORMAT,
    ) {
        let view = D3D12_UNORDERED_ACCESS_VIEW_DESC {
            Format: format,
            ViewDimension: D3D12_UAV_DIMENSION_TEXTURE2D,
            Anonymous: D3D12_UNORDERED_ACCESS_VIEW_DESC_0 {
                Texture2D: D3D12_TEX2D_UAV {
                    MipSlice: 0,
                    PlaneSlice: 0,
                },
            },
        };

        device.CreateUnorderedAccessView(
            &texture,
            counter_resource.as_ref(),
            &view,
            uav_descriptor,
        );
    }

    /// 4.52: Load shader bytecode
    pub unsafe fn load_shader(shader_bytecode: &[u8]) -> D3D12_SHADER_BYTECODE {
        D3D12_SHADER_BYTECODE {
            pShaderBytecode: shader_bytecode.as_ptr() as *const _,
            BytecodeLength: shader_bytecode.len(),
        }
    }

    /// 4.53: SPIR-V to DXIL conversion
    /// Full SPIR-V to DXIL conversion requires external compiler (dxcompiler)
    /// This function provides a placeholder that indicates the requirement
    /// Note: For production use, integrate dxcompiler-rs crate or use HLSL shaders directly
    pub fn convert_spirv_to_dxil(spirv_data: &[u8]) -> Result<Vec<u8>, String> {
        // SPIR-V to DXIL conversion requires dxcompiler or similar library
        // For production use, integrate with dxcompiler-rs or use HLSL shaders directly
        let _ = spirv_data;
        Err("SPIR-V to DXIL conversion requires dxcompiler integration. Use HLSL shaders directly or add dxcompiler-rs dependency.".to_string())
    }

    /// 4.54: Create root signature with descriptor tables
    pub unsafe fn create_root_signature(
        device: &ID3D12Device,
        num_descriptor_tables: u32,
        num_constant_buffers: u32,
        num_samplers: u32,
    ) -> RhiResult<ID3D12RootSignature> {
        use windows::Win32::Graphics::Dx12::{
            D3D12SerializeRootSignature, D3D_ROOT_SIGNATURE_VERSION_1,
        };

        let mut root_params = Vec::new();

        // Descriptor tables для SRVs
        for i in 0..num_descriptor_tables {
            // Create descriptor range for SRV
            let range = D3D12_DESCRIPTOR_RANGE {
                RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
                NumDescriptors: 1,
                BaseShaderRegister: i,
                RegisterSpace: 0,
                OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
            };

            root_params.push(D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE,
                Anonymous: D3D12_ROOT_PARAMETER_0 {
                    DescriptorTable: D3D12_ROOT_DESCRIPTOR_TABLE {
                        NumDescriptorRanges: 1,
                        pDescriptorRanges: &range,
                    },
                },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_PIXEL,
            });
        }

        // Constant buffers
        for i in 0..num_constant_buffers {
            // SAFETY: Transmuting D3D12_ROOT_DESCRIPTOR to union member.
            // This is a POD struct with no padding, standard DirectX 12 FFI pattern.
            root_params.push(D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_CBV,
                Anonymous: D3D12_ROOT_PARAMETER_0 {
                    Descriptor: std::mem::transmute(D3D12_ROOT_DESCRIPTOR {
                        ShaderRegister: i,
                        RegisterSpace: 0,
                    }),
                },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
            });
        }

        // Samplers
        for i in 0..num_samplers {
            // SAFETY: Transmuting D3D12_ROOT_DESCRIPTOR to union member.
            // This is a POD struct with no padding, standard DirectX 12 FFI pattern.
            root_params.push(D3D12_ROOT_PARAMETER {
                ParameterType: D3D12_ROOT_PARAMETER_TYPE_SAMPLER,
                Anonymous: D3D12_ROOT_PARAMETER_0 {
                    Descriptor: std::mem::transmute(D3D12_ROOT_DESCRIPTOR {
                        ShaderRegister: i,
                        RegisterSpace: 0,
                    }),
                },
                ShaderVisibility: D3D12_SHADER_VISIBILITY_ALL,
            });
        }

        let root_sig_desc = D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: root_params.len() as u32,
            pParameters: root_params.as_ptr(),
            NumStaticSamplers: 0,
            pStaticSamplers: std::ptr::null(),
            Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
        };

        let mut serialized_sig = std::mem::zeroed();

        D3D12SerializeRootSignature(
            &root_sig_desc,
            D3D_ROOT_SIGNATURE_VERSION_1,
            &mut serialized_sig,
        )
        .map_err(|e| {
            RhiError::ResourceCreationFailed(format!("Failed to serialize root signature: {:?}", e))
        })?;

        let mut root_signature: Option<ID3D12RootSignature> = None;

        device
            .CreateRootSignature(
                0,
                serialized_sig.GetBufferPointer(),
                serialized_sig.GetBufferSize(),
                &mut root_signature,
            )
            .map_err(|e| {
                RhiError::ResourceCreationFailed(format!(
                    "Failed to create root signature: {:?}",
                    e
                ))
            })?;

        Ok(root_signature.ok_or(RhiError::InitializationFailed("Root signature creation failed"))?)
    }

    /// 4.55: Upload buffer data via upload heap
    pub unsafe fn upload_buffer_data(
        destination_buffer: ID3D12Resource,
        source_data: &[u8],
    ) -> RhiResult<()> {
        let mut mapped_ptr = std::ptr::null_mut();
        let range = D3D12_RANGE {
            Begin: 0,
            End: source_data.len(),
        };

        destination_buffer
            .Map(0, Some(&range), &mut mapped_ptr)
            .map_err(|e| {
                RhiError::ResourceCreationFailed(format!("Failed to map buffer: {:?}", e))
            })?;

        std::ptr::copy_nonoverlapping(
            source_data.as_ptr(),
            mapped_ptr as *mut u8,
            source_data.len(),
        );

        destination_buffer.Unmap(0, None);

        Ok(())
    }

    /// 4.56: Map buffer
    pub unsafe fn map_buffer(
        buffer: ID3D12Resource,
        offset: usize,
        size: usize,
    ) -> RhiResult<*mut u8> {
        let mut mapped_ptr = std::ptr::null_mut();
        let range = D3D12_RANGE {
            Begin: offset,
            End: offset + size,
        };

        buffer
            .Map(0, Some(&range), &mut mapped_ptr)
            .map(|_| mapped_ptr as *mut u8)
            .map_err(|e| RhiError::ResourceCreationFailed(format!("Failed to map buffer: {:?}", e)))
    }

    /// 4.56: Unmap buffer
    pub unsafe fn unmap_buffer(buffer: ID3D12Resource, offset: usize) {
        let range = D3D12_RANGE {
            Begin: offset,
            End: offset,
        };
        buffer.Unmap(0, Some(&range));
    }

    /// 4.57: Destroy resource (reference counting via COM)
    pub fn destroy_resource(_resource: ID3D12Resource) {
        // В DX12 ресурсы используют COM reference counting
        // При последнем drop ресурс автоматически уничтожается
        // Просто дропаем handle
    }

    /// 4.58: Copy data to texture via command list
    pub unsafe fn copy_to_texture(
        command_list: &ID3D12GraphicsCommandList,
        destination: ID3D12Resource,
        source: ID3D12Resource,
        width: u32,
        height: u32,
        depth: u32,
    ) {
        // Вычисляем row pitch для формата R8G8B8A8_UNORM (4 байта на пиксель)
        // В реальном использовании формат должен передаваться или получаться из дескриптора текстуры
        const BYTES_PER_PIXEL: u32 = 4;
        let row_pitch = width * BYTES_PER_PIXEL;

        // Выравниваем row pitch по 256 байт (требование DX12)
        let aligned_row_pitch = ((row_pitch + 255) / 256) * 256;

        let src_location = D3D12_PLACED_SUBRESOURCE_FOOTPRINT {
            Footprint: D3D12_SUBRESOURCE_FOOTPRINT {
                Format: DXGI_FORMAT_R8G8B8A8_UNORM, // Стандартный формат для текстур
                Width: width,
                Height: height,
                Depth: depth,
                RowPitch: aligned_row_pitch,
            },
            Offset: 0,
        };

        command_list.CopyTextureRegion(
            &D3D12_TEXTURE_COPY_LOCATION {
                pResource: Some(destination),
                Type: D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    SubresourceIndex: 0,
                },
            },
            0,
            0,
            0,
            &D3D12_TEXTURE_COPY_LOCATION {
                pResource: Some(source),
                Type: D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT,
                Anonymous: D3D12_TEXTURE_COPY_LOCATION_0 {
                    PlacedFootprint: src_location,
                },
            },
            &D3D12_BOX {
                left: 0,
                top: 0,
                front: 0,
                right: width,
                bottom: height,
                back: depth,
            },
        );
    }
}
