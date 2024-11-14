use std::{thread::sleep, time::Duration};

use nalgebra::Vector2;
use network::servernet::{self, ServerNetwork};
use world::serverchunkmanager::ServerChunkManager;

mod world;
mod network;

pub fn main() {
    let mut servernetwork = ServerNetwork::new();

    let mut chunkmanager = ServerChunkManager::new();
    chunkmanager.generate_range_inclusive(0, 0, 1, 1);

    loop {
        sleep(Duration::new(5, 0));
        servernetwork.recv();
        servernetwork.send_message_to("".to_owned(), shared::network::containers::ServerToClientMessage::ChunkAdded((Vector2::new(0, 0), chunkmanager.chunks.get(&0).unwrap().clone())));
    }
}