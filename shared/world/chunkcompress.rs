use std::{collections::HashMap, sync::Arc};

use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};

use super::{block::BlockType, blocks::air_block::AirBlock, chunk::{index_to_local_xyz, local_xyz_to_index, Chunk}, constructblock::construct_block};

#[derive(Serialize, Deserialize)]
pub struct CompressedChunk {
    position: Vector2<i32>,
    slices: Vec<HashMap<BlockType, Vec<u16>>>
}

pub fn compress_chunk(chunk: Arc<Chunk>) -> CompressedChunk {
    let mut lists: Vec<HashMap<BlockType, Vec<u16>>> = std::iter::repeat_with(|| HashMap::new()).take(16).collect();
    
    for (i, plane) in chunk.grid.iter().enumerate() {
        let position_list = &mut lists[i];
        
        for block in plane {
            let lp = block.get_absolute_position().map(|v| v.rem_euclid(16) as u32);
            let local_index = local_xyz_to_index(lp.x, lp.y, lp.z) as u16;
            
            position_list.entry(block.clone())
                .or_insert_with(Vec::new)
                .push(local_index);
        }
    }

    CompressedChunk {
        position: chunk.position,
        slices: lists
    }
}
pub fn decompress_chunk(chunkc: CompressedChunk) -> Arc<Chunk> {
    let grid = chunkc.slices.into_iter().enumerate().map(|(slice_index, slice)| {
        let mut vec: Vec<BlockType> = Vec::with_capacity(4096);
        vec.resize_with(4096, || Box::new(AirBlock::new(Vector3::new(0, 0, 0))));

        for (block_type, positions) in slice {
            for local_pos in positions {
                let (x, y, z) = index_to_local_xyz(local_pos as u32);
                let lmap = Vector3::new(x as i32, y as i32, z as i32);
                let abs = Vector3::new(
                    chunkc.position.x * 16 + lmap.x, 
                    slice_index as i32 * 16 + lmap.y, 
                    chunkc.position.y * 16 + lmap.z
                );

                let b = construct_block(block_type.get_block(), abs);
                vec[local_xyz_to_index(x, y, z) as usize] = b;
            }
        }

        vec
    }).collect();

    Arc::new(Chunk::from_blocks(chunkc.position, grid))
}