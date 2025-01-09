use std::{collections::HashMap, sync::Arc};

use shared::world::chunk::{xz_to_index, Chunk};

use super::chunkdraw::ChunkDraw;

pub struct ChunkManager {
    pub chunks: HashMap<u32, ChunkDraw>
}

impl ChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new()
        }
    }
}