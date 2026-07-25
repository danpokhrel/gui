use dear_imgui_rs::{MouseButton, Ui};

/// A thin, draggable vertical splitter between two panels (like the one in VS
/// Code / Zed).
///
/// The splitter sits at screen x = `x`. It renders an invisible grab handle and
/// a 1px line. While dragged, it moves the divider to follow the mouse and
/// updates `out_width` accordingly:
///
/// - `SplitterSide::Left`: the splitter is the *right* edge of a panel on the
///   left. Dragging right increases `out_width`. (Width = mouse_x - panel_left.)
/// - `SplitterSide::Right`: the splitter is the *left* edge of a panel on the
///   right. Dragging right decreases `out_width`. (Width = panel_right - mouse_x.)
pub fn vertical_splitter(ui: &Ui, cfg: SplitterConfig) {
    const SPLITTER_W: f32 = 6.0;
    const VISUAL_W: f32 = 1.0;

    let SplitterConfig {
        x,
        y,
        height,
        out_width,
        min,
        max,
        side,
    } = cfg;

    // Position the invisible grab handle centered on the divider line.
    ui.set_cursor_screen_pos([x - SPLITTER_W / 2.0, y]);
    let hovered_or_active = ui.invisible_button("##splitter", [SPLITTER_W, height]);

    // Draw a 1px line centered in the handle. Highlight when hovered/active.
    let draw = ui.get_window_draw_list();
    let base_col = [0.28, 0.28, 0.31, 1.0];
    let active_col = [0.45, 0.62, 0.95, 1.0];
    let col = if hovered_or_active {
        active_col
    } else {
        base_col
    };
    draw.add_line_v(x - VISUAL_W / 2.0, y, y + height, col, VISUAL_W);

    // While the handle is being dragged, move the divider with the mouse.
    if ui.is_item_active() && ui.is_mouse_down(MouseButton::Left) {
        let mouse_x = ui.io().mouse_pos()[0];
        let cur = *out_width;
        let new_width = match side {
            // Left panel's right edge: width grows as mouse moves right.
            SplitterSide::Left => cur + (mouse_x - x),
            // Right panel's left edge: width grows as mouse moves left.
            SplitterSide::Right => cur + (x - mouse_x),
        };
        *out_width = new_width.clamp(min, max);
    }
}

/// Which side of the splitter the resizable panel is on.
#[derive(Clone, Copy)]
pub enum SplitterSide {
    /// The panel being resized is on the left of the splitter.
    Left,
    /// The panel being resized is on the right of the splitter.
    Right,
}

/// Grouped parameters for `vertical_splitter` (keeps the argument list short).
pub struct SplitterConfig<'a> {
    pub x: f32,
    pub y: f32,
    pub height: f32,
    pub out_width: &'a mut f32,
    pub min: f32,
    pub max: f32,
    pub side: SplitterSide,
}
