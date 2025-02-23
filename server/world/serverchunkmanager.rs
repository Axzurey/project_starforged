use std::{collections::{HashMap, VecDeque}, hash::Hash, sync::Arc, thread::sleep, time::Duration};

use nalgebra::{Vector2, Vector3};
use noise::{OpenSimplex, Perlin};
use shared::world::{blockrepr::{get_block_light, has_partial_transparency, is_unbreakable, set_block_light, WorldBlock}, chunk::{get_block_at_absolute, local_xyz_to_index, xz_to_index, Chunk}};

pub struct ServerChunkManager {
    pub chunks: HashMap<u32, Chunk>
}

impl ServerChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new()
        }
    }

    pub fn generate_range_inclusive(&mut self, start_x: i32, start_z: i32, end_x: i32, end_z: i32) {
        let noisegen = OpenSimplex::new(52223);
        for x in start_x..=end_x {
            for z in start_z..=end_z {
                let chunk = Chunk::new(Vector2::new(x, z), noisegen, &mut HashMap::new());
                self.chunks.insert(xz_to_index(x, z), chunk);
            }
        }

        for x in start_x..=end_x {
            for z in start_z..=end_z {
                self.calculate_initial_lighting(Vector2::new(x, z));
            }
        }
    }

    pub fn break_block(&mut self, x: i32, z: i32, y: u32) {
        let xr = x.rem_euclid(16) as u32;
        let zr = z.rem_euclid(16) as u32;
        let chunk = self.chunks.get_mut(&xz_to_index(x.div_euclid(16), z.div_euclid(16))).unwrap();
        let block = chunk.get_block_at(xr as u32, y, zr as u32);

        if is_unbreakable(block) {
            return;
        }

        chunk.grid[y as usize / 16][local_xyz_to_index(xr, y % 16, zr) as usize] = WorldBlock::Air(0);
    }
    pub fn place_block(&mut self, x: i32, z: i32, y: u32, toplace: WorldBlock) {
        let xr = x.rem_euclid(16) as u32;
        let zr = z.rem_euclid(16) as u32;
        let chunk = self.chunks.get_mut(&xz_to_index(x.div_euclid(16), z.div_euclid(16))).unwrap();
        let block = chunk.get_block_at(xr as u32, y, zr as u32);

        if is_unbreakable(block) {
            return;
        }

        chunk.grid[y as usize / 16][local_xyz_to_index(xr, y % 16, zr) as usize] = toplace;
    }

    pub fn calculate_initial_lighting(&mut self, chunk_pos: Vector2<i32>) {
        //ray cast each vertical column of blocks to find the first solid block
        for x in 0..16 {
            for z in 0..16 {
                for y in (0..=255).rev() {
                    let chunk = self.chunks.get_mut(&xz_to_index(chunk_pos.x, chunk_pos.y)).unwrap();
                    let block = chunk.get_block_at_mut(x, y, z);
                    if has_partial_transparency(block) {
                        set_block_light(block, 15);
                        continue;
                    }
                    //now we've hit a solid block and can start flood filling
                    let abs_x = chunk_pos.x * 16 + x as i32;
                    let abs_z = chunk_pos.y * 16 + z as i32;
                    let abs_y = y as i32;

                    let mut queue: VecDeque<Vector3<i32>> = VecDeque::new();

                    set_block_light(block, 15);

                    queue.push_back(Vector3::new(abs_x, abs_y, abs_z));

                    while queue.len() > 0 {
                        let current_pos = queue.pop_front().unwrap();

                        let chunk = self.chunks.get_mut(&xz_to_index(current_pos.x.div_euclid(16), current_pos.z.div_euclid(16))).unwrap();

                        let localspace = current_pos.map(|v| v.rem_euclid(16) as u32);

                        let block = chunk.get_block_at_mut(localspace.x, localspace.y, localspace.z);

                        let light_intensity = get_block_light(block);

                        if light_intensity <= 1 {
                            continue;
                        }

                        //check all the neighbors, and add to queue if required
                        [
                            current_pos + Vector3::new(1, 0, 0),
                            current_pos + Vector3::new(-1, 0, 0),
                            current_pos + Vector3::new(0, 1, 0),
                            current_pos + Vector3::new(0, -1, 0),
                            current_pos + Vector3::new(0, 0, 1),
                            current_pos + Vector3::new(0, 0, -1)
                        ].map(|pos| {
                            if pos.y < 0 || pos.y > 255 || !self.chunks.contains_key(&xz_to_index(pos.x.div_euclid(16), pos.z.div_euclid(16))) {
                                return;
                            }
                            let chunk = self.chunks.get_mut(&xz_to_index(pos.x.div_euclid(16), pos.z.div_euclid(16))).unwrap();
                            let localspace = pos.map(|v| v.rem_euclid(16) as u32);
                            let block = chunk.get_block_at_mut(localspace.x, localspace.y, localspace.z);

                            let block_intensity = get_block_light(block);
                            

                            if block_intensity < light_intensity - 1 {
                                set_block_light(block, light_intensity - 1);
                                if has_partial_transparency(block) {
                                    queue.push_back(pos);
                                }
                            }
                        });
                    }

                    break;
                }
            }
        }
    }
}