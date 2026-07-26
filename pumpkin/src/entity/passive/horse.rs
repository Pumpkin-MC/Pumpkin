use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;

use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase, NBTStorage,
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};

const TEMPT_ITEMS: &[&Item] = &[
    &Item::WHEAT,
    &Item::SUGAR,
    &Item::APPLE,
    &Item::GOLDEN_CARROT,
    &Item::GOLDEN_APPLE,
    &Item::HAY_BLOCK,
];

/// Horse — vanilla AbstractHorse goals (RunAroundLikeCrazy / rear TODO).
///
/// Decompile: Float+MountPanic+Tempt (addBehaviourGoals) + Breed/FollowParent/Stroll.
pub struct HorseEntity {
    pub mob_entity: MobEntity,
    /// Vanilla `AbstractHorse` FLAG_TAME (DATA_ID_FLAGS bit 2).
    pub tamed: std::sync::atomic::AtomicBool,
    /// Vanilla temper, 0..=100 (`AbstractHorse.getMaxTemper`).
    pub temper: std::sync::atomic::AtomicI32,
}

impl HorseEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let horse = Self {
            mob_entity,
            tamed: std::sync::atomic::AtomicBool::new(false),
            temper: std::sync::atomic::AtomicI32::new(0),
        };
        let mob_arc = Arc::new(horse);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            // addBehaviourGoals
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.2)); // MountPanic
            // Vanilla priority 1: untamed horses bolt and buck their rider.
            goal_selector.add_goal(
                1,
                Box::new(
                    crate::entity::ai::goal::run_around_like_crazy::RunAroundLikeCrazyGoal::new(
                        1.2,
                    ),
                ),
            );
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.0)));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(0.7)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }
}

impl HorseEntity {
    /// Vanilla `AbstractHorse.getMaxTemper` (AbstractHorse.java:381).
    pub const MAX_TEMPER: i32 = 100;
    /// Vanilla `FLAG_TAME` (AbstractHorse.java:108).
    const FLAG_TAME: u8 = 2;

    pub fn set_tamed(&self, tamed: bool) {
        self.tamed
            .store(tamed, std::sync::atomic::Ordering::Relaxed);
        self.sync_horse_flags();
    }

    /// Vanilla `modifyTemper`: clamp into 0..=maxTemper.
    pub fn modify_temper(&self, amount: i32) -> i32 {
        use std::sync::atomic::Ordering;
        let new_value = (self.temper.load(Ordering::Relaxed) + amount).clamp(0, Self::MAX_TEMPER);
        self.temper.store(new_value, Ordering::Relaxed);
        new_value
    }

    /// Sends the `DATA_ID_FLAGS` byte (26.2 index via `TrackedData::FLAGS_ID`).
    fn sync_horse_flags(&self) {
        use pumpkin_data::meta_data_type::MetaDataType;
        use pumpkin_protocol::java::client::play::Metadata;
        let mut flags = 0u8;
        if self.tamed.load(std::sync::atomic::Ordering::Relaxed) {
            flags |= Self::FLAG_TAME;
        }
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::TrackedData::FLAGS_ID,
                MetaDataType::BYTE,
                flags,
            )],
            None,
        );
    }

    /// Vanilla `AbstractHorse.handleEating` food table
    /// (AbstractHorse.java:412-451). Returns true when the food was used.
    async fn handle_eating(&self, item: &pumpkin_data::item::Item) -> bool {
        use pumpkin_data::item::Item;
        let (heal, temper) = if item == &Item::WHEAT {
            (2.0f32, 3)
        } else if item == &Item::SUGAR {
            (1.0, 3)
        } else if item == &Item::HAY_BLOCK {
            (20.0, 0)
        } else if item == &Item::APPLE || item == &Item::RED_MUSHROOM || item == &Item::CARROT {
            (3.0, 3)
        } else if item == &Item::GOLDEN_CARROT {
            (4.0, 5)
        } else if item == &Item::GOLDEN_APPLE || item == &Item::ENCHANTED_GOLDEN_APPLE {
            (10.0, 10)
        } else {
            return false;
        };

        use std::sync::atomic::Ordering;
        let living = &self.mob_entity.living_entity;
        let mut used = false;
        if living.health.load() < living.get_max_health() && heal > 0.0 {
            living.heal(heal);
            used = true;
        }
        // Vanilla: temper only counts for untamed horses under max temper.
        let tamed = self.tamed.load(Ordering::Relaxed);
        if temper > 0 && !(!used && tamed) && self.temper.load(Ordering::Relaxed) < Self::MAX_TEMPER
        {
            self.modify_temper(temper);
            used = true;
        }
        if used {
            let entity = &living.entity;
            entity.world.load().play_sound(
                pumpkin_data::sound::Sound::EntityHorseEat,
                pumpkin_data::sound::SoundCategory::Neutral,
                &entity.pos.load(),
            );
        }
        used
    }

    async fn is_saddled(&self) -> bool {
        use pumpkin_data::data_component_impl::EquipmentSlot;
        let equipment = self.mob_entity.living_entity.entity_equipment.lock().await;
        let saddle = equipment.get(&EquipmentSlot::SADDLE);
        let stack = saddle.lock().await;
        !stack.is_empty()
    }
}

impl NBTStorage for HorseEntity {
    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            use std::sync::atomic::Ordering;
            self.get_mob_entity().living_entity.write_nbt(nbt).await;
            nbt.put_bool("Tame", self.tamed.load(Ordering::Relaxed));
            nbt.put_int("Temper", self.temper.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(
        &'a self,
        nbt: &'a pumpkin_nbt::compound::NbtCompound,
    ) -> crate::entity::NbtFuture<'a, ()> {
        Box::pin(async move {
            use std::sync::atomic::Ordering;
            self.get_mob_entity()
                .living_entity
                .read_nbt_non_mut(nbt)
                .await;
            self.tamed
                .store(nbt.get_bool("Tame").unwrap_or(false), Ordering::Relaxed);
            self.temper
                .store(nbt.get_int("Temper").unwrap_or(0), Ordering::Relaxed);
        })
    }
}

impl Mob for HorseEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> crate::entity::EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            // Base Mob::mob_init_data_tracker body (baby flag), then the
            // horse-specific flag byte.
            if self.is_mob_baby() {
                self.get_entity().send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
            self.sync_horse_flags();
        })
    }

    /// Vanilla `AbstractHorse.mobInteract` (AbstractHorse.java:616-637).
    fn mob_interact<'a>(
        &'a self,
        player: &'a std::sync::Arc<crate::entity::player::Player>,
        item_stack: &'a mut pumpkin_data::item_stack::ItemStack,
    ) -> crate::entity::EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            use pumpkin_data::item::Item;
            use std::sync::atomic::Ordering;
            let entity = &self.mob_entity.living_entity.entity;

            // isVehicle() || isBaby() → default animal interaction.
            let has_rider = { !entity.passengers.lock().await.is_empty() };
            if has_rider || entity.age.load(Ordering::Relaxed) < 0 {
                return self.mob_entity.mob_interact(player, item_stack).await;
            }

            let tamed = self.tamed.load(Ordering::Relaxed);
            // Tamed + sneaking opens the horse inventory (screen TODO).
            if tamed && player.get_entity().is_sneaking() {
                return true;
            }

            if !item_stack.is_empty() {
                // fedFood: horse food consumes on use.
                if self.handle_eating(item_stack.item).await {
                    if player.gamemode.load() != pumpkin_util::GameMode::Creative {
                        item_stack.decrement(1);
                    }
                    return true;
                }
                // Saddle (equippable on tamed horses).
                if item_stack.item == &Item::SADDLE && tamed && !self.is_saddled().await {
                    use pumpkin_data::data_component_impl::EquipmentSlot;
                    let saddle_stack = pumpkin_data::item_stack::ItemStack::new(1, &Item::SADDLE);
                    {
                        let mut equipment =
                            self.mob_entity.living_entity.entity_equipment.lock().await;
                        equipment
                            .put(&EquipmentSlot::SADDLE, saddle_stack.clone())
                            .await;
                    }
                    self.mob_entity
                        .living_entity
                        .send_equipment_changes(&[(EquipmentSlot::SADDLE, saddle_stack)]);
                    entity.world.load().play_sound(
                        pumpkin_data::sound::Sound::EntityHorseSaddle,
                        pumpkin_data::sound::SoundCategory::Neutral,
                        &entity.pos.load(),
                    );
                    if player.gamemode.load() != pumpkin_util::GameMode::Creative {
                        item_stack.decrement(1);
                    }
                    return true;
                }
            }

            // doPlayerRide: align the rider to the horse, then mount.
            let world = entity.world.load_full();
            let Some(horse_arc) = world.get_entity_by_id(entity.entity_id) else {
                return false;
            };
            let Some(player_arc) = world.get_player_by_uuid(player.get_entity().entity_uuid) else {
                return false;
            };
            player
                .get_entity()
                .set_rotation(entity.yaw.load(), entity.pitch.load());
            entity
                .add_passenger(
                    horse_arc,
                    player_arc as std::sync::Arc<dyn crate::entity::EntityBase>,
                )
                .await;
            true
        })
    }
}
