use std::{thread::sleep, time::Duration};

use nalgebra::Vector2;
use network::servernet::ServerNetwork;
use world::serverchunkmanager::ServerChunkManager;
use shared::network::containers::ServerToClientMessage::ChunkAdded;
mod world;
mod network;

pub fn main() {
    let mut servernetwork = ServerNetwork::new();

    for i in 0..8 {
        sleep(Duration::from_secs(1));
        servernetwork.recv();
    }
    let mut chunkmanager = ServerChunkManager::new();
    chunkmanager.generate_range_inclusive(-9, -9, 9, 9, &|chunk| {
        let pos = chunk.position;
        servernetwork.send_message_to("".to_owned(), ChunkAdded((Vector2::new(pos.x, pos.y), chunk)));
    });
}