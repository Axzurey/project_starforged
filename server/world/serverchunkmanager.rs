use std::{collections::HashMap, hash::Hash, sync::Arc};

use nalgebra::Vector2;
use noise::Perlin;
use shared::world::chunk::{xz_to_index, Chunk};

pub struct ServerChunkManager {
    pub chunks: HashMap<u32, Arc<Chunk>>
}

impl ServerChunkManager {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new()
        }
    }

    pub fn generate_range_inclusive(&mut self, start_x: i32, start_z: i32, end_x: i32, end_z: i32, on_finish: &dyn Fn(Arc<Chunk>)) {
        let noisegen = Perlin::new(52223);
        for x in start_x..=end_x {
            for z in start_z..=end_z {
                let chunk = Arc::new(Chunk::new(Vector2::new(x, z), noisegen, &mut HashMap::new()));
                self.chunks.insert(xz_to_index(x, z), chunk.clone());
                on_finish(chunk);
            }
        }
    }
}