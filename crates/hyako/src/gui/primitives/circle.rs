use egui::{Color32, Pos2, Shape, Stroke};

use crate::gui::primitives::Primitive2d;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilledCircle {
    pub center: Pos2,
    pub radius: f32,
    pub color: Color32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutlinedCircle {
    pub center: Pos2,
    pub radius: f32,
    pub stroke: Stroke,
}

impl FilledCircle {
    pub fn new(center: Pos2, radius: f32, color: Color32) -> Self {
        Self {
            center,
            radius,
            color,
        }
    }
}

impl OutlinedCircle {
    pub fn new(center: Pos2, radius: f32, stroke: Stroke) -> Self {
        Self {
            center,
            radius,
            stroke,
        }
    }
}

impl Primitive2d for FilledCircle {
    fn to_shape(&self) -> Shape {
        Shape::circle_filled(self.center, self.radius, self.color)
    }
}

impl Primitive2d for OutlinedCircle {
    fn to_shape(&self) -> Shape {
        Shape::circle_stroke(self.center, self.radius, self.stroke)
    }
}

#[cfg(test)]
mod tests {
    use egui::{Color32, Pos2, Shape, Stroke};

    use crate::gui::primitives::{FilledCircle, OutlinedCircle, Primitive2d};

    #[test]
    fn test_filled_circle_converts_to_circle_shape() {
        let primitive = FilledCircle::new(Pos2::new(10.0, 20.0), 8.0, Color32::GREEN);

        let Shape::Circle(shape) = primitive.to_shape() else {
            panic!("expected circle shape");
        };

        assert_eq!(shape.center, Pos2::new(10.0, 20.0));
        assert_eq!(shape.radius, 8.0);
        assert_eq!(shape.fill, Color32::GREEN);
        assert!(shape.stroke.is_empty());
    }

    #[test]
    fn test_outlined_circle_converts_to_circle_shape() {
        let stroke = Stroke::new(3.0, Color32::YELLOW);
        let primitive = OutlinedCircle::new(Pos2::new(10.0, 20.0), 8.0, stroke);

        let Shape::Circle(shape) = primitive.to_shape() else {
            panic!("expected circle shape");
        };

        assert_eq!(shape.center, Pos2::new(10.0, 20.0));
        assert_eq!(shape.radius, 8.0);
        assert_eq!(shape.fill, Color32::TRANSPARENT);
        assert_eq!(shape.stroke, stroke);
    }
}
