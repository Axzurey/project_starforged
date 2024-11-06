use std::sync::Arc;

use nalgebra::Point3;
use winit::window::Window;

use crate::{loaders::texture_loader::initialize_load_textures, view::camera::Camera};

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
    camera: Camera,
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

        let surface_format = surface_capabilities.formats.iter()
            .copied().find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[surface_capabilities.formats.len() - 1]);

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

        let camera = Camera::new(Point3::new(0.0, 0.0, 0.0), 0.0, 0.0, window_size.width as f32 / window_size.height as f32, 70.0, device_arc.clone(), &camera_bindgroup_layout);

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
            camera,
            texture_bindgroup,
            texture_bindgroup_layout
        }
    }

}