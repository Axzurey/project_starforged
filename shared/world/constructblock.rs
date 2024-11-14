use std::{collections::HashMap, sync::{Arc, RwLock}};

use nalgebra::Vector3;
use once_cell::sync::Lazy;

use super::{block::{BlockType, Blocks}, blocks::air_block::AirBlock};

static GLOBAL_DATA: Lazy<Arc<RwLock<HashMap<Blocks, Box<dyn Fn(Vector3<i32>) -> BlockType + Send + Sync>>>>> = Lazy::new(|| {
    let m: HashMap<Blocks, Box<dyn Fn(Vector3<i32>) -> BlockType + Send + Sync>> = HashMap::from([
        (Blocks::AIR, Box::new(|pos| Box::new(AirBlock::new(pos)))),
    ]);
    Arc::new(RwLock::new(m))
});

pub fn construct_block(blocktype: BlockType) {

}