use std::{collections::HashMap, sync::Arc};

use shared::world::chunk::Chunk;

use crate::renderer::surfacevertex::SurfaceVertex;

use super::{binarymesher::{binary_mesh, MeshStageType}, depthsort::Quad};

pub fn mesh_slice_arrayed(chunk_x: i32, chunk_z: i32, y_slice: u32, chunks: &HashMap<u32, Arc<Chunk>>) -> ((Vec<SurfaceVertex>, Vec<u32>, u32), (Vec<SurfaceVertex>, Vec<u32>, u32, Vec<Quad>)) {
    let solidmesh = binary_mesh(chunk_x, chunk_z, y_slice, chunks, MeshStageType::Solid);
    (
        (solidmesh.0, solidmesh.1, solidmesh.2),
        binary_mesh(chunk_x, chunk_z, y_slice, chunks, MeshStageType::Transparent)
    )
}