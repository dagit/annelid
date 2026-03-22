use eframe::egui;

use super::layout_meta::{LayoutMeta, WindowGeometry};

const DEFAULT_WIDTH: f32 = 800.0;
const DEFAULT_HEIGHT: f32 = 600.0;

impl LayoutMeta {
    /// Read current window geometry from the egui context.
    pub fn from_context(ctx: &egui::Context) -> Self {
        let (width, height) = ctx.input(|i| {
            i.viewport().inner_rect.map_or_else(
                || {
                    tracing::warn!("Could not read window inner rect, using defaults");
                    (DEFAULT_WIDTH, DEFAULT_HEIGHT)
                },
                |r| (r.width(), r.height()),
            )
        });
        let (x, y) = ctx.input(|i| {
            i.viewport().outer_rect.map_or_else(
                || {
                    tracing::warn!("Could not read window outer rect, using defaults");
                    (0.0, 0.0)
                },
                |r| (r.left(), r.top()),
            )
        });
        Self::from_geometry(WindowGeometry {
            x,
            y,
            width,
            height,
        })
    }

    /// Apply window geometry to a ViewportBuilder (for initial window creation).
    pub fn apply_to_viewport_builder(
        &self,
        mut builder: egui::ViewportBuilder,
    ) -> egui::ViewportBuilder {
        if let (Some(w), Some(h)) = (self.window_width, self.window_height) {
            builder = builder.with_inner_size([w, h]);
        }
        if let (Some(x), Some(y)) = (self.window_x, self.window_y) {
            builder = builder.with_position([x, y]);
        }
        builder
    }

    /// Apply window geometry to the egui viewport (for runtime changes).
    pub fn apply_to_context(&self, ctx: &egui::Context) {
        if let (Some(w), Some(h)) = (self.window_width, self.window_height) {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::InnerSize(egui::Vec2::new(
                w, h,
            )));
        }
        if let (Some(x), Some(y)) = (self.window_x, self.window_y) {
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::OuterPosition(
                egui::Pos2::new(x, y),
            ));
        }
    }
}
