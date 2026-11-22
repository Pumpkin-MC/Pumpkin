use std::sync::{
    Arc, Weak,
    atomic::{AtomicU8, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::chunk::Biome;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    DataComponentImpl, EquipmentSlot, PigSoundVariantImpl, PigVariantImpl,
};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::pig_sound_variant::PigSoundVariant;
use pumpkin_data::pig_variant::PigVariant;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{entity::EntityType, item::Item};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::EntityBase;
use crate::entity::custom_sound::CustomSound;
use crate::entity::item_steerable::{ItemBasedSteering, ItemSteerable};
use crate::entity::lightning::LightningBoltEntity;
use crate::entity::{
    Entity,
    ageable::AgeableMob,
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity,
        conversion::{ConversionParams, convert_to},
        equipment::{RegionalDifficulty, equip_on_target},
        spawn::{EntitySpawnReason, SpawnGroupData},
    },
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;
use pumpkin_util::Difficulty;

/// Represents a Pig, a common passive mob that provides porkchops.
///
/// Wiki: <https://minecraft.wiki/w/Pig>
pub struct PigEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: crate::entity::ageable::AgeableData,
    pub steering: ItemBasedSteering,
    pub variant: AtomicU8,
    pub sound_variant: AtomicU8,
}

impl PigEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let pig = Self {
            mob_entity,
            ageable_data: crate::entity::ageable::AgeableData::default(),
            steering: ItemBasedSteering::new(pumpkin_data::tracked_data::pig::DATA_BOOST_TIME),
            variant: AtomicU8::new(PigVariant::default().id()),
            sound_variant: AtomicU8::new(PigSoundVariant::Classic as u8),
        };
        let mob_arc = Arc::new(pig);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.25));
            goal_selector.add_goal(3, BreedGoal::new(1.0));
            goal_selector.add_goal(
                4,
                Box::new(TemptGoal::new(1.2, &[&Item::CARROT_ON_A_STICK], false)),
            );
            goal_selector.add_goal(
                4,
                Box::new(TemptGoal::with_tag(
                    1.2,
                    &tag::Item::MINECRAFT_PIG_FOOD,
                    false,
                )),
            );
            goal_selector.add_goal(5, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::water_avoiding(1.0)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    pub fn set_variant(&self, variant: PigVariant) {
        self.set_variant_id(variant.id());
    }

    pub fn set_variant_id(&self, raw_id: u8) {
        self.variant.store(raw_id, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::pig::DATA_VARIANT_ID,
            VarInt(i32::from(raw_id)),
        );
    }

    fn variant_name(&self) -> String {
        let variant_id = u32::from(self.variant.load(Ordering::Relaxed));
        self.resolve_dynamic_entry_id("minecraft:pig_variant", variant_id, |id| {
            let variant = PigVariant::from_id(id as u8).unwrap_or_default();
            format!("minecraft:{}", variant.to_name())
        })
    }

    fn sound_variant(&self) -> PigSoundVariant {
        PigSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed)).unwrap_or_default()
    }

    pub fn set_sound_variant(&self, sound_variant: PigSoundVariant) {
        self.sound_variant
            .store(sound_variant as u8, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::pig::DATA_SOUND_VARIANT_ID,
            VarInt(sound_variant as u8 as i32),
        );
    }

    fn select_spawn_variants(&self, biome: &'static Biome) {
        self.set_variant(PigVariant::select_for_biome(biome.registry_id));
        let sound_variants = PigSoundVariant::ALL;
        self.set_sound_variant(
            sound_variants[self.get_random().random_range(0..sound_variants.len())],
        );
    }
}

impl CustomSound for PigEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(self.sound_variant().death_sound(self.is_baby()))
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(self.sound_variant().hurt_sound(self.is_baby()))
    }
}

impl AgeableMob for PigEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
}

impl Animal for PigEntity {
    fn play_eating_sound(&self) {
        self.make_sound(self.sound_variant().eat_sound(self.is_baby()));
    }

    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_PIG_FOOD)
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::PIG,
        ));
        if let Some(partner) = partner.cast_any().downcast_ref::<Self>() {
            let parent = if self.get_random().random::<bool>() {
                self
            } else {
                partner
            };
            baby.variant
                .store(parent.variant.load(Ordering::Relaxed), Ordering::Relaxed);
        }
        baby
    }
}

impl Mob for PigEntity {
    fn can_dispenser_equip_into_slot(&self, slot: &EquipmentSlot) -> bool {
        *slot == EquipmentSlot::SADDLE || self.mob_entity.can_pick_up_loot()
    }

    fn mob_controlling_passenger(&self) -> Option<Arc<dyn EntityBase>> {
        self.saddled_controlling_passenger(Some(&Item::CARROT_ON_A_STICK))
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(self.sound_variant().step_sound(self.is_baby()), 0.15, 1.0);
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(self.sound_variant().ambient_sound(self.is_baby()))
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        match component {
            DataComponent::PigVariant => Some(Box::new(PigVariantImpl {
                value: self.variant_name().into(),
            })),
            DataComponent::PigSoundVariant => Some(Box::new(PigSoundVariantImpl {
                value: format!("minecraft:{}", self.sound_variant().to_name()).into(),
            })),
            _ => None,
        }
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let biome = world.get_biome(&self.get_entity().block_pos.load());
        self.select_spawn_variants(biome);
        self.finalize_spawn_super(group_data)
    }

    fn finalize_spawn_in_biome(&self, biome: &'static Biome) {
        self.select_spawn_variants(biome);
    }

    fn mob_ai_step(&self) {
        if self
            .mob_controlling_passenger()
            .is_some_and(|rider| rider.get_player().is_some())
        {
            self.steering.tick_boost();
        }
    }

    fn has_audited_tick(&self) -> bool {
        true
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = self.is_baby();
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::pig::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::pig::DATA_VARIANT_ID,
            VarInt(self.variant.load(Ordering::Relaxed) as i32),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::pig::DATA_SOUND_VARIANT_ID,
            VarInt(self.sound_variant.load(Ordering::Relaxed) as i32),
        );
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(id) = self.resolve_dynamic_raw_id("minecraft:pig_variant", name, |n| {
            PigVariant::from_name(n).map(|v| u32::from(v.id()))
        }) && let Ok(variant_id) = u8::try_from(id)
        {
            self.set_variant_id(variant_id);
        }
    }

    fn mob_set_sound_variant_name(&self, name: &str) {
        if let Some(v) = PigSoundVariant::from_name(name) {
            self.set_sound_variant(v);
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("variant", self.variant_name());
        nbt.put_string(
            "sound_variant",
            format!("minecraft:{}", self.sound_variant().to_name()),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.mob_entity
            .read_legacy_saddle(nbt.get_bool("Saddle") == Some(true));
        if let Some(variant_str) = nbt.get_string("variant")
            && let Some(id) =
                self.resolve_dynamic_raw_id("minecraft:pig_variant", variant_str, |n| {
                    PigVariant::from_name(n).map(|v| u32::from(v.id()))
                })
            && let Ok(variant_id) = u8::try_from(id)
        {
            self.variant.store(variant_id, Ordering::Relaxed);
        }
        if let Some(sound_str) = nbt.get_string("sound_variant")
            && let Some(sound_variant) = PigSoundVariant::from_name(sound_str)
        {
            self.sound_variant
                .store(sound_variant as u8, Ordering::Relaxed);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_item_steerable(&self) -> Option<&dyn ItemSteerable> {
        Some(self)
    }

    fn can_use_slot(&self, slot: &EquipmentSlot) -> bool {
        *slot != EquipmentSlot::SADDLE
            || (self.mob_entity.living_entity.entity.is_alive() && !self.is_baby())
    }

    fn get_equip_sound(&self, slot: &EquipmentSlot) -> Option<Sound> {
        (*slot == EquipmentSlot::SADDLE).then_some(Sound::EntityPigSaddle)
    }

    fn mob_on_lightning_strike(&self, caller: &dyn EntityBase, lightning: &LightningBoltEntity) {
        let world = self.get_entity().world.load_full();
        let converted = (world.level_info.load().difficulty != Difficulty::Peaceful)
            .then(|| {
                convert_to(
                    self,
                    &EntityType::ZOMBIFIED_PIGLIN,
                    ConversionParams::single(false, true),
                    |zp| {
                        let difficulty =
                            RegionalDifficulty::at(&world, self.get_entity().pos.load());
                        zp.populate_default_equipment_slots(&world, &difficulty);
                        zp.get_mob_entity()
                            .persistence_required
                            .store(true, Ordering::Relaxed);
                    },
                )
            })
            .flatten();
        if converted.is_none() {
            self.mob_entity
                .living_entity
                .on_lightning_strike(caller, lightning);
        }
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if !self.is_food(item_stack)
            && self.is_saddled()
            && !self.get_entity().has_passengers()
            && !player.get_entity().is_sneaking()
        {
            let world = player.world();
            if let Some(vehicle) = world.get_entity_by_id(self.get_entity().entity_id)
                && let Some(passenger) = world.get_player_by_id(player.entity_id())
            {
                self.get_entity()
                    .add_passenger(vehicle, passenger as Arc<dyn EntityBase>);
            }
            return true;
        }
        if self.animal_interact(player, item_stack) {
            return true;
        }
        self.is_equippable_in_slot(item_stack, &EquipmentSlot::SADDLE)
            && equip_on_target(self, player, item_stack)
    }
}

impl ItemSteerable for PigEntity {
    fn boost(&self) -> bool {
        let mut rng = self.get_random();
        self.steering.boost(self.get_entity(), &mut rng)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
