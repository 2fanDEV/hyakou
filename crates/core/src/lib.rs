#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
#[cfg(not(target_arch = "wasm32"))]
use anyhow::anyhow;
#[cfg(target_arch = "wasm32")]
use anyhow::anyhow;
#[cfg(not(target_arch = "wasm32"))]
use parking_lot::RwLock;

pub mod animations;
pub mod components;
pub mod events;
pub mod geometry;
pub mod traits;
pub mod types;

#[cfg(target_arch = "wasm32")]
pub type Shared<T> = Rc<RefCell<T>>;

#[cfg(not(target_arch = "wasm32"))]
pub type Shared<T> = Arc<RwLock<T>>;

pub fn shared<T>(elem: T) -> Shared<T> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Arc::new(RwLock::new(elem))
    }
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(RefCell::new(elem))
    }
}

pub trait SharedAccess<T> {
    fn read_shared<F>(&self, f: impl FnOnce(&T) -> F) -> F;
    fn write_shared<F>(&self, f: impl FnOnce(&mut T) -> F) -> F;
    fn try_read_shared<F>(&self, f: impl FnOnce(&T) -> F) -> Result<F>;
    fn try_write_shared<F>(&self, f: impl FnOnce(&mut T) -> F) -> Result<F>;
}

#[cfg(target_arch = "wasm32")]
impl<T> SharedAccess<T> for Rc<RefCell<T>> {
    fn read_shared<F>(&self, f: impl FnOnce(&T) -> F) -> F {
        f(&self.borrow())
    }

    fn write_shared<F>(&self, f: impl FnOnce(&mut T) -> F) -> F {
        f(&mut self.borrow_mut())
    }

    fn try_read_shared<F>(&self, f: impl FnOnce(&T) -> F) -> Result<F> {
        self.try_borrow().map(|g| f(&g)).map_err(|e| anyhow!(e))
    }

    fn try_write_shared<F>(&self, f: impl FnOnce(&mut T) -> F) -> Result<F> {
        self.try_borrow_mut()
            .map(|mut g| f(&mut g))
            .map_err(|e| anyhow!(e))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl<T> SharedAccess<T> for Arc<RwLock<T>> {
    fn read_shared<F>(&self, f: impl FnOnce(&T) -> F) -> F {
        let guard = self.read();
        f(&guard)
    }

    fn write_shared<F>(&self, f: impl FnOnce(&mut T) -> F) -> F {
        let mut guard = self.write();
        f(&mut guard)
    }

    fn try_read_shared<F>(&self, f: impl FnOnce(&T) -> F) -> Result<F> {
        self.try_read()
            .map(|guard| f(&guard))
            .ok_or_else(|| anyhow!("Failed to read shared object"))
    }

    fn try_write_shared<F>(&self, f: impl FnOnce(&mut T) -> F) -> Result<F> {
        self.try_write()
            .map(|mut guard| f(&mut guard))
            .ok_or_else(|| anyhow!("Failed to write to shared object"))
    }
}
