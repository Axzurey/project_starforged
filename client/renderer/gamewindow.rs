use std::sync::Arc;

use nalgebra::Point3;
use shared::loaders::texture_loader::{initialize_load_textures, preload_textures};
use wgpu::TextureFormat;
use winit::window::Window;

use crate::{global::globalstate::GlobalState, view::camera::Camera};

use super::gamerenderer::GameRenderer;

pub struct GameWindow<'a> {
    surface: wgpu::Surface<'a>,
    pub device: Arc<wgpu::Device>,
    pub queue: Arc<wgpu::Queue>,
    surface_config: wgpu::SurfaceConfiguration,
    pub window_size: winit::dpi::PhysicalSize<u32>,
    pub window: Arc<Window>,
    surface_format: wgpu::TextureFormat,
    pub renderer: GameRenderer,
    pub camera_bindgroup_layout: wgpu::BindGroupLayout,
    texture_bindgroup: wgpu::BindGroup,
    texture_bindgroup_layout: wgpu::BindGroupLayout
}

impl <'a> GameWindow<'a> {
    pub async fn new(window: Arc<Window>) -> Self {
        let window_size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            flags: wgpu::InstanceFlags::empty(),
            backends: wgpu::Backends::DX12,
            dx12_shader_compiler: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false
            }
        ).await.unwrap();

        println!("ADAPTER: {:?}", adapter.get_info());

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::TEXTURE_BINDING_ARRAY
             | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING 
             | wgpu::Features::BGRA8UNORM_STORAGE | wgpu::Features::DEPTH_CLIP_CONTROL,
            required_limits: wgpu::Limits {
                max_sampled_textures_per_shader_stage: 121,
                max_samplers_per_shader_stage: 121,
                max_bind_groups: 5,
                ..Default::default()
            },
            label: None,
            memory_hints: wgpu::MemoryHints::Performance,
        }, None).await.unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        println!("{:?}", surface_capabilities.formats);

        // let surface_format = surface_capabilities.formats.iter()
        //     .copied().find(|f| f.is_srgb())
        //     .unwrap_or(surface_capabilities.formats[surface_capabilities.formats.len() - 1]);
        let surface_format = TextureFormat::Rgba8UnormSrgb;

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2
        };

        let camera_bindgroup_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera bind group layout :)"),
        });

        surface.configure(&device, &surface_config);

        let device_arc = Arc::new(device);
        let queue_arc = Arc::new(queue);

        preload_textures(&device_arc, &queue_arc, surface_format);

        let (texture_bindgroup, texture_bindgroup_layout) = initialize_load_textures(&device_arc, &queue_arc, surface_format);

        let renderer = GameRenderer::new(
            device_arc.clone(), queue_arc.clone(), (window_size.width, window_size.height), surface_format, 
            &camera_bindgroup_layout, &texture_bindgroup_layout);

        Self {
            device: device_arc,
            queue: queue_arc,
            renderer,
            window,
            window_size,
            surface,
            surface_config,
            surface_format,
            camera_bindgroup_layout,
            texture_bindgroup,
            texture_bindgroup_layout
        }
    }
    pub fn render(&mut self, dt: f32, globalstate: &mut GlobalState) {
        
        globalstate.camera.update_camera(dt);
        globalstate.camera.update_matrices(&self.queue);

        let mut output = self.surface.get_current_texture().unwrap();
        let mut view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Primary Encoder")
        });

        self.renderer.render_surface(&self.device, &self.queue, &mut output, &mut view, &mut encoder, &globalstate.camera, &self.texture_bindgroup, &globalstate.chunk_manager);
    
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}