use crate::{
    components::{LightType, camera::data_structures::CameraAnimationRequest},
    types::shared::{AssetBundleInformation, AssetInformation},
};

pub enum Event {
    AnimateCamera(CameraAnimationRequest),
    StopCameraAnimation,
    AssetUpload(AssetInformation, LightType),
    AssetBundleUpload(AssetBundleInformation, LightType),
    Resize(f64, f64),
}
