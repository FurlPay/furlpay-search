//! Def-Use call hierarchy and cross-file reference graph.

use std::collections::HashMap;
use crate::index::schema::DocId;
use crate::ast::treesitter::SymbolKind;

/// Unique identifier for a symbol in the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u64);

/// A node in the symbol graph representing a definition or reference.
#[derive(Debug, Clone)]
pub struct SymbolNode {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub doc_id: DocId,
    pub byte_range: (usize, usize),
    pub line_range: (usize, usize),
    pub callees: Vec<SymbolId>,
    pub callers: Vec<SymbolId>,
    pub parent: Option<SymbolId>,
    pub children: Vec<SymbolId>,
}

/// The cross-file symbol graph for the entire workspace.
pub struct SymbolGraph {
    nodes: HashMap<SymbolId, SymbolNode>,
    name_index: HashMap<String, Vec<SymbolId>>,
    next_id: u64,
}

impl SymbolGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            name_index: HashMap::new(),
            next_id: 0,
        }
    }

    /// Add a symbol definition to the graph.
    pub fn add_symbol(
        &mut self,
        name: String,
        kind: SymbolKind,
        doc_id: DocId,
        byte_range: (usize, usize),
        line_range: (usize, usize),
    ) -> SymbolId {
        let id = SymbolId(self.next_id);
        self.next_id += 1;

        let node = SymbolNode {
            id,
            name: name.clone(),
            kind,
            doc_id,
            byte_range,
            line_range,
            callees: Vec::new(),
            callers: Vec::new(),
            parent: None,
            children: Vec::new(),
        };

        self.nodes.insert(id, node);
        self.name_index.entry(name).or_default().push(id);
        id
    }

    /// Record a call/reference edge from caller to callee.
    pub fn add_call_edge(&mut self, caller: SymbolId, callee: SymbolId) {
        if let Some(caller_node) = self.nodes.get_mut(&caller) {
            caller_node.callees.push(callee);
        }
        if let Some(callee_node) = self.nodes.get_mut(&callee) {
            callee_node.callers.push(caller);
        }
    }

    /// Set parent-child relationship between symbols.
    pub fn set_parent(&mut self, child: SymbolId, parent: SymbolId) {
        if let Some(child_node) = self.nodes.get_mut(&child) {
            child_node.parent = Some(parent);
        }
        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(child);
        }
    }

    /// Find symbols by name.
    pub fn find_by_name(&self, name: &str) -> Vec<&SymbolNode> {
        self.name_index
            .get(name)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn get(&self, id: SymbolId) -> Option<&SymbolNode> {
        self.nodes.get(&id)
    }

    /// Access all nodes in the symbol graph.
    pub fn nodes(&self) -> &HashMap<SymbolId, SymbolNode> {
        &self.nodes
    }

    /// Find all callers of a symbol.
    pub fn callers_of(&self, id: SymbolId) -> Vec<&SymbolNode> {
        self.nodes
            .get(&id)
            .map(|node| {
                node.callers
                    .iter()
                    .filter_map(|cid| self.nodes.get(cid))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find all callees of a symbol.
    pub fn callees_of(&self, id: SymbolId) -> Vec<&SymbolNode> {
        self.nodes
            .get(&id)
            .map(|node| {
                node.callees
                    .iter()
                    .filter_map(|cid| self.nodes.get(cid))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get all symbols in a specific document.
    pub fn symbols_in_document(&self, doc_id: DocId) -> Vec<&SymbolNode> {
        self.nodes
            .values()
            .filter(|n| n.doc_id == doc_id)
            .collect()
    }
}

impl Default for SymbolGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_graph_basic() {
        let mut graph = SymbolGraph::new();
        let fn_a = graph.add_symbol(
            "handle_payment".into(),
            SymbolKind::Function,
            DocId(0),
            (0, 100),
            (0, 10),
        );
        let fn_b = graph.add_symbol(
            "verify_token".into(),
            SymbolKind::Function,
            DocId(1),
            (0, 80),
            (0, 8),
        );

        graph.add_call_edge(fn_a, fn_b);

        let callers = graph.callers_of(fn_b);
        assert_eq!(callers.len(), 1);
        assert_eq!(callers[0].name, "handle_payment");

        let callees = graph.callees_of(fn_a);
        assert_eq!(callees.len(), 1);
        assert_eq!(callees[0].name, "verify_token");
    }

    #[test]
    fn test_find_by_name() {
        let mut graph = SymbolGraph::new();
        graph.add_symbol("process".into(), SymbolKind::Function, DocId(0), (0, 50), (0, 5));
        graph.add_symbol("process".into(), SymbolKind::Method, DocId(1), (0, 60), (0, 6));

        let results = graph.find_by_name("process");
        assert_eq!(results.len(), 2);
    }
}
