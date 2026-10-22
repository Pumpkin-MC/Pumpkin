use std::sync::{
    Arc, Weak,
    atomic::{AtomicU8, Ordering},
};

use pumpkin_data::cow_sound_variant::CowSoundVariant;
use pumpkin_data::cow_variant::CowVariant;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::{entity::EntityType, item::Item};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase,
    ageable::AgeableMob,
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    custom_sound::CustomSound,
    mob::{Mob, MobEntity, spawn::SpawnGroupData},
    passive::animal::Animal,
    player::Player,
};
use crate::item::items::bucket::fill_bucket_stack;
use crate::world::World;

const TEMPT_ITEMS: &[&Item] = &[&Item::WHEAT];

/// Represents a Cow, a common passive mob that provides milk, leather, and beef.
///
/// Wiki: <https://minecraft.wiki/w/Cow>
pub struct CowEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: crate::entity::ageable::AgeableData,
    pub variant: AtomicU8,
    pub sound_variant: AtomicU8,
}

impl CowEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let cow = Self {
            mob_entity,
            ageable_data: crate::entity::ageable::AgeableData::default(),
            variant: AtomicU8::new(CowVariant::default().id()),
            sound_variant: AtomicU8::new(CowSoundVariant::Classic as u8),
        };
        let mob_arc = Arc::new(cow);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS, false)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::water_avoiding(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    pub fn set_variant(&self, variant: CowVariant) {
        self.variant.store(variant.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::cow::DATA_VARIANT_ID,
            VarInt(variant.id() as i32),
        );
    }

    pub fn set_sound_variant(&self, sound_variant: CowSoundVariant) {
        self.sound_variant
            .store(sound_variant as u8, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::cow::DATA_SOUND_VARIANT_ID,
            VarInt(sound_variant as u8 as i32),
        );
    }
}

impl CustomSound for CowEntity {
    fn death_sound(&self) -> Option<Sound> {
        let is_baby = self.is_baby();
        let sound_variant = CowSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
            .unwrap_or_default();
        Some(sound_variant.death_sound(is_baby))
    }

    fn hurt_sound(&self) -> Option<Sound> {
        let is_baby = self.is_baby();
        let sound_variant = CowSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
            .unwrap_or_default();
        Some(sound_variant.hurt_sound(is_baby))
    }
}

impl AgeableMob for CowEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
}

impl Animal for CowEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        use pumpkin_data::tag::Taggable;
        item_stack
            .item
            .has_tag(&pumpkin_data::tag::Item::MINECRAFT_COW_FOOD)
            || TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::COW,
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

impl Mob for CowEntity {
    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let biome = world.get_biome(&self.get_entity().block_pos.load());
        self.set_variant(CowVariant::select_for_biome(biome.registry_id));
        let sound_variants = CowSoundVariant::ALL;
        self.set_sound_variant(
            sound_variants[self.get_random().random_range(0..sound_variants.len())],
        );
        self.mob_entity.finalize_spawn_base();
        group_data
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

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = self.is_baby();
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::cow::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::cow::DATA_VARIANT_ID,
            VarInt(self.variant.load(Ordering::Relaxed) as i32),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::cow::DATA_SOUND_VARIANT_ID,
            VarInt(self.sound_variant.load(Ordering::Relaxed) as i32),
        );
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(v) = CowVariant::from_name(name) {
            self.set_variant(v);
        }
    }

    fn mob_set_sound_variant_name(&self, name: &str) {
        if let Some(v) = CowSoundVariant::from_name(name) {
            self.set_sound_variant(v);
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        let variant = CowVariant::from_id(self.variant.load(Ordering::Relaxed)).unwrap_or_default();
        nbt.put_string("variant", format!("minecraft:{}", variant.to_name()));
        let sound_variant = CowSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
            .unwrap_or_default();
        nbt.put_string(
            "sound_variant",
            format!("minecraft:{}", sound_variant.to_name()),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(variant_str) = nbt.get_string("variant")
            && let Some(variant) = CowVariant::from_name(variant_str)
        {
            self.variant.store(variant.id(), Ordering::Relaxed);
        }
        if let Some(sound_str) = nbt.get_string("sound_variant")
            && let Some(sound_variant) = CowSoundVariant::from_name(sound_str)
        {
            self.sound_variant
                .store(sound_variant as u8, Ordering::Relaxed);
        }
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        if item_stack.get_item() == &Item::BUCKET && !self.is_baby() {
            self.get_entity().world.load().play_sound_expect(
                player,
                Sound::EntityCowMilk,
                pumpkin_data::sound::SoundCategory::Players,
                &player.position(),
            );
            fill_bucket_stack(item_stack, player, &Item::MILK_BUCKET);
            return true;
        }
        let sound_variant = CowSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
            .unwrap_or_default();
        self.animal_interact(
            player,
            item_stack,
            sound_variant.ambient_sound(self.is_baby()),
        )
    }
}
