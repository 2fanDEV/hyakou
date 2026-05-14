use crate::types::BaseId;
pub use ::shared::id::Id;

impl BaseId for Id {
    fn get_id(&self) -> &str {
        self.as_ref()
    }
}
