use crate::{
    components::{camera::data_structures::CameraAnimationRequest, LightType},
    types::shared::AssetInformation,
};

pub enum Event {
    AnimateCamera(CameraAnimationRequest),
    StopCameraAnimation,
    AssetUpload(AssetInformation, LightType),
    Resize(f64, f64),
}
