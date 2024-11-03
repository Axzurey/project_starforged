use gamedisplay::GameDisplay;
use winit::event_loop::EventLoop;

mod renderer;
mod gamedisplay;
mod view;
mod loaders;
mod network;

pub fn main() {

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let mut display = GameDisplay::default();
    event_loop.run_app(&mut display);

}