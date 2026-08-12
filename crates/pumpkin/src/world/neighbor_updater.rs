use std::cell::RefCell;
use std::sync::Arc;

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

tokio::task_local! {
    static PENDING_UPDATES: RefCell<UpdateStack<Arc<World>>>;
}

pub(super) async fn update_neighbors(
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
                source_block,
                include_fluid: true,
            }),
    )
    .await;
}

pub(super) async fn update_neighbor(world: &Arc<World>, position: &BlockPos, source_block: &Block) {
    submit(
        world,
        std::iter::once(UpdateKind::Neighbor {
            position: *position,
            source_block: source_block.id,
            include_fluid: false,
        }),
    )
    .await;
}

pub(super) async fn update_shape(
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
    )
    .await;
}

fn cascade_in_progress() -> bool {
    PENDING_UPDATES.try_with(|_| ()).is_ok()
}

async fn submit(world: &Arc<World>, updates: impl Iterator<Item = UpdateKind>) {
    if cascade_in_progress() {
        PENDING_UPDATES.with(|pending| pending.borrow_mut().stage(world, updates));
        return;
    }

    let mut stack = UpdateStack::new(MAX_CHAINED_UPDATES);
    stack.stage(world, updates);
    stack.promote_staged();

    PENDING_UPDATES
        .scope(RefCell::new(stack), run_cascade())
        .await;
}

async fn run_cascade() {
    while let Some(update) = PENDING_UPDATES.with(|pending| pending.borrow_mut().remaining.pop()) {
        Box::pin(execute(update)).await;
        PENDING_UPDATES.with(|pending| pending.borrow_mut().promote_staged());
    }
}

async fn execute(update: QueuedUpdate<Arc<World>>) {
    let QueuedUpdate { world, kind } = update;
    match kind {
        UpdateKind::Neighbor {
            position,
            source_block,
            include_fluid,
        } => {
            world
                .execute_neighbor_update(&position, source_block.to_block(), include_fluid)
                .await;
        }
        UpdateKind::Shape {
            position,
            direction,
            flags,
        } => {
            world
                .execute_shape_update(&position, direction, flags)
                .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_CHAINED_UPDATES, UpdateKind, UpdateStack};
    use pumpkin_data::{BlockDirection, BlockId};
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_world::world::BlockFlags;

    const OVERWORLD: &str = "overworld";
    const NETHER: &str = "nether";

    fn neighbor_at(x: i32) -> UpdateKind {
        UpdateKind::Neighbor {
            position: BlockPos(Vector3::new(x, 0, 0)),
            source_block: BlockId::AIR,
            include_fluid: true,
        }
    }

    fn remaining_positions(stack: &UpdateStack<&'static str>) -> Vec<i32> {
        stack
            .remaining
            .iter()
            .map(|update| update.kind.position().0.x)
            .collect()
    }

    fn remaining_worlds(stack: &UpdateStack<&'static str>) -> Vec<&'static str> {
        stack.remaining.iter().map(|update| update.world).collect()
    }

    #[test]
    fn staged_updates_are_popped_in_submission_order() {
        let mut stack = UpdateStack::new(MAX_CHAINED_UPDATES);
        stack.stage(
            &OVERWORLD,
            [neighbor_at(1), neighbor_at(2), neighbor_at(3)].into_iter(),
        );
        stack.promote_staged();

        assert_eq!(remaining_positions(&stack), vec![3, 2, 1]);
        assert_eq!(
            stack
                .remaining
                .pop()
                .map(|update| update.kind.position().0.x),
            Some(1)
        );
    }

    #[test]
    fn updates_staged_while_running_are_handled_before_the_rest() {
        let mut stack = UpdateStack::new(MAX_CHAINED_UPDATES);
        stack.stage(&OVERWORLD, [neighbor_at(1), neighbor_at(2)].into_iter());
        stack.promote_staged();
        stack.remaining.pop();

        stack.stage(&OVERWORLD, [neighbor_at(10), neighbor_at(11)].into_iter());
        stack.promote_staged();

        assert_eq!(remaining_positions(&stack), vec![2, 11, 10]);
    }

    #[test]
    fn updates_stay_bound_to_the_world_that_staged_them() {
        let mut stack = UpdateStack::new(MAX_CHAINED_UPDATES);
        stack.stage(&OVERWORLD, [neighbor_at(1), neighbor_at(2)].into_iter());
        stack.promote_staged();

        let running = stack.remaining.pop().expect("staged update");
        assert_eq!((running.world, running.kind.position().0.x), (OVERWORLD, 1));

        stack.stage(&NETHER, std::iter::once(neighbor_at(10)));
        stack.promote_staged();

        assert_eq!(remaining_positions(&stack), vec![2, 10]);
        assert_eq!(remaining_worlds(&stack), vec![OVERWORLD, NETHER]);
    }

    #[test]
    fn updates_chained_past_the_budget_are_skipped() {
        let mut stack = UpdateStack::new(2);
        stack.stage(
            &OVERWORLD,
            [neighbor_at(1), neighbor_at(2), neighbor_at(3)].into_iter(),
        );
        stack.stage(
            &OVERWORLD,
            std::iter::once(UpdateKind::Shape {
                position: BlockPos(Vector3::new(4, 0, 0)),
                direction: BlockDirection::Up,
                flags: BlockFlags::NOTIFY_ALL,
            }),
        );
        stack.promote_staged();

        assert!(stack.gave_up);
        assert_eq!(remaining_positions(&stack), vec![2, 1]);
    }
}
