use message_io::{network::{NetEvent, Transport}, node};

pub fn main() {
    let (handler, listener) = node::split::<()>();

    handler.network().listen(Transport::Udp, "0.0.0.0:3043").unwrap();

    listener.for_each(move |event| match event.network() {
        NetEvent::Connected(_, _) => unreachable!(), // Used for explicit connections.
        NetEvent::Accepted(_endpoint, _listener) => println!("Client connected"), // Tcp or Ws
        NetEvent::Message(endpoint, data) => {
            println!("Received: {}", String::from_utf8_lossy(data));
            handler.network().send(endpoint, data);
        },
        NetEvent::Disconnected(_endpoint) => println!("Client disconnected"), //Tcp or Ws
    });
}