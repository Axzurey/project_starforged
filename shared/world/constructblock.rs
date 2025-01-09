use nalgebra::Vector3;

use super::blockrepr::WorldBlock;


pub fn construct_block(blocktype: &WorldBlock, absolute_position: Vector3<i32>) -> WorldBlock {
    match blocktype {
        WorldBlock::Air(x) => WorldBlock::Air(*x),
        WorldBlock::Dirt(x) => WorldBlock::Dirt(*x),
        WorldBlock::Grass(x) => WorldBlock::Grass(*x),
        WorldBlock::Stone(x) => WorldBlock::Stone(*x),
        WorldBlock::Sand(x) => WorldBlock::Sand(*x),
        //add separate handling for blocks that require special handling and don't do anything too silly.
    }
}