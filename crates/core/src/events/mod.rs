use crate::{
    components::{
        AssetType,
        camera::data_structures::{CameraAnimationRequest, CameraMode},
    },
    types::shared::{AssetBundleInformation, AssetInformation},
};

pub enum Event {
    AnimateCamera(CameraAnimationRequest),
    StopCameraAnimation,
    SetCameraMode(CameraMode),
    AssetUpload(AssetInformation, AssetType),
    AssetBundleUpload(AssetBundleInformation, AssetType),
    Resize(f64, f64),
}
