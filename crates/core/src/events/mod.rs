use crate::{
    components::{AssetType, camera::data_structures::CameraAnimationRequest},
    types::shared::{AssetBundleInformation, AssetInformation},
};

pub enum Event {
    AnimateCamera(CameraAnimationRequest),
    StopCameraAnimation,
    AssetUpload(AssetInformation, AssetType),
    AssetBundleUpload(AssetBundleInformation, AssetType),
    Resize(f64, f64),
}
