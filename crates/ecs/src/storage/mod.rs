use std::any::Any;
use std::fmt::Debug;

pub trait Storage: Debug {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;

    fn remove_any_key(&mut self, _key: &dyn Any) -> bool {
        false
    }
}

pub trait KeyedStorage<K>: Storage {
    fn remove_key(&mut self, key: &K) -> bool;
}

mod type_storage;
pub use type_storage::TypeStorage;
