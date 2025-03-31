use message_io::network::{Endpoint, NetworkController};
use nalgebra::{Vector2, Vector3};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

use crate::world::{blockrepr::WorldBlock, chunkcompress::CompressedChunk};

#[derive(serde::Serialize, serde::Deserialize)]
pub enum Packet {
    Authenticated(AuthenticatedPacket),
    Unauthenticated(UnauthenticatedPacket)
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AuthenticatedPacket {
    pub data: NetworkMessage,
    pub token: String
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct UnauthenticatedPacket {
    pub data: NetworkMessage
}

pub fn send_authenticated_message(network: &NetworkController, endpoint: Endpoint, token: String, data: NetworkMessage) {
    let message = AuthenticatedPacket {
        data,
        token
    };
    network.send(endpoint, &bincode::serialize(&Packet::Authenticated(message)).unwrap());
}

pub fn send_unauthenticated_message(network: &NetworkController, endpoint: Endpoint, data: NetworkMessage) {
    let message = UnauthenticatedPacket {
        data,
    };
    network.send(endpoint, &bincode::serialize(&Packet::Unauthenticated(message)).unwrap());
}

#[derive(Serialize, Deserialize, Debug, Display)]
pub enum NetworkMessage {
    Auth(AuthMessages),
    ServerToClient(ServerToClientMessage),
    ClientToServer(ClientToServerMessage)
}

#[derive(Serialize, Deserialize, Debug, Display)]
pub enum ServerToClientMessage {
    //position, deflate-compressed data
    ChunkProvided((Vector2<i32>, Vec<u8>)),
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
    RequestInitialChunks,
    RequestChunk(Vector2<i32>),
    SetBlock(Vector3<i32>, WorldBlock),
    BreakBlock(Vector3<i32>)
}