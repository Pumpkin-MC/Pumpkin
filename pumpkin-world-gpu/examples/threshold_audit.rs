//! Audits how exposed the compiled routers are to the f32 branch-flip described in
//! `lib.rs`.
//!
//! `RangeChoice` and `IntervalSelect` compare a selector against a fixed threshold. When
//! the selector lands on the threshold, one ulp of rounding decides which subgraph is
//! evaluated, so the CPU and GPU can return values from different branches rather than
//! near-equal numbers. This reports how many such nodes exist and how often a real
//! sample lands close enough to a threshold to be at risk.
//!
//! Run with: `cargo run -p pumpkin-world-gpu --release --example threshold_audit`

// Reporting the audit to the console is the entire point of this example.
#![expect(clippy::print_stdout)]

use pumpkin_data::noise_router::{NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER};
use pumpkin_world::generation::GlobalRandomConfig;
use pumpkin_world_gpu::graph::{
    BeardifierData, CompiledGraph, OpCode, compile_router, evaluate_cpu_node,
};

/// How close a selector has to sit to a threshold before rounding could flip it.
/// f32 has ~7 significant digits, so this is generous for values of order 1-100.
const AT_RISK: f32 = 1e-4;

fn audit(name: &str, compiled: &CompiledGraph) {
    let beardifier = BeardifierData::default();

    let range_nodes: Vec<u32> = compiled
        .instructions
        .iter()
        .enumerate()
        .filter(|(_, i)| i.opcode == OpCode::RangeChoice as u32)
        .map(|(idx, _)| idx as u32)
        .collect();
    let interval_nodes: Vec<u32> = compiled
        .instructions
        .iter()
        .enumerate()
        .filter(|(_, i)| i.opcode == OpCode::IntervalSelect as u32)
        .map(|(idx, _)| idx as u32)
        .collect();

    // A spread of real block coordinates across chunks and the full height range.
    let points: Vec<[f32; 3]> = (0..2000)
        .map(|i| {
            let f = i as f32;
            [
                (f * 7.0) % 512.0 - 256.0,
                (f % 384.0) - 64.0,
                (f * 11.0) % 512.0 - 256.0,
            ]
        })
        .collect();

    let mut range_at_risk = 0usize;
    let mut range_samples = 0usize;
    for &node in &range_nodes {
        let instruction = compiled.instructions[node as usize];
        let selector = instruction.input0;
        for point in &points {
            let value = evaluate_cpu_node(
                compiled,
                &beardifier,
                selector,
                point[0],
                point[1],
                point[2],
            );
            range_samples += 1;
            if (value - instruction.param0).abs() < AT_RISK
                || (value - instruction.param1).abs() < AT_RISK
            {
                range_at_risk += 1;
            }
        }
    }

    let mut interval_at_risk = 0usize;
    let mut interval_samples = 0usize;
    for &node in &interval_nodes {
        let instruction = compiled.instructions[node as usize];
        let selector = instruction.input0;
        let entries = &compiled.interval_entries
            [instruction.aux0 as usize..(instruction.aux0 + instruction.aux1) as usize];
        for point in &points {
            let value = evaluate_cpu_node(
                compiled,
                &beardifier,
                selector,
                point[0],
                point[1],
                point[2],
            );
            interval_samples += 1;
            if entries
                .iter()
                .any(|e| e.threshold.is_finite() && (value - e.threshold).abs() < AT_RISK)
            {
                interval_at_risk += 1;
            }
        }
    }

    report(
        name,
        compiled,
        &points,
        &range_nodes,
        &RiskCounts {
            range_nodes: range_nodes.len(),
            range_at_risk,
            range_samples,
            interval_nodes: interval_nodes.len(),
            interval_at_risk,
            interval_samples,
        },
    );
}

struct RiskCounts {
    range_nodes: usize,
    range_at_risk: usize,
    range_samples: usize,
    interval_nodes: usize,
    interval_at_risk: usize,
    interval_samples: usize,
}

fn report(
    name: &str,
    compiled: &CompiledGraph,
    points: &[[f32; 3]],
    range_nodes: &[u32],
    counts: &RiskCounts,
) {
    let beardifier = BeardifierData::default();
    let &RiskCounts {
        range_nodes: range_node_count,
        range_at_risk,
        range_samples,
        interval_nodes,
        interval_at_risk,
        interval_samples,
    } = counts;

    println!("\n{name}: {} instructions", compiled.instructions.len());
    println!(
        "  RangeChoice nodes:    {range_node_count:>3}   at-risk samples: {range_at_risk}/{range_samples}"
    );
    println!(
        "  IntervalSelect nodes: {interval_nodes:>3}   at-risk samples: {interval_at_risk}/{interval_samples}"
    );

    // Name the specific nodes that are actually landing on a boundary.
    for &node in range_nodes {
        let instruction = compiled.instructions[node as usize];
        let hits = points
            .iter()
            .filter(|p| {
                let v =
                    evaluate_cpu_node(compiled, &beardifier, instruction.input0, p[0], p[1], p[2]);
                (v - instruction.param0).abs() < AT_RISK || (v - instruction.param1).abs() < AT_RISK
            })
            .count();
        if hits > 0 {
            println!(
                "    node {node}: selector {} on [{}, {}) -> {hits} of {} samples on the edge",
                instruction.input0,
                instruction.param0,
                instruction.param1,
                points.len()
            );
        }
    }
}

fn main() {
    let config = GlobalRandomConfig::new(1234, false);

    for (name, router) in [
        ("overworld", &OVERWORLD_BASE_NOISE_ROUTER.noise),
        ("nether", &NETHER_BASE_NOISE_ROUTER.noise),
    ] {
        match compile_router(router, &config) {
            Ok(compiled) => audit(name, &compiled),
            Err(e) => println!("{name}: does not lower ({e})"),
        }
    }
}
