use std::sync::Arc;

use anyhow::{Result, anyhow};
use hyakou_core::{
    components::light::LightSource,
    traits::BindGroupProvider,
    types::{ModelMatrixBindingMode, Size},
};
use log::warn;
use wgpu::{
    Backends, BindGroupLayout, Device, DeviceDescriptor, ExperimentalFeatures, Features,
    FeaturesWebGPU, Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryHints, Queue,
    RenderPipeline, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat,
    TextureUsages, include_wgsl,
};

use crate::{
    gpu::{
        buffers::camera_buffer::CameraUniform, buffers::model_matrix::ModelMatrixUniform,
        material::GpuMaterial, render_pipeline::create_render_pipeline, texture::Texture,
    },
    renderer::wrappers::SurfaceProvider,
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
    pub model_bind_group_layout: Option<BindGroupLayout>,
    pub material_bind_group_layout: BindGroupLayout,
    pub model_binding_mode: ModelMatrixBindingMode,
    pub depth_texture: Texture,
    pub queue: Queue,
}

impl RenderContext {
    const IMMEDIATE_MODEL_MATRIX_SIZE: u32 = 64;
    const DEPTH_TEXTURE_LABEL: &str = "Depth Texture";

    pub async fn new<T>(provider: Option<T>) -> Result<Self>
    where
        T: SurfaceProvider,
    {
        #[cfg(target_os = "macos")]
        let backends = Backends::METAL;

        #[cfg(target_arch = "wasm32")]
        let backends = Backends::all();

        #[cfg(all(not(target_os = "macos"), not(target_arch = "wasm32")))]
        let backends = Backends::PRIMARY;
        // #[cfg(target_os = "linux")]
        // let backends = Backends::PRIMARY;

        let mut instance_descriptor = InstanceDescriptor::new_without_display_handle();
        instance_descriptor.backends = backends;
        instance_descriptor.flags = InstanceFlags::debugging();
        let instance = wgpu::Instance::new(instance_descriptor);

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

        let model_binding_mode = select_model_binding_mode(&adapter);
        let required_features = required_features_for(model_binding_mode);
        let required_limits = required_limits_for(model_binding_mode);

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Hyakou Device"),
                required_features,
                required_limits,
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

        let depth_texture =
            Texture::create_depth_texture(Self::DEPTH_TEXTURE_LABEL, &device, &size);

        let camera_bind_group_layout = CameraUniform::bind_group_layout(&device);
        let light_bind_group_layout = LightSource::bind_group_layout(&device);
        let model_bind_group_layout = (model_binding_mode == ModelMatrixBindingMode::Uniform)
            .then(|| ModelMatrixUniform::bind_group_layout(&device));
        let material_bind_group_layout = GpuMaterial::bind_group_layout(&device);

        let vertex_shader = create_light_shader_module(&device, model_binding_mode);
        let no_light_vertex_shader = create_no_light_shader_module(&device, model_binding_mode);
        let bind_group_layouts =
            if let Some(model_bind_group_layout) = model_bind_group_layout.as_ref() {
                vec![
                    Some(&camera_bind_group_layout),
                    Some(&light_bind_group_layout),
                    Some(model_bind_group_layout),
                    Some(&material_bind_group_layout),
                ]
            } else {
                vec![
                    Some(&camera_bind_group_layout),
                    Some(&light_bind_group_layout),
                    Some(&material_bind_group_layout),
                ]
            };
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &bind_group_layouts,
                immediate_size: if model_binding_mode == ModelMatrixBindingMode::Immediate {
                    Self::IMMEDIATE_MODEL_MATRIX_SIZE
                } else {
                    0
                },
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
            model_bind_group_layout,
            material_bind_group_layout,
            model_binding_mode,
            queue,
        })
    }

    pub fn resize(&mut self, size: Size) -> Result<()> {
        self.size = size;

        if size.is_zero() {
            warn!(
                "Ignoring resize because wgpu surfaces cannot be configured with zero width or height: {}x{}",
                size.width, size.height
            );
            return Ok(());
        }

        let Some(surface) = self.surface.as_ref() else {
            self.depth_texture =
                Texture::create_depth_texture(Self::DEPTH_TEXTURE_LABEL, &self.device, &self.size);
            return Ok(());
        };

        let Some(surface_configuration) = self.surface_configuration.as_mut() else {
            return Err(anyhow!(
                "Cannot resize render surface because the surface configuration is missing"
            ));
        };

        surface_configuration.width = size.width;
        surface_configuration.height = size.height;
        surface.configure(&self.device, surface_configuration);
        self.depth_texture =
            Texture::create_depth_texture(Self::DEPTH_TEXTURE_LABEL, &self.device, &self.size);

        Ok(())
    }
}

fn select_model_binding_mode(adapter: &wgpu::Adapter) -> ModelMatrixBindingMode {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = adapter;
        ModelMatrixBindingMode::Uniform
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let supported_features = adapter.features();
        if supported_features
            .features_webgpu
            .contains(FeaturesWebGPU::IMMEDIATES)
        {
            ModelMatrixBindingMode::Immediate
        } else {
            ModelMatrixBindingMode::Uniform
        }
    }
}

fn required_features_for(model_binding_mode: ModelMatrixBindingMode) -> Features {
    if model_binding_mode == ModelMatrixBindingMode::Immediate {
        Features {
            features_webgpu: FeaturesWebGPU::IMMEDIATES,
            ..Default::default()
        }
    } else {
        Features::default()
    }
}

fn required_limits_for(model_binding_mode: ModelMatrixBindingMode) -> Limits {
    if model_binding_mode == ModelMatrixBindingMode::Immediate {
        Limits {
            max_immediate_size: RenderContext::IMMEDIATE_MODEL_MATRIX_SIZE,
            ..Default::default()
        }
    } else {
        Limits::default()
    }
}

fn create_light_shader_module(
    device: &Device,
    model_binding_mode: ModelMatrixBindingMode,
) -> wgpu::ShaderModule {
    match model_binding_mode {
        ModelMatrixBindingMode::Immediate => {
            device.create_shader_module(include_wgsl!("../../assets/vertex.wgsl"))
        }
        ModelMatrixBindingMode::Uniform => {
            device.create_shader_module(include_wgsl!("../../assets/vertex_uniform.wgsl"))
        }
    }
}

fn create_no_light_shader_module(
    device: &Device,
    model_binding_mode: ModelMatrixBindingMode,
) -> wgpu::ShaderModule {
    match model_binding_mode {
        ModelMatrixBindingMode::Immediate => {
            device.create_shader_module(include_wgsl!("../../assets/no_light_vertex.wgsl"))
        }
        ModelMatrixBindingMode::Uniform => {
            device.create_shader_module(include_wgsl!("../../assets/no_light_vertex_uniform.wgsl"))
        }
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

            let configured_size = size.clamp_size_for_gpu();

            let surface_configuration = SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format,
                width: configured_size.width,
                height: configured_size.height,
                present_mode: capabilities.present_modes[0],
                desired_maximum_frame_latency: 2,
                alpha_mode: capabilities.alpha_modes[0],
                view_formats: vec![],
            };

            if !size.is_zero() {
                surface.configure(device, &surface_configuration);
            }

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
        if std::env::var("HYAKOU_RUN_GPU_TESTS").ok().as_deref() != Some("1") {
            eprintln!(
                "Skipping GPU-dependent test create_context; set HYAKOU_RUN_GPU_TESTS=1 to enable."
            );
            return;
        }
        let ctx = pollster::block_on(RenderContext::new::<MockSurfaceProvider>(None));
        assert!(ctx.is_ok());
    }
}
