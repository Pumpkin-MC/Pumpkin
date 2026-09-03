use std::{cell::RefCell, sync::Arc};

use pumpkin_data::{Block, BlockDirection, BlockId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use tracing::error;

use super::World;

const MAX_CHAINED_UPDATES: usize = 1_000_000;

#[derive(Clone, Copy)]
enum UpdateKind {
    Neighbor {
        position: BlockPos,
        source_position: BlockPos,
        source_block: BlockId,
        include_fluid: bool,
    },
    Shape {
        position: BlockPos,
        direction: BlockDirection,
        flags: BlockFlags,
    },
}

impl UpdateKind {
    const fn position(&self) -> &BlockPos {
        match self {
            Self::Neighbor { position, .. } | Self::Shape { position, .. } => position,
        }
    }
}

struct QueuedUpdate<W> {
    world: W,
    kind: UpdateKind,
}

struct UpdateStack<W> {
    remaining: Vec<QueuedUpdate<W>>,
    staged: Vec<QueuedUpdate<W>>,
    submitted: usize,
    limit: usize,
    gave_up: bool,
}

impl<W: Clone> UpdateStack<W> {
    const fn new(limit: usize) -> Self {
        Self {
            remaining: Vec::new(),
            staged: Vec::new(),
            submitted: 0,
            limit,
            gave_up: false,
        }
    }

    fn stage(&mut self, world: &W, kinds: impl Iterator<Item = UpdateKind>) {
        for kind in kinds {
            if self.submitted >= self.limit {
                if !self.gave_up {
                    self.gave_up = true;
                    error!(
                        "Skipping block updates chained beyond {}, starting at {}",
                        self.limit,
                        kind.position()
                    );
                }
                return;
            }
            self.submitted += 1;
            self.staged.push(QueuedUpdate {
                world: world.clone(),
                kind,
            });
        }
    }

    fn promote_staged(&mut self) {
        self.remaining.extend(self.staged.drain(..).rev());
    }
}

thread_local! {
    static PENDING_UPDATES: RefCell<Option<UpdateStack<Arc<World>>>> = const { RefCell::new(None) };
}

struct CascadeGuard;

impl Drop for CascadeGuard {
    fn drop(&mut self) {
        PENDING_UPDATES.with(|pending| *pending.borrow_mut() = None);
    }
}

pub(super) fn update_neighbors(
    world: &Arc<World>,
    position: &BlockPos,
    except: Option<BlockDirection>,
) {
    let source_block = world.get_block(position).id;
    submit(
        world,
        BlockDirection::update_order()
            .into_iter()
            .filter(|direction| except != Some(*direction))
            .map(|direction| UpdateKind::Neighbor {
                position: position.offset(direction.to_offset()),
                source_position: *position,
                source_block,
                include_fluid: true,
            }),
    );
}

pub(super) fn update_neighbor(world: &Arc<World>, position: &BlockPos, source_block: &Block) {
    submit(
        world,
        std::iter::once(UpdateKind::Neighbor {
            position: *position,
            source_position: *position,
            source_block: source_block.id,
            include_fluid: false,
        }),
    );
}

pub(super) fn update_shape(
    world: &Arc<World>,
    position: &BlockPos,
    direction: BlockDirection,
    flags: BlockFlags,
) {
    submit(
        world,
        std::iter::once(UpdateKind::Shape {
            position: *position,
            direction,
            flags,
        }),
    );
}

fn submit(world: &Arc<World>, updates: impl Iterator<Item = UpdateKind>) {
    let nested = PENDING_UPDATES.with(|pending| {
        let mut pending = pending.borrow_mut();
        if let Some(stack) = pending.as_mut() {
            stack.stage(world, updates);
            true
        } else {
            let mut stack = UpdateStack::new(MAX_CHAINED_UPDATES);
            stack.stage(world, updates);
            stack.promote_staged();
            *pending = Some(stack);
            false
        }
    });
    if nested {
        return;
    }

    let _guard = CascadeGuard;
    loop {
        let update = PENDING_UPDATES.with(|pending| {
            pending
                .borrow_mut()
                .as_mut()
                .and_then(|stack| stack.remaining.pop())
        });
        let Some(update) = update else {
            break;
        };
        execute(update);
        PENDING_UPDATES.with(|pending| {
            if let Some(stack) = pending.borrow_mut().as_mut() {
                stack.promote_staged();
            }
        });
    }
}

fn execute(update: QueuedUpdate<Arc<World>>) {
    let QueuedUpdate { world, kind } = update;
    match kind {
        UpdateKind::Neighbor {
            position,
            source_position,
            source_block,
            include_fluid,
        } => world.execute_neighbor_update(
            &position,
            &source_position,
            source_block.to_block(),
            include_fluid,
        ),
        UpdateKind::Shape {
            position,
            direction,
            flags,
        } => world.execute_shape_update(&position, direction, flags),
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_CHAINED_UPDATES, UpdateKind, UpdateStack};
    use pumpkin_data::BlockId;
    use pumpkin_util::math::position::BlockPos;

    fn neighbor_at(x: i32) -> UpdateKind {
        let position = BlockPos::new(x, 0, 0);
        UpdateKind::Neighbor {
            position,
            source_position: position,
            source_block: BlockId::AIR,
            include_fluid: true,
        }
    }

    fn remaining_positions(stack: &UpdateStack<()>) -> Vec<i32> {
        stack
            .remaining
            .iter()
            .map(|update| update.kind.position().0.x)
            .collect()
    }

    #[test]
    fn nested_updates_run_before_remaining_siblings_without_recursion() {
        let mut stack = UpdateStack::new(MAX_CHAINED_UPDATES);
        stack.stage(&(), [neighbor_at(1), neighbor_at(2)].into_iter());
        stack.promote_staged();
        assert_eq!(stack.remaining.pop().unwrap().kind.position().0.x, 1);

        stack.stage(&(), [neighbor_at(10), neighbor_at(11)].into_iter());
        stack.promote_staged();

        assert_eq!(remaining_positions(&stack), vec![2, 11, 10]);
    }

    #[test]
    fn chained_updates_stop_at_the_budget() {
        let mut stack = UpdateStack::new(2);
        stack.stage(
            &(),
            [neighbor_at(1), neighbor_at(2), neighbor_at(3)].into_iter(),
        );
        stack.promote_staged();

        assert!(stack.gave_up);
        assert_eq!(remaining_positions(&stack), vec![2, 1]);
    }
}
