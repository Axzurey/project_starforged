use std::{collections::HashMap, io::Write, net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, UdpSocket}, str::FromStr, sync::{mpsc::{channel, Receiver}, Arc, RwLock}, thread};

use message_io::{network::{Endpoint, NetEvent, ResourceId, Transport}, node::{self, NodeHandler, NodeListener, NodeTask}};
use miniz_oxide::{deflate::compress_to_vec, inflate::decompress_to_vec};
use nalgebra::Vector2;
use pollster::FutureExt;
use reqwest::{Method, Request, StatusCode, Url};
use serde::Serialize;
use shared::{network::containers::{send_authenticated_message, send_unauthenticated_message, AuthMessages, ClientToServerMessage, NetworkMessage, ServerToClientMessage}, world::{chunk::Chunk, chunkcompress::{compress_chunk, decompress_chunk, CompressedChunk}}};

const SERVER_HOST: &str = "http://localhost:8000";
const SERVER_HOST_CAST: &str = "http://localhost:8000/clientauthsessiontoken";

#[derive(Serialize)]
pub struct CAST {
    jwt: String,
    expected_host: String,
    auth_secret: String
}

pub struct Client {
    endpoint: Endpoint,
    username: String
}

pub struct CliNet {
    pub handler: Arc<NodeHandler<()>>,
    task: NodeTask,
    pub endpoint: Endpoint,
    join_receiver: Receiver<(Endpoint, NetworkMessage)>,
    target_host: String,
    pub session_token: Option<String>
}

pub enum ClientNetworkEvent {
    ConnectedToServer,
    AcquiredChunk(Vector2<i32>, Arc<Chunk>),
    ServerToClient(ServerToClientMessage),
}

impl CliNet {
    pub fn new(target_host: String) -> Self {
        let (handlerf, listener) = node::split::<()>();

        let (ep, _) = handlerf.network().connect(Transport::Tcp, target_host.clone()).unwrap();

        let handlerarc = Arc::new(handlerf);

        let (send, recv) = channel::<(Endpoint, NetworkMessage)>();

        let ha = handlerarc.clone();

        let task = listener.for_each_async(move |event| match event.network() {
            NetEvent::Connected(ep, _) => {
                send_unauthenticated_message(ha.network(), ep, NetworkMessage::Auth(AuthMessages::ClientRequestJoin("phxie".to_string())));
            },
            NetEvent::Accepted(_endpoint, _listener) => println!("connected"), // Tcp or Ws
            NetEvent::Message(endpoint, data) => {
                //handler.network().send(endpoint, data);
                let message = bincode::deserialize::<NetworkMessage>(data);

                if let Ok(task) = message {
                    send.send((endpoint, task)).unwrap();
                }
                else {
                    println!("Failed to deserialize message.");
                }
            },
            NetEvent::Disconnected(_endpoint) => println!("disconnected"), //Tcp or Ws
        });

        Self {
            handler: handlerarc,
            task,
            endpoint: ep,
            join_receiver: recv,
            target_host,
            session_token: None
        }
    }

    pub async fn handle_auth_message(&mut self, endpoint: Endpoint, msg: AuthMessages) -> Option<ClientNetworkEvent> {
        match msg {
            AuthMessages::AuthRequestUserCredentials(secret) => {
                if endpoint.addr().to_string() != self.target_host {
                    println!("Wrong host 1 {} {}", endpoint.addr().to_string(), self.target_host);
                    return None;
                }
                self.session_token = Some(secret.clone());
                let client = reqwest::Client::new();
                let r = client.post(SERVER_HOST_CAST)
                    .body(serde_json::to_string(&CAST {
                        jwt: "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VybmFtZSI6InBoeGllIiwiZXhwIjoxNzQ4MjIwMTU4fQ.Gvz8liuD4Q7t-epzqbuQILLec1tq540b_gGIQIhExzg".to_string(),
                        expected_host: "127.0.0.1".to_string(),
                        auth_secret: secret
                    }).unwrap())
                    .send().await;

                if let Ok(res) = r {
                    if res.status() == StatusCode::OK {
                        //correct credentials
                    }
                    else {
                        //failed to verify
                        //maybe send a signal
                    }
                }
                else {
                    println!("AUTHENTICATION SERVER OFFLINE");
                }
            },
            AuthMessages::AuthConfirmedUser(username) => {/* not for client */}
            AuthMessages::ClientRequestJoin(username) => {/* not for client */}
            AuthMessages::GiveUserSessionToken(token) => {
                // if endpoint.addr().to_string() != self.target_host {
                //     return;
                // }
                //TODO: READD FOR PROD^
                println!("GOT TOKEN");
                self.session_token = Some(token);
            },
            AuthMessages::JoinConfirmed => {
                //TODO: READD FOR PROD
                // if endpoint.addr().to_string() != self.target_host {
                //     println!("Wrong host 3 {} {}", endpoint.addr().to_string(), self.target_host);
                //     return;
                // }
                println!("JOINED!");
                //send a signal
                return Some(ClientNetworkEvent::ConnectedToServer)
            },
        }
        None
    }

    pub async fn handle_server_to_client_message(&mut self, endpoint: Endpoint, msg: ServerToClientMessage) -> Option<ClientNetworkEvent> {
        match msg {
            ServerToClientMessage::ChunkProvided(c) => {
                let dec = decompress_to_vec(&c.1).unwrap();
                let dc2 = bincode::deserialize::<CompressedChunk>(&dec).unwrap();
                Some(ClientNetworkEvent::AcquiredChunk(c.0, decompress_chunk(dc2)))
            },
            ServerToClientMessage::ConcludeReceiveInitialChunks => {
                Some(ClientNetworkEvent::ServerToClient(msg))
            },
        }
    }

    pub async fn recv(&mut self) -> Vec<ClientNetworkEvent> {
        let mut events = Vec::new();
        loop {
            if let Ok((endpoint, task)) = self.join_receiver.try_recv() {
                match task {
                    NetworkMessage::Auth(msg) => {
                        if let Some(event) = self.handle_auth_message(endpoint, msg).await {
                            events.push(event);
                        }
                    },
                    NetworkMessage::ClientToServer(msg) => {/* nothing to see here */},
                    NetworkMessage::ServerToClient(msg) => {
                        if let Some(event) = self.handle_server_to_client_message(endpoint, msg).await {
                            events.push(event);
                        }
                    }
                }
            }
            else {
                break;
            }
        }
        events
    }
}