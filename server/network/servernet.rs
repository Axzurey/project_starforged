use std::{collections::HashMap, net::{Ipv4Addr, SocketAddrV4}, str::FromStr, sync::{mpsc::{channel, Receiver}, Arc, RwLock}, thread};

use bincode::{deserialize, serialize};
use message_io::{network::{Endpoint, NetEvent, ResourceId, Transport}, node::{self, NodeHandler, NodeListener, NodeTask}};
use shared::network::containers::ServerToClientMessage;

pub struct ServerNetwork {
    handler: Arc<NodeHandler<()>>,
    connected: HashMap<String, Endpoint>,
    task: NodeTask,
    id: ResourceId,
    join_receiver: Receiver<(String, Endpoint)>
}

impl ServerNetwork {
    pub fn new() -> Self {
        let (handlerf, listener) = node::split::<()>();

        let (id, _) = handlerf.network().listen(Transport::Udp, "0.0.0.0:3043").unwrap();

        let handlerarc = Arc::new(handlerf);

        let (send, recv) = channel::<(String, Endpoint)>();

        let handler = handlerarc.clone();
        let task = listener.for_each_async(move |event| match event.network() {
            NetEvent::Connected(_, _) => unreachable!(), // Used for explicit connections.
            NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"), // Tcp or Ws
            NetEvent::Message(endpoint, data) => {
                println!("Received: {} from {}", String::from_utf8_lossy(data), endpoint);
                handler.network().send(endpoint, data);
                send.send(("X".to_owned(), endpoint)).unwrap();
            },
            NetEvent::Disconnected(_endpoint) => println!("Client disconnected"), //Tcp or Ws
        });
        
        
        Self {
            handler: handlerarc,
            connected: HashMap::new(),
            task,
            id,
            join_receiver: recv
        }
    }

    pub fn recv(&mut self) {
        loop {
            let res = self.join_receiver.try_recv();
            if res.is_err() {
                break;
            }
            else {
                let u = res.unwrap();
                self.connected.insert(u.0, u.1);
            }
        }
    }

    pub fn send_message_to(&self, userid: String, message: ServerToClientMessage) {
        match message {
            ServerToClientMessage::ChunkAdded(c) => {
                let encoded = bincode::serialize(&c).unwrap();
                println!("LEN {}", encoded.len());
                let u = self.handler.network().send(*self.connected.get("X").unwrap(), &encoded);
                println!("{:?} {}", u, *self.connected.get("X").unwrap());
            }
        }
    }
}