use std::{collections::HashMap, sync::Arc};

use shared::world::{block::BlockType, chunk::{xz_to_index, Chunk}};

pub fn get_block_at_absolute(x: i32, y: i32, z: i32, chunks: &HashMap<u32, Arc<Chunk>>) -> Option<&BlockType> {
    if y < 0 || y > 255 {return None};
    let chunk_x = x.div_euclid(16);
    let chunk_z = z.div_euclid(16);

    chunks.get(&xz_to_index(chunk_x, chunk_z)).map(|v| v.get_block_at(x.rem_euclid(16) as u32, y as u32, z.rem_euclid(16) as u32))
}

pub struct ChunkManager {
    chunks: HashMap<u32, Arc<Chunk>>
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new()
        }
    }
}