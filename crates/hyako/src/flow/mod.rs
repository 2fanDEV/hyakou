pub mod asset;
pub mod command_sender;
pub mod commands;
pub mod flow;
pub mod frame_composer;
pub mod input_controller;
pub mod render_controller;

use hyakou_core::types::ids::MeshId;

pub use crate::renderer::handlers::camera::CameraController;
pub use asset::{AssetController, AssetUploadController};
pub use command_sender::FlowCommandSender;
pub use commands::FlowCommand;
pub use flow::{FlowController, FlowHandle};
pub use frame_composer::FrameComposer;
pub use input_controller::InputController;
pub use render_controller::RenderController;

#[derive(Clone, Copy)]
pub struct SceneFrameInput<'a> {
    pub outlined_mesh_ids: &'a [MeshId],
}
