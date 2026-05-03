pub trait Component: 'static {}

#[cfg(test)]
#[path = "../tests/component.rs"]
mod tests;
