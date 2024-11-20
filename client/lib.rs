use std::{io::Read, sync::mpsc, time::Duration};

use flate2::read::ZlibDecoder;
use gamedisplay::GameDisplay;
use message_io::{network::{NetEvent, Transport}, node::{self, NodeEvent}};
use miniz_oxide::inflate::decompress_to_vec;
use winit::event_loop::EventLoop;
use shared::world::{self as shared_world, chunkcompress::{decompress_chunk, CompressedChunk}};

mod renderer;
mod gamedisplay;
mod view;
mod network;
mod world;
mod global;

pub fn main() {

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut display = GameDisplay::default();
    event_loop.run_app(&mut display);


}