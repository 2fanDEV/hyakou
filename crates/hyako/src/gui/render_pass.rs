use wgpu::{LoadOp, Operations, RenderPassDepthStencilAttachment, StoreOp};

pub fn color_attachment_operations() -> Operations<wgpu::Color> {
    Operations {
        load: LoadOp::Load,
        store: StoreOp::Store,
    }
}

pub fn depth_stencil_attachment<'a>() -> Option<RenderPassDepthStencilAttachment<'a>> {
    None
}

#[cfg(test)]
mod tests {
    use wgpu::{LoadOp, StoreOp};

    use crate::gui::render_pass::{color_attachment_operations, depth_stencil_attachment};

    #[test]
    fn test_egui_render_pass_loads_existing_color_attachment() {
        let operations = color_attachment_operations();

        assert!(matches!(operations.load, LoadOp::Load));
        assert!(matches!(operations.store, StoreOp::Store));
    }

    #[test]
    fn test_egui_render_pass_has_no_depth_attachment() {
        assert!(depth_stencil_attachment().is_none());
    }
}
