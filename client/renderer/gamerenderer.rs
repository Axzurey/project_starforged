use std::sync::Arc;

use shared::world::chunk::ChunkState;
use stopwatch::Stopwatch;
use wgpu::TextureFormat;

use crate::{loaders::texture::Texture, view::camera::Camera, world::{chunkdraw::ChunkDataVertex, chunkmanager::ChunkManager}};

use super::{pipelines::surface_pipeline::create_surface_pipeline, surfacevertex::SurfaceVertex, vertex::Vertex};

pub struct GameRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    dims: (u32, u32),
    surface_format: wgpu::TextureFormat,
    surface_pipeline: wgpu::RenderPipeline,
    depth_texture: Texture
}

impl GameRenderer {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, dims: (u32, u32), surface_format: wgpu::TextureFormat, camera_bindgroup_layout: &wgpu::BindGroupLayout, texture_bindgroup_layout: &wgpu::BindGroupLayout) -> Self {
        
        let material_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Main renderer material bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable:  true }
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                }
            ]
        });

        let global_bindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("global bindgroup layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ]
        });

        let surface_pipeline = create_surface_pipeline(&device, surface_format, texture_bindgroup_layout, camera_bindgroup_layout, &[SurfaceVertex::desc(), ChunkDataVertex::desc()], Some(TextureFormat::Depth32Float));

        let depth_texture = Texture::from_empty("depth texture", &device, wgpu::TextureFormat::Depth32Float, dims.0, dims.1, wgpu::FilterMode::Linear);
        Self {
            device, queue, dims, surface_format, surface_pipeline, depth_texture
        }
    }

    pub fn render_surface(&mut self, 
        device: &Arc<wgpu::Device>,
        queue: &Arc<wgpu::Queue>,
        output_texture: &mut wgpu::SurfaceTexture, 
        output_view: &wgpu::TextureView, 
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
        texture_bindgroup: &wgpu::BindGroup,
        chunk_manager: &ChunkManager
    ) {
        //if (workspace.chunk_manager.chunks.len() as u32) < (workspace.chunk_manager.render_distance * 2 + 1).pow(2) {return}
        let t = Stopwatch::start_new();
        let camera_bindgroup = &camera.bindgroup;

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("object render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment { 
                view: &output_view, 
                resolve_target: None, 
                ops: wgpu::Operations { 
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0
                    }),
                    store: wgpu::StoreOp::Store
                }
            })],
            depth_stencil_attachment: Some(
                wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(
                        wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store
                        }
                    ),
                    stencil_ops: None,
                }
            ),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&self.surface_pipeline);
        render_pass.set_bind_group(0, texture_bindgroup, &[]);
        render_pass.set_bind_group(1, camera_bindgroup, &[]);


        let mut outeri = 0;
        for (index, chunkref) in chunk_manager.chunks.iter() {
            let chunk = chunk_manager.chunks.get(index).unwrap();
            let out = chunk.get_solid_buffers();

            let mut i = 0;

            for t in out {
                if chunk.states[i] != ChunkState::Ready {i += 1; continue};
                let (vertex_buffer, index_buffer, ilen) = t.as_ref().unwrap();
                if *ilen == 0 {
                    i += 1;
                    continue;
                };
                
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, chunk.slice_vertex_buffers[i].slice(..));
                render_pass.draw_indexed(0..*ilen, 0, 0..1);

                i += 1;
            }
            outeri += 1;
        }
        drop(render_pass);

        println!("frame render: {}ms", t.elapsed_ms());
    }
}