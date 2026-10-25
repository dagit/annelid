use eframe::egui;

use super::layout_meta::LayoutMeta;

impl LayoutMeta {
    /// Read current window geometry from the egui context, or `None` while the
    /// window is minimized and has none.
    pub fn from_context(ctx: &egui::Context) -> Option<Self> {
        ctx.input(|i| {
            Self::from_viewport(
                i.viewport(),
                i.content_rect(),
                crate::platform::is_wayland(),
            )
        })
    }

    /// Window geometry from a viewport's info, or `None` while the window is
    /// minimized and has none. `content_rect` is the area egui lays out into,
    /// and `wayland` says whether the window is on Wayland.
    pub fn from_viewport(
        viewport: &egui::ViewportInfo,
        content_rect: egui::Rect,
        wayland: bool,
    ) -> Option<Self> {
        if viewport.minimized == Some(true) {
            return None;
        }
        let size = match viewport.inner_rect {
            Some(inner) => inner.size(),
            // egui never has an inner rect on Wayland, which can't report
            // window positions, but the content rect is the same size.
            None if wayland => content_rect.size(),
            // Anywhere else, a missing inner rect means the window is minimized.
            None => return None,
        };
        // Wayland does not support reading window position; avoid storing
        // bogus (0, 0) values that would overwrite valid X11 positions in
        // shared layout files.
        let position = if wayland {
            None
        } else {
            viewport.outer_rect.map(|outer| outer.min)
        };
        Some(LayoutMeta {
            window_x: position.map(|p| p.x),
            window_y: position.map(|p| p.y),
            window_width: Some(size.x),
            window_height: Some(size.y),
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
            if crate::platform::is_wayland() {
                tracing::debug!(
                    "Skipping window position ({x}, {y}): Wayland does not support client-side positioning"
                );
            } else {
                builder = builder.with_position([x, y]);
            }
        }
        builder
    }

    /// Apply window geometry to the egui viewport (for runtime changes).
    pub fn apply_to_context(&self, ctx: &egui::Context) {
        if let (Some(w), Some(h)) = (self.window_width, self.window_height) {
            tracing::debug!("Requesting window size: {w}x{h}");
            ctx.send_viewport_cmd(egui::viewport::ViewportCommand::InnerSize(egui::Vec2::new(
                w, h,
            )));
        }
        if let (Some(x), Some(y)) = (self.window_x, self.window_y) {
            if crate::platform::is_wayland() {
                tracing::debug!(
                    "Skipping window position ({x}, {y}): Wayland does not support client-side positioning"
                );
            } else {
                ctx.send_viewport_cmd(egui::viewport::ViewportCommand::OuterPosition(
                    egui::Pos2::new(x, y),
                ));
            }
        }
    }
}
