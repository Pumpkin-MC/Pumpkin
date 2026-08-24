use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, AtomicU64, Ordering},
};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NbtFuture,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    item::ItemEntity,
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

pub const ARMADILLO_FOOD: &[&Item] = &[&Item::SPIDER_EYE];
pub const ARMADILLO_BABY_START_AGE: i32 = -48000;
pub const SCARE_CHECK_INTERVAL: i32 = 80;
pub const SCARE_DISTANCE_HORIZONTAL: f64 = 7.0;
pub const SCARE_DISTANCE_VERTICAL: f64 = 2.0;

fn pick_next_scute_drop_time() -> i32 {
    let rand_ticks = (rand::random::<u32>() % 6000) as i32;
    rand_ticks + 6000
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum ArmadilloState {
    #[default]
    Idle = 0,
    Rolling = 1,
    Scared = 2,
    Unrolling = 3,
}

impl ArmadilloState {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Rolling,
            2 => Self::Scared,
            3 => Self::Unrolling,
            _ => Self::Idle,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn is_threatened(self) -> bool {
        !matches!(self, Self::Idle)
    }

    #[must_use]
    pub const fn animation_duration(self) -> u64 {
        match self {
            Self::Idle => 0,
            Self::Rolling => 10,
            Self::Scared => 50,
            Self::Unrolling => 30,
        }
    }

    #[must_use]
    pub const fn should_hide_in_shell(self, ticks_in_state: u64) -> bool {
        match self {
            Self::Idle => false,
            Self::Rolling => ticks_in_state > 5,
            Self::Scared => true,
            Self::Unrolling => ticks_in_state < 26,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Rolling => "rolling",
            Self::Scared => "scared",
            Self::Unrolling => "unrolling",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "rolling" => Self::Rolling,
            "scared" => Self::Scared,
            "unrolling" => Self::Unrolling,
            _ => Self::Idle,
        }
    }
}

/// Represents an Armadillo, a passive entity that can roll into a ball when threatened.
///
/// Wiki: <https://minecraft.wiki/w/Armadillo>
pub struct ArmadilloEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub state: AtomicI32,
    pub in_state_ticks: AtomicU64,
    pub scute_time: AtomicI32,
    pub danger_detected_recently_ticks: AtomicI32,
}

impl ArmadilloEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let armadillo = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            state: AtomicI32::new(ArmadilloState::Idle.id()),
            in_state_ticks: AtomicU64::new(0),
            scute_time: AtomicI32::new(pick_next_scute_drop_time()),
            danger_detected_recently_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(armadillo);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, ARMADILLO_FOOD)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_state(&self) -> ArmadilloState {
        ArmadilloState::from_id(self.state.load(Ordering::Relaxed))
    }

    pub fn switch_to_state(&self, state: ArmadilloState) {
        self.state.store(state.id(), Ordering::Relaxed);
        self.in_state_ticks.store(0, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::armadillo::ARMADILLO_STATE,
                VarInt(state.id()),
            )],
            None,
        );
    }

    #[must_use]
    pub fn is_scared(&self) -> bool {
        self.get_state() != ArmadilloState::Idle
    }

    #[must_use]
    pub fn should_hide_in_shell(&self) -> bool {
        self.get_state()
            .should_hide_in_shell(self.in_state_ticks.load(Ordering::Relaxed))
    }

    #[must_use]
    pub fn should_switch_to_scared_state(&self) -> bool {
        self.get_state() == ArmadilloState::Rolling
            && self.in_state_ticks.load(Ordering::Relaxed)
                > ArmadilloState::Rolling.animation_duration()
    }

    pub async fn can_stay_rolled_up(&self) -> bool {
        !self.is_panicking()
            && !self.mob_entity.living_entity.is_in_water()
            && !self.get_entity().has_vehicle().await
    }

    pub fn roll_up(&self) {
        if !self.is_scared() {
            self.mob_entity.reset_love_ticks();
            let entity = self.get_entity();
            let world = entity.world.load();
            world.play_sound(
                Sound::EntityArmadilloRoll,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
            self.switch_to_state(ArmadilloState::Rolling);
        }
    }

    pub fn roll_out(&self) {
        if self.is_scared() {
            let entity = self.get_entity();
            let world = entity.world.load();
            world.play_sound(
                Sound::EntityArmadilloUnrollFinish,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
            self.switch_to_state(ArmadilloState::Idle);
        }
    }

    pub fn brush_off_scute<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if self.is_baby() {
                return false;
            }
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            let item_entity = Arc::new(ItemEntity::new(
                Entity::new(world.clone(), pos, &EntityType::ITEM),
                ItemStack::new(1, &Item::ARMADILLO_SCUTE),
            ));
            world.spawn_entity(item_entity).await;
            world.play_sound(Sound::EntityArmadilloBrush, SoundCategory::Neutral, &pos);
            player.damage_held_item(16).await;
            true
        })
    }

    pub async fn is_scared_by(&self, living_entity: &dyn EntityBase) -> bool {
        let entity = self.get_entity();
        let pos = entity.pos.load();
        let target_pos = living_entity.get_entity().pos.load();
        let dx = (pos.x - target_pos.x).abs();
        let dy = (pos.y - target_pos.y).abs();
        let dz = (pos.z - target_pos.z).abs();

        if dx > SCARE_DISTANCE_HORIZONTAL
            || dz > SCARE_DISTANCE_HORIZONTAL
            || dy > SCARE_DISTANCE_VERTICAL
        {
            return false;
        }

        let target_type = living_entity.get_entity().entity_type;
        if target_type.has_tag(&tag::EntityType::MINECRAFT_UNDEAD) {
            return true;
        }

        if target_type == &EntityType::PLAYER {
            let target_ent = living_entity.get_entity();
            if target_ent.is_sprinting() || target_ent.has_vehicle().await {
                return true;
            }
        }

        false
    }
}

impl AgeableMob for ArmadilloEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn get_baby_start_age(&self) -> i32 {
        ARMADILLO_BABY_START_AGE
    }
}

impl Animal for ArmadilloEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_ARMADILLO_FOOD)
            || ARMADILLO_FOOD.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl Mob for ArmadilloEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            nbt.put_string("state", self.get_state().name().to_string());
            nbt.put_int("scute_time", self.scute_time.load(Ordering::Relaxed));
        })
    }

    fn mob_read_nbt<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            if let Some(state_name) = nbt.get_string("state") {
                self.switch_to_state(ArmadilloState::from_name(state_name));
            } else if let Some(state_id) = nbt.get_int("state") {
                self.switch_to_state(ArmadilloState::from_id(state_id));
            }
            if let Some(scute_time) = nbt.get_int("scute_time") {
                self.scute_time.store(scute_time, Ordering::Relaxed);
            }
        })
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn modify_incoming_damage(&self, amount: f32, _damage_type: DamageType) -> f32 {
        if self.is_scared() {
            (amount - 1.0).max(0.0) / 2.0
        } else {
            amount
        }
    }

    fn on_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.get_entity().is_alive()
                && let Some(src) = source
                && src.get_entity().entity_type != &EntityType::ITEM
            {
                self.danger_detected_recently_ticks
                    .store(SCARE_CHECK_INTERVAL, Ordering::Relaxed);
                if self.can_stay_rolled_up().await {
                    self.roll_up();
                }
            }
        })
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.ageable_ai_step();

            self.in_state_ticks.fetch_add(1, Ordering::Relaxed);
            let danger_ticks = self.danger_detected_recently_ticks.load(Ordering::Relaxed);
            if danger_ticks > 0 {
                self.danger_detected_recently_ticks
                    .store(danger_ticks - 1, Ordering::Relaxed);
            }

            let entity = self.get_entity();
            let world = entity.world.load();

            if entity.is_alive() && !self.is_baby() {
                let scute_time = self.scute_time.fetch_sub(1, Ordering::Relaxed) - 1;
                if scute_time <= 0 {
                    let pos = entity.pos.load();
                    let item_entity = Arc::new(ItemEntity::new(
                        Entity::new(world.clone(), pos, &EntityType::ITEM),
                        ItemStack::new(1, &Item::ARMADILLO_SCUTE),
                    ));
                    world.spawn_entity(item_entity).await;
                    world.play_sound(
                        Sound::EntityArmadilloScuteDrop,
                        SoundCategory::Neutral,
                        &pos,
                    );
                    self.scute_time
                        .store(pick_next_scute_drop_time(), Ordering::Relaxed);
                }
            }

            let state = self.get_state();
            let ticks_in_state = self.in_state_ticks.load(Ordering::Relaxed);

            match state {
                ArmadilloState::Rolling => {
                    if ticks_in_state > ArmadilloState::Rolling.animation_duration() {
                        self.switch_to_state(ArmadilloState::Scared);
                    }
                }
                ArmadilloState::Scared => {
                    if !self.can_stay_rolled_up().await {
                        self.roll_out();
                    } else if ticks_in_state > ArmadilloState::Scared.animation_duration()
                        && self.danger_detected_recently_ticks.load(Ordering::Relaxed) == 0
                    {
                        self.switch_to_state(ArmadilloState::Unrolling);
                    }
                }
                ArmadilloState::Unrolling => {
                    if ticks_in_state > ArmadilloState::Unrolling.animation_duration() {
                        self.roll_out();
                    }
                }
                ArmadilloState::Idle => {}
            }
        })
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let is_baby = entity.age.load(Ordering::Relaxed) < 0;
            if is_baby {
                entity.send_meta_data(
                    &[Metadata::new(
                        pumpkin_data::tracked_data::armadillo::BABY_ID,
                        true,
                    )],
                    None,
                );
            }
            entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::armadillo::ARMADILLO_STATE,
                    VarInt(self.get_state().id()),
                )],
                None,
            );
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if item_stack.item == &Item::BRUSH && self.brush_off_scute(player).await {
                return true;
            }
            if self.is_scared() {
                return false;
            }
            self.animal_interact(player, item_stack, Sound::EntityArmadilloAmbient)
                .await
        })
    }
}
