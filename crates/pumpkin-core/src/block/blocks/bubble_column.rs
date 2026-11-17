use std::sync::Arc;

use pumpkin_data::block_properties::BubbleColumnLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockId, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, BlockMetadata, OnEntityCollisionArgs, OnNeighborUpdateArgs,
    OnScheduledTickArgs, PlacedArgs,
};
use crate::entity::{Entity, EntityBase};
use crate::world::World;
use pumpkin_data::particle::Particle;

const CREATE_DELAY_TICKS: u8 = 20;
const REMOVE_DELAY_TICKS: u8 = 5;

const UPWARD_ACCELERATION: f64 = 0.06;
const UPWARD_MAX_SPEED: f64 = 0.7;
pub(crate) const SURFACE_UPWARD_ACCELERATION: f64 = 0.1;
const SURFACE_UPWARD_MAX_SPEED: f64 = 1.8;
const DOWNWARD_ACCELERATION: f64 = -0.03;
const DOWNWARD_MIN_SPEED: f64 = -0.3;

pub struct BubbleColumnBlock;

impl BlockMetadata for BubbleColumnBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::BUBBLE_COLUMN, BlockId::WATER].into()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BubbleColumnKind {
    Upward,
    Downward,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReconcileAction {
    SetBubble(BubbleColumnKind),
    RestoreWater,
    Stop,
}

fn source_water_state() -> BlockStateId {
    Fluid::WATER
        .states
        .iter()
        .find(|state| state.is_still && state.is_source)
        .map_or(BlockStateId::new_or_air(0), |state| state.block_state_id)
}

fn bubble_column_state(kind: BubbleColumnKind) -> BlockStateId {
    BubbleColumnLikeProperties {
        r#drag: matches!(kind, BubbleColumnKind::Downward),
    }
    .to_state_id(&Block::BUBBLE_COLUMN)
}

fn kind_from_support(block: &Block) -> Option<BubbleColumnKind> {
    if block.has_tag(&tag::Block::MINECRAFT_ENABLES_BUBBLE_COLUMN_PUSH_UP) {
        Some(BubbleColumnKind::Upward)
    } else if block.has_tag(&tag::Block::MINECRAFT_ENABLES_BUBBLE_COLUMN_DRAG_DOWN) {
        Some(BubbleColumnKind::Downward)
    } else {
        None
    }
}

fn kind_from_state(state: BlockStateId) -> BubbleColumnKind {
    let props = BubbleColumnLikeProperties::from_state_id(state);
    if props.r#drag {
        BubbleColumnKind::Downward
    } else {
        BubbleColumnKind::Upward
    }
}

fn kind_from_below(block: &Block, state: BlockStateId) -> Option<BubbleColumnKind> {
    if block == &Block::BUBBLE_COLUMN {
        Some(kind_from_state(state))
    } else {
        kind_from_support(block)
    }
}

fn is_source_water_state(state: BlockStateId) -> bool {
    let Some(fluid) = Fluid::from_state_id(state) else {
        return false;
    };

    fluid.has_tag(&tag::Fluid::MINECRAFT_BUBBLE_COLUMN_CAN_OCCUPY)
        && fluid.is_source(state)
        && fluid.states.iter().any(|fluid_state| {
            fluid_state.block_state_id == state && fluid_state.is_still && fluid_state.is_source
        })
}

fn is_source_water(world: &World, position: BlockPos) -> bool {
    is_source_water_state(world.get_block_state_id(&position))
}

fn reconcile_action(
    current_block: &Block,
    current_state: BlockStateId,
    below_block: &Block,
    below_state: BlockStateId,
) -> ReconcileAction {
    let current_is_bubble = current_block == &Block::BUBBLE_COLUMN;
    let current_is_source_water = is_source_water_state(current_state);

    if let Some(kind) = kind_from_below(below_block, below_state)
        && (current_is_bubble || current_is_source_water)
    {
        return ReconcileAction::SetBubble(kind);
    }

    if current_is_bubble {
        ReconcileAction::RestoreWater
    } else {
        ReconcileAction::Stop
    }
}

fn schedule_reconcile(world: &Arc<World>, position: BlockPos, delay: u8) {
    world.schedule_block_tick(&Block::BUBBLE_COLUMN, position, delay, TickPriority::Normal);
}

fn bubble_column_vertical_velocity(
    current_y: f64,
    kind: BubbleColumnKind,
    at_surface: bool,
) -> f64 {
    match (kind, at_surface) {
        (BubbleColumnKind::Upward, true) => {
            (current_y + SURFACE_UPWARD_ACCELERATION).min(SURFACE_UPWARD_MAX_SPEED)
        }
        (BubbleColumnKind::Upward, false) => {
            (current_y + UPWARD_ACCELERATION).min(UPWARD_MAX_SPEED)
        }
        (BubbleColumnKind::Downward, _) => {
            (current_y + DOWNWARD_ACCELERATION).max(DOWNWARD_MIN_SPEED)
        }
    }
}

fn bubble_column_velocity(
    current: Vector3<f64>,
    kind: BubbleColumnKind,
    at_surface: bool,
) -> Vector3<f64> {
    Vector3::new(
        current.x,
        bubble_column_vertical_velocity(current.y, kind, at_surface),
        current.z,
    )
}

/// Pushes an entity in a bubble column. One at the top of the column gets the stronger push.
pub fn entity_inside(world: &World, pos: &BlockPos, state: BlockStateId, entity: &dyn EntityBase) {
    let drag_down = kind_from_state(state) == BubbleColumnKind::Downward;
    if world.get_block(&pos.up()) == &Block::AIR {
        entity.on_above_bubble_column(drag_down, pos);
    } else {
        entity.on_inside_bubble_column(drag_down);
    }
}

/// The regular entity push, capped at a maximum speed.
pub fn push_entity(entity: &Entity, drag_down: bool, at_surface: bool) {
    let kind = if drag_down {
        BubbleColumnKind::Downward
    } else {
        BubbleColumnKind::Upward
    };
    entity.velocity.store(bubble_column_velocity(
        entity.velocity.load(),
        kind,
        at_surface,
    ));
}

/// The push projectiles get: the same acceleration without the speed caps.
pub fn push_projectile(entity: &Entity, drag_down: bool, at_surface: bool) {
    let push = if drag_down {
        DOWNWARD_ACCELERATION
    } else if at_surface {
        SURFACE_UPWARD_ACCELERATION
    } else {
        UPWARD_ACCELERATION
    };
    let mut velocity = entity.velocity.load();
    velocity.y += push;
    entity.velocity.store(velocity);
}

/// Splash and bubble particles at the top of a bubble column.
pub fn send_bubble_column_particles(world: &World, pos: &BlockPos) {
    for _ in 0..2 {
        for (particle, offset_y, speed) in
            [(Particle::Splash, 0.0, 1.0), (Particle::Bubble, 0.01, 0.2)]
        {
            let at = Vector3::new(
                f64::from(pos.0.x) + rand::random::<f64>(),
                f64::from(pos.0.y + 1),
                f64::from(pos.0.z) + rand::random::<f64>(),
            );
            world.spawn_particle(at, Vector3::new(0.0, offset_y, 0.0), speed, 1, particle);
        }
    }
}

impl BlockBehaviour for BubbleColumnBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        {
            if args.block != &Block::BUBBLE_COLUMN {
                return;
            }

            entity_inside(args.world, args.position, args.state.id, args.entity);

            if let Some(player) = args.entity.get_player() {
                player.breath_manager.reset(player);
            }
        }
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        {
            if args.block == &Block::WATER && is_source_water(args.world, *args.position) {
                schedule_reconcile(args.world, *args.position, CREATE_DELAY_TICKS);
            }
        }
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        {
            let state = args.world.get_block_state_id(args.position);
            if args.block == &Block::BUBBLE_COLUMN {
                schedule_reconcile(args.world, *args.position, REMOVE_DELAY_TICKS);
            } else if args.block == &Block::WATER
                && is_source_water_state(state)
                && (args.source_block == &Block::BUBBLE_COLUMN
                    || kind_from_support(args.source_block).is_some())
            {
                schedule_reconcile(args.world, *args.position, CREATE_DELAY_TICKS);
            }
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let state = args.world.get_block_state_id(args.position);
        let block = Block::from_state_id(state);
        if block != &Block::BUBBLE_COLUMN && block != &Block::WATER {
            return;
        }

        let below_pos = args.position.down();
        let below_state = args.world.get_block_state_id(&below_pos);
        let below_block = Block::from_state_id(below_state);

        match reconcile_action(block, state, below_block, below_state) {
            ReconcileAction::SetBubble(kind) => {
                let new_state = bubble_column_state(kind);
                args.world
                    .set_block_state(args.position, new_state, BlockFlags::NOTIFY_ALL);
                schedule_reconcile(args.world, args.position.up(), CREATE_DELAY_TICKS);
            }
            ReconcileAction::RestoreWater => {
                args.world.set_block_state(
                    args.position,
                    source_water_state(),
                    BlockFlags::NOTIFY_ALL,
                );
                schedule_reconcile(args.world, args.position.up(), REMOVE_DELAY_TICKS);
            }
            ReconcileAction::Stop => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::BlockStateId;
    use pumpkin_data::fluid::{Falling, FlowingWaterLikeFluidProperties, FluidProperties, Level};

    use super::*;

    fn flowing_water_state() -> BlockStateId {
        FlowingWaterLikeFluidProperties {
            r#falling: Falling::False,
            r#level: Level::L1,
        }
        .to_state_id(&Fluid::FLOWING_WATER)
    }

    #[test]
    fn support_tags_map_to_expected_kinds() {
        assert_eq!(
            kind_from_support(&Block::SOUL_SAND),
            Some(BubbleColumnKind::Upward)
        );
        assert_eq!(
            kind_from_support(&Block::MAGMA_BLOCK),
            Some(BubbleColumnKind::Downward)
        );
        assert_eq!(kind_from_support(&Block::STONE), None);
    }

    #[test]
    fn reconciliation_creates_from_upward_support() {
        assert_eq!(
            reconcile_action(
                &Block::WATER,
                source_water_state(),
                &Block::SOUL_SAND,
                Block::SOUL_SAND.default_state.id,
            ),
            ReconcileAction::SetBubble(BubbleColumnKind::Upward)
        );
    }

    #[test]
    fn reconciliation_creates_from_downward_support() {
        assert_eq!(
            reconcile_action(
                &Block::WATER,
                source_water_state(),
                &Block::MAGMA_BLOCK,
                Block::MAGMA_BLOCK.default_state.id,
            ),
            ReconcileAction::SetBubble(BubbleColumnKind::Downward)
        );
    }

    #[test]
    fn reconciliation_inherits_direction_from_lower_column() {
        let upward_state = bubble_column_state(BubbleColumnKind::Upward);
        let downward_state = bubble_column_state(BubbleColumnKind::Downward);

        assert_eq!(
            reconcile_action(
                &Block::WATER,
                source_water_state(),
                &Block::BUBBLE_COLUMN,
                upward_state
            ),
            ReconcileAction::SetBubble(BubbleColumnKind::Upward)
        );
        assert_eq!(
            reconcile_action(
                &Block::WATER,
                source_water_state(),
                &Block::BUBBLE_COLUMN,
                downward_state,
            ),
            ReconcileAction::SetBubble(BubbleColumnKind::Downward)
        );
    }

    #[test]
    fn reconciliation_rejects_flowing_water_and_air() {
        assert_eq!(
            reconcile_action(
                &Block::WATER,
                flowing_water_state(),
                &Block::SOUL_SAND,
                Block::SOUL_SAND.default_state.id,
            ),
            ReconcileAction::Stop
        );
        assert_eq!(
            reconcile_action(
                &Block::AIR,
                BlockStateId::AIR,
                &Block::SOUL_SAND,
                Block::SOUL_SAND.default_state.id,
            ),
            ReconcileAction::Stop
        );
    }

    #[test]
    fn reconciliation_restores_invalidated_column() {
        assert_eq!(
            reconcile_action(
                &Block::BUBBLE_COLUMN,
                bubble_column_state(BubbleColumnKind::Upward),
                &Block::STONE,
                Block::STONE.default_state.id,
            ),
            ReconcileAction::RestoreWater
        );
    }
}
