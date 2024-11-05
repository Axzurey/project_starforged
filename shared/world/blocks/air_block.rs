use nalgebra::Vector3;

use crate::world::block::Block;

pub struct AirBlock {
    relative_position: Vector3<u32>,
    orientation: u8,
    light: [u8; 4]
}

impl AirBlock {
    pub fn new(relative_position: Vector3<u32>) -> Self {
        Self {
            relative_position,
            light: [0, 0, 0, 0],
            orientation: 0
        }
    }
}

impl Block for AirBlock {
    fn get_relative_position(&self) -> Vector3<u32> {
        self.relative_position
    }

    fn get_orientation(&self) -> u8 {
        self.orientation
    }

    fn has_partial_transparency(&self) -> bool {
        true
    }

    fn include_in_greedy(&self) -> bool {
        false
    }

    fn does_not_render(&self) -> bool {
        true
    }

    fn get_surface_texture_indices(&self, face: crate::world::block::BlockFace) -> (crate::world::block::BlockFaceTextureConfiguration, crate::world::block::BlockFaceTextureConfiguration, crate::world::block::BlockFaceTextureConfiguration) {
        todo!()
    }

    fn is_fluid(&self) -> bool {
        false
    }

    fn set_sunlight_intensity(&mut self, intensity: u8) {
        self.light[3] = intensity;
    }

    fn set_light(&mut self, with_color: [u8; 4]) {
        self.light = with_color;
    }

    fn get_light(&self) -> &[u8; 4] {
        &self.light
    }

    fn get_sunlight_intensity(&self) -> u8 {
        self.light[3]
    }
}