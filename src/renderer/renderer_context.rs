use anyhow::Result;
use wgpu::{
    Backends, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance,
    InstanceDescriptor, InstanceFlags, Limits, MemoryHints, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureUsages,
};

use crate::renderer::{util::Size, wrappers::SurfaceProvider};

pub struct RendererContext {
    pub instance: Instance,
    pub surface: Option<Surface<'static>>,
    pub surface_configuration: Option<SurfaceConfiguration>,
    pub device: Device,
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
            Some(surface_ref) => {
                init_surface_configuration(Some(surface_ref), adapter, provider.unwrap().get_size(), &device)
            },
            None => None,
        };


        Ok(Self {
            instance,
            surface,
            surface_configuration,
            device,
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

//requires winit window, no test
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
