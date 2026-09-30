use std::sync::Arc;

use pumpkin_data::block_properties::CampfireLikeProperties;
use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::position::BlockPos;

use crate::entity::EntityBase;
use crate::entity::mob::piglin_ai;
use crate::world::World;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, NearestVisibleLivingEntities, types};
use super::Sensor;

const REPELLENT_DETECTION_RANGE_HORIZONTAL: i32 = 8;
const REPELLENT_DETECTION_RANGE_VERTICAL: i32 = 4;

const REQUIRES: &[MemoryModuleId] = &[
    types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
    types::NEAREST_LIVING_ENTITIES.id(),
    types::NEAREST_VISIBLE_NEMESIS.id(),
    types::NEAREST_TARGETABLE_PLAYER_NOT_WEARING_GOLD.id(),
    types::NEAREST_PLAYER_HOLDING_WANTED_ITEM.id(),
    types::NEAREST_VISIBLE_HUNTABLE_HOGLIN.id(),
    types::NEAREST_VISIBLE_BABY_HOGLIN.id(),
    types::NEAREST_VISIBLE_ADULT_PIGLINS.id(),
    types::NEARBY_ADULT_PIGLINS.id(),
    types::VISIBLE_ADULT_PIGLIN_COUNT.id(),
    types::VISIBLE_ADULT_HOGLIN_COUNT.id(),
    types::NEAREST_REPELLENT.id(),
];

pub struct PiglinSpecificSensor;

struct Scan {
    nemesis: Option<Arc<dyn EntityBase>>,
    huntable_hoglin: Option<Arc<dyn EntityBase>>,
    baby_hoglin: Option<Arc<dyn EntityBase>>,
    zombified: Option<Arc<dyn EntityBase>>,
    player_not_wearing_gold: Option<Arc<crate::entity::player::Player>>,
    player_holding_wanted_item: Option<Arc<crate::entity::player::Player>>,
    visible_adult_piglins: Vec<Arc<dyn EntityBase>>,
    visible_adult_hoglin_count: i32,
}

impl Sensor for PiglinSpecificSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let repellent = find_nearest_repellent(tick.world, tick.mob.get_entity().block_pos.load());
        let scan = scan_visible(tick);
        let nearby_adults = piglin_ai::find_nearby_adult_piglins(tick.brain);

        tick.brain.set_optional(types::NEAREST_REPELLENT, repellent);
        tick.brain
            .set_optional(types::NEAREST_VISIBLE_NEMESIS, scan.nemesis);
        tick.brain
            .set_optional(types::NEAREST_VISIBLE_HUNTABLE_HOGLIN, scan.huntable_hoglin);
        tick.brain
            .set_optional(types::NEAREST_VISIBLE_BABY_HOGLIN, scan.baby_hoglin);
        tick.brain
            .set_optional(types::NEAREST_VISIBLE_ZOMBIFIED, scan.zombified);
        tick.brain.set_optional(
            types::NEAREST_TARGETABLE_PLAYER_NOT_WEARING_GOLD,
            scan.player_not_wearing_gold,
        );
        tick.brain.set_optional(
            types::NEAREST_PLAYER_HOLDING_WANTED_ITEM,
            scan.player_holding_wanted_item,
        );
        let visible_count = i32::try_from(scan.visible_adult_piglins.len()).unwrap_or(i32::MAX);
        tick.brain.set(types::NEARBY_ADULT_PIGLINS, nearby_adults);
        tick.brain.set(
            types::NEAREST_VISIBLE_ADULT_PIGLINS,
            scan.visible_adult_piglins,
        );
        tick.brain
            .set(types::VISIBLE_ADULT_PIGLIN_COUNT, visible_count);
        tick.brain.set(
            types::VISIBLE_ADULT_HOGLIN_COUNT,
            scan.visible_adult_hoglin_count,
        );
    }
}

/// The visible list holds erased entities while the player memories want a concrete player.
fn player_arc(
    tick: &BrainTick<'_>,
    entity: &dyn EntityBase,
) -> Option<Arc<crate::entity::player::Player>> {
    let id = entity.get_entity().entity_id;
    tick.world
        .players
        .load()
        .iter()
        .find(|player| player.get_entity().entity_id == id)
        .cloned()
}

fn scan_visible(tick: &BrainTick<'_>) -> Scan {
    let mut scan = Scan {
        nemesis: None,
        huntable_hoglin: None,
        baby_hoglin: None,
        zombified: None,
        player_not_wearing_gold: None,
        player_holding_wanted_item: None,
        visible_adult_piglins: Vec::new(),
        visible_adult_hoglin_count: 0,
    };

    let ctx = tick.visibility();
    let empty = NearestVisibleLivingEntities::empty();
    let visible = ctx
        .brain
        .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
        .unwrap_or(&empty);

    for entity in visible.find_all(&ctx, |_| true) {
        let entity_type = entity.get_entity().entity_type;
        if entity_type == &EntityType::HOGLIN {
            let is_baby = piglin_ai::is_baby_hoglin(entity.as_ref());
            if is_baby {
                if scan.baby_hoglin.is_none() {
                    scan.baby_hoglin = Some(Arc::clone(entity));
                }
            } else {
                scan.visible_adult_hoglin_count += 1;
                if scan.huntable_hoglin.is_none()
                    && piglin_ai::hoglin_can_be_hunted(entity.as_ref())
                {
                    scan.huntable_hoglin = Some(Arc::clone(entity));
                }
            }
        } else if entity_type == &EntityType::PIGLIN_BRUTE {
            scan.visible_adult_piglins.push(Arc::clone(entity));
        } else if entity_type == &EntityType::PIGLIN {
            if piglin_ai::is_adult_piglin(entity.as_ref()) {
                scan.visible_adult_piglins.push(Arc::clone(entity));
            }
        } else if entity_type == &EntityType::PLAYER {
            let Some(player) = player_arc(tick, entity.as_ref()) else {
                continue;
            };
            if scan.player_not_wearing_gold.is_none()
                && !piglin_ai::is_wearing_safe_armor(entity.as_ref())
                && entity
                    .get_living_entity()
                    .is_some_and(|living| ctx.mob.can_attack(living))
            {
                scan.player_not_wearing_gold = Some(Arc::clone(&player));
            }
            if scan.player_holding_wanted_item.is_none()
                && !entity.get_entity().is_spectator()
                && piglin_ai::is_player_holding_loved_item(entity.as_ref())
            {
                scan.player_holding_wanted_item = Some(player);
            }
        } else if scan.nemesis.is_none()
            && (entity_type == &EntityType::WITHER_SKELETON || entity_type == &EntityType::WITHER)
        {
            scan.nemesis = Some(Arc::clone(entity));
        } else if scan.zombified.is_none() && piglin_ai::is_zombified(entity.as_ref()) {
            scan.zombified = Some(Arc::clone(entity));
        }
    }

    scan
}

/// Vanilla walks `BlockPos.withinManhattan`, so equidistant repellents may tie differently.
fn find_nearest_repellent(world: &World, center: BlockPos) -> Option<BlockPos> {
    let max_distance =
        REPELLENT_DETECTION_RANGE_HORIZONTAL * 2 + REPELLENT_DETECTION_RANGE_VERTICAL;
    for distance in 0..=max_distance {
        for y in -REPELLENT_DETECTION_RANGE_VERTICAL..=REPELLENT_DETECTION_RANGE_VERTICAL {
            let remaining = distance - y.abs();
            if remaining < 0 {
                continue;
            }
            for x in -REPELLENT_DETECTION_RANGE_HORIZONTAL..=REPELLENT_DETECTION_RANGE_HORIZONTAL {
                let z_abs = remaining - x.abs();
                if !(0..=REPELLENT_DETECTION_RANGE_HORIZONTAL).contains(&z_abs) {
                    continue;
                }
                for z in [z_abs, -z_abs] {
                    let pos = BlockPos::new(center.0.x + x, center.0.y + y, center.0.z + z);
                    if is_valid_repellent(world, &pos) {
                        return Some(pos);
                    }
                    if z_abs == 0 {
                        break;
                    }
                }
            }
        }
    }
    None
}

fn is_valid_repellent(world: &World, pos: &BlockPos) -> bool {
    let (block, state_id) = world.get_block_and_state_id(pos);
    if !block.has_tag(&tag::Block::MINECRAFT_PIGLIN_REPELLENTS) {
        return false;
    }
    if block == &Block::SOUL_CAMPFIRE {
        return CampfireLikeProperties::from_state_id(state_id).lit;
    }
    true
}

const _: Option<BlockDirection> = None;
