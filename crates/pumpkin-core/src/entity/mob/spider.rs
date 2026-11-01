use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::BlockState;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_util::difficulty::Difficulty;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::ai::behavior::fleeing::{self, Flee};
use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        active_target::TargetCondition, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, revenge::RevengeGoal, spider_attack::SpiderAttackGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity,
        equipment::RegionalDifficulty,
        spawn::{SpawnGroupData, finalize_spawn},
    },
    passive::armadillo::ArmadilloEntity,
    r#type::from_type,
};
use crate::world::World;

pub const SPIDER_FLEES: &[Flee] =
    &[
        Flee::new(2, TargetMatch::Type(&EntityType::ARMADILLO), 6.0, 1.0, 1.2)
            .matching(is_calm_armadillo),
    ];

/// An armadillo out of its shell.
fn is_calm_armadillo(_mob: &dyn Mob, candidate: &dyn EntityBase) -> bool {
    candidate
        .cast_any()
        .downcast_ref::<ArmadilloEntity>()
        .is_some_and(|armadillo| !armadillo.is_scared())
}

pub const SPIDER_HUNTS: &[Hunt] = &[
    Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true).when(TargetCondition::NoDaylight),
    Hunt::new(3, TargetMatch::Type(&EntityType::IRON_GOLEM), true)
        .when(TargetCondition::NoDaylight),
];

pub struct SpiderEntity {
    pub mob_entity: MobEntity,
    pub is_climbing: AtomicBool,
}

impl SpiderEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let spider = Self {
            mob_entity,
            is_climbing: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(spider);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;
            let target_selector = &mob_arc.mob_entity.target_selector;

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            fleeing::apply(goal_selector, SPIDER_FLEES);
            goal_selector.add_goal(3, SpiderAttackGoal::new(1.0, false));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.8)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            hunting::apply(target_selector, &mob_arc.mob_entity, SPIDER_HUNTS);
        };

        mob_arc
    }

    pub fn is_climbing(&self) -> bool {
        self.is_climbing.load(Ordering::Relaxed)
    }

    pub fn set_climbing(&self, climbing: bool) {
        if self.is_climbing.swap(climbing, Ordering::Relaxed) != climbing {
            let flags = i8::from(climbing);
            let entity = &self.mob_entity.living_entity.entity;
            entity.set_synced_data(pumpkin_data::tracked_data::spider::DATA_FLAGS_ID, flags);
            if let Some(meta) = self.mob_metadata() {
                entity.send_bedrock_actor_data(&meta.to_bedrock_actor_data());
            }
        }
    }
}

/// Vanilla `Spider.finalizeSpawn` (also used by the cave spider): a skeleton jockey,
/// and on hard difficulty a random effect shared by the whole spawn group.
pub fn finalize_spider_spawn(
    mob: &MobEntity,
    world: &Arc<World>,
    group_data: Option<SpawnGroupData>,
) -> Option<SpawnGroupData> {
    mob.finalize_spawn_base();
    let mut rng = rand::rng();
    let entity = &mob.living_entity.entity;
    let pos = entity.pos.load();

    if rng.random_range(0..100) == 0 {
        let skeleton = from_type(&EntityType::SKELETON, pos, world, Uuid::new_v4());
        skeleton.get_entity().set_rotation(entity.yaw.load(), 0.0);
        finalize_spawn(&skeleton, world, None);
        mob.add_pending_rider(skeleton);
    }

    let group_data = group_data.unwrap_or_else(|| {
        let difficulty = RegionalDifficulty::at(world, pos);
        let effect = (difficulty.base_difficulty == Difficulty::Hard
            && rng.random::<f32>() < 0.1 * difficulty.special_multiplier)
            .then(|| match rng.random_range(0..5) {
                0 | 1 => &StatusEffect::SPEED,
                2 => &StatusEffect::STRENGTH,
                3 => &StatusEffect::REGENERATION,
                _ => &StatusEffect::INVISIBILITY,
            });
        SpawnGroupData::SpiderEffects(effect)
    });

    if let SpawnGroupData::SpiderEffects(Some(effect_type)) = &group_data {
        mob.living_entity.add_effect(Effect {
            effect_type,
            duration: -1,
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        });
    }
    Some(group_data)
}

impl Mob for SpiderEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntitySpiderStep, 0.15, 1.0);
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        finalize_spider_spawn(&self.mob_entity, world, group_data)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        let mut meta = crate::entity::mob::MobMetadata::new();
        if self.is_climbing() {
            meta = meta.spider_climbing(true);
        }
        Some(meta)
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let vel = entity.velocity.load();
        let is_colliding_horizontally = vel.x.abs() < 1e-4 && vel.z.abs() < 1e-4;
        self.set_climbing(is_colliding_horizontally);
    }
}
