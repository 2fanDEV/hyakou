#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

#[cfg(not(target_arch = "wasm32"))]
use parking_lot::RwLock;

pub mod events;
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
    fn try_read_shared<F>(&self, f: impl FnOnce(&T) -> F) -> Option<F>;
    fn try_write_shared<F>(&self, f: impl FnOnce(&mut T) -> F) -> Option<F>;
}

#[cfg(target_arch = "wasm32")]
impl<T> SharedAccess<T> for Rc<RefCell<T>> {
    fn read_shared<F>(&self, f: impl FnOnce(&T) -> F) -> F {
        f(&self.borrow())
    }

    fn write_shared<F>(&self, f: impl FnOnce(&mut T) -> F) -> F {
        f(&mut self.borrow_mut())
    }

    fn try_read_shared<F>(&self, f: impl FnOnce(&T) -> F) -> Option<F> {
        self.try_borrow().map(|g| f(&g)).ok()
    }

    fn try_write_shared<F>(&self, f: impl FnOnce(&mut T) -> F) -> Option<F> {
        self.try_borrow_mut().map(|mut g| f(&mut g)).ok()
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

    fn try_read_shared<F>(&self, f: impl FnOnce(&T) -> F) -> Option<F> {
        self.try_read().map(|guard| f(&guard))
    }

    fn try_write_shared<F>(&self, f: impl FnOnce(&mut T) -> F) -> Option<F> {
        self.try_write().map(|mut guard| f(&mut guard))
    }
}
