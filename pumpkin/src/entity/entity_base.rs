use super::Entity;
use super::living::LivingEntity;
use super::player::Player;
use crate::{entity::item::ItemEntity, net::bedrock::BedrockClient, server::Server, world::World};
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::bedrock::client::CAddActor;
use pumpkin_protocol::bedrock::client::set_actor_data::{
    EntityMetadata, PropertySyncData, entity_data_flag, entity_data_key,
};
use pumpkin_protocol::codec::var_long::VarLong;
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::hover::HoverEvent;
use std::pin::Pin;
use std::sync::{
    Arc,
    atomic::Ordering::{self, Relaxed},
};

pub type EntityBaseFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub type TeleportFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub trait EntityBase: Send + Sync + NBTStorage + std::any::Any {
    /// Called every tick for this entity.
    ///
    /// The `caller` parameter is a reference to the entity that initiated the tick.
    /// This can be the same entity the method is being called on (`self`),
    /// but in some scenarios (e.g., interactions or events), it might be a different entity.
    ///
    /// The `server` parameter provides access to the game server instance.
    fn tick<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if let Some(living) = self.get_living_entity() {
                living.tick(caller, server).await;
            } else {
                self.get_entity().tick(caller, server).await;
            }
        })
    }

    fn get_job_site_pos(&self) -> Option<pumpkin_util::math::position::BlockPos> {
        None
    }

    fn get_home_pos(&self) -> Option<pumpkin_util::math::position::BlockPos> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any
    where
        Self: Sized,
    {
        self
    }

    fn get_eye_pos(&self) -> Vector3<f64> {
        self.get_entity().get_eye_pos()
    }

    fn get_looking_vector(&self) -> Vector3<f64> {
        let entity = self.get_entity();
        Vector3::from_yaw_pitch(entity.yaw.load(), entity.pitch.load())
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();

            // If the internal age is negative, it's a baby
            let is_baby = entity.age.load(Ordering::Relaxed) < 0;

            if is_baby {
                let mut bedrock_meta = EntityMetadata::new();
                bedrock_meta.set_flag(entity_data_key::FLAGS, entity_data_flag::BABY as u8, true);
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    Some(&bedrock_meta),
                );
            }
        })
    }
    fn set_variant_name(&self, _name: &str) {}

    // This method takes ownership of Arc<Self>, so the lifetime bounds are different.
    fn teleport(
        self: Arc<Self>,
        position: Vector3<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        world: Arc<World>,
    ) -> TeleportFuture
    where
        Self: 'static,
    {
        Box::pin(async move {
            self.get_entity().teleport(position, yaw, pitch, world);
        })
    }

    fn is_pushed_by_fluids(&self) -> bool {
        true
    }

    /// Whether the entity is immune from explosion knockback and damage
    fn is_immune_to_explosion(&self) -> bool {
        false
    }

    fn get_gravity(&self) -> f64 {
        0.0
    }

    fn tick_in_void<'a>(&'a self, _dyn_self: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move { self.get_entity().remove().await })
    }

    /// Returns if damage was successful or not
    fn damage<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            caller
                .damage_with_context(caller, amount, damage_type, None, None, None)
                .await
        })
    }

    fn is_spectator(&self) -> bool {
        false
    }

    fn is_collidable(&self, _entity: Option<Box<dyn EntityBase>>) -> bool {
        false
    }

    fn can_hit(&self) -> bool {
        false
    }

    fn is_flutterer(&self) -> bool {
        false
    }

    /// Custom Y-axis velocity drag multiplier applied during `travel_in_air`.
    /// Bats return `Some(0.6)` to match vanilla's `travel()` override.
    fn get_y_velocity_drag(&self) -> Option<f64> {
        None
    }

    fn send_bedrock_spawn_packet<'a>(
        &'a self,
        client: &'a BedrockClient,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let runtime_id = entity.entity_id as u64;
            let packet = CAddActor::new(
                VarLong(runtime_id as i64),
                VarULong(runtime_id),
                self.get_entity().entity_type.resource_name.to_string(),
                entity.pos.load().to_f32_lossy(),
                entity.velocity.load().to_f32_lossy(),
                entity.pitch.load(),
                entity.yaw.load(),
                entity.head_yaw.load(),
                entity.body_yaw.load(),
                Vec::new(),
                entity.bedrock_metadata(),
                PropertySyncData {
                    int_properties: std::collections::HashMap::new(),
                    float_properties: std::collections::HashMap::new(),
                },
                Vec::new(),
            );
            client.send_game_packet(&packet).await;
        })
    }

    fn damage_with_context<'a>(
        &'a self,
        caller: &'a dyn EntityBase,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if caller.get_living_entity().is_some() {
                return caller
                    .damage_with_context(caller, amount, damage_type, position, source, cause)
                    .await;
            }
            false
        })
    }

    /// Called when a player right-clicks this entity with an item.
    /// Returns true if the interaction was handled.
    fn interact<'a>(
        &'a self,
        _player: &'a Arc<Player>,
        _item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async { false })
    }

    fn set_on_fire_for(&self, seconds: f32) {
        let entity = self.get_entity();
        // Exclude fire-immune entities (ex. certain items) from burn damage
        if !entity.fire_immune.load(Ordering::Relaxed) {
            self.set_on_fire_for_ticks((seconds * 20.0).floor() as u32);
        }
    }

    fn set_on_fire_for_ticks(&self, ticks: u32) {
        let entity = self.get_entity();
        if entity.fire_ticks.load(Ordering::Relaxed) < ticks as i32 {
            entity.fire_ticks.store(ticks as i32, Ordering::Relaxed);
        }
        // TODO: defrost
    }

    /// Called when a player collides with a entity
    fn on_player_collision<'a>(&'a self, _player: &'a Arc<Player>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    fn is_passenger(&self) -> EntityBaseFuture<'_, bool> {
        Box::pin(async move { self.get_entity().has_vehicle().await })
    }

    fn is_vehicle(&self) -> EntityBaseFuture<'_, bool> {
        Box::pin(async move { self.get_entity().has_passengers().await })
    }

    fn has_passenger<'a>(&'a self, other: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            self.get_entity()
                .passengers
                .lock()
                .await
                .iter()
                .any(|p| p.get_entity().entity_id == other.get_entity().entity_id)
        })
    }

    fn move_entity<'a>(
        &'a self,
        caller: &'a Arc<dyn EntityBase>,
        motion: Vector3<f64>,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.get_entity().move_entity(caller, motion).await;
        })
    }

    fn is_pushable(&self) -> bool {
        false
    }

    fn push<'a>(&'a self, entity: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let self_entity = self.get_entity();
            let other_entity = entity.get_entity();

            if self_entity.no_clip.load(Ordering::Relaxed)
                || other_entity.no_clip.load(Ordering::Relaxed)
            {
                return;
            }

            {
                let passengers = self_entity.passengers.lock().await;
                if passengers
                    .iter()
                    .any(|p| p.get_entity().entity_id == other_entity.entity_id)
                {
                    return;
                }
            }
            {
                let passengers = other_entity.passengers.lock().await;
                if passengers
                    .iter()
                    .any(|p| p.get_entity().entity_id == self_entity.entity_id)
                {
                    return;
                }
            }

            let mut dx = other_entity.pos.load().x - self_entity.pos.load().x;
            let mut dz = other_entity.pos.load().z - self_entity.pos.load().z;
            let mut d = dx.abs().max(dz.abs());
            if d >= 0.01 {
                d = d.sqrt();
                dx /= d;
                dz /= d;
                let mut d2 = 1.0 / d;
                if d2 > 1.0 {
                    d2 = 1.0;
                }
                dx *= d2;
                dz *= d2;
                dx *= 0.05;
                dz *= 0.05;

                if !self_entity.has_passengers().await && self.is_pushable() {
                    let mut vel = self_entity.velocity.load();
                    vel.x -= dx;
                    vel.z -= dz;
                    self_entity.velocity.store(vel);
                    self_entity.send_velocity();
                }

                if !other_entity.has_passengers().await && entity.is_pushable() {
                    let mut vel = other_entity.velocity.load();
                    vel.x += dx;
                    vel.z += dz;
                    other_entity.velocity.store(vel);
                    other_entity.send_velocity();
                }
            }
        })
    }

    #[allow(clippy::too_many_lines)]
    fn push_entities<'a>(
        &'a self,
        dyn_self: &'a Arc<dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let mut picked_up = false;
            let mut pushed = false;
            let self_entity = self.get_entity();
            let entity_bb = self_entity.bounding_box.load();

            if !self.is_pushable() {
                return false;
            }

            let world = self_entity.world.load();

            let is_rideable_minecart = self_entity.entity_type.id == EntityType::MINECART.id;
            let is_abstract_minecart = is_rideable_minecart
                || self_entity.entity_type.id == EntityType::CHEST_MINECART.id
                || self_entity.entity_type.id == EntityType::COMMAND_BLOCK_MINECART.id
                || self_entity.entity_type.id == EntityType::FURNACE_MINECART.id
                || self_entity.entity_type.id == EntityType::HOPPER_MINECART.id
                || self_entity.entity_type.id == EntityType::SPAWNER_MINECART.id
                || self_entity.entity_type.id == EntityType::TNT_MINECART.id;

            let is_minecart_fn = |id| -> bool {
                id == EntityType::MINECART.id
                    || id == EntityType::CHEST_MINECART.id
                    || id == EntityType::COMMAND_BLOCK_MINECART.id
                    || id == EntityType::FURNACE_MINECART.id
                    || id == EntityType::HOPPER_MINECART.id
                    || id == EntityType::SPAWNER_MINECART.id
                    || id == EntityType::TNT_MINECART.id
            };

            if is_abstract_minecart {
                let is_vehicle = self.is_vehicle().await;

                if is_rideable_minecart && !is_vehicle {
                    let pickup_bb = entity_bb.expand(0.2, 0.0, 0.2);
                    let other_entities = world.get_entities_at_box(&pickup_bb);

                    for other in other_entities {
                        if other.get_entity().entity_id != self_entity.entity_id {
                            let other_type = other.get_entity().entity_type.id;
                            let is_iron_golem = other_type == EntityType::IRON_GOLEM.id;
                            let is_other_minecart = is_minecart_fn(other_type);

                            if !is_iron_golem
                                && !is_other_minecart
                                && !other.is_passenger().await
                                && other.is_pushable()
                                && other.get_entity().riding_cooldown.load(Relaxed) == 0
                            {
                                dyn_self
                                    .get_entity()
                                    .add_passenger(dyn_self.clone(), other.clone())
                                    .await;
                                picked_up = true;
                                break;
                            }
                        }
                    }
                }

                let push_bb = entity_bb.expand(1.0e-7, 1.0e-7, 1.0e-7);

                let other_entities = world.get_entities_at_box(&push_bb);
                for other in other_entities {
                    if other.get_entity().entity_id != self_entity.entity_id {
                        let other_type = other.get_entity().entity_type.id;
                        let is_other_minecart = is_minecart_fn(other_type);
                        let is_iron_golem = other_type == EntityType::IRON_GOLEM.id;

                        if is_rideable_minecart {
                            if (is_iron_golem
                                || is_other_minecart
                                || is_vehicle
                                || !other.get_entity().has_vehicle().await)
                                && other.is_pushable()
                            {
                                dyn_self.push(&other).await;
                                pushed = true;
                            }
                        } else if !self.has_passenger(&other).await
                            && other.is_pushable()
                            && is_other_minecart
                        {
                            dyn_self.push(&other).await;
                            pushed = true;
                        }
                    }
                }

                let players = world.get_players_at_box(&push_bb);
                for player in players {
                    if player.get_entity().entity_id != self_entity.entity_id
                        && is_rideable_minecart
                    {
                        let player_base: Arc<dyn EntityBase> = player.clone();
                        dyn_self.push(&player_base).await;
                        pushed = true;
                        // Non-rideable minecarts (hoppers, chests) do not push players in vanilla.
                    }
                }
            } else {
                let other_entities = world.get_entities_at_box(&entity_bb);
                for other in other_entities {
                    if other.get_entity().entity_id != self_entity.entity_id {
                        dyn_self.push(&other).await;
                        pushed = true;
                    }
                }

                let players = world.get_players_at_box(&entity_bb);
                for player in players {
                    if player.get_entity().entity_id != self_entity.entity_id {
                        let player_base: Arc<dyn EntityBase> = player.clone();
                        dyn_self.push(&player_base).await;
                        pushed = true;
                    }
                }
            }

            picked_up && !pushed
        })
    }

    fn on_hit(&self, _hit: crate::entity::projectile::ProjectileHit) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {})
    }

    fn set_paddle_state(&self, _left: bool, _right: bool) -> EntityBaseFuture<'_, ()> {
        Box::pin(async {})
    }

    fn is_in_love(&self) -> bool {
        false
    }

    fn is_breeding_ready(&self) -> bool {
        false
    }

    fn reset_love(&self) {}

    fn set_breeding_cooldown(&self, _ticks: i32) {}

    fn is_panicking(&self) -> bool {
        false
    }

    fn get_entity(&self) -> &Entity;

    fn get_living_entity(&self) -> Option<&LivingEntity>;

    fn cast_any(&self) -> &dyn std::any::Any;

    fn get_item_entity(self: Arc<Self>) -> Option<Arc<ItemEntity>> {
        None
    }

    fn get_player(&self) -> Option<&Player> {
        None
    }

    /// Should return the name of the entity without click or hover events.
    fn get_name(&self) -> TextComponent {
        let entity = self.get_entity();
        entity
            .custom_name
            .load()
            .as_ref()
            .clone()
            .unwrap_or(TextComponent::translate_cross(
                format!("entity.minecraft.{}", entity.entity_type.resource_name),
                format!("entity.minecraft.{}", entity.entity_type.resource_name),
                [],
            ))
    }

    fn get_display_name(&self) -> EntityBaseFuture<'_, TextComponent> {
        Box::pin(async move {
            // TODO: team color
            let entity = self.get_entity();
            let mut name = entity.custom_name.load().as_ref().clone().unwrap_or(
                TextComponent::translate_cross(
                    format!("entity.minecraft.{}", entity.entity_type.resource_name),
                    format!("entity.minecraft.{}", entity.entity_type.resource_name),
                    [],
                ),
            );
            let name_clone = name.clone();
            name = name.hover_event(HoverEvent::show_entity(
                entity.entity_uuid.to_string(),
                entity.entity_type.resource_name.into(),
                Some(name_clone),
            ));
            name = name.insertion(entity.entity_uuid.to_string());
            name
        })
    }

    /// Kills the Entity.
    fn kill<'a>(&'a self, caller: &'a dyn EntityBase) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.get_living_entity().is_some() {
                caller
                    .damage(caller, f32::MAX, DamageType::GENERIC_KILL)
                    .await;
            } else {
                // TODO this should be removed once all entities are implemented
                self.get_entity().remove().await;
            }
        })
    }

    /// Returns itself as the nbt storage for saving and loading data.
    fn as_nbt_storage(&self) -> &dyn NBTStorage;

    fn get_experience_reward(&self, _killer: Option<&dyn EntityBase>) -> u32 {
        0
    }

    fn get_base_experience_reward(&self) -> u32 {
        0
    }
}

pub type NbtFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait NBTStorage: Send + Sync {
    fn write_nbt<'a>(&'a self, _nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {})
    }

    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.read_nbt_non_mut(nbt).await;
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, _nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {})
    }
}

pub type NBTInitFuture<'a, T> = Pin<Box<dyn Future<Output = Option<T>> + Send + 'a>>;

pub trait NBTStorageInit: Send + Sync + Sized {
    fn create_from_nbt<'a>(_nbt: &'a mut NbtCompound) -> NBTInitFuture<'a, Self>
    where
        Self: 'a,
    {
        Box::pin(async move { None })
    }
}
