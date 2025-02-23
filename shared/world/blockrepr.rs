//it has been decided that blocks will be stateless until they need state. This means that there will not be classes for each block
//each block will also store its orientation and other data as the following
//u8 -> 5bits for orientation, 4 bits for lighting

//r-primary axis(lower 3 bits): 0: up, 1: down, 2: right, 3: left, 4: front, 5: back (where up vector ends up)
//r-secondary axis(upper 2 bits): 0: up, 1: down, 2: right, 3: left (where the front vector ends up)

use nalgebra::{Matrix3, Vector3};
use serde::{Deserialize, Serialize};

use crate::loaders::texture_loader::get_indices_from_texture;

use super::{block::{BlockFace, FaceTexture}, butils::{get_on_block, perform_op_on_block, UnsignedNumbers}};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Copy, Debug)]
pub enum WorldBlock {
    Air(u8),
    Dirt(u8),
    Grass(u8),
    Stone(u8),
    Sand(u8)
}

pub fn calculate_block_rotation(store: u8, face: BlockFace) -> BlockFace {
    let rotation = store & 0b11100000; //take the first 3 bits for primary orientation axis
    let secondary = (store) & 0b11; //take the 4th and 5th bits for secondary orientation axis TODO: FIRST OF ALL ARE YOU HIGH? THAT'S 9 BITS TOTAL FOR A U8? SECONDLY, WHY EVEN USE THIS?

    let uptarget = match rotation {
        0 => Vector3::new(0, 1, 0),
        1 => Vector3::new(0, -1, 0),
        2 => Vector3::new(1, 0, 0),
        3 => Vector3::new(-1, 0, 0),
        4 => Vector3::new(0, 0, 1),
        5 => Vector3::new(0, 0, -1),
        _ => panic!("Not a valid rotation")
    };

    let forwardtarget = match rotation {
        0 => match secondary {
            0 => Vector3::new(1, 0, 0),
            1 => Vector3::new(-1, 0, 0),
            2 => Vector3::new(0, 0, 1),
            3 => Vector3::new(0, 0, -1),
            _ => panic!("Not a valid rotation sec.")
        },
        1 => match secondary {
            0 => Vector3::new(1, 0, 0),
            1 => Vector3::new(-1, 0, 0),
            2 => Vector3::new(0, 0, 1),
            3 => Vector3::new(0, 0, -1),
            _ => panic!("Not a valid rotation sec.")
        },
        2 => match secondary {
            0 => Vector3::new(0, 1, 0),
            1 => Vector3::new(0, -1, 0),
            2 => Vector3::new(0, 0, 1),
            3 => Vector3::new(0, 0, -1),
            _ => panic!("Not a valid rotation sec.")
        },
        3 => match secondary {
            0 => Vector3::new(0, 1, 0),
            1 => Vector3::new(0, -1, 0),
            2 => Vector3::new(0, 0, 1),
            3 => Vector3::new(0, 0, -1),
            _ => panic!("Not a valid rotation sec.")
        },
        4 => match secondary {
            0 => Vector3::new(0, 1, 0),
            1 => Vector3::new(0, -1, 0),
            2 => Vector3::new(1, 0, 0),
            3 => Vector3::new(-1, 0, 0),
            _ => panic!("Not a valid rotation sec.")
        },
        5 => match secondary {
            0 => Vector3::new(0, 1, 0),
            1 => Vector3::new(0, -1, 0),
            2 => Vector3::new(1, 0, 0),
            3 => Vector3::new(-1, 0, 0),
            _ => panic!("Not a valid rotation sec.")
        },
        _ => panic!("Not a valid rotation")
    };

    let righttarget = uptarget.cross(&forwardtarget);

    let transform = Matrix3::from_columns(&[righttarget, uptarget, forwardtarget]);

    let new_vector = transform * match face {
        BlockFace::Left => Vector3::new(-1, 0, 0),
        BlockFace::Right => Vector3::new(1, 0, 0),
        BlockFace::Bottom => Vector3::new(0, -1, 0),
        BlockFace::Top => Vector3::new(0, 1, 0),
        BlockFace::Front => Vector3::new(0, 0, 1),
        BlockFace::Back => Vector3::new(0, 0, -1),
    };

    let newface = if new_vector.y == 1 {BlockFace::Top} else if new_vector.y == -1 {BlockFace::Bottom} 
        else if new_vector.x == 1 {BlockFace::Right} else if new_vector.x == -1 {BlockFace::Left}
        else if new_vector.z == 1 {BlockFace::Front} else if new_vector.z == -1 {BlockFace::Back}
        else {panic!("Unable to determine face of vector ${}", new_vector)};

    newface
}

pub fn set_block_light(block: &mut WorldBlock, light: u8) {
    perform_op_on_block(block, |currentval| {
        //x here is the block's data. This returns the new lighting value to update
        currentval & 0b11110000 | light
    });
}

pub fn get_block_light(block: &WorldBlock) -> u8 {
    let out = get_on_block(block, |currentval| {
        currentval
    });
    
    match out {
        UnsignedNumbers::U8(x) => x & 0b00001111,
    }
}

pub fn get_surface_texture_indices(block: &WorldBlock, face: BlockFace) -> (FaceTexture, FaceTexture, FaceTexture) {
    match block {
        //these shouldn't render at all actually
        WorldBlock::Air(_) => (0.into(), 0.into(), 0.into()),
        //these do render
        WorldBlock::Dirt(r) => {
            (get_indices_from_texture("dirt").into(), 0.into(), 0.into())
        },
        WorldBlock::Grass(r) => {
            (match calculate_block_rotation(*r, face) {
                BlockFace::Top => get_indices_from_texture("grass-top"),
                _ => get_indices_from_texture("grass-side")
            }.into(), 0.into(), 0.into())
        },
        WorldBlock::Stone(r) => {
            (get_indices_from_texture("stone").into(), 0.into(), 0.into())
        },
        WorldBlock::Sand(r) => {
            (get_indices_from_texture("sand").into(), 0.into(), 0.into())
        }
    }
}

pub fn has_partial_transparency(block: &WorldBlock) -> bool {
    //just add to the first arm to add more ofc because we only want to bother with those which are transparent
    match block {
        WorldBlock::Air(_) => true,
        _ => false
    }
}

pub fn is_unbreakable(block: &WorldBlock) -> bool {
    match block {
        WorldBlock::Air(_) => true,
        _ => false
    }
}

pub fn does_not_render(block: &WorldBlock) -> bool {
    match block {
        WorldBlock::Air(_) => true,
        _ => false
    }
}
pub fn is_fluid(block: &WorldBlock) -> bool {
    match block {
        _ => false
    }
}
pub fn get_block_id(block: &WorldBlock) -> u64 {
    match block {
        WorldBlock::Air(x) => 0,
        WorldBlock::Dirt(x) => 1,
        WorldBlock::Grass(x) => 2,
        WorldBlock::Stone(x) => 3,
        WorldBlock::Sand(x) => 4,
    }
}