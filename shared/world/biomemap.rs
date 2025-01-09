use std::{collections::HashMap, env, fs::File, io::BufReader};

use nalgebra::Vector3;
use once_cell::sync::Lazy;
use serde::Deserialize;

use super::blockrepr::WorldBlock;

#[derive(PartialEq, Eq, Hash, strum_macros::Display, Clone, Copy)]
pub enum Biome {
    Plains,
    Mountains,
    Desert,
    Woodlands,
    SnowyPlains,
    HauntedWoodlands,
    Lake
}

#[derive(Deserialize, Clone)]
pub struct BiomeWeights {
    pub continentalness: f32,
    pub humidity: f32,
    pub arcanity: f32,
    pub peaks: f32,
    pub temperature: f32
}

#[derive(Deserialize, Clone)]
pub struct BiomeData {
    //pub tree_density: f32,
    pub weights: BiomeWeights,
}

unsafe impl Send for Biome {}
unsafe impl Sync for Biome {}

pub trait BiomeGenerator: Send + Sync {
    fn make_surface_block(&self, position: Vector3<i32>) -> WorldBlock;
    fn make_subsurface_block(&self, position: Vector3<i32>) -> WorldBlock;
    //this will not account for ore veins and all of that. It will just be the default block at that level. (ex stone, or )
    fn make_earth_block(&self, position: Vector3<i32>) -> WorldBlock;
}

pub static BIOME_GENERATORS: Lazy<HashMap<Biome, (Box<dyn BiomeGenerator>, BiomeData)>> = Lazy::new(|| {
    let mut dir = env::current_dir().unwrap();
    dir.push(r"res/data/biome_data.json");

    let file = File::open(dir).expect("Unable to open biomes.json");
    let reader = BufReader::new(file);
    let mut data: HashMap<String, BiomeData> = serde_json::from_reader(reader).expect("Invalid biomes.json data");
    for (_, biome) in data.iter_mut() {
        biome.weights.arcanity = biome.weights.arcanity * 0.5 + 0.5;
        biome.weights.continentalness = biome.weights.continentalness * 0.5 + 0.5;
        //biome.weights.peaks = biome.weights.peaks * 0.5 + 0.5;
        biome.weights.temperature = biome.weights.temperature * 0.5 + 0.5;
        biome.weights.humidity = biome.weights.humidity * 0.5 + 0.5;
    }
    let map: HashMap<Biome, (Box<dyn BiomeGenerator>, BiomeData)> = HashMap::from([
        (Biome::Plains, (Box::new(PlainsBiomeGenerator::new()) as Box<dyn BiomeGenerator>, data[&Biome::Plains.to_string()].clone())),
        (Biome::Mountains, (Box::new(MountainsBiomeGenerator::new()) as Box<dyn BiomeGenerator>, data[&Biome::Mountains.to_string()].clone())),
        (Biome::Desert, (Box::new(DesertBiomeGenerator::new()) as Box<dyn BiomeGenerator>, data[&Biome::Desert.to_string()].clone())),
        (Biome::HauntedWoodlands, (Box::new(HauntedWoodlandsBiomeGenerator::new()) as Box<dyn BiomeGenerator>, data[&Biome::HauntedWoodlands.to_string()].clone())),
        (Biome::Lake, (Box::new(LakeBiomeGenerator::new()) as Box<dyn BiomeGenerator>, data[&Biome::Lake.to_string()].clone())),
        (Biome::SnowyPlains, (Box::new(SnowyPlainsBiomeGenerator::new()) as Box<dyn BiomeGenerator>, data[&Biome::SnowyPlains.to_string()].clone())),
        (Biome::Woodlands, (Box::new(WoodlandsBiomeGenerator::new()) as Box<dyn BiomeGenerator>, data[&Biome::Woodlands.to_string()].clone()))
    ]);
    map
});

pub struct PlainsBiomeGenerator {}
impl PlainsBiomeGenerator {pub fn new() -> Self {Self {}}}
impl BiomeGenerator for PlainsBiomeGenerator {
    fn make_surface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Grass(0)
    }

    fn make_subsurface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Dirt(0)
    }

    fn make_earth_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Stone(0)
    }
}

pub struct MountainsBiomeGenerator {}
impl MountainsBiomeGenerator {pub fn new() -> Self {Self {}}}
impl BiomeGenerator for MountainsBiomeGenerator {
    fn make_surface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Stone(0)
    }

    fn make_subsurface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Stone(0)
    }

    fn make_earth_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Stone(0)
    }
}

pub struct DesertBiomeGenerator {}
impl DesertBiomeGenerator {pub fn new() -> Self {Self {}}}
impl BiomeGenerator for DesertBiomeGenerator {
    fn make_surface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Sand(0)
    }

    fn make_subsurface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Dirt(0)
    }

    fn make_earth_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Stone(0)
    }
}

pub struct LakeBiomeGenerator {}
impl LakeBiomeGenerator {pub fn new() -> Self {Self {}}}
impl BiomeGenerator for LakeBiomeGenerator {
    fn make_surface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Sand(0)
    }

    fn make_subsurface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Dirt(0)
    }

    fn make_earth_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Stone(0)
    }
}

pub struct WoodlandsBiomeGenerator {}
impl WoodlandsBiomeGenerator {pub fn new() -> Self {Self {}}}
impl BiomeGenerator for WoodlandsBiomeGenerator {
    fn make_surface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Grass(0)
    }

    fn make_subsurface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Dirt(0)
    }

    fn make_earth_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Stone(0)
    }
}

pub struct HauntedWoodlandsBiomeGenerator {}
impl HauntedWoodlandsBiomeGenerator {pub fn new() -> Self {Self {}}}
impl BiomeGenerator for HauntedWoodlandsBiomeGenerator {
    fn make_surface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Grass(0)
    }

    fn make_subsurface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Dirt(0)
    }

    fn make_earth_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Stone(0)
    }
}

pub struct SnowyPlainsBiomeGenerator {}
impl SnowyPlainsBiomeGenerator {pub fn new() -> Self {Self {}}}
impl BiomeGenerator for SnowyPlainsBiomeGenerator {
    fn make_surface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Sand(0)
    }

    fn make_subsurface_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Dirt(0)
    }

    fn make_earth_block(&self, position: Vector3<i32>) -> WorldBlock {
        WorldBlock::Stone(0)
    }
}