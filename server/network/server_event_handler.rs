use message_io::network::NetworkController;
use miniz_oxide::deflate::compress_to_vec;
use shared::{network::containers::{NetworkMessage, ServerToClientMessage}, world::chunkcompress::compress_chunk};

use crate::world::serverchunkmanager::ServerChunkManager;

use super::servernet::{Client, ServerNetwork, ServerNetworkMessage};

pub struct ServerEventHandler {

}

pub fn send_network_message(network: &NetworkController, client: &Client, data: &NetworkMessage) {
    network.send(client.endpoint, &bincode::serialize(data).unwrap());
}

impl ServerEventHandler {
    pub fn handle_network_messages(msgs: Vec<ServerNetworkMessage>, chunk_manager: &mut ServerChunkManager, network: &mut ServerNetwork) {
        for msg in msgs {
            match msg {
                ServerNetworkMessage::ProvideInitialClientChunks(client) => {
                    for (i, v) in &chunk_manager.chunks {
                        println!("CHUNK");
                        let compressed = compress_chunk(v);
                        let encoded = bincode::serialize(&compressed).unwrap();

                        let cmp = compress_to_vec(&encoded, 6);
                        
                        send_network_message(network.handler.network(), &client, &NetworkMessage::ServerToClient(ServerToClientMessage::ChunkProvided((v.position, cmp))));
                    }
                    send_network_message(network.handler.network(), &client, &NetworkMessage::ServerToClient(ServerToClientMessage::ConcludeReceiveInitialChunks));
                }
            }
        }
    }
}