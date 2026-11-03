use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI8, AtomicI32, Ordering::Relaxed},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityStatus;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tracked_data;
use pumpkin_inventory::Inventory;
use pumpkin_inventory::inventory::{SimpleInventory, sync_read_items_from_nbt};
use pumpkin_inventory::mount_screen_handler::MountScreenHandler;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::set_actor_data::{SyncedActorDataList, entity_data_flag};
use pumpkin_protocol::bedrock::server::actor_event::ActorEventID;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::entity::ageable::AgeableMob;
use crate::entity::ai::target_predicate::TargetPredicate;
use crate::entity::living::prevents_equipment_drop;
use crate::entity::mob::Mob;
use crate::entity::mob::equipment::EquipmentSlotContainer;
use crate::entity::passive::animal::Animal;
use crate::entity::player::Player;
use crate::entity::player::advancement::trigger::AdvancementTrigger;

pub const FLAG_TAME: i8 = 2;
pub const FLAG_BRED: i8 = 8;
pub const FLAG_EATING: i8 = 16;
pub const FLAG_STANDING: i8 = 32;
pub const FLAG_OPEN_MOUTH: i8 = 64;
pub const SOUND_VOLUME: f32 = 0.8;
pub const AMBIENT_SOUND_INTERVAL: i32 = 400;
const MOMMY_RANGE: f64 = 16.0;
const MOMMY_MIN_DISTANCE_SQ: f64 = 4.0;

pub struct AbstractHorseData {
    flags: AtomicI8,
    temper: AtomicI32,
    owner: AtomicCell<Option<Uuid>>,
    has_chest: AtomicBool,
    inventory: Mutex<Arc<SimpleInventory>>,
}

impl Default for AbstractHorseData {
    fn default() -> Self {
        Self {
            flags: AtomicI8::default(),
            temper: AtomicI32::default(),
            owner: AtomicCell::default(),
            has_chest: AtomicBool::default(),
            inventory: Mutex::new(Arc::new(SimpleInventory::new(0))),
        }
    }
}

pub trait AbstractHorse: Animal {
    fn horse_data(&self) -> &AbstractHorseData;

    fn get_max_temper(&self) -> i32 {
        100
    }

    fn get_angry_sound(&self) -> Option<Sound> {
        None
    }

    // TODO: rearing (`standIfPossible`) for the horses that can rear; llamas never do.
    fn make_mad(&self) {
        if !self.is_standing()
            && let Some(sound) = self.get_angry_sound()
        {
            self.make_sound(sound);
        }
    }

    fn tame_with_name(&self, player: &Arc<Player>) {
        self.set_owner(Some(player.gameprofile.id));
        self.set_tamed(true);
        let entity = self.get_entity();
        player.trigger_advancement(AdvancementTrigger::TameAnimal {
            entity_type: format!("minecraft:{}", entity.entity_type.resource_name),
        });
        entity.world.load().send_entity_status(
            entity,
            EntityStatus::TamingSucceeded,
            ActorEventID::TamingSucceeded,
        );
    }

    fn get_flag(&self, flag: i8) -> bool {
        self.horse_data().flags.load(Relaxed) & flag != 0
    }

    fn set_flag(&self, flag: i8, value: bool) {
        let flags = &self.horse_data().flags;
        let new_flags = if value {
            flags.fetch_or(flag, Relaxed) | flag
        } else {
            flags.fetch_and(!flag, Relaxed) & !flag
        };
        self.get_entity()
            .set_synced_data(tracked_data::abstract_horse::DATA_ID_FLAGS, new_flags);
    }

    fn set_tamed(&self, tamed: bool) {
        self.set_flag(FLAG_TAME, tamed);
        let entity = self.get_entity();
        entity.set_bedrock_flag(entity_data_flag::TAMED, tamed);
        entity.send_bedrock_actor_data(&self.bedrock_container_data());
    }

    fn is_bred(&self) -> bool {
        self.get_flag(FLAG_BRED)
    }

    fn set_bred(&self, bred: bool) {
        self.set_flag(FLAG_BRED, bred);
    }

    fn is_eating(&self) -> bool {
        self.get_flag(FLAG_EATING)
    }

    fn set_eating(&self, eating: bool) {
        self.set_flag(FLAG_EATING, eating);
    }

    fn is_standing(&self) -> bool {
        self.get_flag(FLAG_STANDING)
    }

    fn get_temper(&self) -> i32 {
        self.horse_data().temper.load(Relaxed)
    }

    fn set_temper(&self, temper: i32) {
        self.horse_data().temper.store(temper, Relaxed);
    }

    fn modify_temper(&self, amount: i32) -> i32 {
        let temper = (self.get_temper() + amount).clamp(0, self.get_max_temper());
        self.set_temper(temper);
        temper
    }

    fn get_owner(&self) -> Option<Uuid> {
        self.horse_data().owner.load()
    }

    fn set_owner(&self, owner: Option<Uuid>) {
        self.horse_data().owner.store(owner);
    }

    fn has_chest(&self) -> bool {
        self.horse_data().has_chest.load(Relaxed)
    }

    fn set_chest(&self, has_chest: bool) {
        self.horse_data().has_chest.store(has_chest, Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            tracked_data::abstract_chested_horse::DATA_ID_CHEST,
            has_chest,
        );
        entity.set_bedrock_flag(entity_data_flag::CHESTED, has_chest);
        entity.send_bedrock_actor_data(&self.bedrock_container_data());
    }

    fn get_inventory_columns(&self) -> i32 {
        0
    }

    /// The Java mount screen slot Bedrock shows as its one equipment slot.
    fn bedrock_equip_slot(&self) -> usize;

    /// The `UpdateEquip` slot list Bedrock uses to draw the equipment slot.
    fn bedrock_equip_slots(&self) -> NbtCompound;

    /// The actor data Bedrock reads to size and open the mount inventory.
    fn bedrock_container_data(&self) -> SyncedActorDataList;

    fn inventory(&self) -> Arc<SimpleInventory> {
        self.horse_data()
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Resizes the inventory for the current columns, keeping what fits.
    fn create_inventory(&self) {
        let size = MountScreenHandler::get_inventory_size(
            usize::try_from(self.get_inventory_columns()).unwrap_or(0),
        );
        let inventory = SimpleInventory::new(size);
        let mut current = self
            .horse_data()
            .inventory
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for slot in 0..size.min(current.size()) {
            inventory.set_stack(slot, current.get_stack(slot));
        }
        *current = Arc::new(inventory);
    }

    fn open_custom_inventory_screen(&self, player: &Arc<Player>) {
        let entity = self.get_entity();
        if (entity.has_passengers() && !entity.has_passenger(player.entity_id()))
            || !self.is_tamed()
        {
            return;
        }
        let Some(mount) = entity.world.load().get_entity_by_id(entity.entity_id) else {
            return;
        };
        let mount = Arc::downgrade(&mount);
        let inventory = self.inventory();
        let columns = self.get_inventory_columns();
        let bedrock_equip_slot = self.bedrock_equip_slot();
        player.open_mount_screen(
            columns,
            entity.entity_id,
            bedrock_equip_slot,
            self.bedrock_equip_slots(),
            |sync_id, player_inventory| {
                let mut handler = MountScreenHandler::new(
                    sync_id,
                    player_inventory,
                    inventory.clone(),
                    Arc::new(EquipmentSlotContainer::new(
                        mount.clone(),
                        EquipmentSlot::SADDLE,
                    )),
                    Arc::new(EquipmentSlotContainer::new(
                        mount.clone(),
                        EquipmentSlot::BODY,
                    )),
                    usize::try_from(columns).unwrap_or(0),
                );
                handler.bedrock_equip_slot = Some(bedrock_equip_slot);
                handler.validity_check = Some(Box::new(move |player| {
                    let Some(player) = player.as_any().downcast_ref::<Player>() else {
                        return false;
                    };
                    mount.upgrade().is_some_and(|mount| {
                        mount.get_entity().is_alive()
                            && mount
                                .get_mob()
                                .and_then(|mob| mob.as_abstract_horse())
                                .is_some_and(|horse| Arc::ptr_eq(&horse.inventory(), &inventory))
                            && player.can_interact_with_entity(mount.as_ref(), 4.0)
                    })
                }));
                Arc::new(Mutex::new(handler))
            },
        );
    }

    fn drop_horse_inventory(&self) {
        let entity = self.get_entity();
        let inventory = self.inventory();
        for slot in 0..inventory.size() {
            let stack = inventory.get_stack(slot);
            if !stack.is_empty() && !prevents_equipment_drop(&stack) {
                entity.spawn_at_location(stack);
            }
        }
        if self.has_chest() {
            entity.spawn_at_location(ItemStack::new(1, &Item::CHEST));
            self.set_chest(false);
        }
    }

    fn init_horse_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(
            tracked_data::abstract_horse::DATA_ID_FLAGS,
            self.horse_data().flags.load(Relaxed),
        );
        entity.set_synced_data(
            tracked_data::abstract_chested_horse::DATA_ID_CHEST,
            self.has_chest(),
        );
    }

    fn write_horse_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("EatingHaystack", self.is_eating());
        nbt.put_bool("Bred", self.is_bred());
        nbt.put_int("Temper", self.get_temper());
        nbt.put_bool("Tame", self.get_flag(FLAG_TAME));
        if let Some(owner) = self.get_owner() {
            nbt.put_uuid("Owner", owner);
        }
    }

    fn read_horse_nbt(&self, nbt: &NbtCompound) {
        self.set_eating(nbt.get_bool("EatingHaystack").unwrap_or(false));
        self.set_bred(nbt.get_bool("Bred").unwrap_or(false));
        self.set_temper(nbt.get_int("Temper").unwrap_or(0));
        self.set_tamed(nbt.get_bool("Tame").unwrap_or(false));
        self.set_owner(nbt.get_uuid("Owner"));
    }

    fn write_chested_horse_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("ChestedHorse", self.has_chest());
        if self.has_chest() {
            self.inventory().write_inventory_nbt(nbt, true);
        }
    }

    fn read_chested_horse_nbt(&self, nbt: &NbtCompound) {
        self.set_chest(nbt.get_bool("ChestedHorse").unwrap_or(false));
        self.create_inventory();
        if self.has_chest() {
            let inventory = self.inventory();
            sync_read_items_from_nbt(
                nbt,
                &mut inventory
                    .stacks
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
        }
    }
}

pub fn generate_max_health(mut next_int: impl FnMut(i32) -> i32) -> f32 {
    15.0 + next_int(8) as f32 + next_int(9) as f32
}

pub fn generate_jump_strength(mut probability: impl FnMut() -> f64) -> f64 {
    f64::from(0.4f32) + probability() * 0.2 + probability() * 0.2 + probability() * 0.2
}

pub fn generate_speed(mut probability: impl FnMut() -> f64) -> f64 {
    (f64::from(0.45f32) + probability() * 0.3 + probability() * 0.3 + probability() * 0.3) * 0.25
}

/// Fall damage for horse-like mobs: a landing sound and no hurt sound.
pub fn cause_fall_damage(
    mob: &dyn Mob,
    caller: &dyn EntityBase,
    fall_distance: f32,
    damage_modifier: f32,
) -> bool {
    if fall_distance > 1.0 {
        let sound = if mob.as_ageable().is_some_and(AgeableMob::is_baby) {
            Sound::EntityBabyHorseLand
        } else {
            Sound::EntityHorseLand
        };
        caller.play_entity_sound(sound, 0.4, 1.0);
    }
    let damage = mob.calculate_fall_damage(fall_distance, damage_modifier);
    if damage <= 0 {
        return false;
    }
    let living = &mob.get_mob_entity().living_entity;
    living.damage(caller, damage as f32, DamageType::FALL);
    living.propagate_fall_to_passengers(fall_distance, damage_modifier);
    living.play_block_fall_sound(caller);
    true
}

/// The path is dropped on purpose; only the navigator target is kept, so the foal heads for
/// its parent on the next path recompute.
pub fn follow_mommy<H: AbstractHorse + AgeableMob>(horse: &H) {
    if !horse.is_bred() || !horse.is_baby() || horse.is_eating() {
        return;
    }
    let entity = horse.get_entity();
    let world = entity.world.load();
    let pos = entity.pos.load();
    let mommy_targeting = TargetPredicate::create_non_attackable()
        .set_base_max_distance(MOMMY_RANGE)
        .ignore_visibility();
    let area = entity
        .bounding_box
        .load()
        .expand(MOMMY_RANGE, MOMMY_RANGE, MOMMY_RANGE);
    let mommy = world
        .get_entities_at_box(&area)
        .into_iter()
        .filter(|candidate| {
            candidate
                .get_mob()
                .and_then(Mob::as_abstract_horse)
                .is_some_and(AbstractHorse::is_bred)
                && mommy_targeting.test(&world, Some(horse), candidate.as_ref())
        })
        .min_by(|a, b| {
            let a = a.get_entity().pos.load().squared_distance_to_vec(&pos);
            let b = b.get_entity().pos.load().squared_distance_to_vec(&pos);
            a.total_cmp(&b)
        });
    if let Some(mommy) = mommy {
        let mommy_pos = mommy.get_entity().pos.load();
        if pos.squared_distance_to_vec(&mommy_pos) > MOMMY_MIN_DISTANCE_SQ {
            let mob_entity = horse.get_mob_entity();
            mob_entity
                .steered()
                .navigator()
                .create_path(&mob_entity.living_entity, mommy_pos, 0);
        }
    }
}

pub fn can_dispenser_equip_into_slot(mob: &dyn Mob, slot: &EquipmentSlot) -> bool {
    ((*slot == EquipmentSlot::BODY || *slot == EquipmentSlot::SADDLE) && mob.is_tamed())
        || mob.get_mob_entity().can_pick_up_loot()
}
