use egui::Shape;

pub mod circle;
pub mod line;
pub mod rect;

pub use circle::{FilledCircle, OutlinedCircle};
pub use line::LineSegment;
pub use rect::{FilledRect, OutlinedRect};

pub trait Primitive2d {
    fn to_shape(&self) -> Shape;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapePrimitive {
    shape: Shape,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    FilledRect(FilledRect),
    OutlinedRect(OutlinedRect),
    FilledCircle(FilledCircle),
    OutlinedCircle(OutlinedCircle),
    LineSegment(LineSegment),
    Shape(ShapePrimitive),
}

impl ShapePrimitive {
    pub fn new(shape: Shape) -> Self {
        Self { shape }
    }
}

impl Primitive2d for ShapePrimitive {
    fn to_shape(&self) -> Shape {
        self.shape.clone()
    }
}

impl Primitive2d for Primitive {
    fn to_shape(&self) -> Shape {
        match self {
            Primitive::FilledRect(primitive) => primitive.to_shape(),
            Primitive::OutlinedRect(primitive) => primitive.to_shape(),
            Primitive::FilledCircle(primitive) => primitive.to_shape(),
            Primitive::OutlinedCircle(primitive) => primitive.to_shape(),
            Primitive::LineSegment(primitive) => primitive.to_shape(),
            Primitive::Shape(primitive) => primitive.to_shape(),
        }
    }
}

impl From<FilledRect> for Primitive {
    fn from(primitive: FilledRect) -> Self {
        Self::FilledRect(primitive)
    }
}

impl From<OutlinedRect> for Primitive {
    fn from(primitive: OutlinedRect) -> Self {
        Self::OutlinedRect(primitive)
    }
}

impl From<FilledCircle> for Primitive {
    fn from(primitive: FilledCircle) -> Self {
        Self::FilledCircle(primitive)
    }
}

impl From<OutlinedCircle> for Primitive {
    fn from(primitive: OutlinedCircle) -> Self {
        Self::OutlinedCircle(primitive)
    }
}

impl From<LineSegment> for Primitive {
    fn from(primitive: LineSegment) -> Self {
        Self::LineSegment(primitive)
    }
}

impl From<ShapePrimitive> for Primitive {
    fn from(primitive: ShapePrimitive) -> Self {
        Self::Shape(primitive)
    }
}

#[cfg(test)]
mod tests {
    use egui::{Color32, Pos2, Shape};

    use crate::gui::primitives::{Primitive2d, ShapePrimitive};

    #[test]
    fn test_shape_primitive_returns_wrapped_shape() {
        let shape = Shape::circle_filled(Pos2::new(4.0, 5.0), 3.0, Color32::RED);
        let primitive = ShapePrimitive::new(shape.clone());

        assert_eq!(primitive.to_shape(), shape);
    }
}
