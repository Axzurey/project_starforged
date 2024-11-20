use crate::{view::camera::Camera, world::chunkmanager::ChunkManager};

pub struct GlobalState {
    pub chunk_manager: ChunkManager,
    pub camera: Camera
}