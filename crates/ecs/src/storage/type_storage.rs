use std::any::TypeId;
use std::collections::HashMap;

use fxhash::FxHashMap;

use crate::Storage;

#[derive(Debug, Default)]
pub struct TypeStorage {
    map: FxHashMap<TypeId, Box<dyn Storage>>,
}

impl TypeStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get<T: Storage + 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())?
            .as_any()
            .downcast_ref::<T>()
    }

    pub fn get_mut<T: Storage + 'static>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())?
            .as_any_mut()
            .downcast_mut::<T>()
    }

    pub fn contains<T: Storage + 'static>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }

    pub fn insert<T: Storage + 'static>(&mut self, value: T) -> Option<T> {
        self.map
            .insert(TypeId::of::<T>(), Box::new(value))
            .map(|storage| {
                *storage
                    .into_any()
                    .downcast::<T>()
                    .expect("TypeStorage type mismatch")
            })
    }

    pub fn remove<T: Storage + 'static>(&mut self) -> Option<T> {
        self.map.remove(&TypeId::of::<T>()).map(|storage| {
            *storage
                .into_any()
                .downcast::<T>()
                .expect("TypeStorage type mismatch")
        })
    }

    pub fn get_or_insert_with<T: Storage + 'static, F: FnOnce() -> T>(&mut self, f: F) -> &mut T {
        self.map
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(f()))
            .as_any_mut()
            .downcast_mut::<T>()
            .expect("TypeStorage type mismatch")
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut dyn Storage> + '_ {
        self.map
            .values_mut()
            .map(|storage| storage.as_mut() as &mut dyn Storage)
    }

    pub fn filter<'a, F: Fn(&dyn Storage) -> bool + 'a>(
        &'a mut self,
        f: F,
    ) -> impl Iterator<Item = &'a mut dyn Storage> {
        self.map
            .values_mut()
            .filter(move |storage| f(storage.as_ref()))
            .map(|storage| storage.as_mut() as &mut dyn Storage)
    }
}
