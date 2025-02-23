use std::{collections::HashMap, sync::{mpsc::{self, Receiver, Sender}, Arc}};

use message_io::{network::{NetEvent, Transport}, node::{self, NodeTask}};
use miniz_oxide::inflate::decompress_to_vec;
use nalgebra::{Point3, Vector2};
use pollster::FutureExt;
use shared::world::{chunk::{xz_to_index, Chunk, ChunkState}, chunkcompress::{decompress_chunk, CompressedChunk}};
use stopwatch::Stopwatch;
use winit::{application::ApplicationHandler, dpi::{PhysicalSize, Size}, event::WindowEvent, event_loop::EventLoop, window::{Window, WindowAttributes}};

use crate::{global::globalstate::GlobalState, network::clinet::CliNet, renderer::{gamewindow::GameWindow, renderctx::Renderctx}, view::camera::Camera, world::{chunkdraw::ChunkDraw, chunkmanager::ChunkManager, meshthread::spawn_chunk_meshing_loop}};

#[derive(Default)]
pub struct GameDisplay<'a> {
    pub window: Option<Arc<Window>>,
    pub gamewindow: Option<GameWindow<'a>>,
    pub globalstate: Option<GlobalState>,
    pub chunkmesher: Option<(Sender<(i32, i32, u32, std::collections::HashMap<u32, Arc<Chunk>>, Arc<crate::renderer::renderctx::Renderctx>)>, Receiver<(i32, i32, u32, ((wgpu::Buffer, wgpu::Buffer, u32), (wgpu::Buffer, wgpu::Buffer, u32)))>)>,
    pub network: Option<CliNet>
}

impl ApplicationHandler for GameDisplay<'_> {

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let mut win_attr = WindowAttributes::default();
        win_attr = win_attr.with_inner_size(Size::Physical(PhysicalSize::new(1280, 720)));

        self.window = Some(Arc::new(event_loop.create_window(win_attr).unwrap()));

        let mut gamewindow = GameWindow::new(self.window.clone().unwrap()).block_on();

        self.globalstate = Some(GlobalState {
            chunk_manager: ChunkManager::new(),
            camera: Camera::new(Point3::new(0.0, 0.0, 0.0), 0.0, 0.0, gamewindow.window_size.width as f32 / gamewindow.window_size.height as f32, 80.0, gamewindow.device.clone(), &gamewindow.camera_bindgroup_layout)
        });

        self.gamewindow = Some(gamewindow);

        let chunkmesh = spawn_chunk_meshing_loop(4);

        self.chunkmesher = Some(chunkmesh);

        self.network = Some(CliNet::new("127.0.0.1:3043".to_string()));
  
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
                gs.camera.update_camera(0.0166);
                gs.camera.update_matrices(&gamewin.queue);
            },
            winit::event::DeviceEvent::Key(v) => {
                let gamewin = self.gamewindow.as_mut().unwrap();
                
                match v.physical_key {
                    winit::keyboard::PhysicalKey::Code(x) => {
                        let gs = self.globalstate.as_mut().unwrap();
                        gs.camera.controller.process_keyboard_input(x, v.state);
                        gs.camera.update_camera(0.0166);
                        gs.camera.update_matrices(&gamewin.queue);
                    },
                    _ => {}
                }
            }
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
            WindowEvent::RedrawRequested => {
                
                let gamewin = self.gamewindow.as_mut().unwrap();
                
                gamewin.render(0.0166, self.globalstate.as_mut().unwrap());

                self.network.as_mut().unwrap().recv().block_on();

                let t = Stopwatch::start_new();
                // if let Ok(nextchunk) = self.chunkrecv.as_ref().unwrap().try_recv() {
                //     let position = nextchunk.position;
                //     let (x, z) = (nextchunk.position.x, nextchunk.position.y);
                //     let index = xz_to_index(x, z);
                //     let mut chunkdraw = ChunkDraw::new(nextchunk);
                //     chunkdraw.set_slice_vertex_buffers(&gamewin.device);
                //     let gs = self.globalstate.as_mut().unwrap();
                //     gs.chunk_manager.chunks.insert(index, chunkdraw);
                    
                //     let mut nh = HashMap::new();
                //     gs.chunk_manager.chunks.iter().for_each(|(k, v)| {
                //         nh.insert(*k, v.chunk.clone());
                //     });

                //     let renderctx = Arc::new(Renderctx::new(gamewin.device.clone(), gamewin.queue.clone()));
                //     for y in 0..16 {
                //         self.chunkmesher.as_ref().unwrap().0.send((x, z, y, nh.clone(), renderctx.clone())).unwrap();
                //     }
                //     for x in position.x - 1..= position.x + 1 {
                //         for z in position.y - 1..= position.y + 1 {
                //             if !nh.contains_key(&xz_to_index(x, z)) || position == Vector2::new(x, z) {continue}
                //             for y in 0..16 {
                //                 self.chunkmesher.as_ref().unwrap().0.send((x, z, y, nh.clone(), renderctx.clone())).unwrap();
                //             }
                //         }
                //     }
                // }

                for _ in 0..16 {
                    if let Ok((x, z, y, buff)) = self.chunkmesher.as_ref().unwrap().1.try_recv() {
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

