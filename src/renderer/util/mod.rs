

pub type Width = u32;
pub type Height = u32;

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: Width,
    pub height: Height
}

pub trait Concatable {
    fn concat(&mut self, text: &str) -> &str;
}

impl Concatable for String {
    fn concat(&mut self, text: &str) -> &str {
        self.push_str(text);
        self.as_str()
    }
}