use std::{collections::HashMap, sync::{mpsc::{Receiver, Sender}, Arc}};

use miniz_oxide::inflate::decompress_to_vec;
use nalgebra::Vector2;
use shared::{network::containers::{send_authenticated_message, ClientToServerMessage, NetworkMessage, ServerToClientMessage}, world::{chunk::{xz_to_index, Chunk}, chunkcompress::{decompress_chunk, CompressedChunk}}};
use stopwatch::Stopwatch;

use crate::{network::clinet::{CliNet, ClientNetworkEvent}, renderer::renderctx::Renderctx, world::chunkdraw::ChunkDraw};

use super::globalstate::GlobalState;

pub struct EventHandler {

}

impl EventHandler {
    pub fn new() -> Self {
        Self {
        
        }
    }

    pub fn handle_network_events(&mut self, device: &Arc<wgpu::Device>, queue: &Arc<wgpu::Queue>, gs: &mut GlobalState, network: &mut CliNet, chunk_mesher: &mut (Sender<(i32, i32, u32, HashMap<u32, Arc<Chunk>>, Arc<crate::renderer::renderctx::Renderctx>)>, Receiver<(i32, i32, u32, ((wgpu::Buffer, wgpu::Buffer, u32), (wgpu::Buffer, wgpu::Buffer, u32)))>), events: Vec<ClientNetworkEvent>) {
        for event in events {
            match event {
                ClientNetworkEvent::AcquiredChunk(pos, nextchunk) => {
                    let t = Stopwatch::start_new();
                    let position = nextchunk.position;
                    let (x, z) = (nextchunk.position.x, nextchunk.position.y);
                    let index = xz_to_index(x, z);
                    let mut chunkdraw = ChunkDraw::new(nextchunk);
                    chunkdraw.set_slice_vertex_buffers(device);
                    gs.chunk_manager.chunks.insert(index, chunkdraw);

                    // let nh = HashMap::from_iter(gs.chunk_manager.chunks.iter().map(|(k, v)| {
                    //     (*k, v.chunk.clone())
                    // }));

                    //let renderctx = Arc::new(Renderctx::new(device.clone(), queue.clone()));
                    // for y in 0..16 {
                    //     chunk_mesher.0.send((x, z, y, nh.clone(), renderctx.clone())).unwrap();
                    // }
                    // for x in position.x - 1..= position.x + 1 {
                    //     for z in position.y - 1..= position.y + 1 {
                    //         if !nh.contains_key(&xz_to_index(x, z)) || position == Vector2::new(x, z) {continue}
                    //         for y in 0..16 {
                    //             chunk_mesher.0.send((x, z, y, nh.clone(), renderctx.clone())).unwrap();
                    //         }
                    //     }
                    // }
                },
                ClientNetworkEvent::ConnectedToServer => {
                    println!("Connected");
                    //grab chunks from server.
                    send_authenticated_message(network.handler.network(), network.endpoint, network.session_token.clone().unwrap(), NetworkMessage::ClientToServer(ClientToServerMessage::RequestInitialChunks));
                },
                ClientNetworkEvent::ServerToClient(stc) => {
                    match stc {
                        ServerToClientMessage::ConcludeReceiveInitialChunks => {
                            println!("Got initial Chunks");
                            let nh = HashMap::from_iter(gs.chunk_manager.chunks.iter().map(|(k, v)| {
                                (*k, v.chunk.clone())
                            }));

                            let renderctx = Arc::new(Renderctx::new(device.clone(), queue.clone()));
                            for (_, chunk) in &gs.chunk_manager.chunks {
                                for y in 0..16 {
                                    chunk_mesher.0.send((chunk.chunk.position.x, chunk.chunk.position.y, y, nh.clone(), renderctx.clone())).unwrap();
                                }
                            }
                        },
                        ServerToClientMessage::ChunkProvided((pos, data)) => {
                            let dec = decompress_to_vec(&data).unwrap();
                                
                            let deser = bincode::deserialize::<CompressedChunk>(&dec).unwrap();
                            
                            let chunk = decompress_chunk(deser);
                            
                            gs.chunk_manager.chunks.insert(xz_to_index(pos.x, pos.y), ChunkDraw::new(chunk));
                            println!("Added Chunk");
                        }
                    }
                }
            }
        }
    }
}
