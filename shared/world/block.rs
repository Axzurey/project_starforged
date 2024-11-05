use nalgebra::Vector3;

pub type BlockType = Box<dyn Block + Send + Sync>;

pub enum BlockFace {
    Right,
    Left,
    Top,
    Bottom,
    Front,
    Back
}

pub enum BlockFaceTextureConfiguration {
    Static(usize),
    Dynamic(usize, [u8; 4])
}

pub trait Block {
    fn get_relative_position(&self) -> Vector3<u32>;
    fn get_orientation(&self) -> u8;

    fn get_rotation_vector(&self) -> Vector3<i8> {
        let orientation = self.get_orientation();
        match orientation {
            0 => Vector3::new(1, 0, 0),
            1 => Vector3::new(-1, 0, 0),
            2 => Vector3::new(0, 1, 0),
            3 => Vector3::new(0, -1, 0),
            4 => Vector3::new(0, 0, 1),
            5 => Vector3::new(0, 0, -1),
            _ => panic!("{}", format!("Why is the orientation value {}. That shouldn't happen", orientation))
        }
    }

    fn has_partial_transparency(&self) -> bool;

    fn include_in_greedy(&self) -> bool;
    fn does_not_render(&self) -> bool {
        false
    }

    fn get_surface_texture_indices(&self, face: BlockFace) -> (BlockFaceTextureConfiguration, BlockFaceTextureConfiguration, BlockFaceTextureConfiguration);

    fn is_fluid(&self) -> bool;

    fn set_sunlight_intensity(&mut self, intensity: u8);
    //r, g, b, sun
    fn set_light(&mut self, with_color: [u8; 4]);
    //r, g, b, sun
    fn get_light(&self) -> &[u8; 4];
    fn get_sunlight_intensity(&self) -> u8;

    
}