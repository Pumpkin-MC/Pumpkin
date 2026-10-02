use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::cow_variant::CowVariant;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::vector3::Vector3;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    item::ItemEntity,
    mob::{Mob, MobEntity},
    passive::{animal::Animal, cow::CowEntity},
    player::Player,
    shearable::{Shearable, shear_by_player, shearing_loot},
};

const TEMPT_ITEMS: &[&Item] = &[&Item::WHEAT];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum MooshroomVariant {
    #[default]
    Red = 0,
    Brown = 1,
}

impl MooshroomVariant {
    #[must_use]
    pub const fn from_id(id: i32) -> Self {
        match id {
            1 => Self::Brown,
            _ => Self::Red,
        }
    }

    #[must_use]
    pub const fn id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Brown => "brown",
        }
    }

    #[must_use]
    pub fn from_name(name: &str) -> Self {
        match name {
            "brown" => Self::Brown,
            _ => Self::Red,
        }
    }
}

pub struct MooshroomEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    pub variant: AtomicI32,
    pub stew_effect: AtomicCell<Option<u32>>,
    pub last_lightning_bolt_uuid: AtomicCell<Option<Uuid>>,
    /// Claimed by the first shear, so two players shearing at once can't both convert this mooshroom.
    converting: AtomicBool,
}

impl MooshroomEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let mooshroom = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            variant: AtomicI32::new(MooshroomVariant::Red.id()),
            stew_effect: AtomicCell::new(None),
            last_lightning_bolt_uuid: AtomicCell::new(None),
            converting: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(mooshroom);
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
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(3, Box::new(TemptGoal::new(1.25, TEMPT_ITEMS, false)));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));
        };

        mob_arc
    }

    #[must_use]
    pub fn get_variant(&self) -> MooshroomVariant {
        MooshroomVariant::from_id(self.variant.load(Ordering::Relaxed))
    }

    pub fn set_variant(&self, variant: MooshroomVariant) {
        self.variant.store(variant.id(), Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(
            pumpkin_data::tracked_data::mooshroom::DATA_TYPE,
            VarInt(variant.id()),
        );
    }

    /// Vanilla `Mob.convertTo`
    fn convert_to_cow(&self) -> Arc<CowEntity> {
        let entity = self.get_entity();
        let cow = CowEntity::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::COW,
        ));
        // No cow.finalize_spawn(...), vanilla omits this likely to stop a 'biomed' cow being created
        cow.set_variant(CowVariant::default());

        let cow_entity = cow.get_entity();
        cow_entity.set_rotation(entity.yaw.load(), entity.pitch.load());
        cow_entity.head_yaw.store(entity.head_yaw.load());
        cow_entity.body_yaw.store(entity.body_yaw.load());
        cow_entity.velocity.store(entity.velocity.load());
        cow_entity
            .on_ground
            .store(entity.on_ground.load(Ordering::Relaxed), Ordering::Relaxed);
        cow.mob_entity
            .living_entity
            .fall_distance
            .store(self.mob_entity.living_entity.fall_distance.load());

        cow.set_age(self.get_age());
        let (data, cow_data) = (self.get_ageable_data(), cow.get_ageable_data());
        cow_data
            .forced_age
            .store(data.forced_age.load(Ordering::Relaxed), Ordering::Relaxed);
        cow_data.forced_age_timer.store(
            data.forced_age_timer.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );

        cow.mob_entity.set_no_ai(self.mob_entity.is_no_ai());
        cow.mob_entity.persistence_required.store(
            self.mob_entity.persistence_required.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );

        if let Some(custom_name) = &**entity.custom_name.load() {
            cow_entity.set_custom_name(custom_name.clone());
        }
        cow_entity.set_custom_name_visible(entity.custom_name_visible.load(Ordering::Relaxed));
        cow_entity.set_on_fire(entity.is_on_fire());
        cow_entity.invulnerable.store(
            entity.invulnerable.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        cow_entity.set_has_no_gravity(entity.has_no_gravity());
        cow_entity.portal_cooldown.store(
            entity.portal_cooldown.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        cow_entity.set_silent(entity.is_silent());
        cow_entity
            .scoreboard_tags
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone_from(
                &entity
                    .scoreboard_tags
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );

        cow
    }
}

impl Shearable for MooshroomEntity {
    /// Vanilla `MushroomCow.shear`: converts this mooshroom into a cow and drops its mushrooms.
    fn shear(&self, sound_category: SoundCategory, tool: &ItemStack) -> bool {
        if self.converting.swap(true, Ordering::Relaxed) {
            return false;
        }

        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        world.play_sound(Sound::EntityMooshroomShear, sound_category, &pos);

        if entity.is_removed() {
            return false;
        }

        let cow = self.convert_to_cow();
        // Vanilla discards the mooshroom even if the cow fails to spawn, but here that only happens when a
        // plugin cancels the conversion or the spawn. Keep the mooshroom then, so it neither vanishes nor
        // drops free mushrooms.
        if !self.transform(cow.get_entity().entity_id, "sheared".to_string())
            || !world.spawn_entity(cow.clone())
        {
            self.converting.store(false, Ordering::Relaxed);
            return false;
        }

        let height = f64::from(entity.height());
        world.spawn_particle(
            Vector3::new(pos.x, height.mul_add(0.5, pos.y), pos.z),
            Vector3::new(0.0, 0.0, 0.0),
            0.0,
            1,
            Particle::Explosion,
        );

        let loot_key = match self.get_variant() {
            MooshroomVariant::Red => "minecraft:shearing/mooshroom/red",
            MooshroomVariant::Brown => "minecraft:shearing/mooshroom/brown",
        };
        let drop_pos = Vector3::new(pos.x, pos.y + height, pos.z);
        for drop in shearing_loot(entity, loot_key, tool) {
            for _ in 0..drop.item_count {
                let item_entity = Arc::new(ItemEntity::new(
                    Entity::new(world.clone(), drop_pos, &EntityType::ITEM),
                    drop.copy_with_count(1),
                ));
                world.spawn_entity(item_entity);
            }
        }

        // These send packets about the cow, so they wait until clients know it exists.
        let cow_living = &cow.mob_entity.living_entity;
        cow_living.set_absorption(self.mob_entity.living_entity.get_absorption());
        let effects: Vec<_> = self
            .mob_entity
            .living_entity
            .active_effects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .cloned()
            .collect();
        for effect in effects {
            cow_living.add_effect(effect);
        }
        // This is not vanilla: the leash should be removed on the first shear, only once
        // the leash is removed does the mooshroom get sheared. I left this out because leashing
        // is currently very broken
        // TODO fix when leashing works properly
        let holder = entity
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        if let Some(holder) = holder {
            cow_living.entity.leash_to(holder);
        }

        entity.remove();
        true
    }

    fn ready_for_shearing(&self) -> bool {
        !self.is_baby()
    }
}

impl AgeableMob for MooshroomEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for MooshroomEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_COW_FOOD)
            || TEMPT_ITEMS.iter().any(|i| i.id == item_stack.item.id)
    }
}

impl Mob for MooshroomEntity {
    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn as_shearable(&self) -> Option<&dyn Shearable> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("Type", self.get_variant().as_str().to_string());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(variant_str) = nbt.get_string("Type") {
            self.set_variant(MooshroomVariant::from_name(variant_str));
        }
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.ageable_ai_step();
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        let is_baby = entity.age.load(Ordering::Relaxed) < 0;
        if is_baby {
            entity.set_synced_data(pumpkin_data::tracked_data::mooshroom::DATA_BABY_ID, true);
        }
        entity.set_synced_data(
            pumpkin_data::tracked_data::mooshroom::DATA_TYPE,
            VarInt(self.get_variant().id()),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let item = item_stack.get_item();

        if item == &Item::BOWL && !self.is_baby() {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            let is_suspicious = self.stew_effect.swap(None).is_some();
            let sound = if is_suspicious {
                Sound::EntityMooshroomSuspiciousMilk
            } else {
                Sound::EntityMooshroomMilk
            };
            world.play_sound(sound, SoundCategory::Neutral, &pos);
            return true;
        }

        if item == &Item::BUCKET && !self.is_baby() {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let entity = self.get_entity();
            let world = entity.world.load();
            world.play_sound(
                Sound::EntityCowMilk,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );
            return true;
        }

        if item == &Item::SHEARS && self.ready_for_shearing() {
            return shear_by_player(self, player, item_stack);
        }

        if self.get_variant() == MooshroomVariant::Brown
            && !self.is_baby()
            && item.has_tag(&tag::Item::MINECRAFT_SMALL_FLOWERS)
        {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            let entity = self.get_entity();
            let world = entity.world.load();
            let pos = entity.pos.load();
            self.stew_effect.store(Some(1));
            world.play_sound(Sound::EntityMooshroomEat, SoundCategory::Neutral, &pos);
            world.spawn_particle(
                pos + Vector3::new(0.0, 0.5, 0.0),
                Vector3::new(0.5, 0.5, 0.5),
                0.0,
                4,
                Particle::Effect,
            );
            return true;
        }

        self.animal_interact(player, item_stack, Sound::EntityCowAmbient)
    }
}
