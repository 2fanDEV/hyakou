use std::sync::Arc;

use anyhow::Result;
use hyakou_core::types::{ModelMatrixBindingMode, Size};
use wgpu::{
    Backends, BindGroupLayout, Device, DeviceDescriptor, ExperimentalFeatures, Features,
    FeaturesWebGPU, Instance, InstanceDescriptor, InstanceFlags, Limits, MemoryHints, Queue,
    RenderPipeline, RequestAdapterOptions, Surface, SurfaceConfiguration,
};

use crate::{
    gpu::texture::Texture,
    renderer::{pipelines, surface, wrappers::SurfaceProvider},
};

pub struct RenderContext {
    pub instance: Instance,
    pub surface: Option<Surface<'static>>,
    pub surface_configuration: Option<SurfaceConfiguration>,
    pub device: Arc<Device>,
    pub light_render_pipeline: RenderPipeline,
    pub no_light_render_pipeline: RenderPipeline,
    pub outline_render_pipeline: RenderPipeline,
    pub size: Size,
    pub camera_bind_group_layout: BindGroupLayout,
    pub light_bind_group_layout: BindGroupLayout,
    pub model_bind_group_layout: Option<BindGroupLayout>,
    pub material_bind_group_layout: BindGroupLayout,
    pub outline_bind_group_layout: BindGroupLayout,
    pub model_binding_mode: ModelMatrixBindingMode,
    pub depth_texture: Texture,
    pub queue: Arc<Queue>,
}

impl RenderContext {
    pub(super) const IMMEDIATE_MODEL_MATRIX_SIZE: u32 = 64;
    pub(super) const DEPTH_TEXTURE_LABEL: &str = "Depth Texture";

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
        let queue = Arc::new(queue);

        let size = provider.as_ref().map_or(
            Size {
                width: 1920,
                height: 1080,
            },
            SurfaceProvider::get_size,
        );

        let surface_configuration = match surface.as_ref() {
            Some(surface_ref) => {
                surface::init_surface_configuration(Some(surface_ref), &adapter, size, &device)
            }
            None => None,
        };

        let depth_texture =
            Texture::create_depth_texture(Self::DEPTH_TEXTURE_LABEL, &device, &size);

        let pipeline_resources = pipelines::create_pipeline_resources(
            &device,
            model_binding_mode,
            surface::surface_format(surface_configuration.as_ref()),
        );

        Ok(Self {
            instance,
            surface,
            surface_configuration,
            device,
            light_render_pipeline: pipeline_resources.light_render_pipeline,
            no_light_render_pipeline: pipeline_resources.no_light_render_pipeline,
            outline_render_pipeline: pipeline_resources.outline_render_pipeline,
            size,
            depth_texture,
            light_bind_group_layout: pipeline_resources.light_bind_group_layout,
            camera_bind_group_layout: pipeline_resources.camera_bind_group_layout,
            model_bind_group_layout: pipeline_resources.model_bind_group_layout,
            material_bind_group_layout: pipeline_resources.material_bind_group_layout,
            outline_bind_group_layout: pipeline_resources.outline_bind_group_layout,
            model_binding_mode,
            queue,
        })
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
