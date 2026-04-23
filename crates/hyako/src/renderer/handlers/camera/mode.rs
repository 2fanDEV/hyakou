use hyakou_core::components::camera::data_structures::CameraMode;

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
}
