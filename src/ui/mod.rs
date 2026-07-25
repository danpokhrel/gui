pub mod editor;
pub mod file_tree;
pub mod panels;
pub mod toolbar;

use crate::app::App;
use dear_imgui_rs::Ui;

/// Width of the left (Project) and right (Properties) side panels, in logical
/// pixels. Kept constant so the central canvas flexes with the window.
const LEFT_PANEL_WIDTH: f32 = 240.0;
const RIGHT_PANEL_WIDTH: f32 = 300.0;

/// Top-level render function called every frame.
///
/// Layout (all within the main viewport's work area, i.e. below the menu bar):
///
/// ```text
/// ┌────────────┬───────────────────────┬─────────────┐
/// │  Project   │     Node Graph         │ Properties  │
/// │  (240px)   │     (flexible)         │  (300px)    │
/// │            │                        │             │
/// └────────────┴───────────────────────┴─────────────┘
/// ```
pub fn render(ui: &Ui, app: &mut App) {
    crate::ui::toolbar::render_menu_bar(ui, app);

    // Recompute the layout every frame from the viewport work area so all
    // three regions track OS-window resizes and fill the window vertically.
    let viewport = ui.main_viewport();
    let work_pos = viewport.work_pos();
    let work_size = viewport.work_size();
    let x0 = work_pos[0];
    let y0 = work_pos[1];
    let full_w = work_size[0];
    let full_h = work_size[1];

    // Left panel.
    crate::ui::panels::render_project_panel(ui, app, [x0, y0], [LEFT_PANEL_WIDTH, full_h]);

    // Right panel.
    crate::ui::panels::render_properties_panel(
        ui,
        app,
        [x0 + full_w - RIGHT_PANEL_WIDTH, y0],
        [RIGHT_PANEL_WIDTH, full_h],
    );

    // Central canvas: fills the remaining width between the two panels.
    crate::ui::editor::render_editor(
        ui,
        app,
        [x0 + LEFT_PANEL_WIDTH, y0],
        [full_w - LEFT_PANEL_WIDTH - RIGHT_PANEL_WIDTH, full_h],
    );
}
