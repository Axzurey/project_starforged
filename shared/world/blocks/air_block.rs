use nalgebra::Vector3;
use wgpu::naga::Block;

pub struct AirBlock {
    relative_position: Vector3<u32>,
    light_intensity: [u8; 4]
}

impl AirBlock {
    pub fn new(relative_position: Vector3<u32>) -> Self {
        Self {
            relative_position,
            light_intensity: [0, 0, 0, 0]
        }
    }
}

impl Block for AirBlock {
    
}