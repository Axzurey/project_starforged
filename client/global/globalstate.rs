use crate::{view::camera::Camera, world::chunkmanager::ChunkManager};

use super::inputservice::InputService;

pub struct GlobalState {
    pub chunk_manager: ChunkManager,
    pub camera: Camera,
    pub input_service: InputService
}