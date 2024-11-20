use std::{thread::sleep, time::Duration};

use nalgebra::Vector2;
use network::servernet::ServerNetwork;
use shared::world::chunk::xz_to_index;
use world::serverchunkmanager::ServerChunkManager;

mod world;
mod network;

pub fn main() {
    let mut servernetwork = ServerNetwork::new();

    let mut chunkmanager = ServerChunkManager::new();
    chunkmanager.generate_range_inclusive(-1, -1, 2, 2);

    sleep(Duration::new(8, 0));
    servernetwork.recv();
    for x in -1..2 {
        for z in -1..2 {
            servernetwork.send_message_to("".to_owned(), shared::network::containers::ServerToClientMessage::ChunkAdded((Vector2::new(x, z), chunkmanager.chunks.get(&xz_to_index(x, z)).unwrap().clone())));
        }
    }
}