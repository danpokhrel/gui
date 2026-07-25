use super::model::{Link, LinkId, Node, NodeId, PinId, PinKind};

/// Mutable graph state. This is the single source of truth for the
/// graph's structure. The UI layer reads from and writes to this.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GraphState {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
    next_node_id: i32,
    next_pin_id: i32,
    next_link_id: i32,
}

impl GraphState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Allocate the next available node ID.
    pub fn next_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    /// Allocate the next available pin ID.
    pub fn next_pin_id(&mut self) -> PinId {
        let id = PinId(self.next_pin_id);
        self.next_pin_id += 1;
        id
    }

    /// Allocate the next available link ID.
    pub fn next_link_id(&mut self) -> LinkId {
        let id = LinkId(self.next_link_id);
        self.next_link_id += 1;
        id
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Add a link, returning false if it would create a duplicate
    /// (same output → same input) or if the pin directions are invalid
    /// (`from` must be an output pin, `to` must be an input pin).
    pub fn add_link(&mut self, link: Link) -> bool {
        // Validate pin directions: `from` must be Output, `to` must be Input.
        // This enforces the invariant at the model level, not just the UI layer.
        let from_kind = self
            .nodes
            .iter()
            .flat_map(|n| &n.pins)
            .find(|p| p.id == link.from)
            .map(|p| &p.kind);
        let to_kind = self
            .nodes
            .iter()
            .flat_map(|n| &n.pins)
            .find(|p| p.id == link.to)
            .map(|p| &p.kind);

        match (from_kind, to_kind) {
            (Some(PinKind::Output), Some(PinKind::Input)) => {}
            _ => return false, // Invalid direction or unknown pin
        }

        let exists = self
            .links
            .iter()
            .any(|l| l.from == link.from && l.to == link.to);
        if !exists {
            self.links.push(link);
            true
        } else {
            false
        }
    }

    /// Remove a link by ID.
    pub fn remove_link(&mut self, id: LinkId) {
        self.links.retain(|l| l.id != id);
    }

    /// Remove all links connected to a given pin.
    pub fn remove_links_for_pin(&mut self, pin: PinId) {
        self.links.retain(|l| l.from != pin && l.to != pin);
    }

    /// Remove a node and all its links by ID.
    pub fn remove_node(&mut self, id: NodeId) {
        // Collect this node's pins before removing it.
        let pins_to_remove: Vec<PinId> = self
            .nodes
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.pins.iter().map(|p| p.id).collect())
            .unwrap_or_default();

        // Drop every link touching any of the node's pins.
        for pin in pins_to_remove {
            self.remove_links_for_pin(pin);
        }
        self.nodes.retain(|n| n.id != id);
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::*;
    use super::*;

    fn make_test_graph() -> GraphState {
        let mut g = GraphState::new();
        let n1 = g.next_node_id();
        let p1_out = g.next_pin_id();
        let p1_in = g.next_pin_id();
        g.add_node(Node {
            id: n1,
            title: "A".into(),
            pins: vec![
                Pin {
                    id: p1_out,
                    kind: PinKind::Output,
                    label: "Out".into(),
                },
                Pin {
                    id: p1_in,
                    kind: PinKind::Input,
                    label: "In".into(),
                },
            ],
        });

        let n2 = g.next_node_id();
        let p2_in = g.next_pin_id();
        g.add_node(Node {
            id: n2,
            title: "B".into(),
            pins: vec![Pin {
                id: p2_in,
                kind: PinKind::Input,
                label: "In".into(),
            }],
        });

        // Link A.Out -> B.In
        let link_id = g.next_link_id();
        g.add_link(Link {
            id: link_id,
            from: p1_out,
            to: p2_in,
        });
        g
    }

    #[test]
    fn test_add_link_prevents_duplicates() {
        let mut g = make_test_graph();
        let dup = Link {
            id: g.next_link_id(),
            from: PinId(1),
            to: PinId(3),
        };
        assert!(!g.add_link(dup), "Duplicate link should be rejected");
    }

    #[test]
    fn test_add_link_rejects_wrong_direction() {
        let mut g = make_test_graph();
        // Input -> Input (both are input pins) — should be rejected
        let bad = Link {
            id: g.next_link_id(),
            from: PinId(0),
            to: PinId(2),
        };
        assert!(!g.add_link(bad), "Input→Input link should be rejected");
        // Output -> Output — also rejected
        let bad2 = Link {
            id: g.next_link_id(),
            from: PinId(1),
            to: PinId(1),
        };
        assert!(!g.add_link(bad2), "Output→Output link should be rejected");
    }

    #[test]
    fn test_remove_node_removes_links() {
        let mut g = make_test_graph();
        assert_eq!(g.links.len(), 1);
        g.remove_node(NodeId(0));
        assert_eq!(g.links.len(), 0, "Links to removed node should be deleted");
        assert_eq!(g.nodes.len(), 1);
    }

    #[test]
    fn test_id_allocation_is_monotonic() {
        let mut g = GraphState::new();
        assert_eq!(g.next_node_id(), NodeId(0));
        assert_eq!(g.next_node_id(), NodeId(1));
        assert_eq!(g.next_pin_id(), PinId(0));
        assert_eq!(g.next_pin_id(), PinId(1));
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let g = make_test_graph();
        let json = serde_json::to_string(&g).unwrap();
        let g2: GraphState = serde_json::from_str(&json).unwrap();
        assert_eq!(g.nodes.len(), g2.nodes.len());
        assert_eq!(g.links.len(), g2.links.len());
    }
}
