use std::{collections::HashMap, sync::Arc};
use nalgebra::Vector3;
use wgpu::util::DeviceExt;

use crate::renderer::surfacevertex::SurfaceVertex;

#[derive(Clone, Copy)]
pub struct Quad {
    pub center: Vector3<f32>,
    pub vertices: [SurfaceVertex; 4]
}

pub fn sort_chunk_transparent_quads(device: &Arc<wgpu::Device>, camera: &Camera, chunka: &mut Arc<Chunk>, chunkbuff: &mut ChunkBuffers, i: usize) {
    let chunk = Arc::make_mut(chunka);
    let camera_pos = Vector3::new(camera.position.x, camera.position.y, camera.position.z);
    
    if chunk.states[i] != ChunkState::Ready { return };
        
    chunk.transparent_quads[i].sort_by(|a, b| {
        let dista = a.center.distance(camera_pos);
        let distb = b.center.distance(camera_pos);

        dista.partial_cmp(&distb).unwrap()
    });

    let vertices = chunk.transparent_quads[i].clone().iter().flat_map(|v| {
        v.vertices
    }).collect::<Vec<_>>();

    let indices = generate_indices(vertices.len());

    let ilen = indices.len();

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("Transparent Quad Vertex Buffer")),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("Transparent Quad Index Buffer")),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    
    chunkbuff.set_transparent_buffer(i as u32, (vertex_buffer, index_buffer, ilen as u32));
}