//! A small, reusable directed graph.
//!
//! This is intentionally generic and carries no domain semantics: the
//! [`crate::cfg`] module builds a per-method control-flow graph on top of
//! it, and later milestones (Call Graph, dependency graphs) are expected to
//! reuse it too. Node and edge payloads are opaque to the graph itself.
//!
//! Ids are opaque newtypes with private inner indices; callers can compare,
//! copy, and hash them, but cannot construct or index into internal storage
//! directly. All lookups on unknown/foreign ids return `None`/empty
//! iterators rather than panicking.
//!
//! Insertion order is preserved: [`NodeId`]/[`EdgeId`] values are assigned
//! sequentially as nodes/edges are added, and iteration follows that order.

/// Opaque handle to a node in a [`Graph`]. Never panics on unknown ids; all queries return `None`/empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u32);

/// Opaque handle to an edge in a [`Graph`]. Never panics on unknown ids; all queries return `None`/empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeId(u32);

#[derive(Debug, Clone)]
struct EdgeData<E> {
    from: NodeId,
    to: NodeId,
    payload: E,
}

/// A directed graph generic over node payload `N` and edge payload `E`.
///
/// Supports parallel edges (multiple edges between the same pair of nodes)
/// since that is a natural shape for control-flow graphs (e.g. two distinct
/// `switch` cases sharing a target block).
#[derive(Debug, Clone)]
pub struct Graph<N, E> {
    nodes: Vec<N>,
    edges: Vec<EdgeData<E>>,
    outgoing: Vec<Vec<EdgeId>>,
    incoming: Vec<Vec<EdgeId>>,
}

impl<N, E> Default for Graph<N, E> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            outgoing: Vec::new(),
            incoming: Vec::new(),
        }
    }
}

impl<N, E> Graph<N, E> {
    /// Create an empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node carrying `payload`, returning its new [`NodeId`].
    pub fn add_node(&mut self, payload: N) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(payload);
        self.outgoing.push(Vec::new());
        self.incoming.push(Vec::new());
        id
    }

    /// Add a directed edge `from -> to` carrying `payload`, returning its
    /// new [`EdgeId`].
    ///
    /// # Panics
    /// Never panics; edges referencing unknown node ids are simply not
    /// recorded in that node's adjacency lists (the edge itself is still
    /// stored so [`Graph::edge`] and [`Graph::edge_endpoints`] remain
    /// consistent).
    pub fn add_edge(&mut self, from: NodeId, to: NodeId, payload: E) -> EdgeId {
        let id = EdgeId(self.edges.len() as u32);
        self.edges.push(EdgeData { from, to, payload });
        if let Some(list) = self.outgoing.get_mut(from.0 as usize) {
            list.push(id);
        }
        if let Some(list) = self.incoming.get_mut(to.0 as usize) {
            list.push(id);
        }
        id
    }

    /// The payload of a node. Returns `None` for an unknown id.
    pub fn node(&self, id: NodeId) -> Option<&N> {
        self.nodes.get(id.0 as usize)
    }

    /// A mutable reference to a node's payload. Returns `None` for an
    /// unknown id.
    pub fn node_mut(&mut self, id: NodeId) -> Option<&mut N> {
        self.nodes.get_mut(id.0 as usize)
    }

    /// The payload of an edge. Returns `None` for an unknown id.
    pub fn edge(&self, id: EdgeId) -> Option<&E> {
        self.edges.get(id.0 as usize).map(|e| &e.payload)
    }

    /// The `(from, to)` endpoints of an edge. Returns `None` for an unknown id.
    pub fn edge_endpoints(&self, id: EdgeId) -> Option<(NodeId, NodeId)> {
        self.edges.get(id.0 as usize).map(|e| (e.from, e.to))
    }

    /// Total number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Iterate all node ids, in insertion order.
    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        (0..self.nodes.len() as u32).map(NodeId)
    }

    /// Iterate all `(NodeId, &N)` pairs, in insertion order.
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &N)> + '_ {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (NodeId(i as u32), n))
    }

    /// Outgoing edge ids of a node, in insertion order. Returns empty iterator for unknown ids (never panics).
    pub fn out_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + '_ {
        self.outgoing
            .get(id.0 as usize)
            .into_iter()
            .flatten()
            .copied()
    }

    /// Incoming edge ids of a node, in insertion order. Returns empty iterator for unknown ids (never panics).
    pub fn in_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + '_ {
        self.incoming
            .get(id.0 as usize)
            .into_iter()
            .flatten()
            .copied()
    }

    /// Successor node ids reachable via a single outgoing edge, in insertion order; includes duplicates for parallel edges.
    pub fn successors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.out_edges(id)
            .filter_map(move |e| self.edge_endpoints(e).map(|(_, to)| to))
    }

    /// Predecessor node ids with an edge into `id`, in insertion order; includes duplicates for parallel edges.
    pub fn predecessors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.in_edges(id)
            .filter_map(move |e| self.edge_endpoints(e).map(|(from, _)| from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_node_returns_sequential_ids() {
        let mut g: Graph<&str, ()> = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        assert_ne!(a, b);
        assert_eq!(g.node(a), Some(&"a"));
        assert_eq!(g.node(b), Some(&"b"));
        assert_eq!(g.node_count(), 2);
    }

    #[test]
    fn add_edge_and_query_endpoints() {
        let mut g: Graph<&str, &str> = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        let e = g.add_edge(a, b, "edge");
        assert_eq!(g.edge(e), Some(&"edge"));
        assert_eq!(g.edge_endpoints(e), Some((a, b)));
        assert_eq!(g.edge_count(), 1);
    }

    #[test]
    fn successors_and_predecessors() {
        let mut g: Graph<&str, ()> = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        let c = g.add_node("c");
        g.add_edge(a, b, ());
        g.add_edge(a, c, ());
        g.add_edge(b, c, ());

        let succ_a: Vec<_> = g.successors(a).collect();
        assert_eq!(succ_a, vec![b, c]);

        let pred_c: Vec<_> = g.predecessors(c).collect();
        assert_eq!(pred_c, vec![a, b]);

        assert_eq!(g.predecessors(a).count(), 0);
        assert_eq!(g.successors(c).count(), 0);
    }

    #[test]
    fn parallel_edges_are_preserved() {
        let mut g: Graph<&str, u8> = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        g.add_edge(a, b, 1);
        g.add_edge(a, b, 2);

        let succ: Vec<_> = g.successors(a).collect();
        assert_eq!(succ, vec![b, b]);
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn unknown_ids_return_empty_not_panic() {
        let g: Graph<&str, ()> = Graph::new();
        let mut g2: Graph<&str, ()> = Graph::new();
        let real = g2.add_node("real");
        // A NodeId from an unrelated, empty graph of the same node count
        // shape: use a node id that does not exist in `g` (which has none).
        assert_eq!(g.node(real), None);
        assert_eq!(g.successors(real).count(), 0);
        assert_eq!(g.predecessors(real).count(), 0);
        assert_eq!(g.node_count(), 0);
    }

    #[test]
    fn deterministic_iteration_order() {
        let mut g: Graph<u32, ()> = Graph::new();
        let ids: Vec<_> = (0..5).map(|i| g.add_node(i)).collect();
        let collected: Vec<_> = g.nodes().map(|(_, &n)| n).collect();
        assert_eq!(collected, vec![0, 1, 2, 3, 4]);

        // Re-running iteration yields the same order every time.
        let collected2: Vec<_> = g.nodes().map(|(_, &n)| n).collect();
        assert_eq!(collected, collected2);

        let node_ids: Vec<_> = g.node_ids().collect();
        assert_eq!(node_ids, ids);
    }

    #[test]
    fn out_edges_and_in_edges_return_empty_for_isolated_node() {
        let mut g: Graph<&str, ()> = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        // Don't add any edges; nodes are isolated.

        assert_eq!(g.out_edges(a).count(), 0);
        assert_eq!(g.in_edges(a).count(), 0);
        assert_eq!(g.out_edges(b).count(), 0);
        assert_eq!(g.in_edges(b).count(), 0);
    }

    #[test]
    fn successors_includes_duplicates_for_parallel_edges() {
        let mut g: Graph<&str, ()> = Graph::new();
        let a = g.add_node("a");
        let b = g.add_node("b");
        g.add_edge(a, b, ());
        g.add_edge(a, b, ());

        let succs: Vec<_> = g.successors(a).collect();
        // successors() does NOT dedup: it returns all successor nodes, including duplicates for parallel edges.
        assert_eq!(succs, vec![b, b]);
    }
}
