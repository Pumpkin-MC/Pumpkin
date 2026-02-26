use super::*;
use crate::chunk_system::dag::Node;
use slotmap::Key;
use std::collections::BinaryHeap;

#[test]
fn ensure_dependency_chain_builds_multistage_chain() {
    let mut graph = DAG::default();
    let mut queue = BinaryHeap::new();
    let last_level: ChunkLevel = HashMapType::default();
    let last_high_priority: Vec<ChunkPos> = Vec::new();

    let chunk_pos = ChunkPos::new(0, 0);

    // Create a dependency node in the graph which will depend on the chain
    let dependency_task = graph
        .nodes
        .insert(Node::new(ChunkPos::new(10, 10), StagedChunkEnum::Features));

    let mut holder = ChunkHolder {
        current_stage: StagedChunkEnum::None,
        ..Default::default()
    };

    // Build a chain up to Surface (Empty -> ... -> Surface)
    GenerationSchedule::ensure_dependency_chain(
        &mut graph,
        &mut queue,
        &last_level,
        &last_high_priority,
        dependency_task,
        chunk_pos,
        &mut holder,
        StagedChunkEnum::Surface,
    );

    let start = (holder.current_stage as usize + 1).max(StagedChunkEnum::Empty as usize);
    let end = StagedChunkEnum::Surface as u8 as usize;

    // Ensure tasks were created, present in the DAG, and correctly chained
    for idx in start..=end {
        let key = holder.tasks[idx];
        assert!(!key.is_null(), "task {} was not created", idx);

        let node = graph.nodes.get(key).expect("graph missing node");

        if idx == start {
            assert_eq!(node.in_degree, 0, "Start task should have 0 in_degree");
        } else {
            assert_eq!(
                node.in_degree, 1,
                "Intermediate task {} should have in_degree of 1",
                idx
            );
        }
    }

    // The dependency node should have its in_degree incremented
    let dep_node = graph.nodes.get(dependency_task).unwrap();
    assert_eq!(dep_node.in_degree, 1);

    // The entry task should have been queued
    let queued = queue.pop().expect("queue should have entry task");
    assert_eq!(queued.node_key(), holder.tasks[start]);
}

#[test]
fn ensure_dependency_chain_resumes_partial_chain() {
    let mut graph = DAG::default();
    let mut queue = BinaryHeap::new();
    let last_level: ChunkLevel = HashMapType::default();
    let last_high_priority: Vec<ChunkPos> = Vec::new();

    let chunk_pos = ChunkPos::new(0, 0);
    let dependency_task = graph
        .nodes
        .insert(Node::new(ChunkPos::new(10, 10), StagedChunkEnum::Features));

    let mut holder = ChunkHolder {
        current_stage: StagedChunkEnum::Biomes,
        ..Default::default()
    };

    GenerationSchedule::ensure_dependency_chain(
        &mut graph,
        &mut queue,
        &last_level,
        &last_high_priority,
        dependency_task,
        chunk_pos,
        &mut holder,
        StagedChunkEnum::Surface,
    );

    // Dynamically calculate the next stage after Biomes instead of guessing
    let empty = StagedChunkEnum::Empty as usize;
    let start = (holder.current_stage as usize + 1).max(empty);

    let queued = queue.pop().expect("queue should have entry task");
    assert_eq!(
        queued.node_key(),
        holder.tasks[start],
        "Should resume directly from the next stage after Biomes"
    );

    let entry_node = graph.nodes.get(queued.node_key()).unwrap();
    assert_eq!(
        entry_node.in_degree, 0,
        "Resumed task should have 0 in_degree because previous stages are already done"
    );
}

#[test]
fn ensure_dependency_chain_does_nothing_if_already_met() {
    let mut graph = DAG::default();
    let mut queue = BinaryHeap::new();
    let last_level: ChunkLevel = HashMapType::default();
    let last_high_priority: Vec<ChunkPos> = Vec::new();

    let chunk_pos = ChunkPos::new(0, 0);
    let dependency_task = graph
        .nodes
        .insert(Node::new(ChunkPos::new(10, 10), StagedChunkEnum::Features));

    let mut holder = ChunkHolder {
        current_stage: StagedChunkEnum::Full,
        ..Default::default()
    };

    GenerationSchedule::ensure_dependency_chain(
        &mut graph,
        &mut queue,
        &last_level,
        &last_high_priority,
        dependency_task,
        chunk_pos,
        &mut holder,
        StagedChunkEnum::Surface, // Requesting a lower stage than it currently is
    );

    // Ensure the function returned early without creating any tasks or queueing anything
    for task in holder.tasks.iter() {
        assert!(
            task.is_null(),
            "No tasks should be created if the stage requirement is already met"
        );
    }
    assert!(queue.is_empty(), "Nothing should be queued");
}

#[test]
fn ensure_dependency_chain_respects_occupied_lock() {
    let mut graph = DAG::default();
    let mut queue = BinaryHeap::new();
    let last_level: ChunkLevel = HashMapType::default();
    let last_high_priority: Vec<ChunkPos> = Vec::new();

    let chunk_pos = ChunkPos::new(0, 0);
    let dependency_task = graph
        .nodes
        .insert(Node::new(ChunkPos::new(10, 10), StagedChunkEnum::Features));

    // Create an "occupy" node to simulate another thread/process currently working on this chunk
    let occupy_node = graph.nodes.insert(Node::new(
        ChunkPos::new(i32::MAX, i32::MAX),
        StagedChunkEnum::None,
    ));

    let mut holder = ChunkHolder {
        current_stage: StagedChunkEnum::None,
        occupied: occupy_node,
        ..Default::default()
    };

    GenerationSchedule::ensure_dependency_chain(
        &mut graph,
        &mut queue,
        &last_level,
        &last_high_priority,
        dependency_task,
        chunk_pos,
        &mut holder,
        StagedChunkEnum::Surface,
    );

    let start = StagedChunkEnum::Empty as usize;
    let entry_task = holder.tasks[start];

    let entry_node = graph.nodes.get(entry_task).unwrap();
    // The first task should depend on the occupy node finishing!
    assert_eq!(
        entry_node.in_degree, 1,
        "Entry task should be blocked by the occupy node"
    );

    // Therefore, the queue MUST be empty. It shouldn't fire until the occupy node drops.
    assert!(
        queue.is_empty(),
        "Task should not be queued because it is blocked by occupied status"
    );
}
