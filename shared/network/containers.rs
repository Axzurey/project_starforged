use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::world::{blockrepr::WorldBlock, chunkcompress::CompressedChunk};

#[derive(Serialize, Deserialize, Debug, Display)]
pub enum NetworkMessage {
    Auth(AuthMessages),
    ServerToClient(ServerToClientMessage),
    ClientToServer(ClientToServerMessage)
}

#[derive(Serialize, Deserialize, Debug, Display)]
pub enum ServerToClientMessage {
    ChunkProvided((Vector2<i32>, CompressedChunk)),
    PrepareReceiveInitialChunks,
    ConcludeReceiveInitialChunks,
}

#[derive(Serialize, Deserialize, Debug, Display)]
pub enum AuthMessages {
    //auth -> client
    AuthRequestUserCredentials(String),
    //auth -> server (when authentication server confirms user) (username)
    AuthConfirmedUser(String),
    //client -> server (username)
    ClientRequestJoin(String),
    //server -> client (session token)
    GiveUserSessionToken(String),
    //server -> client
    JoinConfirmed,
}
#[derive(Serialize, Deserialize, Debug, Display)]
pub enum ClientToServerMessage {
    RequestChunk(Vector2<i32>),
    SetBlock(Vector3<i32>, WorldBlock),
    BreakBlock(Vector3<i32>)
}