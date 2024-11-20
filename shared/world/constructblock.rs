use nalgebra::Vector3;
use super::{block::{BlockType, Blocks}, blocks::{air_block::AirBlock, grass_block::GrassBlock}};


pub fn construct_block(blocktype: Blocks, absolute_position: Vector3<i32>) -> BlockType {
    match blocktype {
        Blocks::AIR => Box::new(AirBlock::new(absolute_position)),
        Blocks::GRASS => Box::new(GrassBlock::new(absolute_position)),
        Blocks::DIRT => todo!()
    }
}