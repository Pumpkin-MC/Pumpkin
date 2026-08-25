use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::Block;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::boundingbox::EntityDimensions;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

use crate::entity::living::LivingEntity;
use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, open_door::OpenDoorGoal,
        ranged_crossbow_attack::RangedCrossbowAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity, crossbow_attack_mob::CrossbowAttackMob, equipment::RegionalDifficulty,
        piglin_ai::PiglinAi,
    },
};
use crate::world::World;

pub struct PiglinEntity {
    pub mob_entity: MobEntity,
    pub immune_to_zombification: AtomicBool,
    pub time_in_overworld: AtomicI32,
    pub is_baby: AtomicBool,
    pub cannot_hunt: AtomicBool,
    pub is_charging_crossbow: AtomicBool,
    pub is_dancing: AtomicBool,
    pub inventory: Mutex<Vec<ItemStack>>,
    pub admire_timer: AtomicI32,
    pub admiring_disabled_timer: AtomicI32,
    pub eat_cooldown_timer: AtomicI32,
    pub celebration_timer: AtomicI32,
    pub hunt_cooldown_timer: AtomicI32,
    pub admiring_item: Mutex<Option<ItemStack>>,
}

impl PiglinEntity {
    pub const CONVERSION_TIME: i32 = 300;
    pub const INVENTORY_SIZE: usize = 8;
    pub const XP_REWARD: u32 = 5;

    pub const ADULT_DIMENSIONS: EntityDimensions = EntityDimensions {
        width: 0.6,
        height: 1.95,
        eye_height: 1.79,
    };
    pub const BABY_DIMENSIONS: EntityDimensions = EntityDimensions {
        width: 0.49,
        height: 0.98,
        eye_height: 0.78,
    };

    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let piglin = Self {
            mob_entity,
            immune_to_zombification: AtomicBool::new(false),
            time_in_overworld: AtomicI32::new(0),
            is_baby: AtomicBool::new(false),
            cannot_hunt: AtomicBool::new(false),
            is_charging_crossbow: AtomicBool::new(false),
            is_dancing: AtomicBool::new(false),
            inventory: Mutex::new(Vec::new()),
            admire_timer: AtomicI32::new(0),
            admiring_disabled_timer: AtomicI32::new(0),
            eat_cooldown_timer: AtomicI32::new(0),
            celebration_timer: AtomicI32::new(0),
            hunt_cooldown_timer: AtomicI32::new(0),
            admiring_item: Mutex::new(None),
        };
        let mob_arc = Arc::new(piglin);
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
            goal_selector.add_goal(1, Box::new(OpenDoorGoal::new(true)));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(3, Box::new(RangedCrossbowAttackGoal::new(1.0, 8.0)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Retaliate when attacked
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));

            // Hostile to players unless wearing safe gold armor
            target_selector.add_goal(
                2,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::PLAYER,
                    10,
                    true,
                    false,
                    Some(|target: Arc<LivingEntity>, _world: Arc<World>| {
                        Box::pin(async move { !PiglinAi::is_wearing_safe_armor(&target).await })
                    }),
                )),
            );

            // Hostile to wither skeletons and withers
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &mob_arc.mob_entity,
                    &EntityType::WITHER_SKELETON,
                    true,
                ),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::WITHER, true),
            );

            // Adults that can hunt are hostile to hoglins
            let piglin_clone = mob_arc.clone();
            target_selector.add_goal(
                4,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::HOGLIN,
                    10,
                    true,
                    false,
                    Some(move |_target: Arc<LivingEntity>, _world: Arc<World>| {
                        let piglin = piglin_clone.clone();
                        Box::pin(async move { piglin.is_adult() && piglin.can_hunt() })
                    }),
                )),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn is_immune_to_zombification(&self) -> bool {
        self.immune_to_zombification.load(Ordering::Relaxed)
    }

    pub fn set_immune_to_zombification(&self, immune: bool) {
        self.immune_to_zombification
            .store(immune, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                tracked_data::piglin::DATA_IMMUNE_TO_ZOMBIFICATION,
                immune,
            )],
            None,
        );
    }

    #[must_use]
    pub fn is_converting(&self, world: &World) -> bool {
        !self.is_immune_to_zombification()
            && !self.mob_entity.is_no_ai()
            && world.dimension.minecraft_name != Dimension::THE_NETHER.minecraft_name
    }

    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.is_baby.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn is_adult(&self) -> bool {
        !self.is_baby()
    }

    pub fn set_baby(&self, baby: bool) {
        self.is_baby.store(baby, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.send_meta_data(
            &[Metadata::new(tracked_data::piglin::DATA_BABY_ID, baby)],
            None,
        );
        if baby {
            entity.entity_dimension.store(Self::BABY_DIMENSIONS);
        } else {
            entity.entity_dimension.store(Self::ADULT_DIMENSIONS);
        }
    }

    #[must_use]
    pub fn is_charging_crossbow(&self) -> bool {
        self.is_charging_crossbow.load(Ordering::Relaxed)
    }

    pub fn set_charging_crossbow(&self, is_charging: bool) {
        self.is_charging_crossbow
            .store(is_charging, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                tracked_data::piglin::DATA_IS_CHARGING_CROSSBOW,
                is_charging,
            )],
            None,
        );
    }

    #[must_use]
    pub fn is_dancing(&self) -> bool {
        self.is_dancing.load(Ordering::Relaxed)
    }

    pub fn set_dancing(&self, is_dancing: bool) {
        self.is_dancing.store(is_dancing, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                tracked_data::piglin::DATA_IS_DANCING,
                is_dancing,
            )],
            None,
        );
    }

    #[must_use]
    pub fn can_hunt(&self) -> bool {
        !self.cannot_hunt.load(Ordering::Relaxed)
            && self.hunt_cooldown_timer.load(Ordering::Relaxed) <= 0
    }

    pub fn set_cannot_hunt(&self, cannot_hunt: bool) {
        self.cannot_hunt.store(cannot_hunt, Ordering::Relaxed);
    }

    #[must_use]
    pub fn is_admiring(&self) -> bool {
        self.admire_timer.load(Ordering::Relaxed) > 0
    }

    #[must_use]
    pub fn is_admiring_disabled(&self) -> bool {
        self.admiring_disabled_timer.load(Ordering::Relaxed) > 0
    }

    #[must_use]
    pub fn has_eaten_recently(&self) -> bool {
        self.eat_cooldown_timer.load(Ordering::Relaxed) > 0
    }

    pub async fn start_admiring(&self, item: ItemStack) {
        self.admire_timer
            .store(PiglinAi::ADMIRE_DURATION, Ordering::Relaxed);
        *self.admiring_item.lock().await = Some(item.clone());

        let mut equip = self.mob_entity.living_entity.entity_equipment.lock().await;
        equip.put(&EquipmentSlot::OFF_HAND, item);

        let entity = &self.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        entity.world.load().play_sound(
            Sound::EntityPiglinAdmiringItem,
            SoundCategory::Hostile,
            &pos,
        );
    }

    pub async fn stop_holding_off_hand_item(&self, bartering_enabled: bool) {
        let admired_item = {
            let mut guard = self.admiring_item.lock().await;
            guard.take()
        };

        let _ = {
            let mut equip = self.mob_entity.living_entity.entity_equipment.lock().await;
            equip.put(&EquipmentSlot::OFF_HAND, ItemStack::EMPTY.clone())
        };

        let Some(item) = admired_item else {
            return;
        };

        if self.is_adult() {
            let is_barter = PiglinAi::is_barter_currency(&item);
            if bartering_enabled && is_barter {
                let outcomes = PiglinAi::get_barter_response_items();
                let entity = &self.mob_entity.living_entity.entity;

                let mut event =
                    crate::plugin::api::events::entity::piglin_barter::PiglinBarterEvent::new(
                        entity.entity_id,
                        item,
                        outcomes,
                    );
                if let Some(server) = entity.world.load().server.upgrade() {
                    server.plugin_manager.fire(&server, &mut event).await;
                }

                if !event.cancelled {
                    PiglinAi::throw_items(self, event.outcome, None).await;
                }
            } else if !is_barter {
                let remainder = self.add_to_inventory(item).await;
                if let Some(rem) = remainder {
                    PiglinAi::throw_items(self, vec![rem], None).await;
                }
            }
        } else {
            let remainder = self.add_to_inventory(item).await;
            if let Some(rem) = remainder {
                PiglinAi::throw_items(self, vec![rem], None).await;
            }
        }
    }

    pub async fn cancel_admiring(&self) {
        if self.is_admiring() {
            self.admire_timer.store(0, Ordering::Relaxed);
            let item = {
                let mut guard = self.admiring_item.lock().await;
                guard.take()
            };
            if let Some(item) = item {
                let _ = {
                    let mut equip = self.mob_entity.living_entity.entity_equipment.lock().await;
                    equip.put(&EquipmentSlot::OFF_HAND, ItemStack::EMPTY.clone())
                };
                PiglinAi::throw_items(self, vec![item], None).await;
            }
        }
    }

    pub async fn was_hurt_by(&self, attacker: Option<&dyn EntityBase>) {
        self.cancel_admiring().await;
        self.set_dancing(false);
        self.celebration_timer.store(0, Ordering::Relaxed);

        if let Some(attacker_entity) = attacker
            && attacker_entity.get_entity().entity_type.id == EntityType::PLAYER.id
        {
            self.admiring_disabled_timer
                .store(PiglinAi::ADMIRING_DISABLED_DURATION, Ordering::Relaxed);
        }
    }

    pub async fn add_to_inventory(&self, item: ItemStack) -> Option<ItemStack> {
        let mut inv = self.inventory.lock().await;
        if inv.len() < Self::INVENTORY_SIZE {
            inv.push(item);
            None
        } else {
            Some(item)
        }
    }

    pub async fn drop_inventory(&self) {
        let items = {
            let mut inv = self.inventory.lock().await;
            std::mem::take(&mut *inv)
        };
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        for item in items {
            if !item.is_empty() {
                let item_entity = crate::entity::item::ItemEntity::new(
                    Entity::new(world.clone(), pos, &EntityType::ITEM),
                    item,
                );
                world.spawn_entity(Arc::new(item_entity)).await;
            }
        }
    }

    #[must_use]
    pub fn check_piglin_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        let below = BlockPos::new(pos.0.x, pos.0.y - 1, pos.0.z);
        let state = world.get_block_state(&below);
        state.id != Block::NETHER_WART_BLOCK.default_state.id
    }

    async fn convert_to_zombified(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();

        if world.level_info.load().difficulty != pumpkin_util::Difficulty::Peaceful {
            world.play_sound(
                Sound::EntityPiglinConvertedToZombified,
                SoundCategory::Hostile,
                &pos,
            );
        }

        self.drop_inventory().await;

        let zombified = crate::entity::r#type::from_type(
            &EntityType::ZOMBIFIED_PIGLIN,
            pos,
            &world,
            uuid::Uuid::new_v4(),
        );

        let zombified_base = zombified.get_entity();
        zombified_base.set_rotation(entity.yaw.load(), entity.pitch.load());
        zombified_base.head_yaw.store(entity.head_yaw.load());
        zombified_base.velocity.store(entity.velocity.load());

        if let Some(living) = zombified.get_living_entity() {
            living.set_health(self.mob_entity.living_entity.health.load());
        }

        if let Some(custom_name) = &**entity.custom_name.load() {
            zombified_base.set_custom_name(custom_name.clone());
        }

        {
            let src_equip = self.mob_entity.living_entity.entity_equipment.lock().await;
            if let Some(living) = zombified.get_living_entity() {
                let mut dst_equip = living.entity_equipment.lock().await;
                for (slot, item) in &src_equip.equipment {
                    dst_equip.put(slot, item.clone());
                }
            }
        }

        world.spawn_entity(zombified).await;
        entity.remove().await;
    }
}

impl Mob for PiglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn populate_default_equipment_slots<'a>(
        &'a self,
        _world: &'a Arc<World>,
        _difficulty: &'a RegionalDifficulty,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if !self.is_baby.load(Ordering::Relaxed) {
                let living = &self.mob_entity.living_entity;
                let mut equipment = living.entity_equipment.lock().await;

                // Spawn weapon: 50% crossbow, 5% golden spear (10% of remaining 50%), 45% golden sword
                let weapon = if rand::random::<f32>() < 0.5 {
                    &Item::CROSSBOW
                } else if rand::random_range(0..10) == 0 {
                    &Item::GOLDEN_SPEAR
                } else {
                    &Item::GOLDEN_SWORD
                };
                equipment.put(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, weapon));

                // Armor: 10% chance per piece for golden armor
                if rand::random::<f32>() < 0.1 {
                    equipment.put(
                        &EquipmentSlot::HEAD,
                        ItemStack::new(1, &Item::GOLDEN_HELMET),
                    );
                }
                if rand::random::<f32>() < 0.1 {
                    equipment.put(
                        &EquipmentSlot::CHEST,
                        ItemStack::new(1, &Item::GOLDEN_CHESTPLATE),
                    );
                }
                if rand::random::<f32>() < 0.1 {
                    equipment.put(
                        &EquipmentSlot::LEGS,
                        ItemStack::new(1, &Item::GOLDEN_LEGGINGS),
                    );
                }
                if rand::random::<f32>() < 0.1 {
                    equipment.put(&EquipmentSlot::FEET, ItemStack::new(1, &Item::GOLDEN_BOOTS));
                }
            }
        })
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let mut meta = Vec::new();
            if self.is_immune_to_zombification() {
                meta.push(Metadata::new(
                    tracked_data::piglin::DATA_IMMUNE_TO_ZOMBIFICATION,
                    true,
                ));
            }
            if self.is_baby() {
                meta.push(Metadata::new(tracked_data::piglin::DATA_BABY_ID, true));
            }
            if self.is_charging_crossbow() {
                meta.push(Metadata::new(
                    tracked_data::piglin::DATA_IS_CHARGING_CROSSBOW,
                    true,
                ));
            }
            if self.is_dancing() {
                meta.push(Metadata::new(tracked_data::piglin::DATA_IS_DANCING, true));
            }
            if !meta.is_empty() {
                entity.send_meta_data(&meta, None);
            }
        })
    }

    fn mob_write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            if self.is_immune_to_zombification() {
                nbt.put_bool("IsImmuneToZombification", true);
            }
            let time_in_overworld = self.time_in_overworld.load(Ordering::Relaxed);
            if time_in_overworld > 0 {
                nbt.put_int("TimeInOverworld", time_in_overworld);
            }
            nbt.put_bool("CanPickUpLoot", true);
            if self.is_baby() {
                nbt.put_bool("IsBaby", true);
            }
            if !self.can_hunt() {
                nbt.put_bool("CannotHunt", true);
            }
            if self.is_charging_crossbow() {
                nbt.put_bool("IsChargingCrossbow", true);
            }
            if self.is_dancing() {
                nbt.put_bool("IsDancing", true);
            }

            let inv = self.inventory.lock().await;
            if !inv.is_empty() {
                let mut items_tag = Vec::new();
                for item in inv.iter() {
                    if !item.is_empty() {
                        let mut item_nbt = NbtCompound::new();
                        item.write_item_stack(&mut item_nbt);
                        items_tag.push(NbtTag::Compound(item_nbt));
                    }
                }
                if !items_tag.is_empty() {
                    nbt.put_list("Inventory", items_tag);
                }
            }
        })
    }

    fn mob_read_nbt<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            if let Some(immune) = nbt.get_bool("IsImmuneToZombification") {
                self.set_immune_to_zombification(immune);
            }
            if let Some(time) = nbt.get_int("TimeInOverworld") {
                self.time_in_overworld.store(time, Ordering::Relaxed);
            }
            if let Some(baby) = nbt.get_bool("IsBaby") {
                self.set_baby(baby);
            }
            if let Some(cannot_hunt) = nbt.get_bool("CannotHunt") {
                self.set_cannot_hunt(cannot_hunt);
            }
            if let Some(charging) = nbt.get_bool("IsChargingCrossbow") {
                self.set_charging_crossbow(charging);
            }
            if let Some(dancing) = nbt.get_bool("IsDancing") {
                self.set_dancing(dancing);
            }
            if let Some(inv_list) = nbt.get_list("Inventory") {
                let mut inv = self.inventory.lock().await;
                inv.clear();
                for tag in inv_list {
                    if let Some(compound) = tag.extract_compound()
                        && let Some(stack) = ItemStack::read_item_stack(compound)
                    {
                        inv.push(stack);
                    }
                }
            }
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if PiglinAi::can_admire(self, item_stack) {
                let mut given = item_stack.clone();
                given.item_count = 1;
                if player.gamemode.load() != pumpkin_util::GameMode::Creative {
                    item_stack.item_count -= 1;
                }
                self.start_admiring(given).await;
                return true;
            }
            self.mob_entity.mob_interact(player, item_stack).await
        })
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            if !entity.is_alive() {
                return;
            }

            let world = entity.world.load();
            if self.is_converting(&world) {
                let time = self.time_in_overworld.fetch_add(1, Ordering::Relaxed) + 1;
                if time > Self::CONVERSION_TIME {
                    self.convert_to_zombified().await;
                }
            } else {
                self.time_in_overworld.store(0, Ordering::Relaxed);
            }

            if self.admiring_disabled_timer.load(Ordering::Relaxed) > 0 {
                self.admiring_disabled_timer.fetch_sub(1, Ordering::Relaxed);
            }
            if self.eat_cooldown_timer.load(Ordering::Relaxed) > 0 {
                self.eat_cooldown_timer.fetch_sub(1, Ordering::Relaxed);
            }
            if self.hunt_cooldown_timer.load(Ordering::Relaxed) > 0 {
                self.hunt_cooldown_timer.fetch_sub(1, Ordering::Relaxed);
            }
            if self.celebration_timer.load(Ordering::Relaxed) > 0 {
                let remaining = self.celebration_timer.fetch_sub(1, Ordering::Relaxed) - 1;
                if remaining <= 0 {
                    self.set_dancing(false);
                }
            }

            if self.admire_timer.load(Ordering::Relaxed) > 0 {
                let remaining = self.admire_timer.fetch_sub(1, Ordering::Relaxed) - 1;
                if remaining <= 0 {
                    self.stop_holding_off_hand_item(true).await;
                }
            }
        })
    }

    fn on_damage<'a>(
        &'a self,
        _damage_type: pumpkin_data::damage::DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.mob_entity.living_entity.dead.load(Ordering::Relaxed) {
                self.drop_inventory().await;
            } else {
                self.was_hurt_by(source).await;
            }
        })
    }

    fn as_crossbow_attack_mob(&self) -> Option<&dyn CrossbowAttackMob> {
        Some(self)
    }

    fn get_base_experience_reward(&self) -> u32 {
        Self::XP_REWARD
    }
}

impl CrossbowAttackMob for PiglinEntity {
    fn set_charging_crossbow(&self, is_charging: bool) {
        self.set_charging_crossbow(is_charging);
    }

    fn is_charging_crossbow(&self) -> bool {
        self.is_charging_crossbow()
    }
}
