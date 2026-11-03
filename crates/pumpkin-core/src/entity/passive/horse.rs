use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::block_state::SoundType;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, EquipmentSlot, HorseVariantImpl};
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_variant::{HorseMarkings, HorseVariant};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::set_actor_data::{
    MetadataValue, SyncedActorDataList, entity_data_key,
};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, GearDamage,
    ageable::{AgeableData, AgeableMob, DEFAULT_BABY_SPAWN_CHANCE},
    mob::spawn::{EntitySpawnReason, SpawnGroupData},
    mob::{Mob, MobEntity},
    passive::abstract_horse::{
        AbstractHorse, AbstractHorseData, add_horse_behaviour_goals, generate_jump_strength,
        generate_max_health, generate_speed, play_horse_gallop_sound, play_horse_step_sound,
        register_horse_goals,
    },
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;

const HORSE_PARTNERS: &[&EntityType] = &[&EntityType::HORSE, &EntityType::DONKEY];
const MARKINGS_SHIFT: i32 = 8;
const ID_MASK: i32 = 0xFF;
const BREATHE_CHANCE: i32 = 10;
const BREATHE_VOLUME_SCALE: f32 = 0.6;

const fn pack_variant(color: i32, markings: i32) -> i32 {
    (color & ID_MASK) | ((markings & ID_MASK) << MARKINGS_SHIFT)
}

const fn color_id(variant: i32) -> i32 {
    variant & ID_MASK
}

const fn markings_id(variant: i32) -> i32 {
    (variant >> MARKINGS_SHIFT) & ID_MASK
}

pub struct HorseEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    horse_data: AbstractHorseData,
    pub variant: AtomicI32,
}

impl HorseEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let horse = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            horse_data: AbstractHorseData::default(),
            variant: AtomicI32::default(),
        };
        let mob_arc = Arc::new(horse);
        let mob: Arc<dyn Mob> = mob_arc.clone();
        register_horse_goals(&mob);
        add_horse_behaviour_goals(mob.as_ref());

        mob_arc
    }

    #[must_use]
    pub fn get_variant(&self) -> i32 {
        self.variant.load(Ordering::Relaxed)
    }

    pub fn set_variant(&self, variant: i32) {
        self.variant.store(variant, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::horse::DATA_ID_TYPE_VARIANT,
            VarInt(variant),
        );
        entity.send_bedrock_actor_data(&self.bedrock_variant_data());
    }
}

crate::impl_abstract_horse!(HorseEntity, {
    fn finalize_horse_spawn(
        &self,
        _world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let mut rng = self.get_random();
        let (color, group_size) = match group_data {
            Some(SpawnGroupData::Horse { color, group_size }) => (color, group_size),
            _ => (
                HorseVariant::ALL[rng.random_range(0..HorseVariant::ALL.len())],
                0,
            ),
        };
        let markings = HorseMarkings::ALL[rng.random_range(0..HorseMarkings::ALL.len())];
        self.set_variant(pack_variant(color.id(), markings.id()));
        self.randomize_attributes();
        let group_size = match self.finalize_ageable_spawn(Some(SpawnGroupData::Ageable {
            group_size,
            should_spawn_baby: true,
            baby_spawn_chance: DEFAULT_BABY_SPAWN_CHANCE,
        })) {
            Some(SpawnGroupData::Ageable { group_size, .. }) => group_size,
            _ => group_size + 1,
        };
        Some(SpawnGroupData::Horse { color, group_size })
    }

    fn bedrock_variant_data(&self) -> SyncedActorDataList {
        let variant = self.get_variant();
        let mut data = SyncedActorDataList::new();
        data.set(
            entity_data_key::VARIANT,
            MetadataValue::Int(color_id(variant)),
        );
        data.set(
            entity_data_key::MARK_VARIANT,
            MetadataValue::Int(markings_id(variant)),
        );
        data
    }

    fn play_gallop_sound(&self, sound_type: &SoundType) {
        play_horse_gallop_sound(self, sound_type);
        if self.get_random().random_range(0..BREATHE_CHANCE) == 0 {
            let breathe = if self.is_baby() {
                Sound::EntityBabyHorseBreathe
            } else {
                Sound::EntityHorseBreathe
            };
            self.play_entity_sound(
                breathe,
                sound_type.volume * BREATHE_VOLUME_SCALE,
                sound_type.pitch,
            );
        }
    }

    fn ambient_sound(&self) -> Option<Sound> {
        if self.is_baby() {
            Some(Sound::EntityBabyHorseAmbient)
        } else {
            self.get_entity().entity_type.ambient_sound
        }
    }

    fn get_hurt_sound(&self) -> Option<Sound> {
        Some(if self.is_baby() {
            Sound::EntityBabyHorseHurt
        } else {
            Sound::EntityHorseHurt
        })
    }

    fn get_death_sound(&self) -> Option<Sound> {
        Some(if self.is_baby() {
            Sound::EntityBabyHorseDeath
        } else {
            Sound::EntityHorseDeath
        })
    }

    fn randomize_attributes(&self) {
        let mut rng = self.get_random();
        let living = &self.mob_entity.living_entity;
        living.set_max_health(generate_max_health(|bound| rng.random_range(0..bound)));
        living.set_attribute_base(&Attributes::MOVEMENT_SPEED, generate_speed(|| rng.random()));
        living.set_attribute_base(
            &Attributes::JUMP_STRENGTH,
            generate_jump_strength(|| rng.random()),
        );
    }
});

impl Animal for HorseEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_HORSE_FOOD)
    }

    fn can_mate(&self, partner: &dyn EntityBase) -> bool {
        self.can_mate_with(partner, HORSE_PARTNERS)
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let Some(other) = partner.cast_any().downcast_ref::<Self>() else {
            return self.breed_offspring(partner, &EntityType::MULE);
        };
        let baby = self.breed_offspring(partner, &EntityType::HORSE);
        if let Some(foal) = baby.cast_any().downcast_ref::<Self>() {
            let mut rng = self.get_random();
            let color = match rng.random_range(0..9) {
                0..4 => color_id(self.get_variant()),
                4..8 => color_id(other.get_variant()),
                _ => rng.random_range(0..HorseVariant::ALL.len() as i32),
            };
            let markings = match rng.random_range(0..5) {
                0..2 => markings_id(self.get_variant()),
                2..4 => markings_id(other.get_variant()),
                _ => rng.random_range(0..HorseMarkings::ALL.len() as i32),
            };
            foal.set_variant(pack_variant(color, markings));
        }
        baby
    }
}

impl Mob for HorseEntity {
    crate::abstract_horse_mob_hooks!();

    fn mob_hurt_gear_slots(&self, gear: GearDamage) -> &'static [EquipmentSlot] {
        match gear {
            GearDamage::Armor => &[EquipmentSlot::BODY],
            GearDamage::Helmet => &[],
        }
    }

    fn mob_play_step_sound(&self, pos: &BlockPos, state: &BlockState) {
        play_horse_step_sound(self, pos, state);
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        (component == DataComponent::HorseVariant).then(|| {
            Box::new(HorseVariantImpl {
                value: HorseVariant::by_id(color_id(self.get_variant()))
                    .name()
                    .into(),
            }) as Box<dyn DataComponentImpl>
        })
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(color) = HorseVariant::from_name(name) {
            self.set_variant(pack_variant(color.id(), markings_id(self.get_variant())));
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.write_horse_nbt(nbt);
        nbt.put_int("Variant", self.get_variant());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.read_horse_nbt(nbt);
        if let Some(variant) = nbt.get_int("Variant") {
            self.set_variant(variant);
        }
    }

    fn mob_init_data_tracker(&self) {
        self.init_horse_data_tracker();
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::horse::DATA_ID_TYPE_VARIANT,
            VarInt(self.get_variant()),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if let Some(result) = self.handle_important_interactions(player, item_stack) {
            return result;
        }
        if let Some(result) = self.interact_with_held_item(player, item_stack, true) {
            return result;
        }
        self.horse_mob_interact(player, item_stack)
    }
}
