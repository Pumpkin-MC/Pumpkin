use std::sync::Arc;

use pumpkin_data::attributes::Attributes;

use super::super::BrainTick;
use super::super::memory::{MemoryModuleId, types};
use super::{Sensor, is_entity_attackable, is_entity_targetable};
use crate::entity::EntityBase;
use crate::entity::player::Player;

const REQUIRES: [MemoryModuleId; 4] = [
    types::NEAREST_PLAYERS.id(),
    types::NEAREST_VISIBLE_PLAYER.id(),
    types::NEAREST_VISIBLE_ATTACKABLE_PLAYER.id(),
    types::NEAREST_VISIBLE_ATTACKABLE_PLAYERS.id(),
];

pub struct PlayerSensor;

impl Sensor for PlayerSensor {
    fn requires(&self) -> &'static [MemoryModuleId] {
        &REQUIRES
    }

    fn do_tick(&mut self, tick: &mut BrainTick<'_>) {
        let living = &tick.mob.get_mob_entity().living_entity;
        let position = living.entity.pos.load();
        let follow_range = living.get_attribute_value(&Attributes::FOLLOW_RANGE);
        let distance = |player: &Arc<Player>| {
            position.squared_distance_to_vec(&player.get_entity().pos.load())
        };
        let mut players: Vec<Arc<Player>> = tick
            .world
            .players
            .load()
            .iter()
            .filter(|player| {
                !player.is_spectator() && distance(player) < follow_range * follow_range
            })
            .cloned()
            .collect();
        players.sort_by(|a, b| distance(a).total_cmp(&distance(b)));
        tick.state.set(types::NEAREST_PLAYERS, players.clone());

        let ctx = tick.visibility();
        let visible: Vec<Arc<Player>> = players
            .into_iter()
            .filter(|player| is_entity_targetable(&ctx, player.as_ref()))
            .collect();
        let attackable: Vec<Arc<Player>> = visible
            .iter()
            .filter(|player| is_entity_attackable(&ctx, player.as_ref()))
            .cloned()
            .collect();
        tick.state
            .set_optional(types::NEAREST_VISIBLE_PLAYER, visible.first().cloned());
        tick.state.set_optional(
            types::NEAREST_VISIBLE_ATTACKABLE_PLAYER,
            attackable.first().cloned(),
        );
        tick.state
            .set(types::NEAREST_VISIBLE_ATTACKABLE_PLAYERS, attackable);
    }
}
