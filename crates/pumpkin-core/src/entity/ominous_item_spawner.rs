use std::sync::{
    Arc, Mutex,
    atomic::{AtomicI64, Ordering},
};

use pumpkin_data::{
    damage::DamageType,
    entity::EntityType,
    game_event::GameEvent,
    item::Item,
    item_stack::ItemStack,
    sound::{Sound, SoundCategory},
    tracked_data,
    world::WorldEvent,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::{
    entity::{
        Entity, EntityBase,
        item::ItemEntity,
        living::LivingEntity,
        projectile::{
            ThrownItemEntity,
            arrow::{ArrowEntity, ArrowPickup},
            egg::EggEntity,
            experience_bottle::ExperienceBottleEntity,
            firework_rocket::FireworkRocketEntity,
            lingering_potion::LingeringPotionEntity,
            owner::ProjectileOwner,
            small_fireball::SmallFireballEntity,
            snowball::SnowballEntity,
            splash_potion::SplashPotionEntity,
            trident::TridentEntity,
            wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity},
        },
    },
    server::Server,
    world::{World, game_event::GameEventContext},
};

const SPAWN_ITEM_DELAY_MIN: i64 = 60;
const SPAWN_ITEM_DELAY_MAX: i64 = 120;
const TAG_SPAWN_ITEM_AFTER_TICKS: &str = "spawn_item_after_ticks";
const TAG_ITEM: &str = "item";
pub const TICKS_BEFORE_ABOUT_TO_SPAWN_SOUND: i64 = 36;

const DEFAULT_PROJECTILE_POWER: f64 = 1.1;
const DEFAULT_PROJECTILE_UNCERTAINTY: f64 = 6.0;
const POTION_PROJECTILE_POWER: f64 = 1.375;
const POTION_PROJECTILE_UNCERTAINTY: f64 = 3.0;
const CHARGE_PROJECTILE_POWER: f64 = 1.0;
const CHARGE_PROJECTILE_UNCERTAINTY: f64 = 6.666_666_5;
const FIREWORK_PROJECTILE_POWER: f64 = 0.5;
const FIREWORK_PROJECTILE_UNCERTAINTY: f64 = 1.0;

pub struct OminousItemSpawnerEntity {
    pub entity: Entity,
    item_stack: Mutex<ItemStack>,
    spawn_item_after_ticks: AtomicI64,
}

impl OminousItemSpawnerEntity {
    #[must_use]
    pub fn new(entity: Entity) -> Arc<Self> {
        entity.no_physics.store(true, Ordering::Relaxed);
        Arc::new(Self {
            entity,
            item_stack: Mutex::new(ItemStack::EMPTY.clone()),
            spawn_item_after_ticks: AtomicI64::new(0),
        })
    }

    #[must_use]
    pub fn create(world: Arc<World>, pos: Vector3<f64>, item: ItemStack) -> Arc<Self> {
        let entity = Entity::new(world, pos, &EntityType::OMINOUS_ITEM_SPAWNER);
        let spawner = Self::new(entity);
        let delay = rand::rng().random_range(SPAWN_ITEM_DELAY_MIN..=SPAWN_ITEM_DELAY_MAX);
        spawner
            .spawn_item_after_ticks
            .store(delay, Ordering::Relaxed);
        spawner.set_item(item);
        spawner
    }

    #[must_use]
    pub fn get_item(&self) -> ItemStack {
        self.item_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn set_item(&self, item: ItemStack) {
        self.entity.set_synced_data(
            tracked_data::ominous_item_spawner::DATA_ITEM,
            ItemStackSerializer::from(item.clone()),
        );
        *self
            .item_stack
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = item;
    }

    fn spawn_item(&self) {
        let item = self.get_item();
        if item.is_empty() {
            return;
        }

        let world = self.entity.world.load();
        let pos = self.entity.pos.load();
        let spawned_entity = self
            .spawn_projectile(&world, pos, &item)
            .unwrap_or_else(|| {
                let item_entity = Arc::new(ItemEntity::new(
                    Entity::new(world.clone(), pos, &EntityType::ITEM),
                    item,
                ));
                world.spawn_entity(item_entity.clone());
                item_entity as Arc<dyn EntityBase>
            });

        world.sync_world_event(
            WorldEvent::ParticlesTrialSpawnerSpawnItem,
            self.entity.block_pos.load(),
            1,
        );
        world.emit_game_event_with_context(
            GameEvent::EntityPlace.name(),
            pos,
            &GameEventContext {
                source_entity: Some(spawned_entity.as_ref()),
            },
        );
        self.set_item(ItemStack::EMPTY.clone());
    }

    /// Mirrors `OminousItemSpawner.spawnProjectile`: shoots downward `(0, -1, 0)` using each
    /// `ProjectileItem`'s `DispenseConfig` and assigns `this` as owner.
    fn spawn_projectile(
        &self,
        world: &Arc<World>,
        pos: Vector3<f64>,
        item: &ItemStack,
    ) -> Option<Arc<dyn EntityBase>> {
        let id = item.item.id;
        if id == Item::ARROW.id || id == Item::TIPPED_ARROW.id || id == Item::SPECTRAL_ARROW.id {
            let entity = Entity::new(
                world.clone(),
                pos,
                ArrowEntity::entity_type_for_item(item.item),
            );
            let arrow =
                ArrowEntity::new_with_item(entity, Some(&self.entity), item, ArrowPickup::Allowed);
            arrow.set_velocity(
                0.0,
                -1.0,
                0.0,
                DEFAULT_PROJECTILE_POWER,
                DEFAULT_PROJECTILE_UNCERTAINTY,
            );
            let arc: Arc<dyn EntityBase> = Arc::new(arrow);
            world.spawn_entity(arc.clone());
            if let Some(arrow_ref) = arc.cast_any().downcast_ref::<ArrowEntity>() {
                arrow_ref.apply_on_projectile_spawned(item);
            }
            Some(arc)
        } else if id == Item::TRIDENT.id {
            let entity = Entity::new(world.clone(), pos, &EntityType::TRIDENT);
            let trident = TridentEntity::new_with_item(
                entity,
                Some(&self.entity),
                item.copy_with_count(1),
                ArrowPickup::Allowed,
            );
            trident.set_velocity(
                0.0,
                -1.0,
                0.0,
                DEFAULT_PROJECTILE_POWER,
                DEFAULT_PROJECTILE_UNCERTAINTY,
            );
            let arc: Arc<dyn EntityBase> = Arc::new(trident);
            world.spawn_entity(arc.clone());
            if let Some(trident_ref) = arc.cast_any().downcast_ref::<TridentEntity>() {
                trident_ref.apply_on_projectile_spawned(item);
            }
            Some(arc)
        } else {
            self.spawn_thrown_projectile(world, pos, item)
                .or_else(|| self.spawn_charge_or_rocket_projectile(world, pos, item))
        }
    }

    fn spawn_thrown_projectile(
        &self,
        world: &Arc<World>,
        pos: Vector3<f64>,
        item: &ItemStack,
    ) -> Option<Arc<dyn EntityBase>> {
        let id = item.item.id;
        let arc: Arc<dyn EntityBase> = if id == Item::SNOWBALL.id {
            let mut snowball =
                SnowballEntity::new(Entity::new(world.clone(), pos, &EntityType::SNOWBALL));
            self.aim_thrown_downward(
                &mut snowball.thrown,
                DEFAULT_PROJECTILE_POWER,
                DEFAULT_PROJECTILE_UNCERTAINTY,
            );
            Arc::new(snowball)
        } else if id == Item::EGG.id || id == Item::BLUE_EGG.id || id == Item::BROWN_EGG.id {
            let mut egg = EggEntity::new(Entity::new(world.clone(), pos, &EntityType::EGG));
            egg.set_item_stack(item.clone());
            self.aim_thrown_downward(
                &mut egg.thrown,
                DEFAULT_PROJECTILE_POWER,
                DEFAULT_PROJECTILE_UNCERTAINTY,
            );
            Arc::new(egg)
        } else if id == Item::SPLASH_POTION.id {
            let mut potion = SplashPotionEntity::new(Entity::new(
                world.clone(),
                pos,
                &EntityType::SPLASH_POTION,
            ));
            potion.set_item_stack(item.clone());
            self.aim_thrown_downward(
                &mut potion.thrown,
                POTION_PROJECTILE_POWER,
                POTION_PROJECTILE_UNCERTAINTY,
            );
            Arc::new(potion)
        } else if id == Item::LINGERING_POTION.id {
            let mut potion = LingeringPotionEntity::new(Entity::new(
                world.clone(),
                pos,
                &EntityType::LINGERING_POTION,
            ));
            potion.set_item_stack(item.clone());
            self.aim_thrown_downward(
                &mut potion.thrown,
                POTION_PROJECTILE_POWER,
                POTION_PROJECTILE_UNCERTAINTY,
            );
            Arc::new(potion)
        } else if id == Item::EXPERIENCE_BOTTLE.id {
            let mut bottle = ExperienceBottleEntity::new(Entity::new(
                world.clone(),
                pos,
                &EntityType::EXPERIENCE_BOTTLE,
            ));
            bottle.set_item_stack(item.clone());
            self.aim_thrown_downward(
                &mut bottle.thrown,
                POTION_PROJECTILE_POWER,
                POTION_PROJECTILE_UNCERTAINTY,
            );
            Arc::new(bottle)
        } else {
            return None;
        };

        world.spawn_entity(arc.clone());
        Some(arc)
    }

    fn spawn_charge_or_rocket_projectile(
        &self,
        world: &Arc<World>,
        pos: Vector3<f64>,
        item: &ItemStack,
    ) -> Option<Arc<dyn EntityBase>> {
        let block_pos = self.entity.block_pos.load();
        let id = item.item.id;
        let arc: Arc<dyn EntityBase> = if id == Item::FIREWORK_ROCKET.id {
            world.sync_world_event(WorldEvent::SoundFireworkShoot, block_pos, 0);
            let rocket = FireworkRocketEntity::new_with_owner(
                Entity::new(world.clone(), pos, &EntityType::FIREWORK_ROCKET),
                &self.entity,
                item.copy_with_count(1),
            );
            rocket.set_shot_at_angle(true);
            rocket.entity.set_velocity(
                0.0,
                -1.0,
                0.0,
                FIREWORK_PROJECTILE_POWER,
                FIREWORK_PROJECTILE_UNCERTAINTY,
            );
            Arc::new(rocket)
        } else if id == Item::FIRE_CHARGE.id {
            world.sync_world_event(WorldEvent::SoundBlazeFireball, block_pos, 0);
            let mut fireball = SmallFireballEntity::new(Entity::new(
                world.clone(),
                pos,
                &EntityType::SMALL_FIREBALL,
            ));
            fireball.thrown.projectile.owner = ProjectileOwner::new(Some(&self.entity));
            fireball.thrown.set_velocity(
                0.0,
                -1.0,
                0.0,
                CHARGE_PROJECTILE_POWER,
                CHARGE_PROJECTILE_UNCERTAINTY,
            );
            Arc::new(fireball)
        } else if id == Item::WIND_CHARGE.id {
            world.sync_world_event(WorldEvent::SoundWindChargeShoot, block_pos, 0);
            let thrown = ThrownItemEntity::owned(
                Entity::new(world.clone(), pos, &EntityType::WIND_CHARGE),
                &self.entity,
                WIND_CHARGE_GRAVITY,
            );
            thrown.set_velocity(
                0.0,
                -1.0,
                0.0,
                CHARGE_PROJECTILE_POWER,
                CHARGE_PROJECTILE_UNCERTAINTY,
            );
            Arc::new(WindChargeEntity::new_normal(thrown))
        } else {
            return None;
        };

        world.spawn_entity(arc.clone());
        Some(arc)
    }

    fn aim_thrown_downward(&self, thrown: &mut ThrownItemEntity, power: f64, uncertainty: f64) {
        thrown.owner = ProjectileOwner::new(Some(&self.entity));
        thrown.set_velocity(0.0, -1.0, 0.0, power, uncertainty);
    }
}

impl EntityBase for OminousItemSpawnerEntity {
    fn init_data_tracker(&self) {
        let item = self.get_item();
        if !item.is_empty() {
            self.entity.set_synced_data(
                tracked_data::ominous_item_spawner::DATA_ITEM,
                ItemStackSerializer::from(item),
            );
        }
    }

    fn tick(&self, caller: &dyn EntityBase, server: &Server) {
        self.entity.tick(caller, server);
        if self.entity.is_removed() {
            return;
        }

        let tick_count = i64::from(self.entity.tick_count.load(Ordering::Relaxed));
        let spawn_after = self.spawn_item_after_ticks.load(Ordering::Relaxed);
        if tick_count == spawn_after - TICKS_BEFORE_ABOUT_TO_SPAWN_SOUND {
            let world = self.entity.world.load();
            world.play_sound(
                Sound::BlockTrialSpawnerAboutToSpawnItem,
                SoundCategory::Neutral,
                &self.entity.block_pos.load().to_centered_f64(),
            );
        }

        if tick_count >= spawn_after {
            self.spawn_item();
            self.entity.remove();
        }
    }

    fn write_custom_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(item_nbt) = self.get_item().to_nbt() {
            nbt.put_compound(TAG_ITEM, item_nbt);
        }
        nbt.put_long(
            TAG_SPAWN_ITEM_AFTER_TICKS,
            self.spawn_item_after_ticks.load(Ordering::Relaxed),
        );
    }

    fn read_custom_nbt(&self, nbt: &NbtCompound) {
        let item = nbt
            .get_compound(TAG_ITEM)
            .and_then(ItemStack::read_item_stack)
            .unwrap_or_else(|| ItemStack::EMPTY.clone());
        self.set_item(item);

        let ticks = nbt
            .get_long(TAG_SPAWN_ITEM_AFTER_TICKS)
            .or_else(|| nbt.get_int(TAG_SPAWN_ITEM_AFTER_TICKS).map(i64::from))
            .unwrap_or(0);
        self.spawn_item_after_ticks.store(ticks, Ordering::Relaxed);
    }

    fn is_ignoring_block_triggers(&self) -> bool {
        true
    }

    fn is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn is_immune_to_explosion(&self) -> bool {
        true
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&dyn EntityBase>,
        _cause: Option<&dyn EntityBase>,
    ) -> bool {
        false
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
