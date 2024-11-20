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
    let mut i = 0;
    for plane in &chunk.grid {
        let position_list = lists.get_mut(i).unwrap();
        
        for block in plane {
            if position_list.contains_key(block) {
                let lp = block.get_absolute_position().map(|v| v.rem_euclid(16) as u32);
                position_list.get_mut(block).unwrap().push(local_xyz_to_index(lp.x, lp.y, lp.z) as u16);
            }
            else {
                let lp = block.get_absolute_position().map(|v| v.rem_euclid(16) as u32);
                let v = vec![local_xyz_to_index(lp.x, lp.y, lp.z) as u16];
                position_list.insert(block.clone(), v);
            }
        }
        println!("{}", position_list.len());
        i += 1;
    }

    CompressedChunk {
        position: chunk.position,
        slices: lists
    }
}

pub fn decompress_chunk(chunkc: CompressedChunk) -> Arc<Chunk> {
    let mut i = 0;
    let grid = chunkc.slices.into_iter().map(|slice| {
        let mut vec: Vec<BlockType> = Vec::with_capacity(4096);
        vec.resize_with(4096, || Box::new(AirBlock::new(Vector3::new(0, 0, 0))));

        slice.into_iter().for_each(|(k, v)| {
            for local_pos in v {
                let l = index_to_local_xyz(local_pos as u32);
                let lmap = Vector3::new(l.0 as i32, l.1 as i32, l.2 as i32);
                let abs = Vector3::new(chunkc.position.x, i, chunkc.position.y) * 16 + lmap;

                let b = construct_block(k.get_block(), abs);
                vec.insert(local_xyz_to_index(lmap.x as u32, lmap.y as u32, lmap.z as u32) as usize, b);
            }
        });
        i += 1;
        vec
    }).collect();

    Arc::new(Chunk::from_blocks(chunkc.position, grid))
}