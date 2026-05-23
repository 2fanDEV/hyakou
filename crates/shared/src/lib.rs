use std::sync::{Arc, RwLock};

use anyhow::{Result, anyhow};

pub mod id;

pub type Shared<T> = Arc<RwLock<T>>;

pub fn shared<T>(elem: T) -> Shared<T> {
    Arc::new(RwLock::new(elem))
}

pub trait SharedAccess<T> {
    fn read_shared<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R;

    fn write_shared<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R;

    fn try_read_shared<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&T) -> R;

    fn try_write_shared<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut T) -> R;
}

impl<T> SharedAccess<T> for Arc<RwLock<T>> {
    fn read_shared<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        f(&self.read().expect("RwLock poisoned"))
    }

    fn write_shared<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        f(&mut self.write().expect("RwLock poisoned"))
    }

    fn try_read_shared<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&T) -> R,
    {
        self.try_read()
            .map(|guard| f(&guard))
            .map_err(|e| anyhow!("{e}"))
    }

    fn try_write_shared<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.try_write()
            .map(|mut guard| f(&mut guard))
            .map_err(|e| anyhow!("{e}"))
    }
}
