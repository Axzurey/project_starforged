use std::{collections::HashMap, sync::Arc};

use shared::world::chunk::Chunk;
use stopwatch::Stopwatch;

use crate::{renderer::surfacevertex::SurfaceVertex, world::fastmesher::fast_mesh};

use super::{binarymesher::{binary_mesh, MeshStageType}, depthsort::Quad};

pub fn mesh_slice_arrayed(chunk_x: i32, chunk_z: i32, y_slice: u32, chunks: &HashMap<u32, Arc<Chunk>>) -> ((Vec<SurfaceVertex>, Vec<u32>, u32), (Vec<SurfaceVertex>, Vec<u32>, u32, Vec<Quad>)) {
    let t = Stopwatch::start_new();
    let solidmesh = fast_mesh(chunk_x, chunk_z, y_slice, chunks, MeshStageType::Solid);
    let tz = fast_mesh(chunk_x, chunk_z, y_slice, chunks, MeshStageType::Transparent);
    let b = (
        (solidmesh.0, solidmesh.1, solidmesh.2),
        (tz.0, tz.1, tz.2, Vec::new())
    );
    println!("MESH {}ms", t.elapsed_ms());
    b
}