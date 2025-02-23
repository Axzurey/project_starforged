use std::{fs, path::Path, sync::Arc};

pub fn create_surface_pipeline(
    device: &Arc<wgpu::Device>, 
    texture_format: wgpu::TextureFormat, 
    texture_bindgroup_layout: &wgpu::BindGroupLayout, 
    camera_bindgroup_layout: &wgpu::BindGroupLayout,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    depth_format: Option<wgpu::TextureFormat>
) -> wgpu::RenderPipeline {
    let shader_descriptor = wgpu::ShaderModuleDescriptor {
        label: Some("shader descriptor"),
        source: wgpu::ShaderSource::Wgsl(fs::read_to_string(Path::new("res/shaders/surfaceshader.wgsl")).unwrap().into())
    };

    let shader = device.create_shader_module(shader_descriptor);

    let color_targetstate = [Some(wgpu::ColorTargetState {
        format: texture_format,
        blend: Some(wgpu::BlendState {
            alpha: wgpu::BlendComponent::OVER,
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add
            },
        }),
        write_mask: wgpu::ColorWrites::ALL,
    })];

    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("surface pipeline layout"),
        bind_group_layouts: &[&texture_bindgroup_layout, &camera_bindgroup_layout],
        push_constant_ranges: &[]
    });

    let desc = wgpu::RenderPipelineDescriptor {
        label: Some("surface pipeline descriptor"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: vertex_layouts,
            compilation_options: wgpu::PipelineCompilationOptions::default()
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &color_targetstate,
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    };

    device.create_render_pipeline(&desc)
}