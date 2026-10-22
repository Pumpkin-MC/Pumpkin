use std::sync::{Arc, Weak};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::Difficulty;

use crate::entity::{
    Entity,
    ai::behavior::fleeing::{self, Flee},
    ai::behavior::hunting::{self, Hunt},
    ai::goal::{
        bow_attack::BowAttackGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    ai::target_match::TargetMatch,
    mob::{Mob, MobEntity, equipment::RegionalDifficulty},
};
use crate::world::World;

pub mod bogged;
pub mod parched;
#[allow(clippy::module_inception)]
pub mod skeleton;
pub mod stray;
pub mod wither;

const SKELETON_FLEES: &[Flee] = &[Flee::new(
    3,
    TargetMatch::Type(&EntityType::WOLF),
    6.0,
    1.0,
    1.2,
)];

const SKELETON_HUNTS: &[Hunt] = &[
    Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true),
    Hunt::new(3, TargetMatch::Type(&EntityType::IRON_GOLEM), true),
    hunting::baby_turtles(3, true),
];

pub struct SkeletonEntityBase {
    pub mob_entity: MobEntity,
}

impl SkeletonEntityBase {
    pub fn new(entity: Entity) -> Arc<Self> {
        Self::with_hunts(entity, &[])
    }

    /// `extra` goes first, like vanilla subclasses adding targets before `super`.
    pub fn with_hunts(entity: Entity, extra: &[Hunt]) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mob = Self { mob_entity };
        let mob_arc = Arc::new(mob);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let target_selector = &mob_arc.mob_entity.target_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // TODO: RestrictSunGoal(2) and FleeSunGoal(3), once both match vanilla.
            fleeing::apply(goal_selector, SKELETON_FLEES);
            // Vanilla `reassessWeaponGoal`: the bow or the melee goal at 4, by the held weapon.
            // TODO: attack interval per type and difficulty (vanilla 40, 20 on hard; bogged and parched 70, 50 on hard).
            goal_selector.add_goal(4, Box::new(BowAttackGoal::new(1.0, 20, 15.0)));
            goal_selector.add_goal(
                4,
                Box::new(
                    MeleeAttackGoal::new(1.2, false)
                        .gated_by(|mob| !BowAttackGoal::is_holding_bow(mob)),
                ),
            );
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::water_avoiding(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            hunting::apply(target_selector, &mob_arc.mob_entity, extra);
            hunting::apply(target_selector, &mob_arc.mob_entity, SKELETON_HUNTS);
        };

        mob_arc
    }
}

impl Mob for SkeletonEntityBase {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn populate_default_equipment_slots(
        &self,
        _world: &Arc<World>,
        difficulty: &RegionalDifficulty,
    ) {
        // Default armor slots (super.populateDefaultEquipmentSlots)
        if rand::random::<f32>()
            < MobEntity::MAX_WEARING_ARMOR_CHANCE * difficulty.special_multiplier
        {
            let mut armor_type = rand::random_range(0..3);
            for _ in 1..=3 {
                if rand::random::<f32>() < MobEntity::WEARING_ARMOR_UPGRADE_MATERIAL_CHANCE {
                    armor_type += 1;
                }
            }

            let partial_chance = if difficulty.base_difficulty == Difficulty::Hard {
                0.1f32
            } else {
                0.25f32
            };

            let living = &self.mob_entity.living_entity;
            let mut equipment = living
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut first = true;

            for slot in &MobEntity::EQUIPMENT_POPULATION_ORDER {
                let current = equipment.get(slot);
                if !first && rand::random::<f32>() < partial_chance {
                    break;
                }
                first = false;
                if current.is_empty()
                    && let Some(item) = MobEntity::get_equipment_for_slot(slot, armor_type)
                {
                    equipment.put(slot, ItemStack::new(1, item));
                }
            }
        }

        // AbstractSkeleton sets BOW on MAIN_HAND
        let living = &self.mob_entity.living_entity;
        let mut equipment = living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        equipment.put(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::BOW));
    }
}
