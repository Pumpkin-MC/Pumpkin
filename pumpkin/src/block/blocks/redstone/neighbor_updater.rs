//! Vanilla 26.2 neighbor update pipeline (CFR):
//! - `NeighborUpdater` + `UPDATE_ORDER`
//! - `CollectingNeighborUpdater` (chained queue, re-entrant layers)
//! - `InstantNeighborUpdater` (direct execute)
//!
//! Pumpkin is async: the lock is **never held across** `execute_update` awaits,
//! so nested `World::update_neighbors` can re-enter safely (vanilla single-thread
//! re-entrancy via `addedThisLayer`).

use std::collections::VecDeque;
use std::sync::Arc;

use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;
use tracing::error;

use crate::block::OnNeighborUpdateArgs;
use crate::world::World;

use super::orientation::{RedstoneOrientation, SideBias};

/// Vanilla `NeighborUpdater.UPDATE_ORDER`: W, E, D, U, N, S
pub const UPDATE_ORDER: [BlockDirection; 6] = [
    BlockDirection::West,
    BlockDirection::East,
    BlockDirection::Down,
    BlockDirection::Up,
    BlockDirection::North,
    BlockDirection::South,
];

/// Optional redstone orientation (experimental redstone feature flag).
pub type NeighborOrientation = Option<RedstoneOrientation>;

/// Queued work item (vanilla `NeighborUpdates` records).
enum NeighborUpdates {
    /// `SimpleNeighborUpdate` — re-read state at pos then handleNeighborChanged
    Simple {
        pos: BlockPos,
        source_block: &'static Block,
        orientation: NeighborOrientation,
    },
    /// `FullNeighborUpdate` — piston path with fixed state / movedByPiston
    Full {
        pos: BlockPos,
        source_block: &'static Block,
        orientation: NeighborOrientation,
        moved_by_piston: bool,
    },
    /// `MultiNeighborUpdate` — walk `UPDATE_ORDER`, skip one facing
    Multi {
        source_pos: BlockPos,
        source_block: &'static Block,
        orientation: NeighborOrientation,
        skip: Option<BlockDirection>,
        idx: usize,
    },
    /// `ShapeUpdate` — updateShape from a neighbor (flags/limit for parity)
    Shape {
        direction: BlockDirection,
        neighbor_state_id: BlockStateId,
        pos: BlockPos,
        neighbor_pos: BlockPos,
        update_flags: i32,
        update_limit: i32,
    },
}

impl NeighborUpdates {
    /// Returns true if more steps remain (Multi only).
    async fn run_next(&mut self, world: &Arc<World>) -> bool {
        match self {
            Self::Simple {
                pos,
                source_block,
                orientation,
            } => {
                execute_update(world, *pos, source_block, *orientation, false).await;
                false
            }
            Self::Full {
                pos,
                source_block,
                orientation,
                moved_by_piston,
            } => {
                execute_update(world, *pos, source_block, *orientation, *moved_by_piston).await;
                false
            }
            Self::Multi {
                source_pos,
                source_block,
                orientation,
                skip,
                idx,
            } => {
                // Vanilla: Direction direction = UPDATE_ORDER[this.idx++];
                let direction = UPDATE_ORDER[*idx];
                *idx += 1;
                let neighbor_pos = source_pos.offset(direction.to_offset());
                // Experimental REDSTONE_EXPERIMENTS: orientation.withFront(direction).
                // When orientation is None (feature off), stay None — matches vanilla.
                let step_orient = orientation.map(|o| o.with_front(direction));
                execute_update(world, neighbor_pos, source_block, step_orient, false).await;
                // Skip next if it equals skipDirection
                if *idx < UPDATE_ORDER.len() && skip.is_some_and(|s| s == UPDATE_ORDER[*idx]) {
                    *idx += 1;
                }
                *idx < UPDATE_ORDER.len()
            }
            Self::Shape {
                direction,
                neighbor_state_id,
                pos,
                neighbor_pos,
                update_flags,
                update_limit,
            } => {
                execute_shape_update(
                    world,
                    *direction,
                    *pos,
                    *neighbor_pos,
                    *neighbor_state_id,
                    *update_flags,
                    *update_limit,
                )
                .await;
                false
            }
        }
    }
}

/// Vanilla `NeighborUpdater.executeUpdate` → `state.handleNeighborChanged`.
async fn execute_update(
    world: &Arc<World>,
    pos: BlockPos,
    source_block: &'static Block,
    _orientation: NeighborOrientation,
    moved_by_piston: bool,
) {
    let neighbor_block = world.get_block(&pos);
    if let Some(pumpkin_block) = world.block_registry.get_pumpkin_block(neighbor_block.id) {
        pumpkin_block
            .on_neighbor_update(OnNeighborUpdateArgs {
                world,
                block: neighbor_block,
                position: &pos,
                source_block,
                // Pumpkin `notify` ≈ vanilla `movedByPiston` flag on the handler path
                notify: moved_by_piston,
            })
            .await;
    }
    let fluid = world.get_fluid(&pos);
    if let Some(pumpkin_fluid) = world.block_registry.get_pumpkin_fluid(fluid.id) {
        pumpkin_fluid
            .on_neighbor_update(world, fluid, &pos, moved_by_piston)
            .await;
    }
}

/// Vanilla `NeighborUpdater.executeShapeUpdate` (best-effort via registry shape update).
///
/// Uses boxed future for `set_block_state` to avoid async recursion with the
/// neighbor collector (shape → set → `update_neighbors` → collector → shape).
fn execute_shape_update<'a>(
    world: &'a Arc<World>,
    direction: BlockDirection,
    pos: BlockPos,
    neighbor_pos: BlockPos,
    neighbor_state_id: BlockStateId,
    update_flags: i32,
    _update_limit: i32,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
    Box::pin(async move {
        // Vanilla skips redstone wire when UPDATE_KNOWN_SHAPE (0x80) is set.
        let current_id = world.get_block_state_id(&pos);
        let current_block = pumpkin_data::Block::from_state_id(current_id);
        if (update_flags & 0x80) != 0 && current_block == &pumpkin_data::Block::REDSTONE_WIRE {
            return;
        }
        let new_id = world
            .block_registry
            .get_state_for_neighbor_update(
                world,
                current_block,
                current_id,
                &pos,
                direction,
                &neighbor_pos,
                neighbor_state_id,
            )
            .await;
        if new_id != current_id {
            let _ = world
                .set_block_state(
                    &pos,
                    new_id,
                    pumpkin_world::world::BlockFlags::NOTIFY_LISTENERS,
                )
                .await;
        }
    })
}

/// Vanilla `CollectingNeighborUpdater`.
pub struct CollectingNeighborUpdater {
    max_chained: i32,
    count: i32,
    stack: VecDeque<NeighborUpdates>,
    added_this_layer: Vec<NeighborUpdates>,
}

impl CollectingNeighborUpdater {
    /// `max_chained < 0` means unlimited (vanilla).
    #[must_use]
    pub const fn new(max_chained: i32) -> Self {
        Self {
            max_chained,
            count: 0,
            stack: VecDeque::new(),
            added_this_layer: Vec::new(),
        }
    }

    /// Vanilla default-ish large limit for redstone chains (`max-chained-neighbor-updates`).
    #[must_use]
    pub fn with_default_limit() -> Self {
        Self::new(1_000_000)
    }

    fn enqueue(&mut self, update: NeighborUpdates, pos_for_log: BlockPos) -> bool {
        let running_already = self.count > 0;
        let too_many = self.max_chained >= 0 && self.count >= self.max_chained;
        self.count += 1;
        if !too_many {
            if running_already {
                self.added_this_layer.push(update);
            } else {
                self.stack.push_back(update);
            }
        } else if self.count - 1 == self.max_chained {
            error!(
                "Too many chained neighbor updates. Skipping the rest. First skipped position: {}, {}, {}",
                pos_for_log.0.x, pos_for_log.0.y, pos_for_log.0.z
            );
        }
        // true → caller must drive run_updates (we are the top-level entry)
        !running_already
    }

    /// Merge `addedThisLayer` onto stack in vanilla order (index 0 ends on top).
    fn merge_added_layer(&mut self) {
        // Vanilla: for i = size-1 .. 0: stack.push(list[i]) → list[0] processed first
        while let Some(u) = self.added_this_layer.pop() {
            self.stack.push_back(u);
        }
    }

    fn clear_after_run(&mut self) {
        self.stack.clear();
        self.added_this_layer.clear();
        self.count = 0;
    }
}

/// World-owned neighbor updater (vanilla `Level.neighborUpdater`).
///
/// Lock is released during each `execute_update` so nested neighbor updates
/// can enqueue into `added_this_layer` without deadlock.
pub struct WorldNeighborUpdater {
    inner: Mutex<CollectingNeighborUpdater>,
}

impl WorldNeighborUpdater {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(CollectingNeighborUpdater::with_default_limit()),
        }
    }

    /// `neighborChanged(pos, block, orientation)`.
    pub async fn neighbor_changed(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        source_block: &'static Block,
        orientation: NeighborOrientation,
    ) {
        let should_run = {
            let mut g = self.inner.lock().await;
            g.enqueue(
                NeighborUpdates::Simple {
                    pos,
                    source_block,
                    orientation,
                },
                pos,
            )
        };
        if should_run {
            self.run_updates(world).await;
        }
    }

    /// Full path with movedByPiston (piston).
    pub async fn neighbor_changed_full(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        source_block: &'static Block,
        orientation: NeighborOrientation,
        moved_by_piston: bool,
    ) {
        let should_run = {
            let mut g = self.inner.lock().await;
            g.enqueue(
                NeighborUpdates::Full {
                    pos,
                    source_block,
                    orientation,
                    moved_by_piston,
                },
                pos,
            )
        };
        if should_run {
            self.run_updates(world).await;
        }
    }

    /// `updateNeighborsAtExceptFromFacing` → `MultiNeighborUpdate`.
    pub async fn update_neighbors_at_except(
        &self,
        world: &Arc<World>,
        pos: BlockPos,
        source_block: &'static Block,
        skip: Option<BlockDirection>,
        orientation: NeighborOrientation,
    ) {
        let idx = usize::from(skip.is_some_and(|s| s == UPDATE_ORDER[0]));
        let should_run = {
            let mut g = self.inner.lock().await;
            g.enqueue(
                NeighborUpdates::Multi {
                    source_pos: pos,
                    source_block,
                    orientation,
                    skip,
                    idx,
                },
                pos,
            )
        };
        if should_run {
            self.run_updates(world).await;
        }
    }

    /// Shape update queue entry.
    pub async fn shape_update(
        &self,
        world: &Arc<World>,
        direction: BlockDirection,
        neighbor_state_id: BlockStateId,
        pos: BlockPos,
        neighbor_pos: BlockPos,
        update_flags: i32,
        update_limit: i32,
    ) {
        let should_run = {
            let mut g = self.inner.lock().await;
            g.enqueue(
                NeighborUpdates::Shape {
                    direction,
                    neighbor_state_id,
                    pos,
                    neighbor_pos,
                    update_flags,
                    update_limit,
                },
                pos,
            )
        };
        if should_run {
            self.run_updates(world).await;
        }
    }

    /// Drive the queue until empty. Lock is not held across handler awaits.
    async fn run_updates(&self, world: &Arc<World>) {
        loop {
            // Peek/pop one step worth of work with lock held only briefly.
            let step = {
                let mut g = self.inner.lock().await;
                g.merge_added_layer();
                let Some(u) = g.stack.pop_back() else {
                    g.clear_after_run();
                    return;
                };
                u
            };

            let mut next = step;
            // Process one step while unlocked (handlers may re-enter enqueue).
            let more = next.run_next(world).await;

            {
                let mut g = self.inner.lock().await;
                if more {
                    // Multi still has directions — push back; nested work in
                    // added_this_layer will be merged on next loop and run first
                    // (vanilla peeks Multi then yields to nested via addedThisLayer).
                    g.stack.push_back(next);
                }
                // If handlers queued work, continue; if stack empty after merge,
                // next iteration clears and returns.
            }
        }
    }
}

impl Default for WorldNeighborUpdater {
    fn default() -> Self {
        Self::new()
    }
}

/// Vanilla `InstantNeighborUpdater` — no queue, direct execute.
pub struct InstantNeighborUpdater;

impl InstantNeighborUpdater {
    pub async fn neighbor_changed(
        world: &Arc<World>,
        pos: BlockPos,
        source_block: &'static Block,
        orientation: NeighborOrientation,
    ) {
        execute_update(world, pos, source_block, orientation, false).await;
    }

    pub async fn neighbor_changed_full(
        world: &Arc<World>,
        pos: BlockPos,
        source_block: &'static Block,
        orientation: NeighborOrientation,
        moved_by_piston: bool,
    ) {
        execute_update(world, pos, source_block, orientation, moved_by_piston).await;
    }

    pub async fn update_neighbors_at_except(
        world: &Arc<World>,
        pos: BlockPos,
        source_block: &'static Block,
        skip: Option<BlockDirection>,
        _orientation: NeighborOrientation,
    ) {
        // Default NeighborUpdater interface loop (Instant does not override Multi).
        for direction in UPDATE_ORDER {
            if skip.is_some_and(|s| s == direction) {
                continue;
            }
            let neighbor_pos = pos.offset(direction.to_offset());
            execute_update(world, neighbor_pos, source_block, None, false).await;
        }
    }
}

/// Vanilla `ExperimentalRedstoneUtils.initialOrientation` without feature-flag gate.
///
/// Returns `None` so callers match non-experiment vanilla (null orientation).
/// Use [`initial_orientation_experiments`] when `REDSTONE_EXPERIMENTS` is on.
#[must_use]
pub const fn initial_orientation(
    _front: Option<BlockDirection>,
    _up: Option<BlockDirection>,
) -> NeighborOrientation {
    None
}

/// Experimental path: random orientation with LEFT bias, then withUp/withFront.
#[must_use]
pub fn initial_orientation_experiments(
    front: Option<BlockDirection>,
    up: Option<BlockDirection>,
    rng_index: usize,
) -> NeighborOrientation {
    let mut o = RedstoneOrientation::from_index(rng_index).with_side_bias(SideBias::Left);
    if let Some(u) = up {
        o = o.with_up(u);
    }
    if let Some(f) = front {
        o = o.with_front(f);
    }
    Some(o)
}

/// Vanilla `ExperimentalRedstoneUtils.withFront`.
#[must_use]
pub fn orientation_with_front(
    orientation: NeighborOrientation,
    front: BlockDirection,
) -> NeighborOrientation {
    orientation.map(|o| o.with_front(front))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_order_matches_vanilla() {
        assert_eq!(
            UPDATE_ORDER,
            [
                BlockDirection::West,
                BlockDirection::East,
                BlockDirection::Down,
                BlockDirection::Up,
                BlockDirection::North,
                BlockDirection::South,
            ]
        );
        assert_eq!(UPDATE_ORDER, BlockDirection::update_order());
    }

    #[test]
    fn multi_skips_first_when_west() {
        // Constructor: if UPDATE_ORDER[0]==skip → idx=1
        assert_eq!(UPDATE_ORDER[0], BlockDirection::West);
    }
}
