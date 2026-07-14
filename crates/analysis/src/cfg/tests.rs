use ir::{
    BranchKind, CatchHandler, CatchTypeAddr, ClassRef, Instruction, InstructionAt, Method,
    StringRef, SwitchCase, TryRange,
};

use super::{Cfg, EdgeKind};

fn method(instructions: Vec<InstructionAt>, tries: Vec<TryRange>) -> Method {
    Method {
        name: "Ltest;->m".to_string(),
        access_flags: 0,
        code_offset: Some(0),
        instructions,
        tries,
    }
}

fn at(offset: u32, instruction: Instruction) -> InstructionAt {
    InstructionAt {
        offset,
        instruction,
    }
}

fn nop(offset: u32) -> InstructionAt {
    at(offset, Instruction::Nop)
}

// ============================================================================
// Linear (single block)
// ============================================================================

#[test]
fn linear_method_is_a_single_block() {
    let m = method(
        vec![nop(0), nop(1), nop(2), at(3, Instruction::Return)],
        vec![],
    );
    let cfg = Cfg::build(&m);

    assert_eq!(cfg.block_count(), 1);
    assert_eq!(cfg.edge_count(), 0);

    let entry = cfg.entry().expect("entry block");
    let block = cfg.block(entry).expect("block payload");
    assert_eq!(block.start_index, 0);
    assert_eq!(block.end_index, 4);
    assert_eq!(cfg.successors(entry).count(), 0);
}

// ============================================================================
// If/else diamond
// ============================================================================

#[test]
fn if_else_diamond_has_branch_taken_and_fallthrough() {
    // 0: if-eqz -> 3 (taken)
    // 1: nop            (else branch, fallthrough from 0)
    // 2: goto -> 4       (join)
    // 3: nop            (then branch, leader: branch target)
    // 4: return          (join, leader: goto target)
    let m = method(
        vec![
            at(
                0,
                Instruction::Branch {
                    kind: BranchKind::IfEqz,
                    target: 3,
                },
            ),
            nop(1),
            at(
                2,
                Instruction::Branch {
                    kind: BranchKind::Goto,
                    target: 4,
                },
            ),
            nop(3),
            at(4, Instruction::Return),
        ],
        vec![],
    );
    let cfg = Cfg::build(&m);

    // Blocks: [0] (if), [1,2] (else+goto), [3] (then), [4] (join/return).
    assert_eq!(cfg.block_count(), 4);

    let entry = cfg.entry().expect("entry");
    let succs: Vec<_> = cfg.successors(entry).collect();
    assert_eq!(succs.len(), 2, "if block should have 2 successors");

    // Find the "then" block (starts at pc 3) and "join" block (starts at pc 4).
    let then_block = cfg
        .blocks()
        .find(|(_, b)| b.start_pc == 3)
        .map(|(id, _)| id)
        .expect("then block");
    let join_block = cfg
        .blocks()
        .find(|(_, b)| b.start_pc == 4)
        .map(|(id, _)| id)
        .expect("join block");

    assert!(succs.contains(&then_block));

    let else_block = cfg
        .blocks()
        .find(|(_, b)| b.start_pc == 1)
        .map(|(id, _)| id)
        .expect("else block");
    assert!(succs.contains(&else_block));

    // "then" falls through to join; "else+goto" jumps to join.
    let then_succs: Vec<_> = cfg.successors(then_block).collect();
    assert_eq!(then_succs, vec![join_block]);

    let else_succs: Vec<_> = cfg.successors(else_block).collect();
    assert_eq!(else_succs, vec![join_block]);

    // Join block has two predecessors.
    assert_eq!(cfg.predecessors(join_block).count(), 2);
}

// ============================================================================
// Loop (back-edge)
// ============================================================================

#[test]
fn loop_has_back_edge_to_earlier_block() {
    // 0: nop
    // 1: if-nez -> 0   (loop back to start; also falls through to 2)
    // 2: return
    let m = method(
        vec![
            nop(0),
            at(
                1,
                Instruction::Branch {
                    kind: BranchKind::IfNez,
                    target: 0,
                },
            ),
            at(2, Instruction::Return),
        ],
        vec![],
    );
    let cfg = Cfg::build(&m);

    // Blocks: [0,1] (nop + if), [2] (return). Note pc 0 is already the
    // entry leader, so the branch target doesn't split further.
    assert_eq!(cfg.block_count(), 2);

    let entry = cfg.entry().expect("entry");
    let succs: Vec<_> = cfg.successors(entry).collect();
    // Back-edge to itself (entry) + fallthrough to the return block.
    assert!(succs.contains(&entry), "expected back-edge to entry block");
    assert_eq!(succs.len(), 2);
}

// ============================================================================
// Switch
// ============================================================================

#[test]
fn switch_has_case_edges_and_default_fallthrough() {
    // 0: packed-switch -> cases at 1, 2; default falls through to 3.
    // 1: nop
    // 2: nop
    // 3: return
    let m = method(
        vec![
            at(
                0,
                Instruction::Switch {
                    packed: true,
                    cases: vec![
                        SwitchCase { key: 0, target: 1 },
                        SwitchCase { key: 1, target: 2 },
                    ],
                },
            ),
            nop(1),
            nop(2),
            at(3, Instruction::Return),
        ],
        vec![],
    );
    let cfg = Cfg::build(&m);

    let entry = cfg.entry().expect("entry");
    let edges: Vec<_> = cfg.edges_from(entry).collect();

    let case_edges = edges
        .iter()
        .filter(|(_, k)| *k == EdgeKind::SwitchCase)
        .count();
    let fallthrough_edges = edges
        .iter()
        .filter(|(_, k)| *k == EdgeKind::Fallthrough)
        .count();

    assert_eq!(case_edges, 2);
    assert_eq!(fallthrough_edges, 1, "default case is implicit fallthrough");
}

// ============================================================================
// Return/throw terminators: no successors
// ============================================================================

#[test]
fn return_and_throw_have_no_successors() {
    let m = method(vec![at(0, Instruction::Return)], vec![]);
    let cfg = Cfg::build(&m);
    let entry = cfg.entry().expect("entry");
    assert_eq!(cfg.successors(entry).count(), 0);

    let m2 = method(vec![at(0, Instruction::Throw)], vec![]);
    let cfg2 = Cfg::build(&m2);
    let entry2 = cfg2.entry().expect("entry");
    assert_eq!(cfg2.successors(entry2).count(), 0);
}

// ============================================================================
// Branch target splits the middle of a sequence
// ============================================================================

#[test]
fn branch_target_into_middle_of_sequence_splits_block() {
    // 0: goto -> 2
    // 1: nop        (dead code / unreachable in this pass; still a block)
    // 2: nop        (leader: goto target, splits [1] from [2,3])
    // 3: return
    let m = method(
        vec![
            at(
                0,
                Instruction::Branch {
                    kind: BranchKind::Goto,
                    target: 2,
                },
            ),
            nop(1),
            nop(2),
            at(3, Instruction::Return),
        ],
        vec![],
    );
    let cfg = Cfg::build(&m);

    // Blocks: [0] (goto), [1] (dead nop, leader because it follows goto),
    // [2,3] (target of goto through return).
    assert_eq!(cfg.block_count(), 3);

    let target_block = cfg
        .blocks()
        .find(|(_, b)| b.start_pc == 2)
        .map(|(id, _)| id)
        .expect("target block");

    let entry = cfg.entry().expect("entry");
    let succs: Vec<_> = cfg.successors(entry).collect();
    assert_eq!(succs, vec![target_block]);
}

// ============================================================================
// Empty / native / abstract methods
// ============================================================================

#[test]
fn empty_method_has_no_blocks_and_no_entry() {
    let m = method(vec![], vec![]);
    let cfg = Cfg::build(&m);

    assert_eq!(cfg.block_count(), 0);
    assert_eq!(cfg.edge_count(), 0);
    assert_eq!(cfg.entry(), None);
}

// ============================================================================
// Payload is never a leader / never part of a block
// ============================================================================

#[test]
fn payload_is_excluded_from_blocks() {
    // 0: switch -> case at 2; default falls through to 3.
    // 1: payload   (data, immediately after the switch; not code)
    // 2: nop
    // 3: return
    let m = method(
        vec![
            at(
                0,
                Instruction::Switch {
                    packed: true,
                    cases: vec![SwitchCase { key: 0, target: 2 }],
                },
            ),
            at(1, Instruction::Payload),
            nop(2),
            at(3, Instruction::Return),
        ],
        vec![],
    );
    let cfg = Cfg::build(&m);

    // No block should claim to start at pc 1 (the payload).
    assert!(cfg.blocks().all(|(_, b)| b.start_pc != 1));
}

// ============================================================================
// Degenerate/obfuscated: target doesn't land on a real instruction
// ============================================================================

#[test]
fn dangling_branch_target_drops_edge_without_panicking() {
    // 0: goto -> 999 (does not exist)
    let m = method(
        vec![
            at(
                0,
                Instruction::Branch {
                    kind: BranchKind::Goto,
                    target: 999,
                },
            ),
            at(1, Instruction::Return),
        ],
        vec![],
    );
    let cfg = Cfg::build(&m);

    let entry = cfg.entry().expect("entry");
    assert_eq!(cfg.successors(entry).count(), 0);
}

// ============================================================================
// Exception edges (conservative approximation)
// ============================================================================

#[test]
fn try_range_adds_exception_edge_to_handler() {
    // 0: nop         (try start)
    // 1: throw       (still inside protected range)
    // 2: nop         (catch-all handler entry)
    // 3: return
    let m = method(
        vec![
            nop(0),
            at(1, Instruction::Throw),
            nop(2),
            at(3, Instruction::Return),
        ],
        vec![TryRange {
            start_addr: 0,
            end_addr: 2,
            handler: CatchHandler {
                catches: vec![],
                catch_all_addr: Some(2),
            },
        }],
    );
    let cfg = Cfg::build(&m);

    let handler_block = cfg
        .blocks()
        .find(|(_, b)| b.start_pc == 2)
        .map(|(id, _)| id)
        .expect("handler block");

    let entry = cfg.entry().expect("entry");
    let has_exception_edge = cfg
        .edges_from(entry)
        .any(|(to, kind)| to == handler_block && kind == EdgeKind::Exception);
    assert!(has_exception_edge, "expected Exception edge to handler");
}

#[test]
fn try_range_with_typed_catch_adds_exception_edge() {
    let m = method(
        vec![
            nop(0),
            at(1, Instruction::Return),
            nop(2),
            at(3, Instruction::Return),
        ],
        vec![TryRange {
            start_addr: 0,
            end_addr: 1,
            handler: CatchHandler {
                catches: vec![CatchTypeAddr {
                    class: ClassRef { name: StringRef(0) },
                    handler_addr: 2,
                }],
                catch_all_addr: None,
            },
        }],
    );
    let cfg = Cfg::build(&m);

    let handler_block = cfg
        .blocks()
        .find(|(_, b)| b.start_pc == 2)
        .map(|(id, _)| id)
        .expect("handler block");
    let entry = cfg.entry().expect("entry");
    let has_exception_edge = cfg
        .edges_from(entry)
        .any(|(to, kind)| to == handler_block && kind == EdgeKind::Exception);
    assert!(has_exception_edge);
}

#[test]
fn dangling_try_start_does_not_panic() {
    let m = method(
        vec![at(0, Instruction::Return)],
        vec![TryRange {
            start_addr: 999,
            end_addr: 1000,
            handler: CatchHandler {
                catches: vec![],
                catch_all_addr: Some(0),
            },
        }],
    );
    let cfg = Cfg::build(&m);
    // Should not panic; no meaningful exception edge to assert on.
    assert_eq!(cfg.block_count(), 1);
}

// ============================================================================
// Self-loop: a block whose branch target is its own start PC
// ============================================================================

#[test]
fn self_loop_branch_creates_single_self_edge() {
    // 0: if-eqz -> 0 (taken target is the block's own start; self-loop)
    // 1: return      (fallthrough from 0)
    let m = method(
        vec![
            at(
                0,
                Instruction::Branch {
                    kind: BranchKind::IfEqz,
                    target: 0,
                },
            ),
            at(1, Instruction::Return),
        ],
        vec![],
    );
    let cfg = Cfg::build(&m);

    assert_eq!(cfg.block_count(), 2, "should have 2 blocks");
    assert_eq!(
        cfg.edge_count(),
        2,
        "should have 2 edges (BranchTaken to self + Fallthrough)"
    );

    let entry = cfg.entry().expect("entry");
    let block = cfg.block(entry).expect("entry block");
    assert_eq!(block.start_pc, 0);

    let edges: Vec<_> = cfg.edges_from(entry).collect();
    let self_edges: Vec<_> = edges
        .iter()
        .filter(|(to, k)| *to == entry && *k == EdgeKind::BranchTaken)
        .collect();
    assert_eq!(
        self_edges.len(),
        1,
        "should have exactly one BranchTaken self-edge"
    );

    let fallthrough_edges: Vec<_> = edges
        .iter()
        .filter(|(_, k)| *k == EdgeKind::Fallthrough)
        .collect();
    assert_eq!(
        fallthrough_edges.len(),
        1,
        "should have exactly one Fallthrough"
    );
}

// ============================================================================
// Nested loops: two back-edges (outer + inner loop)
// ============================================================================

#[test]
fn nested_loops_has_two_back_edges() {
    // 0: nop           (outer loop header)
    // 1: if-nez -> 3   (inner loop condition)
    // 2: if-nez -> 0   (outer loop condition, back to 0)
    // 3: nop           (inner loop body)
    // 4: goto -> 1     (back to inner loop header)
    // 5: return
    let m = method(
        vec![
            nop(0),
            at(
                1,
                Instruction::Branch {
                    kind: BranchKind::IfNez,
                    target: 3,
                },
            ),
            at(
                2,
                Instruction::Branch {
                    kind: BranchKind::IfNez,
                    target: 0,
                },
            ),
            nop(3),
            at(
                4,
                Instruction::Branch {
                    kind: BranchKind::Goto,
                    target: 1,
                },
            ),
            at(5, Instruction::Return),
        ],
        vec![],
    );
    let cfg = Cfg::build(&m);

    // Blocks: [0,1], [2], [3,4], [5]
    // or similar depending on leader detection. Count should be stable.
    let initial_block_count = cfg.block_count();
    assert!(
        initial_block_count >= 2,
        "should have at least 2 blocks for loops"
    );

    // Look for back-edges. A back-edge is an edge from a later block to an earlier block.
    let mut back_edge_count = 0;
    for (from_id, from_block) in cfg.blocks() {
        for to_id in cfg.successors(from_id) {
            if let Some(to_block) = cfg.block(to_id)
                && to_block.start_pc < from_block.start_pc
            {
                back_edge_count += 1;
            }
        }
    }
    assert_eq!(
        back_edge_count, 2,
        "should have exactly 2 back-edges (outer + inner loop)"
    );
}

// ============================================================================
// Unreachable block: instructions after an unconditional Goto that nothing targets
// ============================================================================

#[test]
fn unreachable_block_after_unconditional_goto_still_exists() {
    // 0: goto -> 2         (unconditional jump)
    // 1: nop               (unreachable: follows unconditional goto and no branch targets it)
    // 2: return            (target of goto)
    let m = method(
        vec![
            at(
                0,
                Instruction::Branch {
                    kind: BranchKind::Goto,
                    target: 2,
                },
            ),
            nop(1),
            at(2, Instruction::Return),
        ],
        vec![],
    );
    let cfg = Cfg::build(&m);

    // The unreachable block should still exist (CFG does not prune unreachable code).
    assert_eq!(
        cfg.block_count(),
        3,
        "CFG should include unreachable blocks (does not prune)"
    );

    // The unreachable block (at PC 1) should have no predecessors.
    let unreachable_block = cfg
        .blocks()
        .find(|(_, b)| b.start_pc == 1)
        .map(|(id, _)| id)
        .expect("unreachable block should exist");

    let preds: Vec<_> = cfg.predecessors(unreachable_block).collect();
    assert_eq!(
        preds.len(),
        0,
        "unreachable block should have no predecessors"
    );
}

// ============================================================================
// Switch cases sharing one target (parallel edges in switch)
// ============================================================================

#[test]
fn switch_multiple_cases_sharing_target_creates_parallel_edges() {
    // 0: packed-switch -> case 0 targets 2, case 1 targets 2 (same target!)
    // 1: nop             (fallthrough default)
    // 2: nop             (target of both cases)
    // 3: return
    let m = method(
        vec![
            at(
                0,
                Instruction::Switch {
                    packed: true,
                    cases: vec![
                        SwitchCase { key: 0, target: 2 },
                        SwitchCase { key: 1, target: 2 }, // Same target as case 0
                    ],
                },
            ),
            nop(1),
            nop(2),
            at(3, Instruction::Return),
        ],
        vec![],
    );
    let cfg = Cfg::build(&m);

    let entry = cfg.entry().expect("entry");
    let target_block = cfg
        .blocks()
        .find(|(_, b)| b.start_pc == 2)
        .map(|(id, _)| id)
        .expect("target block");

    let edges: Vec<_> = cfg.edges_from(entry).collect();
    let case_edges_to_target: Vec<_> = edges
        .iter()
        .filter(|(to, k)| *to == target_block && *k == EdgeKind::SwitchCase)
        .collect();

    // Should have 2 SwitchCase edges to the same target (parallel edges).
    assert_eq!(
        case_edges_to_target.len(),
        2,
        "should have 2 SwitchCase edges to the same target block"
    );

    // Also verify fallthrough exists.
    let fallthrough_edges: Vec<_> = edges
        .iter()
        .filter(|(_, k)| *k == EdgeKind::Fallthrough)
        .collect();
    assert_eq!(
        fallthrough_edges.len(),
        1,
        "should have exactly one Fallthrough"
    );
}

// ============================================================================
// Identical branch targets: if-* where taken target == fallthrough PC
// ============================================================================

#[test]
fn branch_with_target_equal_to_next_instruction_creates_parallel_edges() {
    // 0: if-eqz -> 1 (taken target is the next instruction, same as fallthrough)
    // 1: nop
    // 2: return
    let m = method(
        vec![
            at(
                0,
                Instruction::Branch {
                    kind: BranchKind::IfEqz,
                    target: 1,
                },
            ),
            nop(1),
            at(2, Instruction::Return),
        ],
        vec![],
    );
    let cfg = Cfg::build(&m);

    let entry = cfg.entry().expect("entry");
    let next_block = cfg
        .blocks()
        .find(|(_, b)| b.start_pc == 1)
        .map(|(id, _)| id)
        .expect("next block");

    let edges: Vec<_> = cfg.edges_from(entry).collect();
    let to_next: Vec<_> = edges.iter().filter(|(to, _)| *to == next_block).collect();

    // Both BranchTaken and Fallthrough target the same block, creating 2 edges.
    assert_eq!(
        to_next.len(),
        2,
        "should have 2 edges to the same successor (BranchTaken + Fallthrough)"
    );

    let kinds: Vec<_> = to_next.iter().map(|(_, k)| *k).collect();
    assert!(kinds.contains(&EdgeKind::BranchTaken));
    assert!(kinds.contains(&EdgeKind::Fallthrough));
}

// ============================================================================
// Native/abstract methods: no instructions, access_flags & 0x100 / 0x400
// ============================================================================

#[test]
fn native_method_with_no_code_has_empty_cfg() {
    // Native methods have access_flags & 0x100 and no code.
    let m = Method {
        name: "Ltest;->native_m".to_string(),
        access_flags: 0x100, // 0x100 = native flag
        code_offset: None,
        instructions: vec![],
        tries: vec![],
    };
    let cfg = Cfg::build(&m);

    assert_eq!(cfg.block_count(), 0, "native method should have empty CFG");
    assert_eq!(cfg.entry(), None, "native method should have no entry");
    assert_eq!(cfg.edge_count(), 0);
}

#[test]
fn abstract_method_with_no_code_has_empty_cfg() {
    // Abstract methods have access_flags & 0x400 and no code.
    let m = Method {
        name: "Ltest;->abstract_m".to_string(),
        access_flags: 0x400, // 0x400 = abstract flag
        code_offset: None,
        instructions: vec![],
        tries: vec![],
    };
    let cfg = Cfg::build(&m);

    assert_eq!(
        cfg.block_count(),
        0,
        "abstract method should have empty CFG"
    );
    assert_eq!(cfg.entry(), None, "abstract method should have no entry");
    assert_eq!(cfg.edge_count(), 0);
}
