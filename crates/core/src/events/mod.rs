use crate::types::shared::{AssetInformation, Coordinates3};

pub enum Event {
    SetCoords(Coordinates3),
    AssetUpload(AssetInformation),
}
