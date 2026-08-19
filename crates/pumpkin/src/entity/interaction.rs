use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::Mutex;
use uuid::Uuid;

use pumpkin_data::{damage::DamageType, item_stack::ItemStack};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::{
    boundingbox::{BoundingBox, EntityDimensions},
    vector3::Vector3,
};

use crate::{
    entity::{
        Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
        player::Player,
    },
    server::Server,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerAction {
    pub player: Uuid,
    pub timestamp: i64,
}

impl PlayerAction {
    #[must_use]
    pub fn to_nbt(&self) -> NbtCompound {
        let mut compound = NbtCompound::new();
        compound.put_uuid("player", self.player);
        compound.put_long("timestamp", self.timestamp);
        compound
    }

    #[must_use]
    pub fn from_nbt(compound: &NbtCompound) -> Option<Self> {
        let player = compound.get_uuid("player")?;
        let timestamp = compound.get_long("timestamp")?;
        Some(Self { player, timestamp })
    }
}

pub struct InteractionEntity {
    pub entity: Entity,
    pub width: Mutex<f32>,
    pub height: Mutex<f32>,
    pub response: AtomicBool,
    pub attack: Mutex<Option<PlayerAction>>,
    pub interaction: Mutex<Option<PlayerAction>>,
}

impl InteractionEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        entity.no_clip.store(true, Ordering::Relaxed);
        let width = 1.0;
        let height = 1.0;
        let dimensions = EntityDimensions::new(width, height, height * 0.85);
        let pos = entity.pos.load();
        let aabb = BoundingBox::new_from_pos(pos.x, pos.y, pos.z, &dimensions);
        entity.bounding_box.store(aabb);

        Arc::new(Self {
            entity,
            width: Mutex::new(width),
            height: Mutex::new(height),
            response: AtomicBool::new(false),
            attack: Mutex::new(None),
            interaction: Mutex::new(None),
        })
    }

    pub async fn get_width(&self) -> f32 {
        *self.width.lock().await
    }

    pub async fn set_width(&self, width: f32) {
        *self.width.lock().await = width;
        self.update_dimensions().await;
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::interaction::WIDTH,
                width,
            )],
            None,
        );
    }

    pub async fn get_height(&self) -> f32 {
        *self.height.lock().await
    }

    pub async fn set_height(&self, height: f32) {
        *self.height.lock().await = height;
        self.update_dimensions().await;
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::interaction::HEIGHT,
                height,
            )],
            None,
        );
    }

    pub fn get_response(&self) -> bool {
        self.response.load(Ordering::Relaxed)
    }

    pub fn set_response(&self, response: bool) {
        self.response.store(response, Ordering::Relaxed);
        self.entity.send_meta_data(
            &[Metadata::new(
                pumpkin_data::tracked_data::interaction::RESPONSE,
                response,
            )],
            None,
        );
    }

    pub async fn update_dimensions(&self) {
        let width = *self.width.lock().await;
        let height = *self.height.lock().await;
        let dimensions = EntityDimensions::new(width, height, height * 0.85);
        let pos = self.entity.pos.load();
        let aabb = BoundingBox::new_from_pos(pos.x, pos.y, pos.z, &dimensions);
        self.entity.bounding_box.store(aabb);
    }

    pub async fn get_last_attacker(&self) -> Option<PlayerAction> {
        *self.attack.lock().await
    }

    pub async fn get_target(&self) -> Option<PlayerAction> {
        *self.interaction.lock().await
    }
}

impl NBTStorage for InteractionEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.write_nbt(nbt).await;
            nbt.put_float("width", *self.width.lock().await);
            nbt.put_float("height", *self.height.lock().await);
            nbt.put_bool("response", self.response.load(Ordering::Relaxed));

            let attack = *self.attack.lock().await;
            if let Some(attack) = attack {
                nbt.put("attack", NbtTag::Compound(attack.to_nbt()));
            }

            let interaction = *self.interaction.lock().await;
            if let Some(interaction) = interaction {
                nbt.put("interaction", NbtTag::Compound(interaction.to_nbt()));
            }
        })
    }

    fn read_nbt<'a>(&'a mut self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.read_nbt_non_mut(nbt).await;
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.read_nbt_non_mut(nbt).await;

            let width = nbt.get_float("width").unwrap_or(1.0);
            let height = nbt.get_float("height").unwrap_or(1.0);
            let response = nbt.get_bool("response").unwrap_or(false);

            *self.width.lock().await = width;
            *self.height.lock().await = height;
            self.response.store(response, Ordering::Relaxed);
            self.update_dimensions().await;

            if let Some(attack_compound) = nbt.get_compound("attack") {
                *self.attack.lock().await = PlayerAction::from_nbt(attack_compound);
            } else {
                *self.attack.lock().await = None;
            }

            if let Some(interaction_compound) = nbt.get_compound("interaction") {
                *self.interaction.lock().await = PlayerAction::from_nbt(interaction_compound);
            } else {
                *self.interaction.lock().await = None;
            }
        })
    }
}

impl EntityBase for InteractionEntity {
    fn tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
        _server: &'a Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {})
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let width = *self.width.lock().await;
            let height = *self.height.lock().await;
            let response = self.response.load(Ordering::Relaxed);

            self.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::interaction::WIDTH,
                    width,
                )],
                None,
            );
            self.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::interaction::HEIGHT,
                    height,
                )],
                None,
            );
            self.entity.send_meta_data(
                &[Metadata::new(
                    pumpkin_data::tracked_data::interaction::RESPONSE,
                    response,
                )],
                None,
            );
        })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }

    fn is_pushable(&self) -> bool {
        false
    }

    fn is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn can_hit(&self) -> bool {
        true
    }

    fn is_immune_to_explosion(&self) -> bool {
        true
    }

    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let player = source
                .or(cause)
                .and_then(|e| e.cast_any().downcast_ref::<Player>());
            if let Some(player) = player {
                let timestamp = self.entity.world.load().level_time.lock().await.world_age as i64;
                *self.attack.lock().await = Some(PlayerAction {
                    player: player.gameprofile.id,
                    timestamp,
                });
            }
            false
        })
    }

    fn interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        _item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            let timestamp = self.entity.world.load().level_time.lock().await.world_age as i64;
            *self.interaction.lock().await = Some(PlayerAction {
                player: player.gameprofile.id,
                timestamp,
            });
            true
        })
    }
}
