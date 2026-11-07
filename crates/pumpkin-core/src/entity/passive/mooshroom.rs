use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::BlockState;
use pumpkin_data::cow_sound_variant::CowSoundVariant;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{
    DataComponentImpl, MooshroomVariantImpl, SuspiciousStewEffect, SuspiciousStewEffectsImpl,
};
use pumpkin_data::entity::EntityType;
use pumpkin_data::entity_variant::MooshroomVariant;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::suspicious_effects::get_suspicious_effects;
use pumpkin_data::tag::Taggable;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use uuid::Uuid;

use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    custom_sound::CustomSound,
    item::ItemEntity,
    lightning::LightningBoltEntity,
    mob::conversion::{ConversionParams, convert_to},
    mob::{Mob, MobEntity},
    passive::animal::Animal,
    player::Player,
};
use crate::item::items::bucket::{create_filled_result, fill_bucket_stack};
use crate::world::game_event::GameEventContext;
use crate::world::loot::{LootContextParameters, generate_loot_from_handle};

const MUTATE_CHANCE: i32 = 1024;
const SOUND_VOLUME: f32 = 0.4;
const SHEAR_MOOSHROOM_LOOT_TABLE: &str = "minecraft:shearing/mooshroom";

pub struct MooshroomEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    variant: AtomicI32,
    stew_effects: std::sync::Mutex<Option<SuspiciousStewEffectsImpl>>,
    last_lightning_bolt_uuid: AtomicCell<Option<Uuid>>,
}

impl MooshroomEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mooshroom = Self {
            mob_entity: MobEntity::new(entity),
            ageable_data: AgeableData::default(),
            variant: AtomicI32::new(MooshroomVariant::DEFAULT.id()),
            stew_effects: std::sync::Mutex::new(None),
            last_lightning_bolt_uuid: AtomicCell::new(None),
        };
        let mob_arc = Arc::new(mooshroom);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(2, BreedGoal::new(1.0));
            goal_selector.add_goal(
                3,
                Box::new(TemptGoal::with_tag(
                    1.25,
                    &pumpkin_data::tag::Item::MINECRAFT_COW_FOOD,
                    false,
                )),
            );
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

    #[must_use]
    pub fn get_variant(&self) -> MooshroomVariant {
        MooshroomVariant::by_id(self.variant.load(Ordering::Relaxed))
    }

    pub fn set_variant(&self, variant: MooshroomVariant) {
        self.variant.store(variant.id(), Ordering::Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::mooshroom::DATA_TYPE,
            VarInt(variant.id()),
        );
    }

    fn take_stew_effects(&self) -> Option<SuspiciousStewEffectsImpl> {
        self.stew_effects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }

    fn has_stew_effects(&self) -> bool {
        self.stew_effects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_some()
    }

    fn get_effects_from_item_stack(item_stack: &ItemStack) -> Option<SuspiciousStewEffectsImpl> {
        get_suspicious_effects(item_stack.item.id).map(|effects| SuspiciousStewEffectsImpl {
            effects: effects
                .iter()
                .map(|effect| SuspiciousStewEffect {
                    effect: effect.effect.minecraft_name.into(),
                    duration: effect.duration,
                })
                .collect(),
        })
    }

    fn offspring_variant(&self, mate: &Self) -> MooshroomVariant {
        let variant = self.get_variant();
        let mate_variant = mate.get_variant();
        let mut rng = self.get_random();
        if variant == mate_variant && rng.random_range(0..MUTATE_CHANCE) == 0 {
            if variant == MooshroomVariant::Brown {
                MooshroomVariant::Red
            } else {
                MooshroomVariant::Brown
            }
        } else if rng.random::<bool>() {
            variant
        } else {
            mate_variant
        }
    }

    fn spawn_feeding_particles(&self, particle: Particle, count: usize) {
        let entity = self.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        let y = entity.get_y(0.5);
        let mut rng = self.get_random();
        for _ in 0..count {
            world.spawn_particle(
                Vector3::new(
                    pos.x + rng.random::<f64>() / 2.0,
                    y,
                    pos.z + rng.random::<f64>() / 2.0,
                ),
                Vector3::new(0.0, (rng.random::<f64>() / 5.0) as f32, 0.0),
                1.0,
                0,
                particle,
            );
        }
    }

    pub fn shear(&self, category: SoundCategory, tool: &ItemStack) {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        world.play_sound(Sound::EntityMooshroomShear, category, &entity.pos.load());
        convert_to(
            self,
            &EntityType::COW,
            ConversionParams::single(false, false),
            |_| {
                let pos = entity.pos.load();
                world.spawn_particle(
                    Vector3::new(pos.x, entity.get_y(0.5), pos.z),
                    Vector3::new(0.0, 0.0, 0.0),
                    0.0,
                    1,
                    Particle::Explosion,
                );
                let Some(table) = world.get_loot_table(SHEAR_MOOSHROOM_LOOT_TABLE) else {
                    return;
                };
                let params = LootContextParameters {
                    world: Some(&world),
                    position: Some(pos),
                    this_entity: Some(self),
                    tool: Some(tool.clone()),
                    ..Default::default()
                };
                let drop_pos = Vector3::new(pos.x, entity.get_y(1.0), pos.z);
                for stack in generate_loot_from_handle(&table, self.get_random().random(), &params)
                {
                    for _ in 0..stack.item_count {
                        let item_entity = ItemEntity::new(
                            Entity::new(world.clone(), drop_pos, &EntityType::ITEM),
                            stack.copy_with_count(1),
                        );
                        item_entity.set_pickup_delay(0);
                        world.spawn_entity(Arc::new(item_entity));
                    }
                }
            },
        );
    }

    fn milk_stew(&self, player: &Player, item_stack: &mut ItemStack) {
        let (stew, sound) = self.take_stew_effects().map_or_else(
            || {
                (
                    ItemStack::new(1, &Item::MUSHROOM_STEW),
                    Sound::EntityMooshroomMilk,
                )
            },
            |effects| {
                let mut stew = ItemStack::new(1, &Item::SUSPICIOUS_STEW);
                stew.patch
                    .push((DataComponent::SuspiciousStewEffects, Some(effects.to_dyn())));
                (stew, Sound::EntityMooshroomSuspiciousMilk)
            },
        );
        create_filled_result(item_stack, player, stew, false);
        self.play_entity_sound(sound, 1.0, 1.0);
    }

    fn try_shear(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load();
        if let Some(server) = world.server.upgrade() {
            let mut event =
                crate::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent {
                    player: player.clone(),
                    entity_id: entity.entity_id,
                    hand: 0,
                    cancelled: false,
                };
            server.plugin_manager.fire_blocking(&server, &mut event);
            if event.cancelled {
                return false;
            }
        }
        self.shear(SoundCategory::Players, item_stack);
        world.emit_game_event_with_context(
            GameEvent::Shear.name(),
            entity.pos.load(),
            &GameEventContext {
                source_entity: Some(player.as_ref()),
            },
        );
        let _ = player.damage_item_stack(item_stack, 1);
        true
    }

    fn feed_flower(&self, player: &Player, item_stack: &mut ItemStack) -> bool {
        let Some(effects) = Self::get_effects_from_item_stack(item_stack) else {
            return false;
        };
        if self.has_stew_effects() {
            self.spawn_feeding_particles(Particle::Smoke, 2);
        } else {
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            *self
                .stew_effects
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(effects);
            self.play_entity_sound(Sound::EntityMooshroomEat, 2.0, 1.0);
        }
        true
    }
}

impl CustomSound for MooshroomEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(CowSoundVariant::Classic.death_sound(self.is_baby()))
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(CowSoundVariant::Classic.hurt_sound(self.is_baby()))
    }

    fn sound_volume(&self) -> f32 {
        SOUND_VOLUME
    }
}

impl AgeableMob for MooshroomEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }
}

impl Animal for MooshroomEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack
            .item
            .has_tag(&pumpkin_data::tag::Item::MINECRAFT_COW_FOOD)
    }

    fn get_breed_offspring(&self, partner: &dyn EntityBase) -> Arc<dyn EntityBase> {
        let entity = self.get_entity();
        let baby = Self::new(Entity::new(
            entity.world.load_full(),
            entity.pos.load(),
            &EntityType::MOOSHROOM,
        ));
        if let Some(partner) = partner.cast_any().downcast_ref::<Self>() {
            baby.set_variant(self.offspring_variant(partner));
        }
        baby
    }

    fn animal_walk_target_value(&self, pos: &BlockPos) -> f32 {
        let world = self.get_entity().world.load();
        if world.get_block(&pos.down()) == &pumpkin_data::Block::MYCELIUM {
            10.0
        } else {
            world.get_pathfinding_cost_from_light_levels(pos)
        }
    }
}

impl Mob for MooshroomEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(
            CowSoundVariant::Classic.step_sound(self.is_baby()),
            0.15,
            1.0,
        );
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(CowSoundVariant::Classic.ambient_sound(self.is_baby()))
    }

    fn mob_on_lightning_strike(&self, _caller: &dyn EntityBase, lightning: &LightningBoltEntity) {
        let lightning_uuid = lightning.get_entity().entity_uuid;
        if self.last_lightning_bolt_uuid.load() != Some(lightning_uuid) {
            self.set_variant(if self.get_variant() == MooshroomVariant::Red {
                MooshroomVariant::Brown
            } else {
                MooshroomVariant::Red
            });
            self.last_lightning_bolt_uuid.store(Some(lightning_uuid));
            self.play_entity_sound(Sound::EntityMooshroomConvert, 2.0, 1.0);
        }
    }

    fn mob_get_component(&self, component: DataComponent) -> Option<Box<dyn DataComponentImpl>> {
        match component {
            DataComponent::MooshroomVariant => Some(Box::new(MooshroomVariantImpl {
                value: self.get_variant().name().into(),
            })),
            _ => None,
        }
    }

    fn mob_set_variant_name(&self, name: &str) {
        if let Some(variant) = MooshroomVariant::from_name(name) {
            self.set_variant(variant);
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

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("Type", self.get_variant().name().to_string());
        if let Some(effects) = &*self
            .stew_effects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
        {
            nbt.put("stew_effects", effects.write_data());
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_variant(
            nbt.get_string("Type")
                .and_then(MooshroomVariant::from_name)
                .unwrap_or(MooshroomVariant::DEFAULT),
        );
        *self
            .stew_effects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = nbt
            .get("stew_effects")
            .and_then(SuspiciousStewEffectsImpl::read_data);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        if self.is_baby() {
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
            self.milk_stew(player, item_stack);
            return true;
        }

        if item == &Item::SHEARS && !self.is_baby() {
            return self.try_shear(player, item_stack);
        }

        if self.get_variant() == MooshroomVariant::Brown
            && !self.is_baby()
            && self.feed_flower(player, item_stack)
        {
            return true;
        }

        if item == &Item::BUCKET && !self.is_baby() {
            self.get_entity().world.load().play_sound_expect(
                player,
                Sound::EntityCowMilk,
                SoundCategory::Players,
                &player.position(),
            );
            fill_bucket_stack(item_stack, player, &Item::MILK_BUCKET);
            return true;
        }

        self.animal_interact(player, item_stack)
    }
}
