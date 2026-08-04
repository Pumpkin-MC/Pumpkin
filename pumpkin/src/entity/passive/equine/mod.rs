//! Shared framework for the horse family (`AbstractHorse` in vanilla): Horse, Donkey, Mule,
//! `ZombieHorse`, `SkeletonHorse`. See `designs/equine-framework.md` for the design rationale.
//!
//! Phase 1 scope: taming, feeding, chest inventory, breeding attribute inheritance,
//! `randomizeAttributes`, sounds. Rider-controlled movement (`PlayerRideableJumping`,
//! mounted-input routing) is out of scope -- a horse can be saddled and mounted, but does
//! not yet respond to WASD/jump while ridden.
//!
//! Deliberate simplifications versus vanilla, noted here once rather than at every call site:
//! - Saddle/body-armor equipping is done directly in `abstract_horse_mob_interact` (mirroring
//!   `HappyGhastEntity::try_equip_harness`) rather than through a generic
//!   `ItemStack.interactLivingEntity` dispatch, which doesn't exist in Pumpkin yet.
//! - The chest inventory screen is opened via the existing generic-container screen handler
//!   (`create_generic_9x3`), not a ported `HorseInventoryMenu`/`ClientboundMountScreenOpenPacket`.
//!   Vanilla's real screen has dedicated saddle/armor slots plus the chest columns in one menu;
//!   here the saddle/armor stay equipment-slot-only (no slot in the chest GUI) and only the
//!   chest storage is shown. This is a real, working inventory (right-click+sneak on a tamed
//!   chested horse opens and persists chest contents) but is not pixel-identical to vanilla's
//!   menu layout -- flagged as a follow-up if the exact vanilla GUI is required.
//! - `standIfPossible`'s standing pose is set but never automatically cleared (vanilla ticks
//!   `standCounter` down in `aiStep`); cosmetic-only gap, out of scope without a per-species
//!   `mob_tick` port.

use std::sync::Arc;
use std::sync::atomic::{AtomicI32, AtomicU8, Ordering::Relaxed};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_inventory::generic_container_screen_handler::create_generic_9x3;
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    BoxFuture, InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_world::inventory::{Inventory, SimpleInventory};
use rand::RngExt;
use tokio::sync::Mutex as TokioMutex;
use uuid::Uuid;

use crate::entity::{
    EntityBase, EntityBaseFuture, mob::Mob, passive::animal::Animal, player::Player,
};

/// `AbstractHorse.java` `DATA_ID_FLAGS` bits.
///
/// `FLAG_TAME` (2) is intentionally not here -- `MobEntity::is_tamed`/`set_owner` is the single
/// source of truth for tame state (see the module doc comment on `AbstractHorse::is_tamed`).
pub const FLAG_BRED: u8 = 8;
pub const FLAG_EATING: u8 = 16;
pub const FLAG_STANDING: u8 = 32;
pub const FLAG_OPEN_MOUTH: u8 = 64;

/// Shared per-instance state mirroring vanilla `AbstractHorse`'s fields that aren't already
/// covered by `MobEntity` (owner/love-ticks) or `LivingEntity` (equipment for saddle/armor).
pub struct AbstractHorseData {
    pub flags: AtomicU8,
    pub temper: AtomicI32,
}

impl Default for AbstractHorseData {
    fn default() -> Self {
        Self {
            flags: AtomicU8::new(0),
            temper: AtomicI32::new(0),
        }
    }
}

impl AbstractHorseData {
    #[must_use]
    pub fn get_flag(&self, flag: u8) -> bool {
        self.flags.load(Relaxed) & flag != 0
    }

    pub fn set_flag(&self, flag: u8, value: bool) {
        if value {
            self.flags.fetch_or(flag, Relaxed);
        } else {
            self.flags.fetch_and(!flag, Relaxed);
        }
    }
}

/// Chest-capable species' extra state (`AbstractChestedHorse.java`).
pub struct ChestedHorseData {
    pub has_chest: std::sync::atomic::AtomicBool,
    pub inventory: TokioMutex<Arc<SimpleInventory>>,
}

impl Default for ChestedHorseData {
    fn default() -> Self {
        Self {
            has_chest: std::sync::atomic::AtomicBool::new(false),
            inventory: TokioMutex::new(Arc::new(SimpleInventory::new(0))),
        }
    }
}

// --- Pure attribute-roll helpers: `AbstractHorse.java:938-947,841-863` ---

/// `AbstractHorse.generateMaxHealth`: `15 + rnd(8) + rnd(9)`.
#[must_use]
pub fn generate_max_health(random: &mut impl RngExt) -> f64 {
    15.0 + f64::from(random.random_range(0..8)) + f64::from(random.random_range(0..9))
}

/// `AbstractHorse.generateJumpStrength`: `0.4 + 3x rnd(0..1)*0.2`.
#[must_use]
pub fn generate_jump_strength(random: &mut impl RngExt) -> f64 {
    0.4 + random.random_range(0.0..1.0) * 0.2
        + random.random_range(0.0..1.0) * 0.2
        + random.random_range(0.0..1.0) * 0.2
}

/// `AbstractHorse.generateSpeed`: `(0.45 + 3x rnd(0..1)*0.3) * 0.25`.
#[must_use]
pub fn generate_speed(random: &mut impl RngExt) -> f64 {
    (0.45
        + random.random_range(0.0..1.0) * 0.3
        + random.random_range(0.0..1.0) * 0.3
        + random.random_range(0.0..1.0) * 0.3)
        * 0.25
}

/// `ZombieHorse.generateZombieHorseJumpStrength`.
#[must_use]
pub fn generate_zombie_horse_jump_strength(random: &mut impl RngExt) -> f64 {
    0.5 + random.random_range(0.0..1.0) / 15.0
        + random.random_range(0.0..1.0) / 15.0
        + random.random_range(0.0..1.0) / 15.0
}

/// `ZombieHorse.generateZombieHorseSpeed`.
#[must_use]
pub fn generate_zombie_horse_speed(random: &mut impl RngExt) -> f64 {
    (9.0 + random.random_range(0.0..1.0)
        + random.random_range(0.0..1.0)
        + random.random_range(0.0..1.0))
        / 42.16
}

/// `AbstractHorse.createOffspringAttribute`.
#[must_use]
pub fn create_offspring_attribute(
    parent_a: f64,
    parent_b: f64,
    range_min: f64,
    range_max: f64,
    random: &mut impl RngExt,
) -> f64 {
    assert!(range_max > range_min, "Incorrect range for an attribute");

    let a = parent_a.clamp(range_min, range_max);
    let b = parent_b.clamp(range_min, range_max);
    let margin = 0.15 * (range_max - range_min);
    let range = (a - b).abs() + margin * 2.0;
    let average = f64::midpoint(a, b);
    let quality = (random.random_range(0.0..1.0)
        + random.random_range(0.0..1.0)
        + random.random_range(0.0..1.0))
        / 3.0
        - 0.5;
    let value = average + range * quality;

    if value > range_max {
        range_max - (value - range_max)
    } else if value < range_min {
        range_min + (range_min - value)
    } else {
        value
    }
}

struct HorseChestScreenFactory(Arc<dyn Inventory>);

impl ScreenHandlerFactory for HorseChestScreenFactory {
    fn create_screen_handler<'a>(
        &'a self,
        sync_id: u8,
        player_inventory: &'a Arc<PlayerInventory>,
        _player: &'a dyn InventoryPlayer,
    ) -> BoxFuture<'a, Option<SharedScreenHandler>> {
        Box::pin(async move {
            let handler = create_generic_9x3(sync_id, player_inventory, self.0.clone()).await;
            Some(Arc::new(TokioMutex::new(handler)) as SharedScreenHandler)
        })
    }

    fn get_display_name(&self) -> TextComponent {
        TextComponent::text("Horse")
    }
}

/// Mirrors vanilla `AbstractHorse` (Horse.java's superclass). Species implement this to get
/// taming, feeding, saddling, breeding-attribute-inheritance and `randomizeAttributes` for free.
///
/// Deliberately `Animal`-bound (not just `Mob`) since vanilla's `AbstractHorse` itself extends
/// `Animal`; this also means the required `Animal::is_food` impl species already carry for the
/// generic baby-growth path doubles as the horse-specific food-tag check here.
pub trait AbstractHorse: Animal {
    fn horse_data(&self) -> &AbstractHorseData;

    fn max_temper(&self) -> i32 {
        100
    }

    fn can_perform_rearing(&self) -> bool {
        true
    }

    fn can_fall_in_love(&self) -> bool {
        true
    }

    fn eating_sound(&self) -> Option<Sound> {
        None
    }

    fn angry_sound(&self) -> Option<Sound> {
        None
    }

    fn saddle_sound(&self) -> Sound {
        Sound::EntityHorseSaddle
    }

    /// `randomizeAttributes` -- a no-op default; species override and call this from their
    /// `new()` (before any NBT read overwrites the rolled base values on load, see
    /// `write_horse_attributes_nbt`/`read_horse_attributes_nbt`).
    fn randomize_attributes(&self, _random: &mut impl RngExt)
    where
        Self: Sized,
    {
    }

    fn is_baby(&self) -> bool {
        self.get_entity().age.load(Relaxed) < 0
    }

    /// `AbstractHorse.isTamed`. `FLAG_TAME` is folded into `MobEntity::owner` -- every vanilla
    /// code path sets the owner and the tame flag together, so `owner.is_some()` is equivalent
    /// and avoids a duplicate bool that could desync from the owner reference.
    fn is_tamed(&self) -> bool {
        self.get_mob_entity().is_tamed()
    }

    /// `AbstractHorse.tameWithName`, minus the advancement trigger (no
    /// `CriteriaTriggers.TAME_ANIMAL` equivalent wired up yet).
    fn set_tamed(&self, owner: Uuid) {
        self.get_mob_entity().set_owner(owner);
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load() + Vector3::new(0.0, f64::from(entity.height()), 0.0);
        world.spawn_particle(pos, Vector3::new(0.5, 0.5, 0.5), 1.0, 7, Particle::Heart);
    }

    fn can_use_saddle_slot(&self) -> bool {
        self.get_entity().is_alive() && !self.is_baby() && self.is_tamed()
    }

    fn is_saddled(&self) -> BoxFuture<'_, bool> {
        Box::pin(async move {
            let equipment = self
                .get_mob_entity()
                .living_entity
                .entity_equipment
                .lock()
                .await;
            let stack = equipment.get(&EquipmentSlot::SADDLE);
            let stack = stack.lock().await;
            !stack.is_empty()
        })
    }

    fn equip_saddle<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let new_stack = item_stack.split_unless_creative(player.gamemode.load(), 1);
            {
                let mut equipment = self
                    .get_mob_entity()
                    .living_entity
                    .entity_equipment
                    .lock()
                    .await;
                equipment
                    .put(&EquipmentSlot::SADDLE, new_stack.clone())
                    .await
            };
            self.get_mob_entity()
                .living_entity
                .send_equipment_changes(&[(EquipmentSlot::SADDLE, new_stack)]);

            let entity = self.get_entity();
            let world = entity.world.load();
            world.play_sound(
                self.saddle_sound(),
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
        })
    }

    /// `AbstractHorse.makeMad`.
    fn make_mad(&self) {
        let data = self.horse_data();
        if !data.get_flag(FLAG_STANDING) {
            self.stand_if_possible();
            if let Some(sound) = self.angry_sound() {
                let entity = self.get_entity();
                let world = entity.world.load();
                world.play_sound(sound, SoundCategory::Neutral, &entity.pos.load());
            }
        }
    }

    /// `AbstractHorse.standIfPossible`. See the module doc comment: the standing flag is set
    /// here but not auto-cleared (no `aiStep`-driven `standCounter` tick-down ported).
    fn stand_if_possible(&self) {
        if self.can_perform_rearing() {
            self.horse_data().set_flag(FLAG_EATING, false);
            self.horse_data().set_flag(FLAG_STANDING, true);
        }
    }

    /// `AbstractHorse.doPlayerRide`, using `Entity::add_passenger` the same way
    /// `HappyGhastEntity::try_mount` does (see that file for the established mounting pattern).
    fn do_player_ride<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.horse_data().set_flag(FLAG_EATING, false);
            self.horse_data().set_flag(FLAG_STANDING, false);

            if player.get_entity().has_vehicle().await {
                return;
            }

            let entity = self.get_entity();
            let world = entity.world.load();
            let Some(vehicle) = world.get_entity_by_id(entity.entity_id) else {
                return;
            };
            let Some(passenger) = world.get_player_by_id(player.entity_id()) else {
                return;
            };

            entity
                .add_passenger(vehicle, passenger as Arc<dyn EntityBase>)
                .await;
        })
    }

    /// `AbstractHorse.handleEating`: the default wheat/sugar/hay/apple/mushroom/carrot/golden
    /// carrot/golden apple table (`AbstractHorse.java:423-493`). Species with a different food
    /// table (Llama, out of scope here) override this instead of extending it.
    fn handle_eating<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let id = item_stack.item.id;
            let (heal, age_up_seconds, temper): (f32, i32, i32) = if id == Item::WHEAT.id {
                (2.0, 20, 3)
            } else if id == Item::SUGAR.id {
                (1.0, 30, 3)
            } else if id == Item::HAY_BLOCK.id {
                (20.0, 180, 0)
            } else if id == Item::APPLE.id {
                (3.0, 60, 3)
            } else if id == Item::RED_MUSHROOM.id {
                (3.0, 0, 3)
            } else if id == Item::CARROT.id {
                (3.0, 60, 3)
            } else if id == Item::GOLDEN_CARROT.id {
                (4.0, 60, 5)
            } else if id == Item::GOLDEN_APPLE.id || id == Item::ENCHANTED_GOLDEN_APPLE.id {
                (10.0, 240, 10)
            } else {
                return false;
            };

            let mut item_used = false;
            let is_golden = id == Item::GOLDEN_CARROT.id
                || id == Item::GOLDEN_APPLE.id
                || id == Item::ENCHANTED_GOLDEN_APPLE.id;

            let mob_entity = self.get_mob_entity();
            if is_golden
                && self.can_fall_in_love()
                && self.is_tamed()
                && !self.is_baby()
                && !mob_entity.is_in_love()
            {
                item_used = true;
                mob_entity.set_love_ticks(600, Some(player.gameprofile.id));
            }

            let living = &mob_entity.living_entity;
            if living.health.load() < living.get_max_health() && heal > 0.0 {
                living.heal(heal);
                item_used = true;
            }

            if self.is_baby() && age_up_seconds > 0 {
                let entity = self.get_entity();
                let world = entity.world.load();
                let pos = entity.pos.load();
                world.spawn_particle(
                    pos + Vector3::new(0.0, f64::from(entity.height()) * 0.5, 0.0),
                    Vector3::new(0.5, 0.5, 0.5),
                    1.0,
                    7,
                    Particle::HappyVillager,
                );
                let new_age = (entity.age.load(Relaxed) + age_up_seconds * 20).min(0);
                entity.age.store(new_age, Relaxed);
                item_used = true;
            }

            if temper > 0
                && (item_used || !self.is_tamed())
                && self.horse_data().temper.load(Relaxed) < self.max_temper()
            {
                let new_temper =
                    (self.horse_data().temper.load(Relaxed) + temper).clamp(0, self.max_temper());
                self.horse_data().temper.store(new_temper, Relaxed);
                item_used = true;
            }

            if item_used {
                self.horse_data().set_flag(FLAG_EATING, true);
            }

            item_used
        })
    }

    /// `AbstractHorse.fedFood`.
    fn fed_food<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let ate = self.handle_eating(player, item_stack).await;
            if ate {
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            }
            ate
        })
    }

    /// Whether this species can currently show an inventory screen. Non-chested horses have
    /// nothing to show (saddle/armor are equipment-slot-only here, see the module doc comment),
    /// so the default is a no-op; `AbstractChestedHorse::chested_mob_interact` overrides this
    /// path with the real chest-inventory screen.
    fn open_custom_inventory_screen<'a>(
        &'a self,
        _player: &'a Arc<Player>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    /// `AbstractHorse.mobInteract` (`AbstractHorse.java:644-669`), shared by Horse, `ZombieHorse`
    /// and (post-tame-gate) `SkeletonHorse` -- the species' own `mobInteract` overrides in
    /// vanilla reduce to this exact dispatch once their food/mad early-returns are inlined (see
    /// the per-species files for the trace showing why).
    fn abstract_horse_mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool>
    where
        Self: Sized,
    {
        Box::pin(async move {
            let mob_entity = self.get_mob_entity();
            let entity = &mob_entity.living_entity.entity;
            let is_vehicle = !entity.passengers.lock().await.is_empty();

            if is_vehicle || self.is_baby() {
                return mob_entity
                    .mob_interact(player, item_stack, self.can_be_leashed())
                    .await;
            }

            if self.is_tamed() && player.get_entity().is_sneaking() {
                self.open_custom_inventory_screen(player).await;
                return true;
            }

            if !item_stack.is_empty() {
                if self.is_food(item_stack) {
                    return self.fed_food(player, item_stack).await;
                }

                if !self.is_tamed() {
                    self.make_mad();
                    return true;
                }

                if item_stack.item.id == Item::SADDLE.id
                    && !self.is_saddled().await
                    && self.can_use_saddle_slot()
                {
                    self.equip_saddle(player, item_stack).await;
                    return true;
                }
            }

            self.do_player_ride(player).await;
            true
        })
    }

    /// `AbstractHorse.addAdditionalSaveData` (`AbstractHorse.java:788-793`, minus `Owner`/`Tame`
    /// which `MobEntity`'s generic owner persistence already handles) plus the randomized
    /// attribute base values (see the module doc comment on `randomize_attributes`).
    fn write_horse_nbt(&self, nbt: &mut NbtCompound) {
        let data = self.horse_data();
        nbt.put_bool("EatingHaystack", data.get_flag(FLAG_EATING));
        nbt.put_bool("Bred", data.get_flag(FLAG_BRED));
        nbt.put_int("Temper", data.temper.load(Relaxed));

        let attributes = self
            .get_mob_entity()
            .living_entity
            .attributes
            .read()
            .unwrap();
        if let Some(a) = attributes.get(&pumpkin_data::attributes::Attributes::MAX_HEALTH.id) {
            nbt.put_double("PumpkinHorseMaxHealth", a.base_value);
        }
        if let Some(a) = attributes.get(&pumpkin_data::attributes::Attributes::MOVEMENT_SPEED.id) {
            nbt.put_double("PumpkinHorseSpeed", a.base_value);
        }
        if let Some(a) = attributes.get(&pumpkin_data::attributes::Attributes::JUMP_STRENGTH.id) {
            nbt.put_double("PumpkinHorseJumpStrength", a.base_value);
        }
    }

    fn read_horse_nbt(&self, nbt: &NbtCompound) {
        let data = self.horse_data();
        data.set_flag(FLAG_EATING, nbt.get_bool("EatingHaystack").unwrap_or(false));
        data.set_flag(FLAG_BRED, nbt.get_bool("Bred").unwrap_or(false));
        data.temper
            .store(nbt.get_int("Temper").unwrap_or(0), Relaxed);

        let mut attributes = self
            .get_mob_entity()
            .living_entity
            .attributes
            .write()
            .unwrap();
        if let Some(v) = nbt.get_double("PumpkinHorseMaxHealth")
            && let Some(a) =
                attributes.get_mut(&pumpkin_data::attributes::Attributes::MAX_HEALTH.id)
        {
            a.base_value = v;
            a.dirty.store(true, Relaxed);
        }
        if let Some(v) = nbt.get_double("PumpkinHorseSpeed")
            && let Some(a) =
                attributes.get_mut(&pumpkin_data::attributes::Attributes::MOVEMENT_SPEED.id)
        {
            a.base_value = v;
            a.dirty.store(true, Relaxed);
        }
        if let Some(v) = nbt.get_double("PumpkinHorseJumpStrength")
            && let Some(a) =
                attributes.get_mut(&pumpkin_data::attributes::Attributes::JUMP_STRENGTH.id)
        {
            a.base_value = v;
            a.dirty.store(true, Relaxed);
        }
        drop(attributes);

        let max_health = self.get_mob_entity().living_entity.get_max_health();
        if self.get_mob_entity().living_entity.health.load() > max_health {
            self.get_mob_entity().living_entity.health.store(max_health);
        }
    }
}

/// `AbstractHorse.setOffspringAttribute` (`AbstractHorse.java:835-840`).
///
/// Applied to a freshly spawned baby's attribute base value. `range_min`/`range_max` are the
/// species' randomize-attribute bounds (the same bounds `generate_max_health`/`generate_speed`/
/// `generate_jump_strength` roll within).
pub fn apply_offspring_attribute(
    baby: &dyn Mob,
    attribute: &'static pumpkin_data::attributes::Attributes,
    parent_a_value: f64,
    parent_b_value: f64,
    range_min: f64,
    range_max: f64,
    random: &mut impl RngExt,
) {
    let new_value =
        create_offspring_attribute(parent_a_value, parent_b_value, range_min, range_max, random);
    let mut attributes = baby
        .get_mob_entity()
        .living_entity
        .attributes
        .write()
        .unwrap();
    if let Some(a) = attributes.get_mut(&attribute.id) {
        a.base_value = new_value;
        a.dirty.store(true, Relaxed);
    }
}

/// Mirrors vanilla `AbstractChestedHorse` (Donkey/Mule; Llama is out of scope for this task).
pub trait AbstractChestedHorse: AbstractHorse {
    fn chested_data(&self) -> &ChestedHorseData;

    fn has_chest(&self) -> bool {
        self.chested_data().has_chest.load(Relaxed)
    }

    fn get_inventory_columns(&self) -> u8 {
        if self.has_chest() { 5 } else { 0 }
    }

    fn play_chest_equips_sound(&self) {
        let entity = self.get_entity();
        let world = entity.world.load();
        world.play_sound(
            Sound::EntityDonkeyChest,
            SoundCategory::Neutral,
            &entity.pos.load(),
        );
    }

    /// `AbstractChestedHorse.setChest` + `createInventory`: replaces the backing inventory,
    /// preserving any existing stacks in the overlapping slot range.
    fn set_chest(&self, value: bool) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.chested_data().has_chest.store(value, Relaxed);
            let new_size = if value {
                usize::from(self.get_inventory_columns()) * 3
            } else {
                0
            };

            let mut slot = self.chested_data().inventory.lock().await;
            let old = slot.clone();
            let new_inventory = Arc::new(SimpleInventory::new(new_size));
            let max = old.size().min(new_inventory.size());
            for i in 0..max {
                let stack = old.get_stack(i).await;
                let stack = stack.lock().await;
                if !stack.is_empty() {
                    new_inventory.set_stack(i, stack.clone()).await;
                }
            }
            *slot = new_inventory;
        })
    }

    fn open_chest_inventory<'a>(&'a self, player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let inventory = self.chested_data().inventory.lock().await.clone();
            player
                .open_handled_screen(
                    &HorseChestScreenFactory(inventory as Arc<dyn Inventory>),
                    None,
                )
                .await;
        })
    }

    /// `AbstractChestedHorse.mobInteract` (`AbstractChestedHorse.java:143-167`).
    fn chested_mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool>
    where
        Self: Sized,
    {
        Box::pin(async move {
            let mob_entity = self.get_mob_entity();
            let entity = &mob_entity.living_entity.entity;
            let is_vehicle = !entity.passengers.lock().await.is_empty();
            let should_open_inventory =
                !self.is_baby() && self.is_tamed() && player.get_entity().is_sneaking();
            let baby_with_dandelion =
                self.is_baby() && item_stack.item.id == Item::GOLDEN_DANDELION.id;

            if is_vehicle || should_open_inventory || baby_with_dandelion {
                if should_open_inventory && !is_vehicle {
                    self.open_chest_inventory(player).await;
                    return true;
                }
                return mob_entity
                    .mob_interact(player, item_stack, self.can_be_leashed())
                    .await;
            }

            if !item_stack.is_empty() {
                if self.is_food(item_stack) {
                    return self.fed_food(player, item_stack).await;
                }

                if !self.is_tamed() {
                    self.make_mad();
                    return true;
                }

                if !self.has_chest() && item_stack.item.id == Item::CHEST.id {
                    self.set_chest(true).await;
                    self.play_chest_equips_sound();
                    item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                    return true;
                }
            }

            self.abstract_horse_mob_interact(player, item_stack).await
        })
    }

    fn write_chested_horse_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("ChestedHorse", self.has_chest());
    }

    fn read_chested_horse_nbt<'a>(&'a self, nbt: &'a NbtCompound) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let has_chest = nbt.get_bool("ChestedHorse").unwrap_or(false);
            self.set_chest(has_chest).await;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        create_offspring_attribute, generate_jump_strength, generate_max_health, generate_speed,
        generate_zombie_horse_jump_strength, generate_zombie_horse_speed,
    };
    use rand::rng;

    /// `AbstractHorse.MIN_HEALTH`/`MAX_HEALTH`: `generateMaxHealth(i -> 0)` and
    /// `generateMaxHealth(i -> i - 1)`.
    #[test]
    fn generate_max_health_stays_within_vanilla_bounds() {
        let mut random = rng();
        for _ in 0..1000 {
            let health = generate_max_health(&mut random);
            assert!((15.0..=30.0).contains(&health), "{health} out of range");
        }
    }

    #[test]
    fn generate_jump_strength_stays_within_vanilla_bounds() {
        let mut random = rng();
        for _ in 0..1000 {
            let jump = generate_jump_strength(&mut random);
            assert!((0.4..=1.0).contains(&jump), "{jump} out of range");
        }
    }

    #[test]
    fn generate_speed_stays_within_vanilla_bounds() {
        let mut random = rng();
        for _ in 0..1000 {
            let speed = generate_speed(&mut random);
            assert!((0.1125..=0.3375).contains(&speed), "{speed} out of range");
        }
    }

    #[test]
    fn generate_zombie_horse_jump_strength_stays_within_vanilla_bounds() {
        let mut random = rng();
        for _ in 0..1000 {
            let jump = generate_zombie_horse_jump_strength(&mut random);
            assert!((0.5..=0.7).contains(&jump), "{jump} out of range");
        }
    }

    #[test]
    fn generate_zombie_horse_speed_stays_within_vanilla_bounds() {
        let mut random = rng();
        for _ in 0..1000 {
            let speed = generate_zombie_horse_speed(&mut random);
            assert!((0.213..=0.2847).contains(&speed), "{speed} out of range");
        }
    }

    /// `AbstractHorse.createOffspringAttribute`: result must stay within `[range_min, range_max]`
    /// regardless of the parent values or random rolls.
    #[test]
    fn create_offspring_attribute_stays_within_bounds() {
        let mut random = rng();
        for _ in 0..1000 {
            let value = create_offspring_attribute(15.0, 30.0, 15.0, 30.0, &mut random);
            assert!((15.0..=30.0).contains(&value), "{value} out of range");
        }
    }

    #[test]
    #[should_panic(expected = "Incorrect range for an attribute")]
    fn create_offspring_attribute_rejects_inverted_range() {
        let mut random = rng();
        let _ = create_offspring_attribute(15.0, 30.0, 30.0, 15.0, &mut random);
    }
}
