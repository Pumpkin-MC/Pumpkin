use std::sync::Arc;

use pumpkin_data::attributes::Attributes;
use pumpkin_util::GameMode;

use crate::entity::EntityBase;
use crate::entity::player::Player;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::{Sensor, is_entity_attackable, is_entity_targetable};

const REQUIRES: &[MemoryModuleId] = &[
    types::NEAREST_PLAYERS.id(),
    types::NEAREST_VISIBLE_PLAYER.id(),
    types::NEAREST_VISIBLE_ATTACKABLE_PLAYER.id(),
    types::NEAREST_VISIBLE_ATTACKABLE_PLAYERS.id(),
];

pub struct PlayerSensor;

impl Sensor for PlayerSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let follow_range = tick
            .mob
            .get_mob_entity()
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        let body_pos = tick.mob.get_entity().pos.load();
        let range_squared = follow_range * follow_range;

        let mut nearby: Vec<(f64, Arc<Player>)> = tick
            .world
            .players
            .load()
            .iter()
            .filter(|player| player.gamemode.load() != GameMode::Spectator)
            .filter_map(|player| {
                let distance = player
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&body_pos);
                (distance < range_squared).then(|| (distance, player.clone()))
            })
            .collect();
        nearby.sort_by(|a, b| a.0.total_cmp(&b.0));
        let players: Vec<Arc<Player>> = nearby.into_iter().map(|(_, player)| player).collect();

        let (visible, attackable) = {
            let ctx = tick.visibility();
            let visible: Vec<Arc<Player>> = players
                .iter()
                .filter(|player| is_entity_targetable(&ctx, player.as_ref()))
                .cloned()
                .collect();
            let attackable: Vec<Arc<Player>> = visible
                .iter()
                .filter(|player| is_entity_attackable(&ctx, player.as_ref()))
                .cloned()
                .collect();
            (visible, attackable)
        };

        tick.brain.set(types::NEAREST_PLAYERS, players);
        tick.brain
            .set_optional(types::NEAREST_VISIBLE_PLAYER, visible.into_iter().next());
        tick.brain.set_optional(
            types::NEAREST_VISIBLE_ATTACKABLE_PLAYER,
            attackable.first().cloned(),
        );
        tick.brain
            .set(types::NEAREST_VISIBLE_ATTACKABLE_PLAYERS, attackable);
    }
}
