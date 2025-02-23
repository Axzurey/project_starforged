use std::{collections::HashMap, sync::Arc};

use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};

use super::{blockrepr::WorldBlock, chunk::{index_to_local_xyz, local_xyz_to_index, Chunk}, constructblock::construct_block};

#[derive(Serialize, Deserialize, Debug)]
pub struct CompressedChunk {
    position: Vector2<i32>,
    slices: Vec<HashMap<WorldBlock, Vec<u16>>>
}

pub fn compress_chunk(chunk: &Chunk) -> CompressedChunk {
    let mut lists: Vec<HashMap<WorldBlock, Vec<u16>>> = std::iter::repeat_with(|| HashMap::new()).take(16).collect();
    
    for (i, plane) in chunk.grid.iter().enumerate() {
        let position_list = &mut lists[i];

        for x in 0..16 {
            for z in 0..16 {
                for y in 0..16 {
                    let local_index = local_xyz_to_index(x, y, z) as usize;
                    let block = &plane[local_index];

                    position_list.entry(block.clone())
                        .or_insert_with(Vec::new)
                        .push(local_index as u16);
                }
            }
        }
    }

    CompressedChunk {
        position: chunk.position,
        slices: lists
    }
}
pub fn decompress_chunk(chunkc: CompressedChunk) -> Arc<Chunk> {
    let grid = chunkc.slices.into_iter().enumerate().map(|(slice_index, slice)| {
        let mut vec: Vec<WorldBlock> = Vec::with_capacity(4096);
        vec.resize_with(4096, || WorldBlock::Air(0));

        for (block_type, positions) in slice {
            for local_pos in positions {
                let (x, y, z) = index_to_local_xyz(local_pos as u32);
                let lmap = Vector3::new(x as i32, y as i32, z as i32);
                let abs = Vector3::new(
                    chunkc.position.x * 16 + lmap.x, 
                    slice_index as i32 * 16 + lmap.y, 
                    chunkc.position.y * 16 + lmap.z
                );

                let b = construct_block(&block_type, abs);
                vec[local_xyz_to_index(x, y, z) as usize] = b;
            }
        }

        vec
    }).collect();

    Arc::new(Chunk::from_blocks(chunkc.position, grid))
}