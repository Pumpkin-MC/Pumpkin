use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, AtomicU8, Ordering},
};

use pumpkin_data::chicken_sound_variant::ChickenSoundVariant;
use pumpkin_data::chicken_variant::ChickenVariant;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{entity::EntityType, item::Item};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use rand::RngExt;

use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ageable::AgeableMob,
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity, spawn::SpawnGroupData},
    passive::animal::Animal,
    player::Player,
};
use crate::plugin::api::events::entity::entity_drop_item::EntityDropItemEvent;
use crate::world::World;

const EGG_LAY_TIME: std::ops::Range<i32> = 6000..12000;
const CHICKEN_JOCKEY_EXPERIENCE_REWARD: u32 = 10;

const TEMPT_ITEMS: &[&Item] = &[
    &Item::WHEAT_SEEDS,
    &Item::MELON_SEEDS,
    &Item::PUMPKIN_SEEDS,
    &Item::BEETROOT_SEEDS,
    &Item::TORCHFLOWER_SEEDS,
    &Item::PITCHER_POD,
];

/// Represents a Chicken, a passive mob that lays eggs and is immune to fall damage.
///
/// Wiki: <https://minecraft.wiki/w/Chicken>
pub struct ChickenEntity {
    pub mob_entity: MobEntity,
    pub variant: AtomicU8,
    pub sound_variant: AtomicU8,
    egg_lay_time: AtomicI32,
    is_chicken_jockey: AtomicBool,
    pub ageable_data: crate::entity::ageable::AgeableData,
}

impl ChickenEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let egg_lay_time = rand::rng().random_range(EGG_LAY_TIME);
        let chicken = Self {
            mob_entity,
            variant: AtomicU8::new(ChickenVariant::default().id()),
            sound_variant: AtomicU8::new(ChickenSoundVariant::Classic as u8),
            egg_lay_time: AtomicI32::new(egg_lay_time),
            is_chicken_jockey: AtomicBool::new(false),
            ageable_data: crate::entity::ageable::AgeableData::default(),
        };
        let mob_arc = Arc::new(chicken);
        mob_arc
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_pathfinding_malus(PathType::Water, 0.0);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(1.4));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.0, TEMPT_ITEMS, false)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::water_avoiding(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    fn lay_egg(&self) -> bool {
        let entity = self.get_entity();
        let mut drop_event =
            EntityDropItemEvent::new(entity.entity_id, "minecraft:egg".to_string(), 1);
        if let Some(server) = entity.world.load().server.upgrade() {
            server
                .plugin_manager
                .fire_blocking(&server, &mut drop_event);
        }
        if drop_event.cancelled {
            return false;
        }
        entity.spawn_at_location(ItemStack::new(1, &Item::EGG));
        true
    }

    pub fn is_chicken_jockey(&self) -> bool {
        self.is_chicken_jockey.load(Ordering::Relaxed)
    }

    pub fn set_chicken_jockey(&self, is_chicken_jockey: bool) {
        self.is_chicken_jockey
            .store(is_chicken_jockey, Ordering::Relaxed);
    }

    pub fn set_variant(&self, variant: ChickenVariant) {
        self.variant.store(variant.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::chicken::DATA_VARIANT_ID,
            VarInt(variant.id() as i32),
        );
    }

    pub fn set_sound_variant(&self, sound_variant: ChickenSoundVariant) {
        self.sound_variant
            .store(sound_variant as u8, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::chicken::DATA_SOUND_VARIANT_ID,
            VarInt(sound_variant as u8 as i32),
        );
    }
}

impl CustomSound for ChickenEntity {
    fn death_sound(&self) -> Option<Sound> {
        let is_baby = self.is_baby();
        let sound_variant =
            ChickenSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
                .unwrap_or_default();
        Some(sound_variant.death_sound(is_baby))
    }

    fn hurt_sound(&self) -> Option<Sound> {
        let is_baby = self.is_baby();
        let sound_variant =
            ChickenSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
                .unwrap_or_default();
        Some(sound_variant.hurt_sound(is_baby))
    }
}

impl AgeableMob for ChickenEntity {
    fn get_ageable_data(&self) -> &crate::entity::ageable::AgeableData {
        &self.ageable_data
    }
}

impl Animal for ChickenEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        use pumpkin_data::tag::Taggable;
        item_stack
            .item
            .has_tag(&pumpkin_data::tag::Item::MINECRAFT_CHICKEN_FOOD)
            || TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::CHICKEN,
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

impl Mob for ChickenEntity {
    fn mob_get_component(
        &self,
        component: pumpkin_data::data_component::DataComponent,
    ) -> Option<Box<dyn pumpkin_data::data_component_impl::DataComponentImpl>> {
        use pumpkin_data::{
            data_component::DataComponent, data_component_impl::ChickenVariantImpl,
        };
        match component {
            DataComponent::ChickenVariant => Some(Box::new(ChickenVariantImpl {
                value: format!(
                    "minecraft:{}",
                    ChickenVariant::from_id(self.variant.load(Ordering::Relaxed))?.to_name()
                )
                .into(),
            })),
            _ => None,
        }
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let biome = world.get_biome(&self.get_entity().block_pos.load());
        self.set_variant(ChickenVariant::select_for_biome(biome.registry_id));
        let sound_variants = ChickenSoundVariant::ALL;
        self.set_sound_variant(
            sound_variants[self.get_random().random_range(0..sound_variants.len())],
        );
        self.mob_entity.finalize_spawn_base();
        group_data
    }

    fn has_audited_tick(&self) -> bool {
        true
    }

    fn audited_spawn_distance(&self) -> f64 {
        // The egg drops anywhere in its block.
        if self.egg_lay_time.load(Ordering::Relaxed) <= 1 {
            1.0
        } else {
            0.0
        }
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

    fn remove_when_far_away(&self, _distance_sq: f64) -> bool {
        self.is_chicken_jockey()
    }

    fn get_base_experience_reward(&self) -> u32 {
        if self.is_chicken_jockey() {
            CHICKEN_JOCKEY_EXPERIENCE_REWARD
        } else {
            self.get_entity().entity_type.experience_reward
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_bool("IsChickenJockey", self.is_chicken_jockey());
        nbt.put_int("EggLayTime", self.egg_lay_time.load(Ordering::Relaxed));
        let variant =
            ChickenVariant::from_id(self.variant.load(Ordering::Relaxed)).unwrap_or_default();
        nbt.put_string("variant", format!("minecraft:{}", variant.to_name()));
        let sound_variant =
            ChickenSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
                .unwrap_or_default();
        nbt.put_string(
            "sound_variant",
            format!("minecraft:{}", sound_variant.to_name()),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_chicken_jockey(nbt.get_bool("IsChickenJockey").unwrap_or(false));
        if let Some(egg_lay_time) = nbt.get_int("EggLayTime") {
            self.egg_lay_time.store(egg_lay_time, Ordering::Relaxed);
        }
        if let Some(variant_str) = nbt.get_string("variant")
            && let Some(variant) = ChickenVariant::from_name(variant_str)
        {
            self.variant.store(variant.id(), Ordering::Relaxed);
        }
        if let Some(sound_str) = nbt.get_string("sound_variant")
            && let Some(sound_variant) = ChickenSoundVariant::from_name(sound_str)
        {
            self.sound_variant
                .store(sound_variant as u8, Ordering::Relaxed);
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(v) = ChickenVariant::from_name(name) {
            self.set_variant(v);
        }
    }

    fn mob_set_sound_variant_name(&self, name: &str) {
        if let Some(v) = ChickenSoundVariant::from_name(name) {
            self.set_sound_variant(v);
        }
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::chicken::BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::chicken::DATA_VARIANT_ID,
            VarInt(self.variant.load(Ordering::Relaxed) as i32),
        );
        entity.set_synced_data(
            pumpkin_data::tracked_data::chicken::DATA_SOUND_VARIANT_ID,
            VarInt(self.sound_variant.load(Ordering::Relaxed) as i32),
        );
    }

    fn mob_ai_step(&self) {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        let velocity = entity.velocity.load();
        if !entity.on_ground.load(Ordering::Relaxed) && velocity.y < 0.0 {
            entity.set_velocity(velocity.multiply(1.0, 0.6, 1.0));
        }
        if entity.is_alive()
            && !living.is_dead_or_dying()
            && !self.is_baby()
            && !self.is_chicken_jockey()
            && self.egg_lay_time.fetch_sub(1, Ordering::Relaxed) <= 1
        {
            let mut rng = self.get_random();
            if self.lay_egg() {
                entity.world.load().play_sound_fine(
                    Sound::EntityChickenEgg,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                    1.0,
                    (rng.random::<f32>() - rng.random::<f32>()).mul_add(0.2, 1.0),
                );
                entity
                    .world
                    .load()
                    .emit_game_event(GameEvent::EntityPlace.name(), entity.pos.load());
            }
            self.egg_lay_time
                .store(rng.random_range(EGG_LAY_TIME), Ordering::Relaxed);
        }
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let sound_variant =
            ChickenSoundVariant::from_id(self.sound_variant.load(Ordering::Relaxed))
                .unwrap_or_default();
        self.animal_interact(
            player,
            item_stack,
            sound_variant.ambient_sound(self.is_baby()),
        )
    }
}
