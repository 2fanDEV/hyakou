use crate::{
    components::{LightType, camera::data_structures::CameraAnimationRequest},
    types::shared::AssetInformation,
};

pub enum Event {
    AnimateCamera(CameraAnimationRequest),
    StopCameraAnimation,
    AssetUpload(AssetInformation, LightType),
    Resize(f64, f64),
}
