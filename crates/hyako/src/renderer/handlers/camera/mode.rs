use crate::renderer::actions::camera_actions::CameraHandlerAction;
use hyakou_core::components::camera::data_structures::CameraMode;
use log::debug;

pub struct CameraModeHandler {
    camera_mode: CameraMode,
    camera_mode_index: usize,
}

impl CameraModeHandler {
    pub fn new(mode: CameraMode) -> Self {
        let index = CameraMode::get_variants().iter().position(|p| mode.eq(p));
        Self {
            camera_mode: mode,
            camera_mode_index: index.unwrap(),
        }
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
        self.camera_mode = match action {
            CameraHandlerAction::SwitchCameraModeForward => {
                let variants = CameraMode::get_variants();
                debug!("{:?}", variants.len());
                self.camera_mode_index = if self.camera_mode_index == variants.len() - 1 {
                    0
                } else {
                    self.camera_mode_index + 1
                };
                debug!("Forwards camera mode switch");
                debug!("{:?}", self.camera_mode_index);
                variants.get(self.camera_mode_index).unwrap().clone()
            }
            CameraHandlerAction::SwitchCameraModeBackwards => {
                let variants = CameraMode::get_variants();
                self.camera_mode_index = if self.camera_mode_index == 0 {
                    variants.len() - 1
                } else {
                    self.camera_mode_index - 1
                };
                debug!("backwards camera mode switch");
                debug!("{:?}", self.camera_mode_index);
                variants.get(self.camera_mode_index).unwrap().clone()
            }
        }
    }
}
