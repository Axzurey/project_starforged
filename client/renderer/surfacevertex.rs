use std::{mem, ops::BitOrAssign};

use shared::world::{block::FaceTexture, blockrepr::{get_block_light, WorldBlock}};
use wgpu::vertex_attr_array;

use crate::shared_world::block::BlockFace;

use super::vertex::Vertex;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, PartialEq, Eq, Hash)]
pub struct SurfaceVertex {
    pub d0: u32,
    pub d1: u32,
    pub illumination: u32,
}

impl SurfaceVertex {
    pub fn from_position(pos: [u32; 3], face: BlockFace, nth: u32, texture_indices: (FaceTexture, FaceTexture, FaceTexture), illumination: u32) -> SurfaceVertex {
        let face_dir = match face {
            BlockFace::Top => 0,
            BlockFace::Bottom => 1,
            BlockFace::Right => 2,
            BlockFace::Left => 3,
            BlockFace::Front => 4,
            BlockFace::Back => 5,
        };

        // 15 bits for pos
        // 3 bits for direction
        // 2 bits for normal
        // 4 bits for width
        // 4 bits for height

        let mut d0 = 0;
        let mut d1 = 0;

        d0.bitor_assign(pos[0]);
        d0.bitor_assign(pos[1] << 5);
        d0.bitor_assign(pos[2] << 10);
        d0.bitor_assign(face_dir << 15);
        d0.bitor_assign(nth << 18);

        match texture_indices.0 {
            FaceTexture::Static(v) => d1.bitor_assign(v as u32),
            //FaceTexture::Dynamic(v, _) => d1.bitor_assign(1),
        }

        match texture_indices.1 {
            FaceTexture::Static(v) => d1.bitor_assign((v as u32) << 8),
            //FaceTexture::Dynamic(v, _) => d1.bitor_assign(1 << 1),
        }

        match texture_indices.2 {
            FaceTexture::Static(v) => d1.bitor_assign((v as u32) << 16),
            //FaceTexture::Dynamic(v, _) => {},
        }

        //we're going to have reactive textures soon, rather than baking color information into the vertex
    
        SurfaceVertex {
            d0, d1, illumination
        }
    }
}

impl Vertex for SurfaceVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SurfaceVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[u32; 1]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[u32; 2]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint32,
                }
            ]
        }
    }
}

pub fn calculate_illumination_bytes(block: &WorldBlock) -> u32 {
    let mut val: u32 = 0;
    
    //TODO: actually calculate sunlight intensity.
    let sunlight = get_block_light(block);
    let light = [0, 0, 0]; //this will be removed

    //sunlight: 4 bits

    val.bitor_assign(light[0] as u32);
    val.bitor_assign((light[1] as u32) << 8);
    val.bitor_assign((light[2] as u32) << 16);
    val.bitor_assign((sunlight as u32) << 24);

    val
}