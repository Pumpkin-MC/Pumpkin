use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_data::{
    Block,
    damage::DamageType,
    data_component_impl::EquipmentSlot,
    entity::EntityType,
    item::Item,
    sound::{Sound, SoundCategory},
    tag::{self, Taggable},
};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use rand::{RngExt, rng};

use crate::entity::{
    Entity, EntityBase, NBTStorage,
    ai::{
        goal::{
            Controls, Goal, GoalFuture, active_target::ActiveTargetGoal,
            zombie_attack::ZombieAttackGoal,
        },
        pathfinder::NavigatorGoal,
    },
    living::LivingEntity,
    mob::{Mob, MobEntity, zombie::ZombieEntity},
};
use crate::world::World;

const DROWNED_TRIDENT_DAMAGE: f32 = 8.0;

pub struct DrownedEntity {
    entity: Arc<ZombieEntity>,
    searching_for_land: AtomicBool,
}

impl DrownedEntity {
    #[expect(clippy::too_many_lines)]
    pub async fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntity::new(entity).await;
        let drowned = Self {
            entity,
            searching_for_land: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(drowned);
        let drowned_weak = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc.entity.mob_entity.goals_selector.lock().await;
            let mut target_selector = mob_arc.entity.mob_entity.target_selector.lock().await;

            // Drowned keeps inherited zombie register-goals entries (egg breaking + look goals)
            // but replaces zombie behavior and target goals.
            goal_selector
                .remove_goal::<ZombieAttackGoal>(mob_arc.as_ref())
                .await;
            target_selector
                .remove_goal::<ActiveTargetGoal>(mob_arc.as_ref())
                .await;

            goal_selector.add_goal(
                1,
                Box::new(DrownedGoToWaterGoal::new(drowned_weak.clone(), 1.0)),
            );
            goal_selector.add_goal(
                2,
                Box::new(DrownedTridentAttackGoal::new(
                    drowned_weak.clone(),
                    1.0,
                    40,
                    10.0,
                )),
            );
            goal_selector.add_goal(
                2,
                Box::new(DrownedAttackGoal::new(drowned_weak.clone(), 1.0, false)),
            );
            goal_selector.add_goal(
                5,
                Box::new(DrownedGoToBeachGoal::new(drowned_weak.clone(), 1.0)),
            );
            goal_selector.add_goal(
                6,
                Box::new(DrownedSwimUpGoal::new(drowned_weak.clone(), 1.0)),
            );
            goal_selector.add_goal(7, Box::new(DrownedRandomStrollGoal::new(1.0, 120)));

            target_selector.add_goal(1, Box::new(DrownedHurtByTargetGoal::new()));

            let drowned_player_target = drowned_weak.clone();
            target_selector.add_goal(
                2,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.entity.mob_entity,
                    &EntityType::PLAYER,
                    10,
                    true,
                    false,
                    Some(move |target: Arc<LivingEntity>, _world: Arc<World>| {
                        let drowned_player_target = drowned_player_target.clone();
                        async move {
                            if let Some(drowned) = drowned_player_target.upgrade() {
                                !drowned.is_bright_outside().await || target.is_in_water().await
                            } else {
                                false
                            }
                        }
                    }),
                )),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.entity.mob_entity,
                    &EntityType::VILLAGER,
                    false,
                ),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.entity.mob_entity,
                    &EntityType::IRON_GOLEM,
                    true,
                ),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.entity.mob_entity,
                    &EntityType::AXOLOTL,
                    true,
                ),
            );

            target_selector.add_goal(
                5,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.entity.mob_entity,
                    &EntityType::TURTLE,
                    10,
                    true,
                    false,
                    Some(|target: Arc<LivingEntity>, _world: Arc<World>| async move {
                        target.entity.age.load(Ordering::Relaxed) < 0 && !target.is_in_water().await
                    }),
                )),
            );
        };

        mob_arc
    }

    async fn is_bright_outside(&self) -> bool {
        let world = self.entity.mob_entity.living_entity.entity.world.load();
        let level_time = world.level_time.lock().await;
        !level_time.is_night()
    }

    fn set_searching_for_land(&self, searching_for_land: bool) {
        self.searching_for_land
            .store(searching_for_land, Ordering::Relaxed);
    }

    async fn has_trident_in_main_hand(&self) -> bool {
        let main_hand_item = {
            let equipment = self
                .entity
                .mob_entity
                .living_entity
                .entity_equipment
                .lock()
                .await;
            equipment.get(&EquipmentSlot::MAIN_HAND)
        };
        let main_hand_item = main_hand_item.lock().await;
        !main_hand_item.is_empty() && main_hand_item.item.id == Item::TRIDENT.id
    }

    async fn ok_target_entity(&self, target: &dyn EntityBase) -> bool {
        if !self.is_bright_outside().await {
            return true;
        }

        if let Some(target_living) = target.get_living_entity() {
            return target_living.is_in_water().await;
        }

        false
    }
}

impl NBTStorage for DrownedEntity {}

impl Mob for DrownedEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }
}

async fn set_navigation_goal(mob: &dyn Mob, destination: Vector3<f64>, speed: f64) {
    let mut navigator = mob.get_mob_entity().navigator.lock().await;
    navigator.set_progress(NavigatorGoal::new(
        mob.get_entity().pos.load(),
        destination,
        speed,
    ));
}

struct DrownedHurtByTargetGoal {
    pending_target: Option<Arc<dyn EntityBase>>,
    last_seen_attacker_time: i64,
}

impl DrownedHurtByTargetGoal {
    fn new() -> Self {
        Self {
            pending_target: None,
            last_seen_attacker_time: 0,
        }
    }
}

impl Goal for DrownedHurtByTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let living = &mob.get_mob_entity().living_entity;
            let attacker_time = living.last_attacker_time();
            if attacker_time <= self.last_seen_attacker_time {
                return false;
            }

            let Some(attacker) = living.get_last_attacker() else {
                return false;
            };
            if !attacker.get_entity().is_alive() || attacker.get_living_entity().is_none() {
                return false;
            }
            if attacker.get_entity().entity_type == &EntityType::DROWNED {
                return false;
            }

            self.pending_target = Some(attacker);
            true
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let mob_target = mob.get_mob_entity().target.lock().await;
            mob_target.as_ref().is_some_and(|target| {
                target.get_entity().is_alive() && target.get_living_entity().is_some()
            })
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.last_seen_attacker_time = mob.get_mob_entity().living_entity.last_attacker_time();
            let mut target = mob.get_mob_entity().target.lock().await;
            (*target).clone_from(&self.pending_target);
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.pending_target = None;
        })
    }

    fn controls(&self) -> Controls {
        Controls::TARGET
    }
}

struct DrownedAttackGoal {
    drowned: Weak<DrownedEntity>,
    zombie_attack_goal: Box<ZombieAttackGoal>,
}

impl DrownedAttackGoal {
    fn new(drowned: Weak<DrownedEntity>, speed: f64, pause_when_mob_idle: bool) -> Self {
        Self {
            drowned,
            zombie_attack_goal: ZombieAttackGoal::new(speed, pause_when_mob_idle),
        }
    }

    async fn ok_target(&self, mob: &dyn Mob) -> bool {
        let target = mob.get_mob_entity().target.lock().await.clone();
        let Some(target) = target else {
            return false;
        };
        let Some(drowned) = self.drowned.upgrade() else {
            return false;
        };
        drowned.ok_target_entity(target.as_ref()).await
    }
}

impl Goal for DrownedAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.zombie_attack_goal.can_start(mob).await && self.ok_target(mob).await
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.zombie_attack_goal.should_continue(mob).await && self.ok_target(mob).await
        })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.zombie_attack_goal.start(mob).await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.zombie_attack_goal.stop(mob).await;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.zombie_attack_goal.tick(mob).await;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        self.zombie_attack_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.zombie_attack_goal.controls()
    }
}

struct DrownedTridentAttackGoal {
    drowned: Weak<DrownedEntity>,
    speed: f64,
    attack_interval_ticks: i32,
    max_attack_distance_sq: f64,
    cooldown: i32,
}

impl DrownedTridentAttackGoal {
    fn new(
        drowned: Weak<DrownedEntity>,
        speed: f64,
        attack_interval_ticks: i32,
        max_attack_distance: f32,
    ) -> Self {
        Self {
            drowned,
            speed,
            attack_interval_ticks,
            max_attack_distance_sq: f64::from(max_attack_distance * max_attack_distance),
            cooldown: 0,
        }
    }

    async fn has_valid_target(&self, mob: &dyn Mob) -> Option<Arc<dyn EntityBase>> {
        let drowned = self.drowned.upgrade()?;
        if !drowned.has_trident_in_main_hand().await {
            return None;
        }

        let target = mob.get_mob_entity().target.lock().await.clone()?;
        if !target.get_entity().is_alive() {
            return None;
        }
        if !drowned.ok_target_entity(target.as_ref()).await {
            return None;
        }

        let mob_pos = mob.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        if mob_pos.squared_distance_to_vec(&target_pos) > self.max_attack_distance_sq {
            return None;
        }

        Some(target)
    }
}

impl Goal for DrownedTridentAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.has_valid_target(mob).await.is_some() })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.has_valid_target(mob).await.is_some() })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.cooldown = 0;
            mob.get_mob_entity().set_attacking(true).await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.get_mob_entity().set_attacking(false).await;
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(target) = self.has_valid_target(mob).await else {
                return;
            };

            let target_pos = target.get_entity().pos.load();
            mob.get_mob_entity()
                .look_control
                .lock()
                .await
                .look_at_entity_with_range(&target, 30.0, 30.0);

            set_navigation_goal(mob, target_pos, self.speed).await;

            if self.cooldown > 0 {
                self.cooldown -= 1;
                return;
            }

            self.cooldown = self.get_tick_count(self.attack_interval_ticks);
            mob.get_mob_entity().living_entity.swing_hand().await;
            target
                .damage_with_context(
                    target.as_ref(),
                    DROWNED_TRIDENT_DAMAGE,
                    DamageType::TRIDENT,
                    None,
                    Some(mob as &dyn EntityBase),
                    Some(mob as &dyn EntityBase),
                )
                .await;

            let world = mob.get_entity().world.load();
            world
                .play_sound(
                    Sound::EntityDrownedShoot,
                    SoundCategory::Hostile,
                    &mob.get_entity().pos.load(),
                )
                .await;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

struct DrownedGoToWaterGoal {
    drowned: Weak<DrownedEntity>,
    speed: f64,
    wanted_pos: Option<Vector3<f64>>,
}

impl DrownedGoToWaterGoal {
    const fn new(drowned: Weak<DrownedEntity>, speed: f64) -> Self {
        Self {
            drowned,
            speed,
            wanted_pos: None,
        }
    }

    async fn find_water_pos(&self, mob: &dyn Mob) -> Option<Vector3<f64>> {
        let world = mob.get_entity().world.load();
        let origin = mob.get_entity().block_pos.load();

        for _ in 0..10 {
            let candidate = origin.add(
                rng().random_range(-10..11),
                2 - rng().random_range(0..8),
                rng().random_range(-10..11),
            );
            if world
                .get_fluid(&candidate)
                .await
                .has_tag(&tag::Fluid::MINECRAFT_WATER)
            {
                return Some(candidate.to_f64());
            }
        }

        None
    }
}

impl Goal for DrownedGoToWaterGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(drowned) = self.drowned.upgrade() else {
                return false;
            };
            if !drowned.is_bright_outside().await {
                return false;
            }
            if mob.get_mob_entity().living_entity.is_in_water().await {
                return false;
            }

            self.wanted_pos = self.find_water_pos(mob).await;
            self.wanted_pos.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { !mob.get_mob_entity().navigator.lock().await.is_idle() })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(wanted_pos) = self.wanted_pos {
                set_navigation_goal(mob, wanted_pos, self.speed).await;
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

struct DrownedGoToBeachGoal {
    drowned: Weak<DrownedEntity>,
    speed: f64,
    wanted_pos: Option<Vector3<f64>>,
}

impl DrownedGoToBeachGoal {
    const fn new(drowned: Weak<DrownedEntity>, speed: f64) -> Self {
        Self {
            drowned,
            speed,
            wanted_pos: None,
        }
    }

    async fn find_beach_pos(&self, mob: &dyn Mob) -> Option<Vector3<f64>> {
        let world = mob.get_entity().world.load();
        let origin = mob.get_entity().block_pos.load();

        for _ in 0..20 {
            let candidate = origin.add(
                rng().random_range(-8..9),
                rng().random_range(-2..3),
                rng().random_range(-8..9),
            );

            let block = world.get_block(&candidate).await;
            let above = world.get_block(&candidate.up()).await;
            let above2 = world.get_block(&candidate.up_height(2)).await;

            let fluid_is_water = world
                .get_fluid(&candidate)
                .await
                .has_tag(&tag::Fluid::MINECRAFT_WATER);
            if !fluid_is_water
                && above == &Block::AIR
                && above2 == &Block::AIR
                && block != &Block::AIR
            {
                return Some(candidate.up().to_f64());
            }
        }

        None
    }
}

impl Goal for DrownedGoToBeachGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(drowned) = self.drowned.upgrade() else {
                return false;
            };
            let world = mob.get_entity().world.load();

            if drowned.is_bright_outside().await {
                return false;
            }
            if !mob.get_mob_entity().living_entity.is_in_water().await {
                return false;
            }
            if mob.get_entity().pos.load().y < f64::from(world.sea_level - 3) {
                return false;
            }

            self.wanted_pos = self.find_beach_pos(mob).await;
            self.wanted_pos.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { !mob.get_mob_entity().navigator.lock().await.is_idle() })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(drowned) = self.drowned.upgrade() {
                drowned.set_searching_for_land(false);
            }
            if let Some(wanted_pos) = self.wanted_pos {
                set_navigation_goal(mob, wanted_pos, self.speed).await;
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

struct DrownedSwimUpGoal {
    drowned: Weak<DrownedEntity>,
    speed: f64,
    stuck: bool,
}

impl DrownedSwimUpGoal {
    const fn new(drowned: Weak<DrownedEntity>, speed: f64) -> Self {
        Self {
            drowned,
            speed,
            stuck: false,
        }
    }

    async fn can_swim_up(&self, mob: &dyn Mob) -> bool {
        let Some(drowned) = self.drowned.upgrade() else {
            return false;
        };
        let world = mob.get_entity().world.load();
        !drowned.is_bright_outside().await
            && mob.get_mob_entity().living_entity.is_in_water().await
            && mob.get_entity().pos.load().y < f64::from(world.sea_level - 2)
    }

    async fn find_swim_up_pos(&self, mob: &dyn Mob) -> Option<Vector3<f64>> {
        let world = mob.get_entity().world.load();
        let origin = mob.get_entity().block_pos.load();
        let target_y = world.sea_level - 1;

        for _ in 0..15 {
            let candidate = BlockPos::new(
                origin.0.x + rng().random_range(-4..5),
                target_y + rng().random_range(-1..2),
                origin.0.z + rng().random_range(-4..5),
            );
            if world
                .get_fluid(&candidate)
                .await
                .has_tag(&tag::Fluid::MINECRAFT_WATER)
            {
                return Some(candidate.to_f64());
            }
        }

        None
    }
}

impl Goal for DrownedSwimUpGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.can_swim_up(mob).await })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.can_swim_up(mob).await && !self.stuck })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(drowned) = self.drowned.upgrade() {
                drowned.set_searching_for_land(true);
            }
            self.stuck = false;
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(drowned) = self.drowned.upgrade() {
                drowned.set_searching_for_land(false);
            }
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let world = mob.get_entity().world.load();
            if mob.get_entity().pos.load().y >= f64::from(world.sea_level - 1) {
                return;
            }

            if !mob.get_mob_entity().navigator.lock().await.is_idle() {
                return;
            }

            if let Some(target) = self.find_swim_up_pos(mob).await {
                set_navigation_goal(mob, target, self.speed).await;
            } else {
                self.stuck = true;
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

struct DrownedRandomStrollGoal {
    speed: f64,
    interval: i32,
    target_pos: Option<Vector3<f64>>,
}

impl DrownedRandomStrollGoal {
    const fn new(speed: f64, interval: i32) -> Self {
        Self {
            speed,
            interval,
            target_pos: None,
        }
    }

    async fn find_stroll_pos(&self, mob: &dyn Mob) -> Option<Vector3<f64>> {
        let world = mob.get_entity().world.load();
        let origin = mob.get_entity().block_pos.load();

        for _ in 0..10 {
            let candidate = origin.add(
                rng().random_range(-10..11),
                rng().random_range(-3..4),
                rng().random_range(-10..11),
            );
            let above = world.get_block(&candidate.up()).await;
            if above == &Block::AIR {
                return Some(candidate.to_f64());
            }
        }

        None
    }
}

impl Goal for DrownedRandomStrollGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            if mob.get_mob_entity().target.lock().await.is_some() {
                return false;
            }

            if rng().random_range(0..self.interval) != 0 {
                return false;
            }

            self.target_pos = self.find_stroll_pos(mob).await;
            self.target_pos.is_some()
        })
    }

    fn should_continue<'a>(&'a self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { !mob.get_mob_entity().navigator.lock().await.is_idle() })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if let Some(target_pos) = self.target_pos {
                set_navigation_goal(mob, target_pos, self.speed).await;
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
