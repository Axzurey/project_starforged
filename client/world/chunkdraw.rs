use std::{iter, mem, sync::Arc};

use shared::world::chunk::{Chunk, ChunkState};
use wgpu::util::DeviceExt;

use crate::renderer::vertex::Vertex;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkDataVertex {
    pub position_sliced: [i32; 3],
}

impl Vertex for ChunkDataVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ChunkDataVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Sint32x3,
                }
            ],
        }
    }
}
pub struct ChunkDraw {
    pub chunk: Arc<Chunk>,
    pub solid_buffers: Vec<Option<(wgpu::Buffer, wgpu::Buffer, u32)>>,
    pub transparent_buffers: Vec<Option<(wgpu::Buffer, wgpu::Buffer, u32)>>,
    pub slice_vertex_buffers: Vec<wgpu::Buffer>,
    pub states: Vec<ChunkState>
}

impl ChunkDraw {
    pub fn new(chunk: Arc<Chunk>) -> Self {
        Self {
            chunk,
            solid_buffers: iter::repeat_with(|| None).take(16).collect(),
            transparent_buffers: iter::repeat_with(|| None).take(16).collect(),
            slice_vertex_buffers: Vec::new(),
            states: iter::repeat(ChunkState::PreMesh).take(16).collect()
        }
    }
    pub fn set_slice_vertex_buffers(&mut self, device: &Arc<wgpu::Device>) {
        let slice_vertex_buffers = (0..16).map(|y| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Chunk Data Buffer")),
                contents: bytemuck::cast_slice(&[ChunkDataVertex {
                    position_sliced: [self.chunk.position.x, y, self.chunk.position.y]
                }]),
                usage: wgpu::BufferUsages::VERTEX,
            })
        }).collect::<Vec<wgpu::Buffer>>();
        self.slice_vertex_buffers = slice_vertex_buffers;
    }
    pub fn set_solid_buffer(&mut self, slice: u32, buffers: (wgpu::Buffer, wgpu::Buffer, u32)) {
        self.solid_buffers[slice as usize] = Some(buffers);
    }
    pub fn set_solid_buffers(&mut self, buffers: Vec<Option<(wgpu::Buffer, wgpu::Buffer, u32)>>) {
        self.solid_buffers = buffers;
    }
    pub fn get_solid_buffer(&self, slice: u32) -> Option<&(wgpu::Buffer, wgpu::Buffer, u32)> {
        self.solid_buffers[slice as usize].as_ref()
    }
    pub fn get_solid_buffers(&self) -> &Vec<Option<(wgpu::Buffer, wgpu::Buffer, u32)>> {
        &self.solid_buffers
    }
    pub fn set_transparent_buffer(&mut self, slice: u32, buffers: (wgpu::Buffer, wgpu::Buffer, u32)) {
        self.transparent_buffers[slice as usize] = Some(buffers);
    }
    pub fn set_transparent_buffers(&mut self, buffers: Vec<Option< (wgpu::Buffer, wgpu::Buffer, u32)>>) {
        self.transparent_buffers = buffers;
    }
    pub fn get_transparent_buffer(&self, slice: u32) -> Option<&(wgpu::Buffer, wgpu::Buffer, u32)> {
        self.transparent_buffers[slice as usize].as_ref()
    }
    pub fn get_transparent_buffers(&self) -> &Vec<Option<(wgpu::Buffer, wgpu::Buffer, u32)>> {
        &self.transparent_buffers
    }
}