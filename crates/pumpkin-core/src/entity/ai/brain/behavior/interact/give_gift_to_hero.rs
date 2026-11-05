use std::sync::Arc;

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::villager::VillagerProfession;
use rand::RngExt;

use super::super::super::BrainTick;
use super::super::super::memory::{MemoryModuleId, MemoryStatus, types};
use super::super::timed::Behavior;
use super::super::utils;
use crate::entity::EntityBase;
use crate::entity::passive::villager::VillagerEntity;
use crate::entity::player::Player;
use crate::world::loot::{LootContextParameters, generate_loot_from_handle};

const THROW_GIFT_AT_DISTANCE: i32 = 5;
const MIN_TIME_BETWEEN_GIFTS: i32 = 600;
const MAX_TIME_BETWEEN_GIFTS: i32 = 6600;
const TIME_TO_DELAY_FOR_HEAD_TO_FINISH_TURNING: i64 = 20;
const SPEED_MODIFIER: f32 = 0.5;

const ENTRY_CONDITIONS: [(MemoryModuleId, MemoryStatus); 4] = [
    (types::WALK_TARGET.id(), MemoryStatus::Registered),
    (types::LOOK_TARGET.id(), MemoryStatus::Registered),
    (types::INTERACTION_TARGET.id(), MemoryStatus::Registered),
    (
        types::NEAREST_VISIBLE_PLAYER.id(),
        MemoryStatus::ValuePresent,
    ),
];

pub struct GiveGiftToHero {
    time_until_next_gift: i32,
    gift_given_during_this_run: bool,
    time_since_start: i64,
    timeout: i32,
}

impl GiveGiftToHero {
    #[must_use]
    pub const fn new(timeout: i32) -> Self {
        Self {
            time_until_next_gift: MIN_TIME_BETWEEN_GIFTS,
            gift_given_during_this_run: false,
            time_since_start: 0,
            timeout,
        }
    }
}

fn nearest_targetable_hero(tick: &BrainTick<'_>) -> Option<Arc<Player>> {
    tick.state
        .get(types::NEAREST_VISIBLE_PLAYER)
        .cloned()
        .filter(|player| {
            player
                .living_entity
                .has_effect(&StatusEffect::HERO_OF_THE_VILLAGE)
        })
}

fn is_hero_visible(tick: &BrainTick<'_>) -> bool {
    nearest_targetable_hero(tick).is_some()
}

fn is_within_throwing_distance(tick: &BrainTick<'_>, player: &Arc<Player>) -> bool {
    tick.mob
        .get_entity()
        .block_pos
        .load()
        .squared_distance(&player.get_entity().block_pos.load())
        < THROW_GIFT_AT_DISTANCE * THROW_GIFT_AT_DISTANCE
}

fn throw_gift(tick: &BrainTick<'_>, target: &Arc<dyn EntityBase>) {
    let is_baby = tick.mob.get_mob_entity().living_entity.is_baby();
    let profession = tick.mob.cast_any().downcast_ref::<VillagerEntity>().map_or(
        VillagerProfession::None,
        |villager| {
            villager
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .profession_enum()
        },
    );
    let Some(table) = tick
        .world
        .get_loot_table(loot_table_to_throw(is_baby, profession))
    else {
        return;
    };
    let position = tick.mob.get_entity().pos.load();
    let params = LootContextParameters {
        world: Some(tick.world),
        position: Some(position),
        this_entity: Some(tick.mob),
        ..Default::default()
    };
    for stack in generate_loot_from_handle(&table, tick.mob.get_random().random::<i64>(), &params) {
        utils::throw_item(tick.mob, stack, target.get_entity().pos.load());
    }
}

impl Behavior for GiveGiftToHero {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        &ENTRY_CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        self.timeout
    }

    fn max_duration(&self) -> i32 {
        self.timeout
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        if !is_hero_visible(tick) {
            return false;
        }
        if self.time_until_next_gift > 0 {
            self.time_until_next_gift -= 1;
            return false;
        }
        true
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        self.gift_given_during_this_run = false;
        self.time_since_start = tick.time;
        let Some(player) = nearest_targetable_hero(tick) else {
            return;
        };
        let target: Arc<dyn EntityBase> = player;
        tick.state.set(types::INTERACTION_TARGET, target.clone());
        utils::look_at_entity(tick.state, target);
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        is_hero_visible(tick) && !self.gift_given_during_this_run
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(player) = nearest_targetable_hero(tick) else {
            return;
        };
        let target: Arc<dyn EntityBase> = player.clone();
        utils::look_at_entity(tick.state, target.clone());
        if is_within_throwing_distance(tick, &player) {
            if tick.time - self.time_since_start > TIME_TO_DELAY_FOR_HEAD_TO_FINISH_TURNING {
                throw_gift(tick, &target);
                self.gift_given_during_this_run = true;
            }
        } else {
            utils::set_walk_and_look_target_memories_to_entity(
                tick.state,
                target,
                SPEED_MODIFIER,
                THROW_GIFT_AT_DISTANCE,
            );
        }
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>) {
        self.time_until_next_gift = calculate_time_until_next_gift(tick);
        tick.state.erase(types::INTERACTION_TARGET.id());
        tick.state.erase(types::WALK_TARGET.id());
        tick.state.erase(types::LOOK_TARGET.id());
    }

    fn debug_name(&self) -> &'static str {
        "GiveGiftToHero"
    }
}

fn calculate_time_until_next_gift(tick: &BrainTick<'_>) -> i32 {
    MIN_TIME_BETWEEN_GIFTS
        + tick
            .mob
            .get_random()
            .random_range(0..=MAX_TIME_BETWEEN_GIFTS - MIN_TIME_BETWEEN_GIFTS)
}

const fn loot_table_to_throw(is_baby: bool, profession: VillagerProfession) -> &'static str {
    if is_baby {
        return "minecraft:gameplay/hero_of_the_village/baby_gift";
    }
    match profession {
        VillagerProfession::Armorer => "minecraft:gameplay/hero_of_the_village/armorer_gift",
        VillagerProfession::Butcher => "minecraft:gameplay/hero_of_the_village/butcher_gift",
        VillagerProfession::Cartographer => {
            "minecraft:gameplay/hero_of_the_village/cartographer_gift"
        }
        VillagerProfession::Cleric => "minecraft:gameplay/hero_of_the_village/cleric_gift",
        VillagerProfession::Farmer => "minecraft:gameplay/hero_of_the_village/farmer_gift",
        VillagerProfession::Fisherman => "minecraft:gameplay/hero_of_the_village/fisherman_gift",
        VillagerProfession::Fletcher => "minecraft:gameplay/hero_of_the_village/fletcher_gift",
        VillagerProfession::Leatherworker => {
            "minecraft:gameplay/hero_of_the_village/leatherworker_gift"
        }
        VillagerProfession::Librarian => "minecraft:gameplay/hero_of_the_village/librarian_gift",
        VillagerProfession::Mason => "minecraft:gameplay/hero_of_the_village/mason_gift",
        VillagerProfession::Shepherd => "minecraft:gameplay/hero_of_the_village/shepherd_gift",
        VillagerProfession::Toolsmith => "minecraft:gameplay/hero_of_the_village/toolsmith_gift",
        VillagerProfession::Weaponsmith => {
            "minecraft:gameplay/hero_of_the_village/weaponsmith_gift"
        }
        VillagerProfession::None | VillagerProfession::Nitwit => {
            "minecraft:gameplay/hero_of_the_village/unemployed_gift"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gift_loot_table_follows_baby_profession_and_default_rules() {
        assert_eq!(
            loot_table_to_throw(true, VillagerProfession::Farmer),
            "minecraft:gameplay/hero_of_the_village/baby_gift"
        );
        assert_eq!(
            loot_table_to_throw(false, VillagerProfession::Farmer),
            "minecraft:gameplay/hero_of_the_village/farmer_gift"
        );
        assert_eq!(
            loot_table_to_throw(false, VillagerProfession::None),
            "minecraft:gameplay/hero_of_the_village/unemployed_gift"
        );
        assert_eq!(
            loot_table_to_throw(false, VillagerProfession::Nitwit),
            "minecraft:gameplay/hero_of_the_village/unemployed_gift"
        );
    }
}
