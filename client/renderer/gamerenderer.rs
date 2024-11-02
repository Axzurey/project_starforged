use std::sync::Arc;

pub struct GameRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    dims: (u32, u32),
    surface_format: wgpu::TextureFormat,
}

impl GameRenderer {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, dims: (u32, u32), surface_format: wgpu::TextureFormat, camera_bindgroup_layout: &wgpu::BindGroupLayout) -> Self {
        Self {
            device, queue, dims, surface_format
        }
    }
}