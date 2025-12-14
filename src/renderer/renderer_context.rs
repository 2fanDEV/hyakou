use std::{ops::Range, sync::Arc};

use anyhow::Result;
use glam::Vec3;
use wgpu::{
    Backends, BindGroupLayout, BufferUsages, Device, DeviceDescriptor, ExperimentalFeatures,
    Features, FeaturesWGPU, Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryHints,
    PushConstantRange, Queue, RenderPipeline, RequestAdapterOptions, ShaderStages, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
};

use crate::renderer::{
    components::{
        camera::CameraUniform, light::LightSource, render_pipeline::create_render_pipeline,
        texture::Texture,
    },
    geometry::BindGroupProvider,
    util::Size,
    wrappers::SurfaceProvider,
};

pub struct RenderContext {
    pub instance: Instance,
    pub surface: Option<Surface<'static>>,
    pub surface_configuration: Option<SurfaceConfiguration>,
    pub device: Arc<Device>,
    pub light_render_pipeline: RenderPipeline,
    pub no_light_render_pipeline: RenderPipeline,
    pub size: Size,
    pub camera_bind_group_layout: BindGroupLayout,
    pub light_bind_group_layout: BindGroupLayout,
    pub depth_texture: Texture,
    pub queue: Queue,
}

impl RenderContext {
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

        let required_features = Features {
            features_wgpu: FeaturesWGPU::PUSH_CONSTANTS,
            ..Default::default()
        };

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Hyakou Device"),
                required_features,
                required_limits: Limits {
                    max_push_constant_size: 128,
                    ..Default::default()
                },
                experimental_features: ExperimentalFeatures::default(),
                memory_hints: MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await?;

        let device = Arc::new(device);

        let size = if provider.is_some() {
            provider.unwrap().get_size()
        } else {
            Size {
                width: 1920,
                height: 1080,
            }
        };

        let surface_configuration = match surface.as_ref() {
            Some(surface_ref) => {
                init_surface_configuration(Some(surface_ref), adapter, size, &device)
            }
            None => None,
        };

        let light = LightSource::new(Vec3::new(0.0, 3.0, 3.0), Vec3::new(1.0, 1.0, 1.0));
        let light_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Light Source Buffer"),
            contents: bytemuck::bytes_of(&light),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let depth_texture = Texture::create_depth_texture("Depth Texture", &device, &size);

        let camera_bind_group_layout = CameraUniform::bind_group_layout(&device);
        let light_bind_group_layout = LightSource::bind_group_layout(&device);
        // let (mesh_bind_group_layout, meshes_bind_group) = Vertex::create_bind_group(&device, &depth_texture.view, &depth_texture.sampler);

        let vertex_shader = device.create_shader_module(include_wgsl!("../../assets/vertex.wgsl"));
        let no_light_vertex_shader =
            device.create_shader_module(include_wgsl!("../../assets/no_light_vertex.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &light_bind_group_layout],
                push_constant_ranges: &[PushConstantRange {
                    stages: ShaderStages::VERTEX,
                    range: Range { start: 0, end: 64 },
                }],
            });

        let format = if surface_configuration.is_some() {
            surface_configuration.as_ref().unwrap().format
        } else {
            TextureFormat::Bgra8UnormSrgb
        };

        let no_light_render_pipeline = create_render_pipeline(
            &device,
            "no light render pass",
            &render_pipeline_layout,
            format,
            no_light_vertex_shader,
            Some(TextureFormat::Depth32Float),
        );

        let light_render_pipeline = create_render_pipeline(
            &device,
            "light render pass",
            &render_pipeline_layout,
            format,
            vertex_shader,
            Some(TextureFormat::Depth32Float),
        );

        Ok(Self {
            instance,
            surface,
            surface_configuration,
            device,
            light_render_pipeline,
            no_light_render_pipeline,
            size,
            depth_texture,
            light_bind_group_layout,
            camera_bind_group_layout,
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
                format,
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
    use crate::renderer::{renderer_context::RenderContext, wrappers::MockSurfaceProvider};

    #[test]
    fn create_context() {
        let ctx = pollster::block_on(RenderContext::new::<MockSurfaceProvider>(None));
        assert!(ctx.is_ok());
    }
}
