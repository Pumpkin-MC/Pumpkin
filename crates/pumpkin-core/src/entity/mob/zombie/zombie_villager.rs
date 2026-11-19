use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockState;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, EquipmentSlot, VillagerVariantImpl};
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data;
use pumpkin_data::villager::{VillagerProfession, VillagerType};
use pumpkin_data::world::WorldEvent;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::server::ActorEventID;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::custom_sound::CustomSound;
use crate::entity::mob::Mob;
use crate::entity::mob::conversion::{ConversionParams, convert_to};
use crate::entity::mob::equipment::RegionalDifficulty;
use crate::entity::mob::spawn::{EntitySpawnReason, SpawnGroupData};
use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::passive::villager::breeding::villager_type_for_biome;
use crate::entity::passive::villager::data::VillagerData;
use crate::entity::passive::villager::{ReputationEventType, VillagerEntity};
use crate::entity::player::Player;
use crate::entity::player::advancement::trigger::AdvancementTrigger;
use crate::entity::{Entity, EntityBase};
use crate::world::World;

/// Every equipment slot, in drop order.
const EQUIPMENT_SLOTS: [EquipmentSlot; 8] = [
    EquipmentSlot::MAIN_HAND,
    EquipmentSlot::OFF_HAND,
    EquipmentSlot::FEET,
    EquipmentSlot::LEGS,
    EquipmentSlot::CHEST,
    EquipmentSlot::HEAD,
    EquipmentSlot::BODY,
    EquipmentSlot::SADDLE,
];

pub struct ZombieVillagerEntity {
    pub mob_entity: Arc<ZombieEntityBase>,
    villager_data: std::sync::Mutex<VillagerData>,
    villager_data_finalized: AtomicBool,
    /// `Offers` and `Gossips` kept as NBT, so a cure can hand them straight back to the villager.
    villager_nbt: std::sync::Mutex<NbtCompound>,
    villager_xp: AtomicI32,
    converting: AtomicBool,
    conversion_time: AtomicI32,
    conversion_starter: AtomicCell<Option<Uuid>>,
}

impl ZombieVillagerEntity {
    const VILLAGER_CONVERSION_WAIT_MIN: i32 = 3600;
    const VILLAGER_CONVERSION_WAIT_MAX: i32 = 6000;
    const MAX_SPECIAL_BLOCKS_COUNT: i32 = 14;
    const SPECIAL_BLOCK_RADIUS: i32 = 4;
    const NOT_CONVERTING: i32 = -1;
    /// Nausea the cured villager wakes up with.
    const NAUSEA_DURATION: i32 = 200;

    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = ZombieEntityBase::new(entity);
        Arc::new(Self {
            mob_entity,
            villager_data: std::sync::Mutex::new(Self::initialize_zombie_villager_data()),
            villager_data_finalized: AtomicBool::new(false),
            villager_nbt: std::sync::Mutex::new(NbtCompound::new()),
            villager_xp: AtomicI32::new(0),
            converting: AtomicBool::new(false),
            conversion_time: AtomicI32::new(0),
            conversion_starter: AtomicCell::new(None),
        })
    }

    /// Default villager data with a random profession, so a cured villager has trades.
    fn initialize_zombie_villager_data() -> VillagerData {
        VillagerData::new(
            VillagerType::Plains,
            VillagerProfession::from_i32(rand::random_range(0..15))
                .unwrap_or(VillagerProfession::None),
            1,
        )
    }

    #[must_use]
    pub fn get_villager_data(&self) -> VillagerData {
        *self
            .villager_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Changing the profession forgets the trades it carried.
    pub fn set_villager_data(&self, data: VillagerData) {
        let previous = std::mem::replace(
            &mut *self
                .villager_data
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            data,
        );
        if previous.profession_enum() != data.profession_enum() {
            self.villager_nbt
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .child_tags
                .remove("Offers");
        }
        self.get_entity()
            .set_synced_data(tracked_data::zombie_villager::VILLAGER_DATA, data);
    }

    #[must_use]
    pub fn get_villager_data_finalized(&self) -> bool {
        self.villager_data_finalized.load(Ordering::Relaxed)
    }

    pub fn set_villager_data_finalized(&self, finalized: bool) {
        self.villager_data_finalized
            .store(finalized, Ordering::Relaxed);
        self.get_entity().set_synced_data(
            tracked_data::zombie_villager::VILLAGER_DATA_FINALIZED,
            finalized,
        );
    }

    fn finalize_villager_type(&self, world: &World) {
        if self.get_villager_data_finalized() {
            return;
        }
        let biome = world.get_biome(&self.get_entity().block_pos.load());
        let data = self.get_villager_data();
        self.set_villager_data(VillagerData::new(
            villager_type_for_biome(biome.registry_id),
            data.profession_enum(),
            data.level.0,
        ));
        self.set_villager_data_finalized(true);
    }

    #[must_use]
    pub fn is_converting(&self) -> bool {
        self.converting.load(Ordering::Relaxed)
    }

    fn set_converting(&self, converting: bool) {
        self.converting.store(converting, Ordering::Relaxed);
        self.get_entity().set_synced_data(
            tracked_data::zombie_villager::DATA_CONVERTING_ID,
            converting,
        );
    }

    #[must_use]
    pub fn is_baby(&self) -> bool {
        self.mob_entity.is_baby()
    }

    /// Takes the profession, trades, gossips and trading xp of the villager it was.
    pub fn copy_villager(&self, villager: &VillagerEntity) {
        let mut nbt = NbtCompound::new();
        villager.mob_write_nbt(&mut nbt);
        self.read_villager_nbt(&nbt);
    }

    fn read_villager_nbt(&self, nbt: &NbtCompound) {
        if let Some(villager_data_nbt) = nbt.get_compound("VillagerData") {
            self.set_villager_data(VillagerData::read_nbt(
                villager_data_nbt,
                self.get_villager_data(),
            ));
        }

        self.villager_xp
            .store(nbt.get_int("Xp").unwrap_or(0), Ordering::Relaxed);

        let mut carried = NbtCompound::new();
        if let Some(offers) = nbt.get_compound("Offers") {
            carried.put_compound("Offers", offers.clone());
        }
        if let Some(gossips) = nbt.get_list("Gossips") {
            carried.put_list("Gossips", gossips.to_vec());
        }
        *self
            .villager_nbt
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = carried;
    }

    /// Begins the cure, swapping Weakness for Strength and telling clients to play the cure sound.
    pub fn start_converting(&self, starter: Option<Uuid>, ticks: i32) {
        let living = &self.mob_entity.mob_entity.living_entity;
        let entity = &living.entity;

        self.conversion_starter.store(starter);
        self.conversion_time.store(ticks, Ordering::Relaxed);
        self.set_converting(true);
        living.remove_effect(&StatusEffect::WEAKNESS);
        living.add_effect(Effect {
            effect_type: &StatusEffect::STRENGTH,
            duration: ticks,
            // min(difficulty - 1, 0) is 0 everywhere but Peaceful.
            amplifier: 0,
            ambient: false,
            show_particles: true,
            show_icon: true,
            blend: false,
        });
        entity.world.load().send_entity_status(
            entity,
            EntityStatus::ZombieConverting,
            ActorEventID::ZombieConverting,
        );
    }

    /// Iron bars and beds nearby speed the cure up.
    #[expect(clippy::cast_possible_truncation)]
    fn conversion_progress(&self) -> i32 {
        let mut progress = 1;
        let mut rng = self.get_random();
        if rng.random::<f32>() >= 0.01 {
            return progress;
        }

        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        let (x, y, z) = (pos.x as i32, pos.y as i32, pos.z as i32);
        let radius = Self::SPECIAL_BLOCK_RADIUS;
        let mut special_blocks = 0;

        for block_x in x - radius..x + radius {
            for block_y in y - radius..y + radius {
                for block_z in z - radius..z + radius {
                    if special_blocks >= Self::MAX_SPECIAL_BLOCKS_COUNT {
                        return progress;
                    }
                    let block = world.get_block(&BlockPos::new(block_x, block_y, block_z));
                    if block.has_tag(&tag::Block::MINECRAFT_SPEEDS_UP_ZOMBIE_VILLAGER_CURING) {
                        if rng.random::<f32>() < 0.3 {
                            progress += 1;
                        }
                        special_blocks += 1;
                    }
                }
            }
        }

        progress
    }

    /// Drops guaranteed-drop gear and keeps the slots whose items must not leave the body.
    fn drop_preserved_equipment(&self) -> Vec<(EquipmentSlot, ItemStack)> {
        let mob = &self.mob_entity.mob_entity;
        let mut kept = Vec::new();
        for slot in &EQUIPMENT_SLOTS {
            let stack = mob
                .living_entity
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(slot);
            if stack.is_empty() {
                continue;
            }
            if prevents_armor_change(&stack) {
                kept.push((slot.clone(), stack));
            } else if mob.drop_chance(slot) > 1.0 {
                mob.set_item_slot(slot, ItemStack::EMPTY.clone());
                self.get_entity().spawn_at_location(stack);
            }
        }
        kept
    }

    /// Replaces the zombie villager with the villager it remembers being.
    fn finish_conversion(&self) {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        let kept = self.drop_preserved_equipment();
        convert_to(
            self,
            &EntityType::VILLAGER,
            ConversionParams::single(false, false),
            |converted| {
                let Some(villager) = converted.cast_any().downcast_ref::<VillagerEntity>() else {
                    return;
                };
                {
                    let mut inventory = villager
                        .inventory
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    for (slot, stack) in kept {
                        let index = slot.get_slot_index() as usize;
                        if let Some(inventory_slot) = inventory.get_mut(index) {
                            *inventory_slot = stack;
                        }
                    }
                }
                villager.set_villager_data_finalized(self.get_villager_data_finalized());
                let mut nbt = self
                    .villager_nbt
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                nbt.put_int("Xp", self.villager_xp.load(Ordering::Relaxed));
                villager.set_villager_data(self.get_villager_data());
                villager.mob_read_nbt(&nbt);
                villager.finalize_spawn(&world, EntitySpawnReason::Conversion, None);
                villager.mob_entity.refresh_brain(villager);

                if let Some(starter) = self.conversion_starter.load()
                    && let Some(player) = world.get_player_by_uuid(starter)
                {
                    player.trigger_advancement(AdvancementTrigger::CuredZombieVillager);
                    villager.on_reputation_event_from(
                        ReputationEventType::ZombieVillagerCured,
                        starter,
                    );
                }

                villager.mob_entity.living_entity.add_effect(Effect {
                    effect_type: &StatusEffect::NAUSEA,
                    duration: Self::NAUSEA_DURATION,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                });
                if !entity.is_silent() {
                    world.sync_world_event(
                        WorldEvent::SoundZombieConverted,
                        entity.block_pos.load(),
                        0,
                    );
                }
            },
        );
    }
}

fn prevents_armor_change(stack: &ItemStack) -> bool {
    stack
        .get_data_component::<pumpkin_data::data_component_impl::EnchantmentsImpl>()
        .is_some_and(|enchantments| {
            enchantments
                .enchantment
                .iter()
                .any(|(enchantment, level)| *level > 0 && enchantment.effects.prevent_armor_change)
        })
}

impl CustomSound for ZombieVillagerEntity {
    fn death_sound(&self) -> Option<Sound> {
        None
    }

    fn hurt_sound(&self) -> Option<Sound> {
        None
    }

    fn voice_pitch(&self) -> f32 {
        let mut rng = self.get_random();
        let spread = (rng.random::<f32>() - rng.random::<f32>()) * 0.2;
        if self.is_baby() {
            spread + 2.0
        } else {
            spread + 1.0
        }
    }
}

impl Mob for ZombieVillagerEntity {
    crate::zombie_mob_hooks!(mob_entity);

    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        super::wants_to_pick_up(self, stack)
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        self.get_entity().entity_type.ambient_sound
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityZombieVillagerStep, 0.15, 1.0);
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        !self.is_converting() && self.villager_xp.load(Ordering::Relaxed) == 0
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        self.finalize_villager_type(world);
        self.mob_entity
            .finalize_zombie_spawn(self, world, reason, group_data)
    }

    fn do_hurt_target(&self, target: &dyn EntityBase) -> bool {
        self.mob_entity.do_hurt_target(self, target)
    }

    fn populate_default_equipment_slots(
        &self,
        world: &Arc<World>,
        difficulty: &RegionalDifficulty,
    ) {
        self.mob_entity
            .populate_default_equipment_slots(world, difficulty);
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        let mut meta = crate::entity::mob::MobMetadata::new()
            .zombie_villager_data(
                self.get_villager_data(),
                self.villager_xp.load(Ordering::Relaxed),
            )
            .converting(self.is_converting());
        if self.is_baby() {
            meta = meta.baby(true);
        }
        Some(meta)
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        (component == DataComponent::VillagerVariant).then(|| {
            Box::new(VillagerVariantImpl {
                value: format!(
                    "minecraft:{}",
                    self.get_villager_data().type_enum().to_name()
                )
                .into(),
            }) as Box<dyn DataComponentImpl>
        })
    }

    fn mob_set_variant_name(&self, name: &str) {
        let Some(villager_type) = VillagerType::from_name(name) else {
            return;
        };
        let data = self.get_villager_data();
        self.set_villager_data(VillagerData::new(
            villager_type,
            data.profession_enum(),
            data.level.0,
        ));
        self.set_villager_data_finalized(true);
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if item_stack.get_item() == &Item::GOLDEN_APPLE {
            if self
                .mob_entity
                .mob_entity
                .living_entity
                .has_effect(&StatusEffect::WEAKNESS)
            {
                item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                self.start_converting(
                    Some(player.get_entity().entity_uuid),
                    rand::random_range(
                        Self::VILLAGER_CONVERSION_WAIT_MIN..=Self::VILLAGER_CONVERSION_WAIT_MAX,
                    ),
                );
            }
            return true;
        }
        self.mob_entity.mob_interact(player, item_stack)
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        if !self.get_entity().is_alive() || !self.is_converting() {
            return;
        }
        let progress = self.conversion_progress();
        if self.conversion_time.fetch_sub(progress, Ordering::Relaxed) - progress <= 0 {
            self.finish_conversion();
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.mob_entity.write_nbt(nbt);

        nbt.put_compound("VillagerData", self.get_villager_data().write_nbt());
        nbt.put_bool("VillagerDataFinalized", self.get_villager_data_finalized());
        {
            let carried = self
                .villager_nbt
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(offers) = carried.get_compound("Offers") {
                nbt.put_compound("Offers", offers.clone());
            }
            if let Some(gossips) = carried.get_list("Gossips") {
                nbt.put_list("Gossips", gossips.to_vec());
            }
        }

        nbt.put_int(
            "ConversionTime",
            if self.is_converting() {
                self.conversion_time.load(Ordering::Relaxed)
            } else {
                Self::NOT_CONVERTING
            },
        );
        if let Some(starter) = self.conversion_starter.load() {
            nbt.put_uuid("ConversionPlayer", starter);
        }
        nbt.put_int("Xp", self.villager_xp.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.mob_entity.read_nbt(self, nbt);

        let has_villager_data = nbt.get_compound("VillagerData").is_some();
        if nbt.get_bool("VillagerDataFinalized").unwrap_or(false) || has_villager_data {
            self.set_villager_data_finalized(true);
            if !has_villager_data {
                self.set_villager_data(Self::initialize_zombie_villager_data());
            }
        }
        self.read_villager_nbt(nbt);

        match nbt
            .get_int("ConversionTime")
            .unwrap_or(Self::NOT_CONVERTING)
        {
            Self::NOT_CONVERTING => {
                self.conversion_time
                    .store(Self::NOT_CONVERTING, Ordering::Relaxed);
                self.set_converting(false);
            }
            conversion_time => {
                self.start_converting(nbt.get_uuid("ConversionPlayer"), conversion_time);
            }
        }
    }
}
