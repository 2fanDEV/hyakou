use egui::{Color32, CornerRadius, Rect, Shape, Stroke, StrokeKind};

use crate::gui::primitives::Primitive2d;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FilledRect {
    pub rect: Rect,
    pub corner_radius: CornerRadius,
    pub color: Color32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OutlinedRect {
    pub rect: Rect,
    pub corner_radius: CornerRadius,
    pub stroke: Stroke,
    pub stroke_kind: StrokeKind,
}

impl FilledRect {
    pub fn new(rect: Rect, color: Color32) -> Self {
        Self {
            rect,
            corner_radius: CornerRadius::ZERO,
            color,
        }
    }
}

impl OutlinedRect {
    pub fn new(rect: Rect, stroke: Stroke) -> Self {
        Self {
            rect,
            corner_radius: CornerRadius::ZERO,
            stroke,
            stroke_kind: StrokeKind::Middle,
        }
    }
}

impl Primitive2d for FilledRect {
    fn to_shape(&self) -> Shape {
        Shape::rect_filled(self.rect, self.corner_radius, self.color)
    }
}

impl Primitive2d for OutlinedRect {
    fn to_shape(&self) -> Shape {
        Shape::rect_stroke(self.rect, self.corner_radius, self.stroke, self.stroke_kind)
    }
}

#[cfg(test)]
mod tests {
    use egui::{Color32, Pos2, Rect, Shape, Stroke};

    use crate::gui::primitives::{FilledRect, OutlinedRect, Primitive2d};

    #[test]
    fn test_filled_rect_converts_to_rect_shape() {
        let rect = Rect::from_min_max(Pos2::new(1.0, 2.0), Pos2::new(3.0, 4.0));
        let primitive = FilledRect::new(rect, Color32::RED);

        let Shape::Rect(shape) = primitive.to_shape() else {
            panic!("expected rect shape");
        };

        assert_eq!(shape.rect, rect);
        assert_eq!(shape.fill, Color32::RED);
    }

    #[test]
    fn test_outlined_rect_converts_to_rect_shape() {
        let rect = Rect::from_min_max(Pos2::new(1.0, 2.0), Pos2::new(3.0, 4.0));
        let stroke = Stroke::new(2.0, Color32::BLUE);
        let primitive = OutlinedRect::new(rect, stroke);

        let Shape::Rect(shape) = primitive.to_shape() else {
            panic!("expected rect shape");
        };

        assert_eq!(shape.rect, rect);
        assert_eq!(shape.stroke, stroke);
    }
}
