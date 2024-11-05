use gamedisplay::GameDisplay;
use winit::event_loop::EventLoop;
use shared::world as shared_world;

mod renderer;
mod gamedisplay;
mod view;
mod loaders;
mod network;
mod world;

pub fn main() {

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut display = GameDisplay::default();
    event_loop.run_app(&mut display);


}