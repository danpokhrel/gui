use crate::app::App;
use crate::graph::model::PinKind;
use crate::ui::file_tree::FileEntry;
use dear_imgui_rs::{Condition, TreeNodeFlags, Ui, WindowFlags};
use std::path::PathBuf;

/// Zed-like palette: compact rows, subtle hover, accent selection, no frame.
/// All colors are RGBA floats 0..=1.
const COL_ROW_HOVER: [f32; 4] = [0.30, 0.31, 0.34, 0.55];
const COL_ROW_SELECTED: [f32; 4] = [0.20, 0.46, 0.74, 0.80];
const COL_ICON_DIR: [f32; 4] = [0.90, 0.78, 0.40, 1.0];
const COL_ICON_FILE: [f32; 4] = [0.55, 0.66, 0.82, 1.0];

/// Render the left-hand project panel as a Zed-style file tree of the project
/// directory. The tree is scanned once and cached; a Refresh button rescans.
pub fn render_project_panel(ui: &Ui, app: &mut App, pos: [f32; 2], size: [f32; 2]) {
    ui.window("Project")
        .flags(panel_flags())
        .position(pos, Condition::Always)
        .size(size, Condition::Always)
        .build(|| {
            apply_tree_style(ui);

            ui.text("Project");
            ui.same_line();
            let btn_w = 60.0;
            let avail = ui.content_region_avail_width();
            ui.set_cursor_pos_x(ui.cursor_pos()[0] + avail - btn_w);
            if ui.button("Refresh") {
                app.ui.file_tree = scan_project_root();
            }
            ui.separator();

            if app.ui.file_tree.is_none() {
                app.ui.file_tree = scan_project_root();
            }

            if let Some(root) = &app.ui.file_tree {
                let selected = app.ui.selected_file.clone();
                render_tree_entry(ui, root, &selected, &mut app.ui.selected_file, 0);
            } else {
                ui.text_wrapped("Could not read project directory.");
            }
        });
}

/// Recursively render one file-tree entry with a Zed-like appearance:
/// - Folders show a chevron + folder glyph, expandable.
/// - Files show a file glyph, single-click selects.
/// - Hover and selection are drawn as full-width rounded rects behind the row.
///
/// To avoid "DrawListMut already in use" panics, we never hold a draw-list borrow
/// across other `Ui` calls: we query hover/click state first, capture the row
/// rect, then draw the background in a separate draw-list handle.
fn render_tree_entry(
    ui: &Ui,
    entry: &FileEntry,
    selected: &Option<PathBuf>,
    out: &mut Option<PathBuf>,
    depth: u32,
) {
    let is_selected = selected.as_ref().is_some_and(|p| *p == entry.path);

    let mut node = ui
        .tree_node_config(entry.name.as_str())
        .span_avail_width(true)
        .open_on_arrow(true);

    if entry.is_dir {
        node = node.default_open(depth == 0);
    } else {
        node = node.leaf(true);
    }

    let token = node.push();
    let was_open = token.is_some();

    // Capture row geometry and interaction state now (before any draw list use).
    let row_min = ui.item_rect_min();
    let row_max = ui.item_rect_max();
    let hovered = ui.is_item_hovered();
    let clicked = ui.is_item_clicked();

    // Draw the hover/selection highlight behind the row. We acquire the draw
    // list, draw, and let it drop before any further Ui calls below.
    if is_selected || hovered {
        let avail_w = ui.content_region_avail_width();
        let bg_max = [
            row_min[0] + avail_w.max(row_max[0] - row_min[0]),
            row_max[1],
        ];
        let col = if is_selected {
            COL_ROW_SELECTED
        } else {
            COL_ROW_HOVER
        };
        let draw = ui.get_window_draw_list();
        draw.add_rect(row_min, bg_max, col)
            .filled(true)
            .rounding(3.0)
            .build();
    }

    // Draw the leading icon glyph to the left of the built-in label.
    {
        let (icon, icon_col) = if entry.is_dir {
            ("▸ ", COL_ICON_DIR)
        } else {
            ("• ", COL_ICON_FILE)
        };
        let draw = ui.get_window_draw_list();
        draw.add_text([row_min[0] + 2.0, row_min[1] + 1.0], icon_col, icon);
    }

    // Recurse into directories (only while open).
    if was_open
        && let Some(_t) = token
        && entry.is_dir
    {
        for child in &entry.children {
            render_tree_entry(ui, child, selected, out, depth + 1);
        }
    }

    // Files: select on click. (Directories toggle open/close via the arrow.)
    if !entry.is_dir && clicked {
        *out = Some(entry.path.clone());
    }
}

/// Scan the current working directory (the project root when run via cargo).
fn scan_project_root() -> Option<FileEntry> {
    let root = std::env::current_dir().ok()?;
    FileEntry::scan(&root)
}

/// Render the right-hand properties panel for the selected node.
pub fn render_properties_panel(ui: &Ui, app: &mut App, pos: [f32; 2], size: [f32; 2]) {
    ui.window("Properties")
        .flags(panel_flags())
        .position(pos, Condition::Always)
        .size(size, Condition::Always)
        .build(|| {
            ui.text("Properties");
            ui.separator();

            let Some(node_id) = app.ui.selected_node else {
                ui.text_wrapped(
                    "No node selected. Select a node from the Project panel or the canvas.",
                );
                if let Some(hover) = app.ui.minimap_hovered {
                    ui.spacing();
                    ui.text(format!("(minimap hover: node #{})", hover.0));
                }
                return;
            };

            let idx = app.graph.nodes.iter().position(|n| n.id == node_id);
            let Some(idx) = idx else {
                app.ui.selected_node = None;
                ui.text("Selected node no longer exists.");
                return;
            };

            ui.text(format!("Node #{}", node_id.0));
            ui.spacing();

            ui.text("Title");
            ui.input_text("##title", &mut app.graph.nodes[idx].title)
                .build();
            ui.spacing();

            if ui.collapsing_header(
                "Pins",
                TreeNodeFlags::DEFAULT_OPEN | TreeNodeFlags::SPAN_AVAIL_WIDTH,
            ) {
                ui.indent();
                let pin_count = app.graph.nodes[idx].pins.len();
                for i in 0..pin_count {
                    let pin = &mut app.graph.nodes[idx].pins[i];
                    let kind = match pin.kind {
                        PinKind::Input => "IN ",
                        PinKind::Output => "OUT",
                    };
                    ui.text(format!("{kind} pin #{}", pin.id.0));
                    ui.indent();
                    ui.input_text("##pin_label", &mut pin.label).build();
                    ui.unindent();
                    ui.spacing();
                }
                ui.unindent();
            }
        });
}

/// Flags shared by both side panels: no title bar / dock tab, no resize/move,
/// so they stay pinned and fill the window vertically like the canvas.
/// Unlike `NO_DECORATION`, we keep `NO_SCROLLBAR` *off* so long file trees /
/// pin lists can scroll.
fn panel_flags() -> WindowFlags {
    WindowFlags::NO_TITLE_BAR
        | WindowFlags::NO_RESIZE
        | WindowFlags::NO_MOVE
        | WindowFlags::NO_COLLAPSE
}

/// Apply a compact, Zed-like style to the tree: tight row spacing, minimal
/// frame padding, smaller indent. Scoped via push_style_var so it only affects
/// this panel.
fn apply_tree_style(ui: &Ui) {
    let _t1 = ui.push_style_var(dear_imgui_rs::StyleVar::WindowPadding([8.0, 6.0]));
    let _t2 = ui.push_style_var(dear_imgui_rs::StyleVar::ItemSpacing([2.0, 2.0]));
    let _t3 = ui.push_style_var(dear_imgui_rs::StyleVar::FramePadding([4.0, 2.0]));
    let _t4 = ui.push_style_var(dear_imgui_rs::StyleVar::IndentSpacing(14.0));
}
