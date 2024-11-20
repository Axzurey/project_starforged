use std::hash::Hash;

use nalgebra::Vector3;
use serde::{Deserialize, Serialize};

use super::constructblock::construct_block;

pub type BlockType = Box<dyn Block>;

#[derive(PartialEq, Eq, Debug, Deserialize, Clone, Copy, Serialize, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Blocks {
    AIR = 0,
    DIRT = 1,
    GRASS = 2,
}

unsafe impl Send for Blocks {}
unsafe impl Sync for Blocks {}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum BlockFace {
    Top = 0,
    Bottom = 1,
    Right = 2,
    Left = 3,
    Front = 4,
    Back = 5,
}

impl BlockFace {
    pub fn normal_index(&self) -> u32 {
        match self {
            BlockFace::Left => 0u32,
            BlockFace::Right => 1u32,
            BlockFace::Bottom => 2u32,
            BlockFace::Top => 3u32,
            BlockFace::Front => 4u32,
            BlockFace::Back => 5u32,
        }
    }

    pub fn normal(&self) -> Vector3<i32> {
        match self {
            BlockFace::Left => Vector3::new(-1, 0, 0),
            BlockFace::Right => Vector3::new(1, 0, 0),
            BlockFace::Bottom => Vector3::new(0, -1, 0),
            BlockFace::Top => Vector3::new(0, 1, 0),
            BlockFace::Front => Vector3::new(0, 0, -1),
            BlockFace::Back => Vector3::new(0, 0, 1),
        }
    }

    pub fn world_to_sample(&self, axis: i32, x: i32, y: i32) -> [u32; 3] {
        match self {
            BlockFace::Top => [x as u32, axis as u32 + 1, y as u32],
            BlockFace::Bottom => [x as u32, axis as u32, y as u32],
            BlockFace::Left => [axis as u32, y as u32, x as u32],
            BlockFace::Right => [axis as u32 + 1, y as u32, x as u32],
            BlockFace::Front => [x as u32, y as u32, axis as u32],
            BlockFace::Back => [x as u32, y as u32, axis as u32 + 1],
        }
    }

    pub fn reverse_order(&self) -> bool {
        match self {
            BlockFace::Top => true,      //+1
            BlockFace::Bottom => false,   //-1
            BlockFace::Left => false,   //-1
            BlockFace::Right => true,   //+1
            BlockFace::Front => true, //-1
            BlockFace::Back => false,   //+1
        }
    }
}

impl From<usize> for BlockFace {
    fn from(value: usize) -> Self {
        match value {
            0 => BlockFace::Top,
            1 => BlockFace::Bottom,
            2 => BlockFace::Right,
            3 => BlockFace::Left,
            4 => BlockFace::Front,
            5 => BlockFace::Back,
            _ => panic!("Value {} is invalid", value)
        }
    }
}

#[derive(Clone, Copy)]
pub enum FaceTexture {
    //static index
    Static(usize),
    //static index + color/alpha
    //Dynamic(usize, [u8; 4])
    //better idea^ use some sort of reactive texture that is coded into the shader, rather than sending all that data.
}

impl Default for FaceTexture {
    fn default() -> Self {
        FaceTexture::Static(0)
    }
}

impl Hash for dyn Block {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.calc_hash());
        state.finish();
    }
}

impl PartialEq for dyn Block {
    fn eq(&self, other: &Self) -> bool {
        self.calc_hash() == other.calc_hash()
    }
}

impl Clone for BlockType {
    fn clone(&self) -> Self {
        construct_block(self.get_block(), self.get_absolute_position())
    }
    
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}

impl Eq for dyn Block {}
#[typetag::serde(tag="type")]
pub trait Block: Send + Sync {
    fn calc_hash(&self) -> u64 {
        let mut i = self.get_block() as u64;
        
        let l = self.get_light();
        i += (l[0] as u64) >> 16;
        i +=  (l[1] as u64) >> 24;
        i +=  (l[2] as u64) >> 32;
        i +=  (l[3] as u64) >> 40;
        //48 bits
        i
    }
    fn get_block(&self) -> Blocks;

    fn get_absolute_position(&self) -> Vector3<i32>;
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

    fn get_surface_texture_indices(&self, face: BlockFace) -> (FaceTexture, FaceTexture, FaceTexture);

    fn is_fluid(&self) -> bool;

    fn set_sunlight_intensity(&mut self, intensity: u8);
    //r, g, b, sun
    fn set_light(&mut self, with_color: [u8; 4]);
    //r, g, b, sun
    fn get_light(&self) -> &[u8; 4];
    fn get_sunlight_intensity(&self) -> u8;
}