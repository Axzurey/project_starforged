use nalgebra::Vector3;

use super::blockrepr::WorldBlock;

/**
 * 1bit: isleaf,
 * 8bits: child mask
 * 23bits: first child index
 * 32 bits: whatever else
 */
pub struct OctreeNode {
    data: u64
}

pub struct VoxelOctree {
    nodes: Vec<OctreeNode>
}

impl VoxelOctree {
    fn get_child(&self, node_index: usize, child_num: u8) -> Option<usize> {
        
    }

    fn insert_voxel(&mut self, position: Vector3<i32>, voxel_data: u32) {
        let mut current_index = 0;

        for depth in (0..3).rev() {
            
        }
    }
}