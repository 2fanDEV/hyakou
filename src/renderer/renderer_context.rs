use std::{ops::Range, path::Path};

use anyhow::Result;
use log::debug;
use nalgebra::{Matrix4, Point3, Vector3};
use wgpu::{
    Backends, BindGroup, Buffer, BufferUsages, Device, DeviceDescriptor, ExperimentalFeatures, Features, FeaturesWGPU, Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryHints, PushConstantRange, Queue, RenderPipeline, RequestAdapterOptions, ShaderStages, Surface, SurfaceConfiguration, TextureFormat, TextureUsages, include_wgsl, util::{BufferInitDescriptor, DeviceExt}
};

use crate::renderer::{
    components::{
        camera::{Camera, CameraUniform}, glTF, light::LightSource, render_pipeline::create_render_pipeline, texture::Texture
    }, geometry::vertices::Vertex, util::Size, wrappers::SurfaceProvider
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
    pub mesh_bind_group: BindGroup,
    pub camera: Camera,
    pub camera_uniform_buffer: Buffer,
    pub camera_bind_group: BindGroup,
    pub light: LightSource,
    pub light_uniform_buffer: Buffer,
    pub light_bind_group: BindGroup,
    pub model_matrix: Matrix4<f32>,
    pub depth_texture: Texture,
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

        let size = provider.unwrap().get_size();
        let surface_configuration = match surface.as_ref() {
            Some(surface_ref) => {
                init_surface_configuration(Some(surface_ref), adapter, size, &device)
            }
            None => None,
        };

        debug!("SIZE: {:?}, {:?}", size.width, size.height);
        let camera = Camera::new(
            Point3::new(0.0, 0.0, 10.0),
            Point3::new(0.0, 0.0, -1.0),
            Vector3::y_axis().into_inner(),
            (size.width / size.height) as f32,
            45.0_f32.to_radians(),
            0.1,
            100.0,
        );

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update(&camera);
        println!("{:?}", camera_uniform.view_projection_matrix);

        let camera_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::bytes_of(&camera_uniform),
            usage: BufferUsages::UNIFORM,
        });

        let light = LightSource::new(Vector3::new(1.0, 1.0, 1.0), Vector3::new(1.0, 1.0, 1.0));
        let light_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Light Source Buffer"),
            contents: bytemuck::bytes_of(&light),
            usage: BufferUsages::UNIFORM
        });
        let path = Path::new("/Users/zapzap/Projects/hyako/assets/gltf/monkey.glb");
        let meshes = glTF::GLTFLoader::load_from_path(path).unwrap();
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&meshes[0].vertices),
            usage: BufferUsages::VERTEX,
        });

        let indices: &[u32] = &meshes[0].indices;
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: BufferUsages::INDEX,
        });

        let model_matrix = meshes[0].model_matrix;

        let depth_texture = Texture::create_depth_texture(
            "Depth Texture",
            &device,
            surface_configuration.as_ref().unwrap(),
        );
        
        let (mesh_bind_group_layout, meshes_bind_group) = Vertex::create_bind_group(&device, &depth_texture.view, &depth_texture.sampler);
        let (camera_bind_group_layout, camera_bind_group) = CameraUniform::bind_group(&device, &camera_uniform_buffer);
        let (light_bind_group_layout, light_bind_group) = LightSource::bind_group(&device, &light_uniform_buffer);

        let vertex_shader =
            device.create_shader_module(include_wgsl!("../../assets/vertex_hc.wgsl"));
        let fragment_shader =
            device.create_shader_module(include_wgsl!("../../assets/vertex_hc.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&mesh_bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[PushConstantRange {
                    stages: ShaderStages::VERTEX,
                    range: Range { start: 0, end: 64 },
                }],
            });
        

        let light_render_pass = create_render_pipeline(
            &device,
             "depth render pass",
              &render_pipeline_layout, 
              surface_configuration.as_ref().unwrap().format, 
              vertex_shader.clone(), Some(TextureFormat::Depth32Float));
        


        let render_pipeline = create_render_pipeline(
              &device,
             "Scene Render Pass",
              &render_pipeline_layout,
              surface_configuration.as_ref().unwrap().format,
              vertex_shader,
              None
         );

        Ok(Self {
            instance,
            surface,
            surface_configuration,
            device,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            camera,
            camera_uniform_buffer,
            light,
            light_uniform_buffer,
            depth_texture,
            num_indices: indices.len(),
            mesh_bind_group: meshes_bind_group,
            light_bind_group,
            camera_bind_group,
            queue,
            model_matrix,
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
