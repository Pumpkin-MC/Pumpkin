use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_data::damage::DamageType;

use crate::entity::{Entity, EntityBase, EntityBaseFuture, NBTStorage};

/// `minecraft:marker`.
///
/// Vanilla `Marker` (`Marker.java`) has empty overrides for `tick`,
/// `defineSynchedData`, `readAdditionalSaveData`, `addAdditionalSaveData`
/// (:20-34), sets `noPhysics = true` in its constructor (:17), and `hurtServer`
/// is `final` and always returns false (:67-69). Its `getAddEntityPacket`
/// override (:37-39) throws because vanilla never tracks it to clients
/// (`EntityTypes.java:662-663`, `clientTrackingRange(0)`); Pumpkin has no
/// per-type client tracking range, so that suppression is not replicated here.
pub struct MarkerEntity {
    pub entity: Entity,
}

impl MarkerEntity {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(entity: Entity) -> Arc<dyn EntityBase> {
        entity.no_clip.store(true, Ordering::Relaxed);
        Arc::new(Self { entity })
    }
}

impl NBTStorage for MarkerEntity {}

impl EntityBase for MarkerEntity {
    fn as_nbt_storage(&self) -> &dyn NBTStorage {
        self
    }

    fn tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
        _server: &'a crate::server::Server,
    ) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {})
    }

    fn init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {})
    }

    /// Mirrors vanilla's `final hurtServer` (`Marker.java:67-69`): always rejects damage.
    fn damage_with_context<'a>(
        &'a self,
        _caller: &'a dyn EntityBase,
        _amount: f32,
        _damage_type: DamageType,
        _position: Option<pumpkin_util::math::vector3::Vector3<f64>>,
        _source: Option<&'a dyn EntityBase>,
        _cause: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move { false })
    }

    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&crate::entity::living::LivingEntity> {
        None
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
