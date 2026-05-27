use crate::renderer::actions::camera_actions::CameraHandlerAction;
use hyakou_core::components::camera::data_structures::CameraMode;
use log::debug;

pub struct CameraModeHandler {
    camera_mode: CameraMode,
}

impl CameraModeHandler {
    pub fn new(mode: CameraMode) -> Self {
        Self { camera_mode: mode }
    }

    pub fn set(&mut self, mode: CameraMode) {
        self.camera_mode = mode;
    }

    pub fn mode(&self) -> &CameraMode {
        &self.camera_mode
    }

    pub fn handle_action(&mut self, action: &CameraHandlerAction, is_pressed: bool) {
        if is_pressed {
            self.switch_camera(action);
        }
    }

    pub fn switch_camera(&mut self, action: &CameraHandlerAction) {
        let variants = CameraMode::get_variants();
        if variants.is_empty() {
            return;
        }

        let current_index = variants
            .iter()
            .position(|mode| self.camera_mode.eq(mode))
            .unwrap_or(0);

        let next_index = match action {
            CameraHandlerAction::SwitchCameraModeForward => {
                if current_index == variants.len() - 1 {
                    0
                } else {
                    current_index + 1
                }
            }
            CameraHandlerAction::SwitchCameraModeBackwards => {
                if current_index == 0 {
                    variants.len() - 1
                } else {
                    current_index - 1
                }
            }
        };

        if let Some(next_mode) = variants.get(next_index) {
            self.camera_mode = next_mode.clone();
            match action {
                CameraHandlerAction::SwitchCameraModeForward => {
                    debug!("Forwards camera mode switch");
                }
                CameraHandlerAction::SwitchCameraModeBackwards => {
                    debug!("backwards camera mode switch");
                }
            }
            debug!("{:?}", next_index);
        }
    }
}
