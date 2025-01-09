use std::{collections::{HashMap, VecDeque}, sync::Arc};

use nalgebra::Vector3;
use shared::world::{block::BlockFace, blockrepr::{does_not_render, get_block_id, get_surface_texture_indices, has_partial_transparency, is_fluid, WorldBlock}, chunk::{get_block_at_absolute, xz_to_index, Chunk}};

use crate::renderer::surfacevertex::{calculate_illumination_bytes, SurfaceVertex};

use super::{binarymesher::MeshStageType};

//this was ai generated!
pub fn fast_mesh(
    chunk_x: i32,
    chunk_z: i32,
    y_slice: u32,
    chunks: &HashMap<u32, Arc<Chunk>>,
    stage: MeshStageType,
) -> (Vec<SurfaceVertex>, Vec<u32>, u32) {
    let chunk = chunks.get(&xz_to_index(chunk_x, chunk_z)).unwrap();
    if chunk.fullair[y_slice as usize] {
        println!("SKIP");
        return (Vec::new(), Vec::new(), 0);
    }
    let mut vertices = Vec::with_capacity(16 * 16 * 16 * 24);
    let mut indices = Vec::with_capacity(16 * 16 * 16 * 36);

    let rel_abs_x = chunk_x * 16;
    let rel_abs_z = chunk_z * 16;

    let y_start = 16 * y_slice;

    let mut cached_blocks = [None; 18 * 18 * 18];

    for x in 0..=17 {
        for z in 0..=17 {
            for y in 0..=17 {
                let block = get_block_at_absolute(rel_abs_x + x - 1, y_start as i32 + y - 1, rel_abs_z + z - 1, chunks);
                cached_blocks[((z * 18 * 18) + (y * 18) + x) as usize] = block;
            }
        }
    }

    let get_block_at = |x: i32, y: i32, z: i32| -> Option<&WorldBlock> {
        cached_blocks[(((z + 1) * 18 * 18) + ((y + 1) * 18) + (x + 1)) as usize]
    };
    
    for x in 0..16 {
        for z in 0..16 {

            for y in 0..16 {

                let block_at = get_block_at(x as i32, y as i32, z as i32).unwrap();

                if does_not_render(block_at) {
                    continue;
                }
                
                let neighbors = [
                    get_block_at(x as i32, y as i32, z as i32 + 1),
                    get_block_at(x as i32, y as i32, z as i32 - 1),
                    get_block_at(x as i32 + 1, y as i32, z as i32),
                    get_block_at(x as i32 - 1, y as i32, z as i32),
                    get_block_at(x as i32, y as i32 + 1, z as i32),
                    get_block_at(x as i32, y as i32 - 1, z as i32),
                ];

                let faces = [
                    BlockFace::Front,
                    BlockFace::Back,
                    BlockFace::Right,
                    BlockFace::Left,
                    BlockFace::Top,
                    BlockFace::Bottom,
                ];

                let is_transparent = has_partial_transparency(block_at);

                if is_transparent {
                    continue;
                }

                for (i, neighbor_block) in neighbors.iter().enumerate() {
                    let neighbor = if neighbor_block.is_some() {
                        neighbor_block.unwrap()
                    } else {continue;};
                    
                    if has_partial_transparency(neighbor) {
                        let current_l = vertices.len();

                        let face = faces[i];

                        let (face_vertices, face_indices) = match face {
                            BlockFace::Front => (
                                [
                                    [x, y, z + 1],
                                    [x + 1, y, z + 1],
                                    [x, y + 1, z + 1],
                                    [x + 1, y + 1, z + 1],
                                ],
                                [0, 1, 2, 1, 3, 2],
                            ),
                            BlockFace::Back => (
                                [
                                    [x, y, z],
                                    [x + 1, y, z],
                                    [x, y + 1, z],
                                    [x + 1, y + 1, z],
                                ],
                                [2, 1, 0, 2, 3, 1],
                            ),
                            BlockFace::Right => (
                                [
                                    [x + 1, y, z],
                                    [x + 1, y, z + 1],
                                    [x + 1, y + 1, z],
                                    [x + 1, y + 1, z + 1],
                                ],
                                [2, 1, 0, 2, 3, 1],
                            ),
                            BlockFace::Left => (
                                [
                                    [x, y, z],
                                    [x, y, z + 1],
                                    [x, y + 1, z],
                                    [x, y + 1, z + 1],
                                ],
                                [0, 1, 2, 1, 3, 2],
                            ),
                            BlockFace::Top => (
                                [
                                    [x, y + 1, z],
                                    [x, y + 1, z + 1],
                                    [x + 1, y + 1, z],
                                    [x + 1, y + 1, z + 1],
                                ],
                                [0, 1, 2, 1, 3, 2],
                            ),
                            BlockFace::Bottom => (
                                [
                                    [x, y, z],
                                    [x, y, z + 1],
                                    [x + 1, y, z],
                                    [x + 1, y, z + 1],
                                ],
                                [2, 1, 0, 2, 3, 1],
                            ),
                        };

                        let illumination = calculate_illumination_bytes(neighbor);
                        indices.extend(face_indices.iter().map(|&index| (index + current_l) as u32));
                        for (j, &pos) in face_vertices.iter().enumerate() {
                            vertices.push(SurfaceVertex::from_position(
                                pos, face, j as u32, get_surface_texture_indices(block_at, face), illumination
                            ));
                        }
                    }
                }
            }
        }
    }
    
    let l = indices.len();
    (vertices, indices, l as u32)
}