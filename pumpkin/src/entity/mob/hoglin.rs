use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityType;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::zoglin::ZoglinEntity,
    mob::{Mob, MobEntity},
};

pub struct HoglinEntity {
    pub mob_entity: MobEntity,
    pub time_in_overworld: AtomicI32,
}

impl HoglinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let hoglin = Self {
            mob_entity,
            time_in_overworld: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(hoglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for HoglinEntity {}

impl Mob for HoglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = caller.get_entity();
            let overworld_time = self.time_in_overworld.load(Ordering::Relaxed);
            let world = entity.world.load_full();

            if world.dimension == Dimension::THE_NETHER {
                if overworld_time > 0 {
                    self.time_in_overworld
                        .store(overworld_time - 1, Ordering::Relaxed);
                }
                return;
            }

            let new_time = overworld_time + 1;
            self.time_in_overworld.store(new_time, Ordering::Relaxed);

            if new_time >= 300 {
                convert_to_zoglin(caller).await;
            }
        })
    }
}

async fn convert_to_zoglin(caller: &Arc<dyn EntityBase>) {
    let entity = caller.get_entity();
    let pos = entity.pos.load();
    let world = entity.world.load_full();
    let uuid = entity.entity_uuid;
    let yaw = entity.yaw.load();
    let pitch = entity.pitch.load();
    let age = entity.age.load(Ordering::Relaxed);
    let custom_name = entity.custom_name.load().clone();
    let custom_name_visible = entity.custom_name_visible.load(Ordering::Relaxed);
    let velocity = entity.velocity.load();

    let living = caller
        .get_living_entity()
        .expect("Hoglin must be a living entity");
    let health = living.health.load();
    let equipment = {
        let eq = living.entity_equipment.lock().await;
        eq.clone()
    };
    let active_effects = {
        let effects = living.active_effects.lock().await;
        effects.clone()
    };

    entity.remove().await;

    let new_entity = Entity::from_uuid(uuid, world.clone(), pos, &EntityType::ZOGLIN);
    let zoglin = ZoglinEntity::new(new_entity);

    let zoglin_entity = zoglin.get_entity();
    let zoglin_living = zoglin
        .get_living_entity()
        .expect("Zoglin must be a living entity");

    zoglin_living.health.store(health);
    zoglin_entity.age.store(age, Ordering::Relaxed);
    zoglin_entity.custom_name.store(custom_name);
    zoglin_entity
        .custom_name_visible
        .store(custom_name_visible, Ordering::Relaxed);
    zoglin_entity.yaw.store(yaw);
    zoglin_entity.pitch.store(pitch);
    zoglin_entity.velocity.store(velocity);

    {
        let mut eq = zoglin_living.entity_equipment.lock().await;
        *eq = equipment;
        drop(eq);
    };
    {
        let mut effects = zoglin_living.active_effects.lock().await;
        *effects = active_effects;
        drop(effects);
    };

    world.spawn_entity(zoglin).await;
}
