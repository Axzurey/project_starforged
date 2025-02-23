use std::{thread::sleep, time::Duration};

use nalgebra::Vector2;
use network::servernet::ServerNetwork;
use world::serverchunkmanager::ServerChunkManager;
use shared::network::containers::ServerToClientMessage;
mod world;
mod network;

#[tokio::main]
pub async fn main() {
    let mut servernetwork = ServerNetwork::new();

    // for i in 0..10 {
    //     sleep(Duration::from_secs(1));
    //     servernetwork.recv();
    // }
    let mut chunkmanager = ServerChunkManager::new();

    loop {
        sleep(Duration::from_millis(33)); //1000ms/30ticks ~= 33
        servernetwork.recv().await;
    }
    // chunkmanager.generate_range_inclusive(-10, -10, 10, 10);
    // for (_, chunk) in &chunkmanager.chunks {
    //     let pos = chunk.position;
    //     servernetwork.send_message_to("".to_owned(), ServerToClientMessage::ChunkProvided((Vector2::new(pos.x, pos.y), chunk)));
    // }
}