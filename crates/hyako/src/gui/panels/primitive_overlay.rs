use egui::{Context, Id, LayerId, Order};

use crate::gui::primitives::{Primitive, Primitive2d};

#[derive(Debug, Default)]
pub struct PrimitiveOverlay {
    pub primitives: Vec<Primitive>,
}

impl PrimitiveOverlay {
    pub fn new(primitives: Vec<Primitive>) -> Self {
        Self { primitives }
    }

    pub fn should_be_rendered(&self) -> bool {
        true
    }

    pub fn show(&self, ctx: &Context) {
        if !self.should_be_rendered() {
            return;
        }

        let painter = ctx.layer_painter(LayerId::new(
            Order::Foreground,
            Id::new("primitive_overlay"),
        ));
        for primitive in &self.primitives {
            painter.add(primitive.to_shape());
        }
    }
}

#[cfg(test)]
mod tests {
    use egui::{Color32, Pos2, Stroke};

    use crate::gui::panels::primitive_overlay::PrimitiveOverlay;
    use crate::gui::primitives::{FilledCircle, LineSegment};

    #[test]
    fn test_overlay_defaults_to_no_primitives() {
        let overlay = PrimitiveOverlay::default();

        assert!(overlay.primitives.is_empty());
        assert!(overlay.should_be_rendered());
    }

    #[test]
    fn test_overlay_stores_supplied_primitives() {
        let overlay = PrimitiveOverlay::new(vec![
            FilledCircle::new(Pos2::new(10.0, 10.0), 5.0, Color32::RED).into(),
            LineSegment::new(
                Pos2::new(0.0, 0.0),
                Pos2::new(10.0, 10.0),
                Stroke::new(2.0, Color32::WHITE),
            )
            .into(),
        ]);

        assert_eq!(overlay.primitives.len(), 2);
        assert!(overlay.should_be_rendered());
    }
}
