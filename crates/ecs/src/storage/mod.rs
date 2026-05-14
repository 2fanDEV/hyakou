use std::any::Any;
use std::fmt::Debug;

pub trait Storage: Debug + Send {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    fn remove_any_key(&mut self, _key: &dyn Any) -> bool {
        false
    }

    fn has_outstanding_commands(&self) -> bool {
        false
    }

    fn apply_outstanding_commands(&mut self) {
        debug!("apply_outstanding_commands not implemented!: {:?}", self);
    }
}

pub trait KeyedStorage<K>: Storage {
    fn remove_key(&mut self, key: &K) -> bool;
}

mod type_storage;
use log::debug;
pub use type_storage::TypeStorage;
