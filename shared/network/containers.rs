use std::sync::Arc;

use nalgebra::Vector2;

use crate::world::chunk::Chunk;

pub enum ServerToClientMessage {
    ChunkAdded((Vector2<i32>, Arc<Chunk>))
}