pub trait Component: 'static {}

#[cfg(test)]
#[path = "tests/component_trait_test.rs"]
mod component_trait_test;
