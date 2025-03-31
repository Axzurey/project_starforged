use std::{collections::HashMap, io::Write, net::{Ipv4Addr, SocketAddrV4}, str::FromStr, sync::{mpsc::{channel, Receiver}, Arc, RwLock}, thread};

use bincode::{deserialize, serialize};
use flate2::{write::ZlibEncoder, Compression};
use message_io::{network::{Endpoint, NetEvent, ResourceId, Transport}, node::{self, NodeHandler, NodeListener, NodeTask}};
use miniz_oxide::deflate::compress_to_vec;
use pollster::FutureExt;
use reqwest::{StatusCode, Url};
use serde::Serialize;
use shared::{network::containers::{AuthMessages, AuthenticatedPacket, ClientToServerMessage, NetworkMessage, Packet, ServerToClientMessage, UnauthenticatedPacket}, world::chunkcompress::compress_chunk};

const SERVER_HOST: &str = "http://localhost:8000";
const SERVER_HOST_SRST: &str = "http://localhost:8000/servergetsessiontoken";

#[derive(Serialize)]
pub struct SRST {
    pub expected_username: String,
    pub callback: String
}

#[derive(Clone)]
pub struct Client {
    pub endpoint: Endpoint,
    pub username: String
}

pub enum ServerNetworkMessage {
    ProvideInitialClientChunks(Client),
}

pub struct ServerNetwork {
    pub handler: Arc<NodeHandler<()>>,
    connected: HashMap<String, Endpoint>,
    task: NodeTask,
    id: ResourceId,
    join_receiver: Receiver<(Endpoint, Packet)>,
    queued_auth_responses: HashMap<String, (String, Endpoint)>,
    valid_tokens: HashMap<String, Client>,
}

impl ServerNetwork {
    pub fn new() -> Self {
        let (handlerf, listener) = node::split::<()>();

        let (id, _) = handlerf.network().listen(Transport::Tcp, "0.0.0.0:3043").unwrap();

        let handlerarc = Arc::new(handlerf);

        let (send, recv) = channel::<(Endpoint, Packet)>();

        let handler = handlerarc.clone();
        let task = listener.for_each_async(move |event| match event.network() {
            NetEvent::Connected(_, _) => unreachable!(), // Used for explicit connections.
            NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"), // Tcp or Ws
            NetEvent::Message(endpoint, data) => {
                //handler.network().send(endpoint, data);
                let msg = bincode::deserialize::<Packet>(data);

                if let Ok(message) = msg {
                    send.send((endpoint, message)).unwrap();
                }
                else {
                    println!("Failed to deserialize message");
                }
                
                
            },
            NetEvent::Disconnected(_endpoint) => println!("Client disconnected"), //Tcp or Ws
        });
        
        
        Self {
            handler: handlerarc,
            connected: HashMap::new(),
            task,
            id,
            join_receiver: recv,
            queued_auth_responses: HashMap::new(),
            valid_tokens: HashMap::new()
        }
    }

    pub fn validate_packet(&self, packet: &AuthenticatedPacket) -> Option<Client> {
        if let Some(user) = self.valid_tokens.get(&packet.token) {
            return Some(user.clone())
        }
        None
    }

    async fn handle_auth_message(&mut self, endpoint: Endpoint, msg: AuthMessages) -> Option<ServerNetworkMessage> {
        match msg {
            AuthMessages::AuthRequestUserCredentials(_) => None,
            AuthMessages::AuthConfirmedUser(username) => {
                //if &endpoint.addr().ip().to_string() != SERVER_HOST {return};
                //TODO: READD FOR PROD^^
                
                let qar = self.queued_auth_responses.remove(&username).unwrap();
                self.valid_tokens.insert(qar.0, Client {
                    username: username.clone(),
                    endpoint: qar.1.clone()
                });
                self.handler.network().send(qar.1, &bincode::serialize(&NetworkMessage::Auth(AuthMessages::JoinConfirmed)).unwrap());
                None
            },
            AuthMessages::ClientRequestJoin(username) => {
                let client = reqwest::Client::new();
                let req = client.get(Url::parse(SERVER_HOST_SRST).unwrap())
                    .body(serde_json::to_string(&SRST {
                        expected_username: username.clone(),
                        callback: "127.0.0.1:3043".to_string()
                    }).unwrap()).send().await;

                if let Ok(res) = req {
                    if res.status() == StatusCode::OK {
                        let code = res.text().await.unwrap();
                        self.queued_auth_responses.insert(username, (code.clone(), endpoint));
                        println!("WILL GIVE");
                        self.handler.network().send(endpoint, &bincode::serialize(&NetworkMessage::Auth(AuthMessages::AuthRequestUserCredentials(code.clone()))).unwrap());
                        self.handler.network().send(endpoint, &bincode::serialize(&NetworkMessage::Auth(AuthMessages::GiveUserSessionToken(code))).unwrap());
                    }
                    else {
                        println!("BAD STATUS");
                    }
                }
                else {
                    println!("AUTHENTICATION SERVER OFFLINE");
                }
                None
            },
            AuthMessages::GiveUserSessionToken(_) => None,
            AuthMessages::JoinConfirmed => None,
        }
    }

    pub async fn handle_client_to_server_message(&self, client: Client, msg: ClientToServerMessage) -> Option<ServerNetworkMessage> {
        match msg {
            ClientToServerMessage::RequestInitialChunks => {
                Some(ServerNetworkMessage::ProvideInitialClientChunks(client))
            },
            _ => None
        }
    }

    pub async fn recv(&mut self) -> Vec<ServerNetworkMessage> {
        let mut msgs: Vec<ServerNetworkMessage> = Vec::new();
        loop {
            if let Ok((endpoint, packet)) = self.join_receiver.try_recv() {
                let message: (Option<NetworkMessage>, Option<Client>) = match packet {
                    Packet::Authenticated(p) => {
                        if let Some(client) = self.validate_packet(&p) {
                            (Some(p.data), Some(client))
                        }
                        else {
                            (None, None)
                        }
                    }
                    Packet::Unauthenticated(p) => (Some(p.data), None)
                };
                if message.0.is_none() {continue}; //unauthenticated message.
                
                let msg = message.0.unwrap();

                match msg {
                    NetworkMessage::ServerToClient(msg) => {
                        if message.1.is_none() {continue;}
                    },
                    NetworkMessage::Auth(msg) => {
                        self.handle_auth_message(endpoint, msg).await;
                    },
                    NetworkMessage::ClientToServer(msg) => {
                        if message.1.is_none() {continue;}
                        println!("RC SV {}", msg);
                        if let Some(res) = self.handle_client_to_server_message(message.1.unwrap(), msg).await {
                            msgs.push(res);
                        }
                    },
                }
            }
            else {
                break;
            }
        }
        msgs
    }
    //199480(new) 7240448(old) 133944(newer) 14595(newest!!! zlib!!!)
    pub fn send_message_to(&self, userid: String, message: ServerToClientMessage) {
        match message {
            ServerToClientMessage::ChunkProvided(c) => {
                let compressed = c.1;
                let encoded = bincode::serialize(&compressed).unwrap();
                
                let b = compress_to_vec(&encoded, 6);
                println!("LEN {}, {}", b.len(), encoded.len());
                let u = self.handler.network().send(*self.connected.get("X").unwrap(), &b);
                println!("{:?} {}", u, *self.connected.get("X").unwrap());
            },
            _ => {}
        }
    }
}