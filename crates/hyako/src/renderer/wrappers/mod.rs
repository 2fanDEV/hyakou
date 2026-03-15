use std::sync::Arc;

use mockall::automock;
use wgpu::{Instance, Surface};
use winit::window::Window;

use crate::renderer::util::Size;

#[automock]
pub trait SurfaceProvider {
    fn create_surface(&self, instance: &Instance) -> Option<Surface<'static>>;
    fn get_size(&self) -> Size;
}

pub struct WinitSurfaceProvider {
    pub window: Arc<Window>,
}

impl SurfaceProvider for WinitSurfaceProvider {
    fn create_surface(&self, instance: &Instance) -> Option<wgpu::Surface<'static>> {
        match instance.create_surface(self.window.clone()) {
            Ok(surface) => Some(surface),
            Err(_) => None,
        }
    }

    fn get_size(&self) -> Size {
        let size = self.window.inner_size();
        Size {
            width: size.width,
            height: size.height,
        }
    }
}
