use wgpu::{
    BlendState, ColorTargetState, ColorWrites, Device, FragmentState, MultisampleState, PipelineCompilationOptions, PipelineLayout, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderModule, TextureFormat, VertexState
};

use crate::renderer::geometry::{BufferLayoutProvider, vertices::Vertex};

pub fn create_render_pipeline(
    device: &Device,
    label: &str,
    pipeline_layout: &PipelineLayout,
    color_format: TextureFormat,
    shader_module: ShaderModule,
    depth_format: Option<TextureFormat>,
) -> RenderPipeline {
    device.create_render_pipeline(&RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(pipeline_layout),
        vertex: VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            buffers: &[Vertex::vertex_buffer_layout()],
        },
        primitive: PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: depth_format.map(|format| wgpu::DepthStencilState {
            format,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: MultisampleState {
            count: 1,
            mask: 0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            compilation_options: PipelineCompilationOptions::default(),
            targets: &[Some(ColorTargetState {
                format: color_format,
                blend: Some(BlendState::REPLACE),
                write_mask: ColorWrites::ALL
            })],
        }),
        multiview: None,
        cache: None,
    })
}
