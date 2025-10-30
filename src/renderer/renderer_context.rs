use anyhow::Result;
use nalgebra::Vector3;
use wgpu::{
    include_wgsl, util::{BufferInitDescriptor, DeviceExt}, Backends, BlendState, Buffer, BufferUsages, ColorTargetState, ColorWrites, Device, DeviceDescriptor, ExperimentalFeatures, Extent3d, Features, FragmentState, Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryHints, PipelineCompilationOptions, PipelineLayout, PrimitiveState, Queue, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModuleDescriptor, ShaderSource, Surface, SurfaceConfiguration, TextureDescriptor, TextureUsages, VertexState
};

use crate::renderer::{
    draw_entities::{vertices::Vertex, BufferLayouts},
    util::Size,
    wrappers::SurfaceProvider,
};

pub struct RendererContext {
    pub instance: Instance,
    pub surface: Option<Surface<'static>>,
    pub surface_configuration: Option<SurfaceConfiguration>,
    pub device: Device,
    pub render_pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: usize,
    pub queue: Queue,
}

impl RendererContext {
    pub async fn new<T>(provider: Option<T>) -> Result<Self>
    where
        T: SurfaceProvider,
    {
        #[cfg(target_os = "macos")]
        let backends = Backends::METAL;

        let instance = wgpu::Instance::new(&InstanceDescriptor {
            backends,
            flags: InstanceFlags::debugging(),
            ..Default::default()
        });

        let surface = match provider.as_ref() {
            Some(prov) => prov.create_surface(&instance),
            None => None,
        };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: surface.as_ref(),
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Hyakou Device"),
                required_features: Features::default(),
                required_limits: Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                experimental_features: ExperimentalFeatures::default(),
                memory_hints: MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_configuration = match surface.as_ref() {
            Some(surface_ref) => init_surface_configuration(
                Some(surface_ref),
                adapter,
                provider.unwrap().get_size(),
                &device,
            ),
            None => None,
        };

        const VERTICES: &[Vertex] = &[
            Vertex::new(
                Vector3::new(-0.0868241, 0.49240386, 0.0),
                Vector3::new(0.5, 0.0, 0.5),
            ),
            Vertex::new(
                Vector3::new(-0.49513406, 0.06958647, 0.0),
                Vector3::new(0.5, 0.0, 0.5),
            ),
            Vertex::new(
                Vector3::new(-0.21918549, -0.44939706, 0.0),
                Vector3::new(0.5, 0.0, 0.5),
            ),
            Vertex::new(
                Vector3::new(0.35966998, -0.3473291, 0.0),
                Vector3::new(0.5, 0.0, 0.5),
            ),
            Vertex::new(
                Vector3::new(0.44147372, 0.2347359, 0.0),
                Vector3::new(0.5, 0.0, 0.5),
            ),
        ];

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        let vertex_shader =
            device.create_shader_module(include_wgsl!("../../assets/vertex_hc.wgsl"));
        let fragment_shader =
            device.create_shader_module(include_wgsl!("../../assets/vertex_hc.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &vertex_shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[Vertex::layouts()],
            },
            fragment: Some(FragmentState {
                module: &fragment_shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: surface_configuration.as_ref().unwrap().format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: 0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let image_bytes = include_bytes!("../../images/happy-tree.png");
        let image = image::load_from_memory(image_bytes)?;
        let rgba_image = image.to_rgba8();
        let (img_width, img_height) = rgba_image.dimensions();
        
        let texture_size = Extent3d {
            width: img_width,
            height: img_height,
            depth_or_array_layers: 1
        };

        let texture_img = device.create_texture(&TextureDescriptor {
            label: todo!(),
            size: todo!(),
            mip_level_count: todo!(),
            sample_count: todo!(),
            dimension: todo!(),
            format: todo!(),
            usage: todo!(),
            view_formats: todo!(),
        });

        Ok(Self {
            instance,
            surface,
            surface_configuration,
            device,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len(),
            queue,
        })
    }

    // requires winit window, no test until figured out how to do headless
    pub fn resize(&mut self, size: Size) {
        self.surface_configuration.as_mut().map(|cfg| {
            cfg.width = size.width;
            cfg.height = size.height;
            self.surface.as_ref().unwrap().configure(&self.device, &cfg);
            cfg
        });
    }
}

fn init_surface_configuration(
    surface: Option<&Surface<'static>>,
    adapter: wgpu::Adapter,
    size: Size,
    device: &Device,
) -> Option<wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>>> {
    let surface_configuration = match surface {
        Some(surface) => {
            let capabilities = surface.get_capabilities(&adapter);
            let format = capabilities
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(capabilities.formats[0]);

            let surface_configuration = SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: format,
                width: size.width,
                height: size.height,
                present_mode: capabilities.present_modes[0],
                desired_maximum_frame_latency: 2,
                alpha_mode: capabilities.alpha_modes[0],
                view_formats: vec![],
            };

            surface.configure(device, &surface_configuration);
            Some(surface_configuration)
        }
        None => None,
    };
    surface_configuration
}

#[cfg(test)]
mod tests {
    use crate::renderer::{renderer_context::RendererContext, wrappers::MockSurfaceProvider};

    #[test]
    fn create_context() {
        let mut mock = MockSurfaceProvider::new();
        let ctx = pollster::block_on(RendererContext::new::<MockSurfaceProvider>(None));
        assert!(ctx.is_ok());
    }
}
