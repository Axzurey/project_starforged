use std::sync::Arc;

use pollster::FutureExt;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::EventLoop, window::Window};

use crate::{renderer::gamewindow::GameWindow, view::camera::Camera};

#[derive(Default)]
pub struct GameDisplay {
    pub window: Option<Arc<Window>>,
}

impl ApplicationHandler for GameDisplay {

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(Arc::new(event_loop.create_window(Window::default_attributes()).unwrap()));

        let mut gamewindow = GameWindow::new(self.window.clone().unwrap()).block_on();
        gamewindow.load_textures();
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
               
                //draw before this line
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

