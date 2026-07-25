use serde::{Deserialize, Serialize};

/// A unique identifier for a node in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub i32);

/// A unique identifier for a pin (input or output endpoint).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PinId(pub i32);

/// A unique identifier for a link connecting two pins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LinkId(pub i32);

/// The kind of pin — determines which side of the node it appears on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinKind {
    Input,
    Output,
}

/// A node in the graph. The data model is intentionally simple —
/// the UI layer is responsible for rendering title bars, pins, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub title: String,
    pub pins: Vec<Pin>,
}

/// A pin on a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pin {
    pub id: PinId,
    pub kind: PinKind,
    pub label: String,
}

/// A link connecting an output pin to an input pin.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Link {
    pub id: LinkId,
    pub from: PinId, // output pin
    pub to: PinId,   // input pin
}
