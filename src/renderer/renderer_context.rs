use anyhow::Result;
use wgpu::{Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance, InstanceDescriptor, Limits, MemoryHints, Queue, RequestAdapterOptions, Surface};
use winit::window::Window;

pub struct RendererContext {
    pub instance: Instance,
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue
}

impl RendererContext {
    pub async fn new(window: &Window) -> Result<Self> {
        let instance = wgpu::Instance::new(&InstanceDescriptor::from_env_or_default());
        let surface = instance.create_surface(window)?;
        let adapter = instance.request_adapter(&RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false ,
            compatible_surface: Some(&surface),
        }).await?;

        let (device, queue) = adapter.request_device(&DeviceDescriptor {
            label: Some("Hyakou Device"),
            required_features: Features::default(), 
            required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
            experimental_features: ExperimentalFeatures::default(),
            memory_hints: MemoryHints::MemoryUsage,
            trace: wgpu::Trace::Off,
        }).await?;

        Ok(Self {
            instance,
            surface,
            device,
            queue
        })
    }
}
