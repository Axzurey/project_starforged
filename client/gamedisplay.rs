use std::{collections::HashMap, sync::{mpsc::{self, Receiver, Sender}, Arc}, time::{UNIX_EPOCH}};

use instant::{Instant, SystemTime};
use message_io::{network::{NetEvent, Transport}, node::{self, NodeTask}};
use miniz_oxide::inflate::decompress_to_vec;
use nalgebra::{Point3, Vector2};
use pollster::FutureExt;
use shared::world::{chunk::{xz_to_index, Chunk, ChunkState}, chunkcompress::{decompress_chunk, CompressedChunk}};
use stopwatch::Stopwatch;
use winit::{application::ApplicationHandler, dpi::{PhysicalPosition, PhysicalSize, Size}, event::WindowEvent, event_loop::EventLoop, window::{Window, WindowAttributes}};

use crate::{global::{event_handler::EventHandler, globalstate::GlobalState, inputservice::{InputService, MouseLockState}}, network::clinet::CliNet, renderer::{gamewindow::GameWindow, renderctx::Renderctx}, view::camera::Camera, world::{chunkdraw::ChunkDraw, chunkmanager::ChunkManager, meshthread::spawn_chunk_meshing_loop}};

#[derive(Default)]
pub struct GameDisplay<'a> {
    pub window: Option<Arc<Window>>,
    pub gamewindow: Option<GameWindow<'a>>,
    pub globalstate: Option<GlobalState>,
    pub chunkmesher: Option<(Sender<(i32, i32, u32, std::collections::HashMap<u32, Arc<Chunk>>, Arc<crate::renderer::renderctx::Renderctx>)>, Receiver<(i32, i32, u32, ((wgpu::Buffer, wgpu::Buffer, u32), (wgpu::Buffer, wgpu::Buffer, u32)))>)>,
    pub network: Option<CliNet>,
    pub event_handler: Option<EventHandler>,
    pub last_frame: u128,
    pub last_mouse_position: PhysicalPosition<f64>
}

impl ApplicationHandler for GameDisplay<'_> {

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut win_attr = WindowAttributes::default();
        win_attr = win_attr.with_inner_size(Size::Physical(PhysicalSize::new(1280, 720)));

        self.window = Some(Arc::new(event_loop.create_window(win_attr).unwrap()));

        let mut gamewindow = GameWindow::new(self.window.clone().unwrap()).block_on();

        self.globalstate = Some(GlobalState {
            chunk_manager: ChunkManager::new(),
            camera: Camera::new(Point3::new(0.0, 0.0, 0.0), 0.0, 0.0, gamewindow.window_size.width as f32 / gamewindow.window_size.height as f32, 80.0, gamewindow.device.clone(), &gamewindow.camera_bindgroup_layout),
            input_service: InputService::new(self.window.clone().unwrap())
        });

        self.gamewindow = Some(gamewindow);

        let chunkmesh = spawn_chunk_meshing_loop(4);

        self.chunkmesher = Some(chunkmesh);

        self.network = Some(CliNet::new("127.0.0.1:3043".to_string()));
        self.event_handler = Some(EventHandler::new());

        self.globalstate.as_mut().unwrap().input_service.set_mouse_lock_state(MouseLockState::LockCenter);

        self.last_frame = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
    }
    fn device_event(
            &mut self,
            event_loop: &winit::event_loop::ActiveEventLoop,
            device_id: winit::event::DeviceId,
            event: winit::event::DeviceEvent,
        ) {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                let gamewin = self.gamewindow.as_mut().unwrap();
                
                
                let gs = self.globalstate.as_mut().unwrap();
                gs.camera.controller.process_mouse_input(delta.0, delta.1);
                gs.input_service.process_mouse_move(delta);
            },
            _ => {}
        }
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
            WindowEvent::CursorMoved { device_id, position } => {
                let (lx, ly) = (self.last_mouse_position.x, self.last_mouse_position.y);
                let (cx, cy) = (position.x, position.y);
                let delta = (cx - lx, cy - ly);
                let gs = self.globalstate.as_mut().unwrap();
                
                gs.input_service.process_mouse_move(delta);
            }
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                let gs = self.globalstate.as_mut().unwrap();
                
                gs.input_service.process_key_input(&event, false);

                match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(x) => {
                        let gs = self.globalstate.as_mut().unwrap();
                        gs.camera.controller.process_keyboard_input(x, event.state);
                    },
                    _ => {}
                }
            },
            WindowEvent::RedrawRequested => {

                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();
                let dt = (((now as f64) - (self.last_frame as f64)) / 1000.0) as f32;
                self.last_frame = now;
                
                let gamewin = self.gamewindow.as_mut().unwrap();
                let net = self.network.as_mut().unwrap();
                
                let gs = self.globalstate.as_mut().unwrap();
                gs.input_service.update();
                
                gs.on_world_tick(net, dt);
                
                gamewin.render(dt, gs);

                let network_events = net.recv().block_on();
                self.event_handler.as_mut().unwrap().handle_network_events(&gamewin.device, &gamewin.queue, gs, net, self.chunkmesher.as_mut().unwrap(), network_events);

                for _ in 0..16 {
                    if let Ok((x, z, y, buff)) = self.chunkmesher.as_ref().unwrap().1.try_recv() {
                        println!("Meshed chunk");
                        let index = xz_to_index(x, z);
                        let chunkdraw = self.globalstate.as_mut().unwrap().chunk_manager.chunks.get_mut(&index).unwrap();
                        
                        chunkdraw.set_solid_buffer(y, buff.0);
                        chunkdraw.set_transparent_buffer(y, buff.1);
                        chunkdraw.states[y as usize] = ChunkState::Ready;
                    }
                    else {
                        break;
                    }
                }
                
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}

