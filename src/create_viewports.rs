use egui::ahash::HashMapExt;

pub(crate) fn create_viewports(
    bottom_screen_size: egui::Vec2,
    bottom_rect: egui::Rect,
    top_screen_size: egui::Vec2,
    top_rect: egui::Rect,
    bottom_viewport_id: egui::ViewportId,
    top_viewport_id: egui::ViewportId,
) -> egui::ViewportIdMap<egui::ViewportInfo> {
    let mut viewports = egui::ViewportIdMap::new();
    viewports.insert(bottom_viewport_id, egui::ViewportInfo {
        native_pixels_per_point: Some(1.0),
        parent: None,
        title: None,
        events: vec![],
        monitor_size: Some(bottom_screen_size),
        inner_rect: Some(bottom_rect),
        outer_rect: Some(bottom_rect),
        minimized: Some(false),
        maximized: Some(true),
        fullscreen: Some(true),
        focused: Some(true),
    });
    viewports.insert(top_viewport_id, egui::ViewportInfo {
        native_pixels_per_point: Some(1.0),
        parent: None,
        title: None,
        events: vec![],
        monitor_size: Some(top_screen_size),
        inner_rect: Some(top_rect),
        outer_rect: Some(top_rect),
        minimized: Some(false),
        maximized: Some(true),
        fullscreen: Some(true),
        focused: Some(true),
    });
    viewports
}
