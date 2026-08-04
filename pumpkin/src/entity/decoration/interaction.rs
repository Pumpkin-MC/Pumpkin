use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::Mutex;
use uuid::Uuid;

use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture, living::LivingEntity,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::vector3::Vector3;

const DEFAULT_WIDTH: f32 = 1.0;
const DEFAULT_HEIGHT: f32 = 1.0;

fn scaled_dimensions(width: f32, height: f32) -> EntityDimensions {
    EntityDimensions::new(width, height, height * 0.85)
}

#[derive(Clone, Copy)]
struct PlayerAction {
    player: Uuid,
    timestamp: i64,
}

/// Invisible, non-physical entity used to attach a custom hitbox to something else (e.g. a display entity).
///
/// It cannot take damage; it only records that it was hit or right-clicked so the
/// thing that spawned it can react.
pub struct InteractionEntity {
    entity: Entity,
    response: AtomicBool,
    attack: Mutex<Option<PlayerAction>>,
    interaction: Mutex<Option<PlayerAction>>,
}

impl InteractionEntity {
    pub fn new(entity: Entity) -> Self {
        entity.no_clip.store(true, Ordering::Relaxed);
        Self::apply_dimensions(&entity, scaled_dimensions(DEFAULT_WIDTH, DEFAULT_HEIGHT));
        Self {
            entity,
            response: AtomicBool::new(false),
            attack: Mutex::new(None),
            interaction: Mutex::new(None),
        }
    }

    fn apply_dimensions(entity: &Entity, dimensions: EntityDimensions) {
        let pos = entity.pos.load();
        entity
            .bounding_box
            .store(BoundingBox::new_from_pos(pos.x, pos.y, pos.z, &dimensions));
        entity.entity_dimension.store(dimensions);
    }

    async fn record_action(&self, slot: &Mutex<Option<PlayerAction>>, player: &Player) {
        let timestamp = self
            .entity
            .world
            .load()
            .level_time
            .lock()
            .await
            .query_gametime();
        *slot.lock().await = Some(PlayerAction {
            player: player.get_entity().entity_uuid,
            timestamp,
        });
    }
}

impl NBTStorage for InteractionEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            let dimensions = self.entity.entity_dimension.load();
            nbt.put_float("width", dimensions.width);
            nbt.put_float("height", dimensions.height);

            let attack = *self.attack.lock().await;
            if let Some(attack) = attack {
                let mut compound = NbtCompound::new();
                compound.put_uuid("player", attack.player);
                compound.put_long("timestamp", attack.timestamp);
                nbt.put_compound("attack", compound);
            }
            let interaction = *self.interaction.lock().await;
            if let Some(interaction) = interaction {
                let mut compound = NbtCompound::new();
                compound.put_uuid("player", interaction.player);
                compound.put_long("timestamp", interaction.timestamp);
                nbt.put_compound("interaction", compound);
            }

            nbt.put_bool("response", self.response.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            let width = nbt.get_float("width").unwrap_or(DEFAULT_WIDTH);
            let height = nbt.get_float("height").unwrap_or(DEFAULT_HEIGHT);
            Self::apply_dimensions(&self.entity, scaled_dimensions(width, height));

            let read_action = |key: &str| -> Option<PlayerAction> {
                let compound = nbt.get_compound(key)?;
                Some(PlayerAction {
                    player: compound.get_uuid("player")?,
                    timestamp: compound.get_long("timestamp")?,
                })
            };
            *self.attack.lock().await = read_action("attack");
            *self.interaction.lock().await = read_action("interaction");

            self.response
                .store(nbt.get_bool("response").unwrap_or(false), Ordering::Relaxed);
        })
    }
}

impl EntityBase for InteractionEntity {
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

    // Vanilla's `Interaction.tick()` is an empty override with no `super.tick()`
    // call, so the entity never runs normal entity ticking.
    fn tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
        _server: &'a crate::server::Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async {})
    }

    // Vanilla `hurtServer` unconditionally returns false; this entity cannot take
    // damage. Vanilla additionally records the attacker in `skipAttackInteraction`,
    // which runs just before `hurtServer` on the same attack path - this codebase
    // only exposes that path through `damage_with_context`, so the recording is
    // folded in here.
    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if let Some(player) = source.and_then(EntityBase::get_player) {
                self.record_action(&self.attack, player).await;
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
            self.record_action(&self.interaction, player).await;
            true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_HEIGHT, DEFAULT_WIDTH, scaled_dimensions};

    #[test]
    fn default_dimensions_are_one_by_one() {
        let dims = scaled_dimensions(DEFAULT_WIDTH, DEFAULT_HEIGHT);
        assert_eq!(dims.width, 1.0);
        assert_eq!(dims.height, 1.0);
        assert_eq!(dims.eye_height, 0.85);
    }

    #[test]
    fn eye_height_tracks_height_at_vanilla_ratio() {
        let dims = scaled_dimensions(2.0, 4.0);
        assert_eq!(dims.width, 2.0);
        assert_eq!(dims.height, 4.0);
        assert_eq!(dims.eye_height, 3.4);
    }
}
