use std::collections::HashMap;

use cached::proc_macro::cached;
use nalgebra::Vector2;
use stopwatch::Stopwatch;

use crate::world::worldgen::{generate_surface_height, is_cave};

use super::block::BlockType;

#[cached]
pub fn local_xyz_to_index(x: u32, y: u32, z: u32) -> u32 {
    ((z * 16 * 16) + (y * 16) + x) as u32
}

#[cached]   
pub fn xz_to_index(x: i32, z: i32) -> u32 {
    let x0 = if x >= 0 {2 * x} else {-2 * x - 1}; //converting integers to natural numbers
    let z0 = if z >= 0 {2 * z} else {-2 * z - 1};

    (0.5 * (x0 + z0) as f32 * (x0 + z0 + 1) as f32 + z0 as f32) as u32 //cantor pairing https://math.stackexchange.com/questions/3003672/convert-infinite-2d-plane-integer-coords-to-1d-number
}

pub type ChunkGridType = Vec<Vec<BlockType>>;

pub struct Chunk {
    position: Vector2<i32>,
    grid: ChunkGridType
}

impl Chunk {
    pub fn new(position: Vector2<i32>, noisegen: Perlin, extra_blocks: &mut HashMap<u32, Vec<BlockType>>) -> Self {
        let t = Stopwatch::start_new();

        let iter_layers = (0..16).into_iter();

        let mut extra_blocks_same: Vec<BlockType> = Vec::new();

        let mut blocks = iter_layers.map(|y_slice| {
            let mut out: Vec<BlockType> = Vec::with_capacity(4096);

            let uninit = out.spare_capacity_mut();

            for x in 0..16 {
                for z in 0..16 {
                    let abs_x = ((x as i32) + position.x * 16) as i32;
                    let abs_z = ((z as i32) + position.y * 16) as i32;

                    let floor_level = generate_surface_height(noisegen, abs_x, abs_z);
                    
                    for y in 0..16 {
                        let abs_y = (y + y_slice as u32 * 16) as i32;
                        let is_cave = is_cave(noisegen, abs_x, abs_y, abs_z);
                        let block: BlockType =
                        if is_cave {
                            Box::new(AirBlock::new(
                                Vector3::new(x, y as u32, z), 
                                Vector3::new(abs_x, abs_y, abs_z))
                            )
                        }
                        else if abs_y == floor_level && abs_y < 100 {
                            Box::new(GrassBlock::new(
                                Vector3::new(x, y as u32, z), 
                                Vector3::new(abs_x, abs_y, abs_z))
                            )
                        }
                        else if abs_y + 3 < floor_level || (abs_y == floor_level && abs_y >= 100) {
                            Box::new(StoneBlock::new(
                                Vector3::new(x, y as u32, z), 
                                Vector3::new(abs_x, abs_y, abs_z))
                            )
                        }
                        else if abs_y < floor_level {
                            if abs_y < 100 {
                                Box::new(DirtBlock::new(
                                    Vector3::new(x, y as u32, z), 
                                    Vector3::new(abs_x, abs_y, abs_z))
                                )
                            }
                            else {
                                Box::new(StoneBlock::new(
                                    Vector3::new(x, y as u32, z), 
                                    Vector3::new(abs_x, abs_y, abs_z))
                                )
                            }
                        }
                        else {
                            Box::new(AirBlock::new(
                                Vector3::new(x, y as u32, z), 
                                Vector3::new(abs_x, abs_y, abs_z))
                            )
                        };

                        if abs_y == floor_level + 1 {
                            let should_tree = density_map_plane(noisegen, abs_x, abs_z);

                            if should_tree {
                                let mut blocks = get_blocks_for_structure_at_point("tree", 0, Vector3::new(abs_x, abs_y, abs_z));

                                loop {

                                    let nblock = blocks.pop();

                                    if nblock.is_none() {break}

                                    let block = nblock.unwrap();

                                    let abs_dived = block.get_absolute_position().map(|v| {
                                        v.div_euclid(16)
                                    });
                                    
                                    if abs_dived.x != position.x || abs_dived.y != position.y {
                                        let xz = xz_to_index(abs_dived.x, abs_dived.z);
                                        if extra_blocks.contains_key(&xz) {
                                            let mutlist = extra_blocks.get_mut(&xz).unwrap();

                                            mutlist.push(block);

                                        }
                                        else {
                                            let list = vec![block];
                                            extra_blocks.insert(xz, list);
                                        }
                                    }
                                }

                                extra_blocks_same.extend(blocks);

                            }
                        }

                        uninit[local_xyz_to_index(x, y as u32, z) as usize].write(block);
                    }
                }
            }

            unsafe { out.set_len(4096) };

            out
        }).collect::<Vec<Vec<BlockType>>>();

        let k = xz_to_index(position.x, position.y);

        if extra_blocks.contains_key(&k) {
            let new_blocks = extra_blocks.remove(&k).unwrap();

            for block in new_blocks {
                if block.get_block() == Blocks::AIR {continue};

                let p = block.get_absolute_position();

                if p.x.div_euclid(16) == position.x && p.z.div_euclid(16) == position.y {
                    let rel = block.get_relative_position();
                    blocks[p.y.div_euclid(16) as usize][local_xyz_to_index(rel.x, rel.y, rel.z) as usize] = block;
                }
            }
        }

        for block in extra_blocks_same {
            if block.get_block() == Blocks::AIR {continue};

            let p = block.get_absolute_position();

            if p.x.div_euclid(16) == position.x && p.z.div_euclid(16) == position.y {
                let rel = block.get_relative_position();
                blocks[p.y.div_euclid(16) as usize][local_xyz_to_index(rel.x, rel.y, rel.z) as usize] = block;
            }
        }

        

        println!("Took {}ms to generate chunk", t.elapsed_ms());

        Self {
            position,
            grid: blocks,
        }
    }

    pub fn set_slice_vertex_buffers(&mut self, device: &wgpu::Device) {
        let slice_vertex_buffers = (0..16).map(|y| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("Chunk Data Buffer")),
                contents: bytemuck::cast_slice(&[ChunkDataVertex {
                    position_sliced: [self.position.x, y, self.position.y]
                }]),
                usage: wgpu::BufferUsages::VERTEX,
            })
        }).collect::<Vec<wgpu::Buffer>>();
        self.slice_vertex_buffers = slice_vertex_buffers;
    }

    pub fn get_block_at(&self, x: u32, y: u32, z: u32) -> &BlockType {
        &self.grid[(y / 16) as usize][local_xyz_to_index(x % 16, y % 16, z % 16) as usize]
    }

    pub fn get_surface_block_y(&self, x: u32, z: u32) -> u32 {
        for y in (1..=255).rev() {
            let ys = y / 16;
            
            let block = &self.grid[ys as usize][local_xyz_to_index(x, y % 16, z) as usize];

            if !block.has_partial_transparency() {
                return y;
            }
        }
        0
    }

    pub fn modify_block_at<F>(&mut self, x: u32, y: u32, z: u32, mut callback: F) where F: FnMut(&mut BlockType) {
        callback(&mut self.grid[(y / 16) as usize][local_xyz_to_index(x % 16, y % 16, z % 16) as usize]);
    }
}