use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use crate::world::block::Block;

#[derive(Serialize, Deserialize)]
pub struct GrassBlock {
    absolute_position: Vector3<i32>,
    orientation: u8,
    light: [u8; 4]
}

impl GrassBlock {
    pub fn new(absolute_position: Vector3<i32>) -> Self {
        Self {
            absolute_position,
            light: [0, 0, 0, 0],
            orientation: 0
        }
    }
}
#[typetag::serde]
impl Block for GrassBlock {
    fn get_block(&self) -> crate::world::block::Blocks {
        crate::world::block::Blocks::GRASS
    }
    fn get_absolute_position(&self) -> Vector3<i32> {
        self.absolute_position
    }

    fn get_orientation(&self) -> u8 {
        self.orientation
    }

    fn has_partial_transparency(&self) -> bool {
        false
    }

    fn include_in_greedy(&self) -> bool {
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