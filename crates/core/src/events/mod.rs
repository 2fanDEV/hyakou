use crate::types::shared::{AssetInformation, Coordinates};

pub enum Event {
    SetCoords(Coordinates),
    AssetUpload(AssetInformation),
}
