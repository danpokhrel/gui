use crate::app::App;
use crate::graph::model::{LinkId, NodeId, PinId, PinKind};
use crate::graph::{Node, Pin};
use crate::ui::toolbar::PendingAction;
use dear_imgui_rs::{Condition, Key, MouseButton, Ui, WindowFlags};
use dear_imnodes::{self as imnodes, ImNodesExt, MiniMapLocation};

use std::cell::Cell;

fn to_im_id(id: NodeId) -> imnodes::NodeId {
    imnodes::NodeId::new(id.0)
}
fn to_im_pin(id: PinId) -> imnodes::PinId {
    imnodes::PinId::new(id.0)
}
fn to_im_link(id: LinkId) -> imnodes::LinkId {
    imnodes::LinkId::new(id.0)
}

/// Collected interaction results from the editor frame, captured while only
/// immutable borrows of `app` are live so we can mutate `app` afterwards.
struct Interactions {
    editor_hovered: bool,
    hovered_node: Option<NodeId>,
    hovered_link: Option<LinkId>,
    minimap_hovered: Option<NodeId>,
    new_link: Option<(i32, i32)>,
    destroyed: Option<i32>,
    selected_nodes: Vec<i32>,
    selected_links: Vec<i32>,
}

pub fn render_editor(ui: &Ui, app: &mut App, pos: [f32; 2], size: [f32; 2]) {
    // Central node graph canvas. The caller computes the position/size so this
    // window fits between the side panels and resizes with the OS window.
    // `NO_DECORATION` removes the title bar / dock tab; `NO_MOVE` keeps it pinned.
    ui.window("Node Graph Editor")
        .flags(WindowFlags::NO_DECORATION | WindowFlags::NO_MOVE)
        .position(pos, Condition::Always)
        .size(size, Condition::Always)
        .build(|| {
            // --- Editor frame (immutable borrow of contexts + graph reads) ---
            let interactions = {
                let mut editor = ui.imnodes_editor(&app.nodes_context, Some(&app.editor_context));

                // Collect the minimap-hovered node into a local; the editor holds an
                // immutable borrow of the contexts, so we can't touch `app` inside
                // the callback. `Cell<Option<NodeId>>` lets the callback write without
                // a mutable borrow that would conflict with reading it afterwards
                // (`NodeId` is `Copy`). Written to `app.ui.minimap_hovered` after `.end()`.
                let minimap_hovered: Cell<Option<NodeId>> = Cell::new(None);

                // Apply theme on the first frame only (persistent on the EditorContext).
                // The tutorial explains that this must be deferred to the first frame (not
                // `App::new()`): the style setters need a `NodeEditor` token, which requires
                // a `&Ui` that isn't available in `on_setup`.
                if !app.ui.theme_applied {
                    app.theme.apply(&editor);
                    app.ui.theme_applied = true;
                }

                // Apply pending node position (from context-menu node creation).
                if let Some((node_id, pos)) = app.ui.pending_node_pos.take() {
                    editor.set_node_pos_screen(to_im_id(node_id), pos);
                }

                editor.enable_link_detach_with_ctrl();
                editor.enable_multiple_select_with_ctrl();

                // Set initial positions on first frame
                if !app.ui.positions_initialized {
                    for (i, node) in app.graph.nodes.iter().enumerate() {
                        let x = 100.0 + (i as f32) * 250.0;
                        editor.set_node_pos_grid(to_im_id(node.id), [x, 100.0]);
                    }
                    app.ui.positions_initialized = true;
                }

                // Render nodes
                for node in &app.graph.nodes {
                    let n = editor.node(to_im_id(node.id));
                    n.title_bar(|| ui.text(&node.title));
                    for pin in &node.pins {
                        let im_pin = to_im_pin(pin.id);
                        match pin.kind {
                            PinKind::Input => {
                                let _a = editor.input_attr(im_pin, imnodes::PinShape::CircleFilled);
                                ui.text(&pin.label);
                            }
                            PinKind::Output => {
                                let _a = editor.output_attr(im_pin, imnodes::PinShape::QuadFilled);
                                ui.text(&pin.label);
                            }
                        }
                    }
                    n.end();
                }

                // Render links
                for link in &app.graph.links {
                    editor.link(
                        to_im_link(link.id),
                        to_im_pin(link.from),
                        to_im_pin(link.to),
                    );
                }

                editor.minimap_with_callback(0.25, MiniMapLocation::BottomRight, |node_id| {
                    minimap_hovered.set(Some(NodeId(node_id.raw())));
                });
                let post = editor.end();

                // Collect interaction results
                Interactions {
                    editor_hovered: post.is_editor_hovered(),
                    hovered_node: post.hovered_node().map(|id| NodeId(id.raw())),
                    hovered_link: post.hovered_link().map(|id| LinkId(id.raw())),
                    minimap_hovered: minimap_hovered.get(),
                    new_link: post
                        .is_link_created()
                        .map(|lc| (lc.start_attr.raw(), lc.end_attr.raw())),
                    destroyed: post.is_link_destroyed().map(|id| id.raw()),
                    selected_nodes: post.selected_nodes().iter().map(|id| id.raw()).collect(),
                    selected_links: post.selected_links().iter().map(|id| id.raw()).collect(),
                }
            };

            // Sync the minimap-hovered node into app state (used by the
            // properties panel to show which node the cursor is over in the minimap).
            app.ui.minimap_hovered = interactions.minimap_hovered;

            // --- Right-click context menu trigger ---
            // The tutorial uses `ui.is_mouse_clicked(MouseButton::Right)` here and
            // `ui.io().mouse_pos()` / `ui.io().key_ctrl()` (methods, not array fields).
            // `mouse_clicked` is not an array field on `Io`.
            if interactions.editor_hovered && ui.is_mouse_clicked(MouseButton::Right) {
                app.ui.ctx_open_pos = Some(ui.io().mouse_pos());
                app.ui.ctx_hovered_node = interactions.hovered_node;
                app.ui.ctx_hovered_link = interactions.hovered_link;
                ui.open_popup("editor_ctx");
            }

            // --- Left-click selects the hovered node for the properties panel ---
            // Only act when no new link was just created (so drag-connect doesn't
            // also clobber the selection). Clicking empty space clears it.
            if interactions.editor_hovered
                && interactions.new_link.is_none()
                && ui.is_mouse_clicked(MouseButton::Left)
            {
                app.ui.selected_node = interactions.hovered_node;
            }

            // --- Context menu popup ---
            ui.popup("editor_ctx", || {
                if let Some(link_id) = app.ui.ctx_hovered_link {
                    if ui.selectable_config("Delete Link").build() {
                        app.graph.remove_link(link_id);
                        ui.close_current_popup();
                    }
                } else if let Some(node_id) = app.ui.ctx_hovered_node {
                    if ui.selectable_config("Delete Node").build() {
                        app.graph.remove_node(node_id);
                        ui.close_current_popup();
                    }
                } else if ui.selectable_config("Add Node").build() {
                    let pos = app.ui.ctx_open_pos.unwrap_or([0.0, 0.0]);
                    let id = app.graph.next_node_id();
                    let pin_in = app.graph.next_pin_id();
                    let pin_out = app.graph.next_pin_id();
                    app.graph.add_node(Node {
                        id,
                        title: format!("Node {}", id.0),
                        pins: vec![
                            Pin {
                                id: pin_in,
                                kind: PinKind::Input,
                                label: "In".into(),
                            },
                            Pin {
                                id: pin_out,
                                kind: PinKind::Output,
                                label: "Out".into(),
                            },
                        ],
                    });
                    // Position the new node at the click location on the next frame.
                    app.ui.pending_node_pos = Some((id, pos));
                    ui.close_current_popup();
                }
            });

            // --- Process new link creation ---
            if let Some((a, b)) = interactions.new_link
                && let Some((from, to)) = classify_pins(a, b, &app.graph)
            {
                let link = crate::graph::Link {
                    id: app.graph.next_link_id(),
                    from,
                    to,
                };
                app.graph.add_link(link);
            }

            // --- Process link destruction ---
            if let Some(raw) = interactions.destroyed {
                app.graph.remove_link(LinkId(raw));
            }

            // --- Process pending save/load actions ---
            match app.ui.pending.take() {
                Some(PendingAction::SaveGraph(path)) => {
                    if let Err(e) = app.save_graph(&path) {
                        eprintln!("Save error: {e}");
                    }
                }
                Some(PendingAction::LoadGraph(path)) => {
                    if let Err(e) = app.load_graph(&path) {
                        eprintln!("Load error: {e}");
                    }
                }
                Some(PendingAction::SaveIni(path)) => {
                    // INI save/load must happen on a PostEditor during a frame.
                    let editor = ui.imnodes_editor(&app.nodes_context, Some(&app.editor_context));
                    let post = editor.end();
                    post.save_state_to_ini_file(&path);
                }
                Some(PendingAction::LoadIni(path)) => {
                    let editor = ui.imnodes_editor(&app.nodes_context, Some(&app.editor_context));
                    let post = editor.end();
                    post.load_state_from_ini_file(&path);
                }
                Some(PendingAction::NewGraph) => {
                    app.graph = crate::graph::GraphState::new();
                    app.ui.positions_initialized = false;
                }
                None => {}
            }

            // --- Keyboard: Delete selected nodes and links ---
            if ui.is_key_pressed(Key::Delete) {
                for raw in &interactions.selected_nodes {
                    app.graph.remove_node(NodeId(*raw));
                }
                for raw in &interactions.selected_links {
                    app.graph.remove_link(LinkId(*raw));
                }
            }
        });
}

/// Determine which pin is an output and which is an input.
/// Returns `Some((output_pin, input_pin))` when the pair is valid,
/// or `None` if both pins have the same direction or aren't found.
fn classify_pins(a: i32, b: i32, graph: &crate::graph::GraphState) -> Option<(PinId, PinId)> {
    let find = |raw: i32| {
        graph
            .nodes
            .iter()
            .flat_map(|n| &n.pins)
            .find(|p| p.id.0 == raw)
    };

    match (find(a), find(b)) {
        (Some(pa), Some(pb)) if pa.kind == PinKind::Output && pb.kind == PinKind::Input => {
            Some((pa.id, pb.id))
        }
        (Some(pa), Some(pb)) if pa.kind == PinKind::Input && pb.kind == PinKind::Output => {
            Some((pb.id, pa.id))
        }
        _ => None, // Invalid: same direction or not found
    }
}
