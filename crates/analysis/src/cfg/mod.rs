//! Per-method control-flow graph (CFG), built over the Rutken IR using the
//! reusable [`crate::graph`] abstraction.
//!
//! CFG construction is entirely analysis-owned: it does not populate or
//! depend on `ir::Function`/`ir::BasicBlock` (those remain reserved
//! placeholders in the IR for a possible future pass). Nothing here mutates
//! the IR.
//!
//! # Leader detection
//!
//! A "leader" is the first instruction of a basic block. An instruction is
//! a leader if:
//!
//! 1. it is the first (non-payload) instruction of the method;
//! 2. its program counter (PC) is the target of some `Branch`/`Switch` in
//!    the method, or the entry point of some exception handler
//!    (`CatchTypeAddr::handler_addr` / `CatchHandler::catch_all_addr`);
//! 3. it immediately follows a `Branch`, `Switch`, `Return`, or `Throw`.
//!
//! `Instruction::Payload` (inline switch-payload data) is never a leader
//! and never part of any block's body -- it is data, not code.
//!
//! # Exception edges (approximation)
//!
//! For each `ir::TryRange`, an [`EdgeKind::Exception`] edge is added from
//! the block containing the try-protected range's start PC to each handler
//! entry block (typed catches and the catch-all, if present). This is a
//! deliberately conservative approximation: it does not model exactly
//! which instructions within the protected range can raise which
//! exception, only that control may transfer from somewhere in the range
//! to the handler. Precise exception-edge modeling is left to a future
//! milestone; normal (non-exceptional) edges are exact.
//!
//! Degenerate/obfuscated input (a branch/switch/handler target that does
//! not land on any real instruction) never panics: the corresponding edge
//! is simply dropped.

use std::collections::HashMap;

use ir::{BranchKind, Instruction, Method};

use crate::graph::{Graph, NodeId};

/// The kind of control-flow relationship an edge represents.
///
/// Used as payload in the underlying [`crate::graph::Graph`] to classify each edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// Falls through to the next block because the terminating instruction
    /// did not divert control (e.g. a conditional branch not taken, or the
    /// default case of a switch).
    Fallthrough,
    /// The taken target of a conditional (`if-*`) branch.
    BranchTaken,
    /// The unconditional target of a `goto`.
    Goto,
    /// One case of a `packed-switch`/`sparse-switch`.
    SwitchCase,
    /// A conservative exception-handler edge (see module docs); from try-start block to handler entry.
    Exception,
}

/// A basic block: a contiguous run of instructions with a single entry point and no internal control-flow branches.
///
/// Lightweight by design: references instructions by index range into `ir::Method::instructions`.
/// Payload entries are never logically part of the block (callers must skip them defensively).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BasicBlock {
    /// Index of the first instruction of this block, into `Method::instructions`.
    pub start_index: u32,
    /// Exclusive end index into `Method::instructions`.
    pub end_index: u32,
    /// Code-unit offset (PC) of the first instruction of this block.
    pub start_pc: u32,
}

/// The control-flow graph of a single method.
///
/// Built over the reusable [`crate::graph::Graph`] abstraction. Does not mutate or depend on
/// `ir::Function`/`ir::BasicBlock` (reserved for future passes). Never panics on malformed input.
#[derive(Debug, Clone)]
pub struct Cfg {
    graph: Graph<BasicBlock, EdgeKind>,
    entry: Option<NodeId>,
}

impl Cfg {
    /// Build the CFG of `method`. Methods with no instructions yield empty CFG (no blocks, no entry).
    ///
    /// Never panics on malformed/degenerate input (e.g., dangling branch targets, obfuscated code).
    pub fn build(method: &Method) -> Cfg {
        build(method)
    }

    /// The entry block node id, if the method has any instructions.
    pub fn entry(&self) -> Option<NodeId> {
        self.entry
    }

    /// The payload (BasicBlock metadata) of a node. Returns `None` for unknown ids (never panics).
    pub fn block(&self, id: NodeId) -> Option<&BasicBlock> {
        self.graph.node(id)
    }

    /// Iterate all blocks with their node ids, in PC order.
    pub fn blocks(&self) -> impl Iterator<Item = (NodeId, &BasicBlock)> + '_ {
        self.graph.nodes()
    }

    /// Successor blocks of `id`, in insertion order; includes duplicates for parallel edges.
    pub fn successors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.graph.successors(id)
    }

    /// Predecessor blocks of `id`, in insertion order; includes duplicates for parallel edges.
    pub fn predecessors(&self, id: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.graph.predecessors(id)
    }

    /// Outgoing edges of `id` as `(target_block, edge_kind)` pairs, in insertion order.
    pub fn edges_from(&self, id: NodeId) -> impl Iterator<Item = (NodeId, EdgeKind)> + '_ {
        self.graph.out_edges(id).filter_map(move |e| {
            self.graph
                .edge_endpoints(e)
                .zip(self.graph.edge(e))
                .map(|((_, to), kind)| (to, *kind))
        })
    }

    /// Total number of blocks in this CFG.
    pub fn block_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Total number of edges in this CFG.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}

fn build(method: &Method) -> Cfg {
    let instructions = &method.instructions;

    // Map every PC (including Payload) to instruction index, and collect all real (non-Payload) indices.
    let mut pc_to_index: HashMap<u32, usize> = HashMap::with_capacity(instructions.len());
    let mut real_indices: Vec<usize> = Vec::with_capacity(instructions.len());
    for (i, ia) in instructions.iter().enumerate() {
        pc_to_index.insert(ia.offset, i);
        if !matches!(ia.instruction, Instruction::Payload) {
            real_indices.push(i);
        }
    }

    if real_indices.is_empty() {
        return Cfg {
            graph: Graph::new(),
            entry: None,
        };
    }

    // Build reverse index: for each real instruction, record its position in `real_indices`.
    // This allows O(1) lookup when marking leaders.
    let mut index_to_pos: HashMap<usize, usize> = HashMap::with_capacity(real_indices.len());
    for (pos, &idx) in real_indices.iter().enumerate() {
        index_to_pos.insert(idx, pos);
    }

    // Helper to mark a PC as a leader. Silently ignores PCs that don't map to real instructions.
    let mark_leader = |pc: u32, is_leader: &mut [bool]| {
        if let Some(&idx) = pc_to_index.get(&pc)
            && let Some(&pos) = index_to_pos.get(&idx)
        {
            is_leader[pos] = true;
        }
    };

    // Mark leaders: first instruction is always a leader.
    let mut is_leader = vec![false; real_indices.len()];
    is_leader[0] = true;

    // Mark branch/switch targets and instructions following terminators as leaders.
    for (pos, &idx) in real_indices.iter().enumerate() {
        match &instructions[idx].instruction {
            Instruction::Branch { target, .. } => {
                mark_leader(*target, &mut is_leader);
                // Instruction after conditional branch is a leader (fallthrough start).
                if pos + 1 < is_leader.len() {
                    is_leader[pos + 1] = true;
                }
            }
            Instruction::Switch { cases, .. } => {
                for case in cases {
                    mark_leader(case.target, &mut is_leader);
                }
                // Instruction after switch is a leader (default/fallthrough start).
                if pos + 1 < is_leader.len() {
                    is_leader[pos + 1] = true;
                }
            }
            Instruction::Return | Instruction::Throw if pos + 1 < is_leader.len() => {
                // Instruction after terminator is a leader (dead code block).
                is_leader[pos + 1] = true;
            }
            _ => {}
        }
    }

    // Exception handler entry points must be leaders so the exception edge
    // from the try-start block lands exactly at a block boundary.
    for try_range in &method.tries {
        for catch in &try_range.handler.catches {
            mark_leader(catch.handler_addr, &mut is_leader);
        }
        if let Some(addr) = try_range.handler.catch_all_addr {
            mark_leader(addr, &mut is_leader);
        }
    }

    // Collect block start positions (in `real_indices` space, ascending order).
    let block_starts: Vec<usize> = is_leader
        .iter()
        .enumerate()
        .filter_map(|(pos, &leader)| leader.then_some(pos))
        .collect();

    let mut graph: Graph<BasicBlock, EdgeKind> = Graph::new();
    let mut node_ids: Vec<NodeId> = Vec::with_capacity(block_starts.len());
    let mut pc_to_block: HashMap<u32, NodeId> = HashMap::with_capacity(block_starts.len());

    // Create nodes for each block, mapping start_pc -> NodeId for edge resolution.
    for (i, &start_pos) in block_starts.iter().enumerate() {
        let end_pos = block_starts
            .get(i + 1)
            .copied()
            .unwrap_or(real_indices.len());
        let start_index = real_indices[start_pos];
        let end_index = real_indices[end_pos - 1] + 1;
        let start_pc = instructions[start_index].offset;

        let node_id = graph.add_node(BasicBlock {
            start_index: start_index as u32,
            end_index: end_index as u32,
            start_pc,
        });
        node_ids.push(node_id);
        pc_to_block.insert(start_pc, node_id);
    }

    // Construct edges by analyzing the last instruction of each block.
    for (i, _) in block_starts.iter().enumerate() {
        let end_pos = block_starts
            .get(i + 1)
            .copied()
            .unwrap_or(real_indices.len());
        let last_idx = real_indices[end_pos - 1];
        let from = node_ids[i];
        let next_block = node_ids.get(i + 1).copied();

        match &instructions[last_idx].instruction {
            Instruction::Branch {
                kind: BranchKind::Goto,
                target,
            } => {
                // Unconditional goto: exactly one successor (target).
                if let Some(&to) = pc_to_block.get(target) {
                    graph.add_edge(from, to, EdgeKind::Goto);
                }
            }
            Instruction::Branch { target, .. } => {
                // Conditional branch: two successors (taken target + fallthrough).
                if let Some(&to) = pc_to_block.get(target) {
                    graph.add_edge(from, to, EdgeKind::BranchTaken);
                }
                if let Some(to) = next_block {
                    graph.add_edge(from, to, EdgeKind::Fallthrough);
                }
            }
            Instruction::Switch { cases, .. } => {
                // Switch: multiple case edges + default fallthrough.
                for case in cases {
                    if let Some(&to) = pc_to_block.get(&case.target) {
                        graph.add_edge(from, to, EdgeKind::SwitchCase);
                    }
                }
                if let Some(to) = next_block {
                    graph.add_edge(from, to, EdgeKind::Fallthrough);
                }
            }
            Instruction::Return | Instruction::Throw => {
                // Terminators: block has no successors.
            }
            _ => {
                // Non-terminating instruction: fallthrough to next block (which must be a leader).
                if let Some(to) = next_block {
                    graph.add_edge(from, to, EdgeKind::Fallthrough);
                }
            }
        }
    }

    // Add conservative exception edges: from the block containing each try-start PC
    // to all handler entry points (typed catches and catch-all).
    for try_range in &method.tries {
        let Some(from) = pc_to_index
            .get(&try_range.start_addr)
            .and_then(|&idx| index_to_pos.get(&idx))
            .and_then(|&pos| block_for_pos(pos, &block_starts, &node_ids))
        else {
            continue;
        };

        for catch in &try_range.handler.catches {
            if let Some(&to) = pc_to_block.get(&catch.handler_addr) {
                graph.add_edge(from, to, EdgeKind::Exception);
            }
        }
        if let Some(addr) = try_range.handler.catch_all_addr
            && let Some(&to) = pc_to_block.get(&addr)
        {
            graph.add_edge(from, to, EdgeKind::Exception);
        }
    }

    let entry = node_ids.first().copied();
    Cfg { graph, entry }
}

/// Find the block (NodeId) whose range covers `pos` (a position into `real_indices`).
/// Uses binary search over `block_starts` to find the block; returns `None` if `pos` precedes all blocks.
fn block_for_pos(pos: usize, block_starts: &[usize], node_ids: &[NodeId]) -> Option<NodeId> {
    let idx = block_starts.partition_point(|&s| s <= pos);
    if idx == 0 {
        None
    } else {
        node_ids.get(idx - 1).copied()
    }
}

#[cfg(test)]
mod tests;
