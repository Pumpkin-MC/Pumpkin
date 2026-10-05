use std::sync::Arc;

use pumpkin_data::BlockState;
use pumpkin_data::enchantment::{LevelBasedValue, ReplaceDiskPredicate};
use pumpkin_data::game_event::GameEvent;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

use crate::world::World;

/// Enchantment entity effect that replaces a disk of blocks around an offset position.
pub struct ReplaceDisk {
    pub radius: LevelBasedValue,
    pub height: LevelBasedValue,
    pub offset: Vector3<i32>,
    pub predicate: Option<ReplaceDiskPredicate>,
    pub block_state: &'static BlockState,
    pub trigger_game_event: Option<GameEvent>,
}

pub trait ReplaceDiskPredicateExt {
    fn test_world(&self, world: &World, pos: &BlockPos) -> bool;
}

impl ReplaceDiskPredicateExt for ReplaceDiskPredicate {
    fn test_world(&self, world: &World, pos: &BlockPos) -> bool {
        match self {
            Self::MatchingBlockTag { offset, tag } => {
                let test_pos = pos.offset(*offset);
                let state_id = world.get_block_state_id(&test_pos);
                state_id.to_block_id().has_tag(**tag)
            }
            Self::MatchingBlocks { offset, blocks } => {
                let test_pos = pos.offset(*offset);
                let block = world.get_block(&test_pos);
                blocks.iter().any(|b| {
                    let stripped = b.strip_prefix("minecraft:").unwrap_or(b);
                    stripped == block.name
                })
            }
            Self::MatchingFluids { offset, fluids } => {
                let test_pos = pos.offset(*offset);
                let (_, fluid) = world.get_block_and_fluid(&test_pos);
                fluids.iter().any(|f| {
                    let stripped = f.strip_prefix("minecraft:").unwrap_or(f);
                    stripped == fluid.name
                })
            }
            Self::Unobstructed => true,
            Self::AllOf(predicates) => predicates.iter().all(|p| p.test_world(world, pos)),
        }
    }
}

impl ReplaceDisk {
    #[must_use]
    pub const fn new(
        radius: LevelBasedValue,
        height: LevelBasedValue,
        offset: Vector3<i32>,
        predicate: Option<ReplaceDiskPredicate>,
        block_state: &'static BlockState,
        trigger_game_event: Option<GameEvent>,
    ) -> Self {
        Self {
            radius,
            height,
            offset,
            predicate,
            block_state,
            trigger_game_event,
        }
    }

    #[must_use]
    pub const fn simple(
        radius: LevelBasedValue,
        height: LevelBasedValue,
        offset: Vector3<i32>,
        block_state: &'static BlockState,
        trigger_game_event: Option<GameEvent>,
    ) -> Self {
        Self {
            radius,
            height,
            offset,
            predicate: None,
            block_state,
            trigger_game_event,
        }
    }

    #[must_use]
    pub fn calculate_radius(&self, level: i32) -> f32 {
        self.radius.calculate(level)
    }

    #[must_use]
    pub fn calculate_height(&self, level: i32) -> f32 {
        self.height.calculate(level)
    }

    #[must_use]
    pub fn center_position(&self, origin: Vector3<f64>) -> BlockPos {
        BlockPos::containing_vec(origin).offset(self.offset)
    }

    #[must_use]
    pub fn is_in_radius(pos: &BlockPos, position: Vector3<f64>, dist: i32) -> bool {
        pos.dist_to_center_sqr(position.x, f64::from(pos.0.y) + 0.5, position.z)
            < f64::from(dist * dist)
    }

    /// Iterates through all candidate block positions within the disk region.
    pub fn iterate_blocks(
        &self,
        level: i32,
        position: Vector3<f64>,
    ) -> impl Iterator<Item = BlockPos> {
        let center_block = self.center_position(position);
        let dist = self.radius.calculate(level) as i32;
        let height = self.height.calculate(level) as i32;
        let start = center_block.offset(Vector3::new(-dist, 0, -dist));
        let end = center_block.offset(Vector3::new(dist, (height - 1).min(0), dist));
        BlockPos::between_closed(start, end)
            .filter(move |pos| Self::is_in_radius(pos, position, dist))
    }

    /// Applies the replace disk effect on the given world and position.
    pub fn apply(
        &self,
        world: &Arc<World>,
        enchantment_level: i32,
        _entity: Option<&crate::entity::Entity>,
        position: Vector3<f64>,
    ) {
        let mut changed = false;
        for pos in self.iterate_blocks(enchantment_level, position) {
            let passes_predicate = self
                .predicate
                .as_ref()
                .is_none_or(|p| p.test_world(world.as_ref(), &pos));
            if !passes_predicate {
                continue;
            }

            let new_state = self.block_state;
            let old_state_id = world.set_block_state(&pos, new_state.id, BlockFlags::NOTIFY_ALL);
            if old_state_id != new_state.id {
                changed = true;
                if let Some(event) = self.trigger_game_event {
                    world.emit_game_event(event.name(), pos.to_centered_f64());
                }
            }
        }
        if changed {
            world.flush_block_updates();
        }
    }
}

impl super::EnchantmentEntityEffectExt for ReplaceDisk {
    fn apply(
        &self,
        world: &Arc<World>,
        enchantment_level: i32,
        _owner: Option<&Arc<crate::entity::player::Player>>,
        entity: Option<&crate::entity::Entity>,
        position: Vector3<f64>,
    ) {
        self.apply(world, enchantment_level, entity, position);
    }
}
