/// Returns `true` when the application is running under a Wayland compositor.
///
/// Detection mirrors winit's own logic: check for `WAYLAND_DISPLAY` or
/// `WAYLAND_SOCKET` environment variables.
pub fn is_wayland() -> bool {
    std::env::var("WAYLAND_DISPLAY")
        .ok()
        .filter(|v| !v.is_empty())
        .is_some()
        || std::env::var("WAYLAND_SOCKET")
            .ok()
            .filter(|v| !v.is_empty())
            .is_some()
}
