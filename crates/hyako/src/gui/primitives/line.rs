use egui::{Pos2, Shape, Stroke};

use crate::gui::primitives::Primitive2d;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LineSegment {
    pub start: Pos2,
    pub end: Pos2,
    pub stroke: Stroke,
}

impl LineSegment {
    pub fn new(start: Pos2, end: Pos2, stroke: Stroke) -> Self {
        Self { start, end, stroke }
    }
}

impl Primitive2d for LineSegment {
    fn to_shape(&self) -> Shape {
        Shape::line_segment([self.start, self.end], self.stroke)
    }
}

#[cfg(test)]
mod tests {
    use egui::{Color32, Pos2, Shape, Stroke};

    use crate::gui::primitives::{LineSegment, Primitive2d};

    #[test]
    fn test_line_segment_converts_to_line_segment_shape() {
        let start = Pos2::new(1.0, 2.0);
        let end = Pos2::new(3.0, 4.0);
        let stroke = Stroke::new(4.0, Color32::WHITE);
        let primitive = LineSegment::new(start, end, stroke);

        let Shape::LineSegment { points, stroke } = primitive.to_shape() else {
            panic!("expected line segment shape");
        };

        assert_eq!(points, [start, end]);
        assert_eq!(stroke, Stroke::new(4.0, Color32::WHITE));
    }
}
