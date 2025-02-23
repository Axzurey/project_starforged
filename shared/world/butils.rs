use super::blockrepr::WorldBlock;

pub enum UnsignedNumbers {
    U8(u8)
}

pub fn perform_op_on_block<F>(block: &mut WorldBlock, conditionu8: F) -> UnsignedNumbers
 where F: Fn(u8) -> u8 {
    match block {
        WorldBlock::Air(x) | WorldBlock::Dirt(x) | WorldBlock::Grass(x)
        | WorldBlock::Stone(x) | WorldBlock::Sand(x) => {
            let updated_value = conditionu8(*x);
            *x = updated_value;
            UnsignedNumbers::U8(updated_value)
        },
    }
}

pub fn get_on_block<F>(block: &WorldBlock, conditionu8: F) -> UnsignedNumbers
 where F: Fn(u8) -> u8 {
    match block {
        WorldBlock::Air(x) | WorldBlock::Dirt(x) | WorldBlock::Grass(x)
        | WorldBlock::Stone(x) | WorldBlock::Sand(x) => {
            let newvalue = conditionu8(*x);
            UnsignedNumbers::U8(newvalue)
        },
    }
}