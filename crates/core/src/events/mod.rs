use crate::{
    components::camera::data_structures::CameraAnimationRequest, types::shared::AssetInformation,
};

pub enum Event {
    AnimateCamera(CameraAnimationRequest),
    StopCameraAnimation,
    AssetUpload(AssetInformation),
    Resize(f64, f64),
}
