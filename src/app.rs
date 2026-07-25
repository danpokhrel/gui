use crate::graph::{GraphState, Node, NodeId, Pin, PinKind};
use crate::theme::EditorTheme;
use crate::ui::file_tree::FileEntry;
use dear_imnodes as imnodes;
use std::path::PathBuf;

/// Ephemeral UI interaction state — resets every session.
/// Separated from App so the data model and contexts stay clean.
#[derive(Default)]
pub struct UiState {
    pub show_about: bool,
    pub show_demo: bool,
    pub positions_initialized: bool,
    pub theme_applied: bool,
    pub ctx_open_pos: Option<[f32; 2]>,
    pub ctx_hovered_node: Option<NodeId>,
    pub ctx_hovered_link: Option<crate::graph::LinkId>,
    /// Node currently inspected in the properties panel.
    pub selected_node: Option<NodeId>,
    /// Node under the minimap cursor (used for the properties panel tooltip).
    pub minimap_hovered: Option<NodeId>,
    pub pending_node_pos: Option<(NodeId, [f32; 2])>,
    pub pending: Option<crate::ui::toolbar::PendingAction>,
    /// Cached project file tree (scanned once, refreshed on demand).
    pub file_tree: Option<FileEntry>,
    /// Currently selected file in the project panel.
    pub selected_file: Option<PathBuf>,
}

/// Core application state — graph, contexts, and theme.
/// This is what you test and persist; UiState is ephemeral.
pub struct App {
    pub graph: GraphState,
    pub nodes_context: imnodes::Context,
    pub editor_context: imnodes::EditorContext,
    pub theme: EditorTheme,
    pub ui: UiState, // UI interaction state, nested
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ProjectFile {
    graph: crate::graph::GraphState,
    version: String,
}

impl App {
    pub fn new(imgui_context: &mut dear_imgui_rs::Context) -> Self {
        let nodes_context = imnodes::Context::create(imgui_context);
        let editor_context = nodes_context.create_editor_context();
        let theme = crate::theme::EditorTheme::dark();

        // The tutorial explains that the theme can't be applied here in `on_setup`:
        // the ImNodes style setters need a `NodeEditor` token (a method on `NodesUi`,
        // created via `ui.imnodes(ctx)`), which requires a `&Ui`. In `on_setup` we only
        // have `&mut Context`, so we defer theme application to the first frame in
        // `render_editor` (see `editor.rs`'s `theme_applied` guard).
        let mut graph = GraphState::new();

        // Seed with a couple of example nodes
        let n1 = graph.next_node_id();
        let p1_in = graph.next_pin_id();
        let p1_out = graph.next_pin_id();
        graph.add_node(Node {
            id: n1,
            title: "Source".to_string(),
            pins: vec![
                Pin {
                    id: p1_in,
                    kind: PinKind::Input,
                    label: "In".into(),
                },
                Pin {
                    id: p1_out,
                    kind: PinKind::Output,
                    label: "Out".into(),
                },
            ],
        });

        let n2 = graph.next_node_id();
        let p2_in = graph.next_pin_id();
        let p2_out = graph.next_pin_id();
        graph.add_node(Node {
            id: n2,
            title: "Sink".to_string(),
            pins: vec![
                Pin {
                    id: p2_in,
                    kind: PinKind::Input,
                    label: "In".into(),
                },
                Pin {
                    id: p2_out,
                    kind: PinKind::Output,
                    label: "Out".into(),
                },
            ],
        });

        Self {
            graph,
            nodes_context,
            editor_context,
            theme,
            ui: UiState::default(),
        }
    }

    /// Save the graph structure to a JSON file.
    pub fn save_graph(&self, path: &str) -> Result<(), String> {
        let project = ProjectFile {
            graph: self.graph.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let json = serde_json::to_string_pretty(&project)
            .map_err(|e| format!("Serialization error: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("File write error: {e}"))?;
        Ok(())
    }

    /// Load the graph structure from a JSON file.
    pub fn load_graph(&mut self, path: &str) -> Result<(), String> {
        let json = std::fs::read_to_string(path).map_err(|e| format!("File read error: {e}"))?;
        let project: ProjectFile =
            serde_json::from_str(&json).map_err(|e| format!("Deserialization error: {e}"))?;
        self.graph = project.graph;
        self.ui.positions_initialized = false; // re-init positions
        Ok(())
    }
}
