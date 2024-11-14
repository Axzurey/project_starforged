use std::{collections::HashMap, sync::Arc};

use nalgebra::Vector3;

use super::{block::BlockType, chunk::Chunk};

pub fn compress_chunk(chunk: Arc<Chunk>) {
    let mut position_list: HashMap<BlockType, Vec<Vector3<i32>>> = HashMap::new();

    for plane in &chunk.grid {
        for block in plane {
            if position_list.contains_key(block) {
                position_list.get_mut(block).unwrap().push(block.get_absolute_position());
            }
            else {
                let v = vec![block.get_absolute_position()];
                position_list.insert(block, v);
            }
        }
    }
}