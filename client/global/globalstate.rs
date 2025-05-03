use winit::event::MouseButton;

use crate::{network::clinet::CliNet, view::camera::Camera, world::chunkmanager::ChunkManager};

use super::inputservice::{InputEvent, InputService};

pub struct GlobalState {
    pub chunk_manager: ChunkManager,
    pub camera: Camera,
    pub input_service: InputService
}

impl GlobalState {
    pub fn on_world_tick(&mut self, net: &mut CliNet, dt: f32) {
        let input_events = self.input_service.consume_events();
        
        for event in input_events {
            match event {
                InputEvent::MouseButtonClicked(button) => {
                    if button == MouseButton::Left {
                        
                    }
                },
                _ => {}
            }
        }
    }
}