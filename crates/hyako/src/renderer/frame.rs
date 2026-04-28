use wgpu::{CommandEncoder, Queue, SurfaceTexture, TextureView};

pub struct FrameTarget<'a> {
    pub encoder: &'a mut CommandEncoder,
    pub queue: &'a Queue,
    pub color_view: &'a TextureView,
    pub depth_view: &'a TextureView,
    pub size_in_pixels: [u32; 2],
}

pub struct SurfaceFrame {
    output: SurfaceTexture,
    encoder: CommandEncoder,
    queue: Queue,
    color_view: TextureView,
    depth_view: TextureView,
    size_in_pixels: [u32; 2],
    should_reconfigure_surface: bool,
}

impl SurfaceFrame {
    pub fn new(
        output: SurfaceTexture,
        encoder: CommandEncoder,
        queue: Queue,
        color_view: TextureView,
        depth_view: TextureView,
        size_in_pixels: [u32; 2],
        should_reconfigure_surface: bool,
    ) -> Self {
        Self {
            output,
            encoder,
            queue,
            color_view,
            depth_view,
            size_in_pixels,
            should_reconfigure_surface,
        }
    }

    pub fn target(&mut self) -> FrameTarget<'_> {
        FrameTarget {
            encoder: &mut self.encoder,
            queue: &self.queue,
            color_view: &self.color_view,
            depth_view: &self.depth_view,
            size_in_pixels: self.size_in_pixels,
        }
    }

    pub fn finish(self) -> bool {
        self.queue.submit(std::iter::once(self.encoder.finish()));
        self.output.present();
        self.should_reconfigure_surface
    }
}
