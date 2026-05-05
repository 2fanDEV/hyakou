use std::fmt::Debug;

pub trait Component: 'static + Debug {}

#[cfg(test)]
#[path = "tests/component_trait_test.rs"]
mod component_trait_test;
