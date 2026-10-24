use std::sync::atomic::Ordering;
use std::sync::{Arc, PoisonError, Weak};

use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::potion::Effect;
use pumpkin_data::tracked_data::spider::DATA_FLAGS_ID;
use pumpkin_util::difficulty::Difficulty;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::{
    Entity,
    ai::goal::{
        active_target::{ActiveTargetGoal, TargetCondition},
        leap_at_target::LeapAtTargetGoal,
        look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal,
        revenge::RevengeGoal,
        spider_attack::SpiderAttackGoal,
        swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    ai::pathfinder::Navigator,
    mob::{
        Mob, MobEntity, MobMetadata,
        equipment::RegionalDifficulty,
        spawn::{SpawnGroupData, finalize_spawn},
    },
    r#type::from_type,
};
use crate::world::World;

pub struct SpiderEntity {
    pub mob_entity: MobEntity,
}

impl SpiderEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        initialize_spider_navigation(&mut mob_entity);
        let spider = Self { mob_entity };
        let mob_arc = Arc::new(spider);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        register_spider_goals(&mob_arc.mob_entity, mob_weak);

        mob_arc
    }
}

pub fn initialize_spider_navigation(mob: &mut MobEntity) {
    *mob.navigator
        .get_mut()
        .unwrap_or_else(PoisonError::into_inner) = Navigator::wall_climber();
}

pub fn is_climbing(mob: &MobEntity) -> bool {
    mob.living_entity.climbing.load(Ordering::Relaxed)
}

pub fn set_climbing(mob: &MobEntity, climbing: bool) {
    if mob.living_entity.climbing.swap(climbing, Ordering::Relaxed) != climbing {
        let entity = &mob.living_entity.entity;
        entity.set_synced_data(DATA_FLAGS_ID, i8::from(climbing));
        let meta = MobMetadata::new().spider_climbing(climbing);
        entity.send_bedrock_actor_data(&meta.to_bedrock_actor_data());
    }
}

pub fn spider_metadata(mob: &MobEntity) -> MobMetadata {
    let meta = MobMetadata::new();
    if is_climbing(mob) {
        meta.spider_climbing(true)
    } else {
        meta
    }
}

pub fn register_spider_goals(mob_entity: &MobEntity, mob_weak: Weak<dyn Mob>) {
    let goal_selector = &mob_entity.goals_selector;
    let target_selector = &mob_entity.target_selector;

    goal_selector.add_goal(1, Box::new(SwimGoal::default()));
    // TODO: 2: avoid armadillos that aren't scared
    goal_selector.add_goal(3, Box::new(LeapAtTargetGoal::new(0.4)));
    goal_selector.add_goal(4, SpiderAttackGoal::new(1.0, true));
    goal_selector.add_goal(5, Box::new(WanderAroundGoal::water_avoiding(0.8)));
    goal_selector.add_goal(
        6,
        LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
    );
    goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));

    target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
    target_selector.add_goal(
        2,
        ActiveTargetGoal::with_default(mob_entity, &EntityType::PLAYER, true)
            .when(TargetCondition::NoDaylight),
    );
    target_selector.add_goal(
        3,
        ActiveTargetGoal::with_default(mob_entity, &EntityType::IRON_GOLEM, true)
            .when(TargetCondition::NoDaylight),
    );
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

    fn mob_metadata(&self) -> Option<MobMetadata> {
        Some(spider_metadata(&self.mob_entity))
    }

    fn post_tick(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        set_climbing(
            &self.mob_entity,
            entity.horizontal_collision.load(Ordering::SeqCst),
        );
    }
}
