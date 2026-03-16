use crate::types::shared::{Coordinates, FileInformation};

pub enum Event {
    SetCoords(Coordinates),
    UploadFile(FileInformation),
}
