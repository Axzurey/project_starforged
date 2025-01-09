use std::{collections::HashMap, sync::Arc};

use cached::proc_macro::cached;
use nalgebra::{Vector2, Vector3};
use noise::{OpenSimplex, Perlin, Seedable};
use serde::{Deserialize, Serialize};
use stopwatch::Stopwatch;

use crate::world::{blockrepr::WorldBlock, worldgen::{generate_surface_height, get_biome, get_gen_config, is_cave}};

use super::blockrepr::has_partial_transparency;

pub fn get_block_at_absolute(x: i32, y: i32, z: i32, chunks: &HashMap<u32, Arc<Chunk>>) -> Option<&WorldBlock> {
    if y < 0 || y > 255 {return None};
    let chunk_x = x.div_euclid(16);
    let chunk_z = z.div_euclid(16);

    chunks.get(&xz_to_index(chunk_x, chunk_z)).map(|v| v.get_block_at(x.rem_euclid(16) as u32, y as u32, z.rem_euclid(16) as u32))
}

#[cached]
pub fn local_xyz_to_index(x: u32, y: u32, z: u32) -> u32 {
    ((y * 16 * 16) + (z * 16) + x) as u32
}
pub fn index_to_local_xyz(index: u32) -> (u32, u32, u32) {
    let x = index % 16;
    let z = (index / 16) % 16;
    let y = (index / 256) % 16;
    (x, y, z)
}

#[cached]   
pub fn xz_to_index(x: i32, z: i32) -> u32 {
    let x0 = if x >= 0 {2 * x} else {-2 * x - 1}; //converting integers to natural numbers
    let z0 = if z >= 0 {2 * z} else {-2 * z - 1};

    (0.5 * (x0 + z0) as f32 * (x0 + z0 + 1) as f32 + z0 as f32) as u32 //cantor pairing https://math.stackexchange.com/questions/3003672/convert-infinite-2d-plane-integer-coords-to-1d-number
}

pub type ChunkGridType = Vec<Vec<WorldBlock>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct Chunk {
    pub position: Vector2<i32>,
    pub grid: ChunkGridType,
    pub fullair: [bool; 16]
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChunkState {
    PreMesh,
    Mesh,
    Ready
}

impl Chunk {
    pub fn from_blocks(position: Vector2<i32>, grid: ChunkGridType) -> Self {
        let mut fullair = [true; 16];
        for x in 0..16 {
            for z in 0..16 {
                for y_slice in 0..16 {
                    for y in 0..16 {
                        match grid[y_slice / 16][local_xyz_to_index(x, y, z) as usize] {
                            WorldBlock::Air(_) => {},
                            _ => {
                                fullair[y_slice] = false;
                            }
                        }
                        if !fullair[y_slice] {
                            break;
                        }
                    }
                }
            }
        }
        Self {
            position,
            grid,
            fullair
        }
    }
    pub fn new(position: Vector2<i32>, noisegen: OpenSimplex, extra_blocks: &mut HashMap<u32, Vec<WorldBlock>>) -> Self {
        let t = Stopwatch::start_new();

        let iter_layers = (0..16).into_iter();

        let mut extra_blocks_same: Vec<WorldBlock> = Vec::new();

        let gencfg = get_gen_config(noisegen.seed());

        let mut blocks = iter_layers.map(|y_slice| {
            let mut out: Vec<WorldBlock> = Vec::with_capacity(4096);

            let uninit = out.spare_capacity_mut();

            for x in 0..16 {
                for z in 0..16 {
                    let abs_x = ((x as i32) + position.x * 16) as i32;
                    let abs_z = ((z as i32) + position.y * 16) as i32;

                    let floor_level = generate_surface_height(noisegen, abs_x, abs_z, &gencfg);

                    let (biome, biomegen) = get_biome(noisegen, abs_x, abs_z, &gencfg);
                    
                    for y in 0..16 {
                        let abs_y = (y + y_slice as u32 * 16) as i32;
                        let is_cave = is_cave(noisegen, abs_x, abs_y, abs_z);
                        let block: WorldBlock =
                        if abs_y > floor_level || is_cave {
                            WorldBlock::Air(0)
                        }
                        else if abs_y == floor_level {
                            biomegen.make_surface_block(Vector3::new(abs_x, abs_y, abs_z))
                        }
                        else if abs_y >= floor_level - 3 {
                            biomegen.make_subsurface_block(Vector3::new(abs_x, abs_y, abs_z))
                        }
                        else if abs_y < floor_level {
                            biomegen.make_earth_block(Vector3::new(abs_x, abs_y, abs_z))
                        }
                        else {
                            WorldBlock::Air(0)
                        };

                        if abs_y == floor_level + 1 {
                            // let should_tree = density_map_plane(noisegen, abs_x, abs_z);

                            // if should_tree {
                            //     let mut blocks = get_blocks_for_structure_at_point("tree", 0, Vector3::new(abs_x, abs_y, abs_z));

                            //     loop {

                            //         let nblock = blocks.pop();

                            //         if nblock.is_none() {break}

                            //         let block = nblock.unwrap();

                            //         let abs_dived = block.get_absolute_position().map(|v| {
                            //             v.div_euclid(16)
                            //         });
                                    
                            //         if abs_dived.x != position.x || abs_dived.y != position.y {
                            //             let xz = xz_to_index(abs_dived.x, abs_dived.z);
                            //             if extra_blocks.contains_key(&xz) {
                            //                 let mutlist = extra_blocks.get_mut(&xz).unwrap();

                            //                 mutlist.push(block);

                            //             }
                            //             else {
                            //                 let list = vec![block];
                            //                 extra_blocks.insert(xz, list);
                            //             }
                            //         }
                            //     }

                            //     extra_blocks_same.extend(blocks);

                            // }
                        }

                        uninit[local_xyz_to_index(x, y as u32, z) as usize].write(block);
                    }
                }
            }

            unsafe { out.set_len(4096) };

            out
        }).collect::<Vec<Vec<WorldBlock>>>();

        let k = xz_to_index(position.x, position.y);

        // if extra_blocks.contains_key(&k) {
        //     let new_blocks = extra_blocks.remove(&k).unwrap();

        //     for block in new_blocks {
        //         if block.get_block() == Blocks::AIR {continue};

        //         let p = block.get_absolute_position();

        //         if p.x.div_euclid(16) == position.x && p.z.div_euclid(16) == position.y {
        //             let rel = block.get_relative_position();
        //             blocks[p.y.div_euclid(16) as usize][local_xyz_to_index(rel.x, rel.y, rel.z) as usize] = block;
        //         }
        //     }
        // }

        // for block in extra_blocks_same {
        //     if block.get_block() == Blocks::AIR {continue};

        //     let p = block.get_absolute_position();

        //     if p.x.div_euclid(16) == position.x && p.z.div_euclid(16) == position.y {
        //         let rel = block.get_relative_position();
        //         blocks[p.y.div_euclid(16) as usize][local_xyz_to_index(rel.x, rel.y, rel.z) as usize] = block;
        //     }
        // }

        

        println!("Took {}ms to generate chunk", t.elapsed_ms());

        Self {
            position,
            grid: blocks,
            fullair: [false; 16]
        }
    }

    // pub fn set_slice_vertex_buffers(&mut self, device: &wgpu::Device) {
    //     let slice_vertex_buffers = (0..16).map(|y| {
    //         device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
    //             label: Some(&format!("Chunk Data Buffer")),
    //             contents: bytemuck::cast_slice(&[ChunkDataVertex {
    //                 position_sliced: [self.position.x, y, self.position.y]
    //             }]),
    //             usage: wgpu::BufferUsages::VERTEX,
    //         })
    //     }).collect::<Vec<wgpu::Buffer>>();
    //     self.slice_vertex_buffers = slice_vertex_buffers;
    // }

    pub fn get_block_at(&self, x: u32, y: u32, z: u32) -> &WorldBlock {
        &self.grid[(y / 16) as usize][local_xyz_to_index(x % 16, y % 16, z % 16) as usize]
    }

    pub fn get_surface_block_y(&self, x: u32, z: u32) -> u32 {
        for y in (1..=255).rev() {
            let ys = y / 16;
            
            let block = &self.grid[ys as usize][local_xyz_to_index(x, y % 16, z) as usize];

            if !has_partial_transparency(block) {
                return y;
            }
        }
        0
    }

    pub fn modify_block_at<F>(&mut self, x: u32, y: u32, z: u32, mut callback: F) where F: FnMut(&mut WorldBlock) {
        callback(&mut self.grid[(y / 16) as usize][local_xyz_to_index(x % 16, y % 16, z % 16) as usize]);
    }
}