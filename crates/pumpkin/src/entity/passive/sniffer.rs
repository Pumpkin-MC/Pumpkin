use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, swim::SwimGoal, tempt::TemptGoal,
        wander_around::WanderAroundGoal,
    },
    item::ItemEntity,
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};

pub const SNIFFER_FOOD: &[&Item] = &[&Item::TORCHFLOWER_SEEDS, &Item::PITCHER_POD];
pub const SNIFFER_BABY_START_AGE: i32 = -48000;
pub const DIGGING_DROP_SEED_OFFSET_TICKS: i32 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum SnifferState {
    #[default]
    Idling = 0,
    FeelingHappy = 1,
    Scenting = 2,
    Sniffing = 3,
    Searching = 4,
    Digging = 5,
    Rising = 6,
}

impl SnifferState {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::FeelingHappy,
            2 => Self::Scenting,
            3 => Self::Sniffing,
            4 => Self::Searching,
            5 => Self::Digging,
            6 => Self::Rising,
            _ => Self::Idling,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }
}

/// Represents a Sniffer, a large passive mob that digs up ancient seeds.
///
/// Wiki: <https://minecraft.wiki/w/Sniffer>
pub struct SnifferEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub state: AtomicI32,
    pub drop_seed_at_tick: AtomicI32,
    pub explored_positions: std::sync::Mutex<Vec<BlockPos>>,
}

impl SnifferEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let sniffer = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            state: AtomicI32::new(SnifferState::Idling.id()),
            drop_seed_at_tick: AtomicI32::new(0),
            explored_positions: std::sync::Mutex::new(Vec::new()),
        };
        let mob_arc = Arc::new(sniffer);
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
            goal_selector.add_goal(1, BreedGoal::new(1.0));
            goal_selector.add_goal(2, Box::new(TemptGoal::new(1.2, SNIFFER_FOOD)));
            goal_selector.add_goal(3, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_state(&self) -> SnifferState {
        SnifferState::from_id(self.state.load(Ordering::Relaxed))
    }

    pub fn set_state(&self, state: SnifferState) {
        self.state.store(state.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::sniffer::STATE,
                VarInt(state.id()),
            )],
            None,
        );
    }

    pub fn transition_to(&self, state: SnifferState) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();

        match state {
            SnifferState::Idling => {
                self.set_state(SnifferState::Idling);
            }
            SnifferState::FeelingHappy => {
                world.play_sound(Sound::EntitySnifferHappy, SoundCategory::Neutral, &pos);
                self.set_state(SnifferState::FeelingHappy);
            }
            SnifferState::Scenting => {
                self.set_state(SnifferState::Scenting);
                self.on_scenting_start();
            }
            SnifferState::Sniffing => {
                world.play_sound(Sound::EntitySnifferSniffing, SoundCategory::Neutral, &pos);
                self.set_state(SnifferState::Sniffing);
            }
            SnifferState::Searching => {
                self.set_state(SnifferState::Searching);
            }
            SnifferState::Digging => {
                self.set_state(SnifferState::Digging);
                self.on_digging_start();
            }
            SnifferState::Rising => {
                world.play_sound(
                    Sound::EntitySnifferDiggingStop,
                    SoundCategory::Neutral,
                    &pos,
                );
                self.set_state(SnifferState::Rising);
            }
        }
    }

    fn on_scenting_start(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        let pitch = if self.is_baby() { 1.3 } else { 1.0 };
        world.play_sound_fine(
            Sound::EntitySnifferScenting,
            SoundCategory::Neutral,
            &pos,
            1.0,
            pitch,
        );
    }

    fn on_digging_start(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let current_ticks = world.level_time.try_lock().map_or(0, |t| t.world_age);
        let drop_tick = current_ticks as i32 + DIGGING_DROP_SEED_OFFSET_TICKS;
        self.drop_seed_at_tick.store(drop_tick, Ordering::Relaxed);
        entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::sniffer::DROP_SEED_AT_TICK,
                VarInt(drop_tick),
            )],
            None,
        );
        world.send_entity_status(entity, EntityStatus::SnifferDiggingSound, None);
    }

    pub fn on_digging_complete(&self, success: bool) {
        if success {
            let head_block = self.get_head_block().down();
            self.store_explored_position(head_block);
        }
    }

    #[must_use]
    pub fn get_head_position(&self) -> Vector3<f64> {
        let entity = self.get_entity();
        let pos = entity.pos.load();
        let yaw = f64::from(entity.yaw.load());
        let yaw_rad = yaw.to_radians();
        let forward = Vector3::new(-yaw_rad.sin(), 0.0, yaw_rad.cos());
        pos + forward * 2.25
    }

    #[must_use]
    pub fn get_head_block(&self) -> BlockPos {
        let head_pos = self.get_head_position();
        let entity = self.get_entity();
        let pos = entity.pos.load();
        BlockPos::floored(head_pos.x, pos.y + 0.2, head_pos.z)
    }

    #[must_use]
    pub fn can_dig(&self) -> bool {
        let entity = self.get_entity();
        !self.is_panicking()
            && !self.is_baby()
            && !self.mob_entity.living_entity.is_in_water()
            && entity.on_ground.load(Ordering::Relaxed)
            && self.can_dig_at(self.get_head_block().down())
    }

    fn can_dig_at(&self, pos: BlockPos) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load();
        let block_state = world.get_block_state(&pos);
        let block = pumpkin_data::Block::from_state_id(block_state.id);

        if !block.has_tag(&tag::Block::MINECRAFT_SNIFFER_DIGGABLE_BLOCK) {
            return false;
        }

        let explored = self
            .explored_positions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        !explored.contains(&pos)
    }

    fn store_explored_position(&self, pos: BlockPos) {
        let mut explored = self
            .explored_positions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if explored.len() >= 20 {
            explored.pop();
        }
        explored.insert(0, pos);
    }

    pub async fn drop_seed(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let current_tick = world.level_time.lock().await.world_age as i32;

        if self.drop_seed_at_tick.load(Ordering::Relaxed) == current_tick {
            let head_pos = self.get_head_position();
            let seed_item = if rand::random::<bool>() {
                &Item::TORCHFLOWER_SEEDS
            } else {
                &Item::PITCHER_POD
            };
            let item_stack = ItemStack::new(1, seed_item);

            let item_entity = Entity::new(world.clone(), head_pos, &EntityType::ITEM);
            let item_arc = Arc::new(ItemEntity::new(item_entity, item_stack));
            world.spawn_entity(item_arc).await;

            world.play_sound(
                Sound::EntitySnifferDropSeed,
                SoundCategory::Neutral,
                &head_pos,
            );
        }
    }

    pub async fn spawn_child_from_breeding(&self, partner: &dyn EntityBase) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();

        let item_stack = ItemStack::new(1, &Item::SNIFFER_EGG);
        let egg_entity = Entity::new(world.clone(), pos, &EntityType::ITEM);
        let egg_arc = Arc::new(ItemEntity::new(egg_entity, item_stack));
        world.spawn_entity(egg_arc).await;

        world.play_sound(Sound::BlockSnifferEggPlop, SoundCategory::Neutral, &pos);

        self.mob_entity.reset_love_ticks();
        self.mob_entity
            .breeding_cooldown
            .store(6000, Ordering::Relaxed);
        partner.reset_love();
        partner.set_breeding_cooldown(6000);
    }
}

impl AgeableMob for SnifferEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn get_baby_start_age(&self) -> i32 {
        SNIFFER_BABY_START_AGE
    }
}

impl NBTStorage for SnifferEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_ageable_nbt(nbt);
            self.write_animal_nbt(nbt);
            nbt.put_int("State", self.get_state().id());
            nbt.put_int(
                "DropSeedAtTick",
                self.drop_seed_at_tick.load(Ordering::Relaxed),
            );
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_ageable_nbt(nbt);
            self.read_animal_nbt(nbt);
            if let Some(state_id) = nbt.get_int("State") {
                self.state.store(state_id, Ordering::Relaxed);
            }
            if let Some(drop_tick) = nbt.get_int("DropSeedAtTick") {
                self.drop_seed_at_tick.store(drop_tick, Ordering::Relaxed);
            }
        })
    }
}

impl Animal for SnifferEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_SNIFFER_FOOD)
            || SNIFFER_FOOD.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl Mob for SnifferEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.ageable_ai_step();
            let state = self.get_state();
            match state {
                SnifferState::Searching => {
                    let entity = self.get_entity();
                    let world = entity.world.load();
                    let ticks = world.level_time.lock().await.world_age;
                    if ticks % 20 == 0 {
                        world.play_sound(
                            Sound::EntitySnifferSearching,
                            SoundCategory::Neutral,
                            &entity.pos.load(),
                        );
                    }
                }
                SnifferState::Digging => {
                    self.drop_seed().await;
                }
                _ => {}
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
                        pumpkin_data::tracked_data::sniffer::BABY_ID,
                        true,
                    )],
                    None,
                );
            }
            entity.send_meta_data(
                &[
                    Metadata::new(
                        pumpkin_data::tracked_data::sniffer::STATE,
                        VarInt(self.get_state().id()),
                    ),
                    Metadata::new(
                        pumpkin_data::tracked_data::sniffer::DROP_SEED_AT_TICK,
                        VarInt(self.drop_seed_at_tick.load(Ordering::Relaxed)),
                    ),
                ],
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
            self.animal_interact(player, item_stack, Sound::EntitySnifferEat)
                .await
        })
    }
}
