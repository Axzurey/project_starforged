use std::sync::Arc;

#[derive(Clone)]
pub struct Renderctx {
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>
}

impl Renderctx {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device, queue
        }
    }
}