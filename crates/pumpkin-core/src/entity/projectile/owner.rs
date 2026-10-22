use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, Mutex, PoisonError, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_nbt::compound::NbtCompound;
use uuid::Uuid;

use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase};
use crate::world::World;

/// Vanilla `EntityReference`: an entity known by UUID, found in any dimension.
#[derive(Default)]
pub struct EntityReference {
    uuid: AtomicCell<Option<Uuid>>,
    /// Vanilla keeps the resolved entity until it is removed. Players are always looked up,
    /// since leaving the server does not mark them removed.
    resolved: Mutex<Option<Weak<dyn EntityBase>>>,
}

impl EntityReference {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            uuid: AtomicCell::new(None),
            resolved: Mutex::new(None),
        }
    }

    #[must_use]
    pub fn new(entity: Option<&Entity>) -> Self {
        Self {
            uuid: AtomicCell::new(entity.map(|entity| entity.entity_uuid)),
            resolved: Mutex::new(None),
        }
    }

    #[must_use]
    pub fn uuid(&self) -> Option<Uuid> {
        self.uuid.load()
    }

    #[must_use]
    pub fn matches(&self, entity: &Entity) -> bool {
        self.uuid() == Some(entity.entity_uuid)
    }

    #[must_use]
    pub fn get(&self, world: &World) -> Option<Arc<dyn EntityBase>> {
        let uuid = self.uuid()?;
        let mut resolved = self.resolved.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(entity) = resolved.as_ref().and_then(Weak::upgrade)
            && !entity.get_entity().is_removed()
        {
            return Some(entity);
        }
        *resolved = None;
        if let Some(player) = self.get_player(world) {
            return Some(player);
        }
        let entity = world.get_entity_by_uuid(uuid).or_else(|| {
            world
                .server
                .upgrade()?
                .worlds
                .load()
                .iter()
                .find_map(|other| other.get_entity_by_uuid(uuid))
        })?;
        *resolved = Some(Arc::downgrade(&entity));
        Some(entity)
    }

    #[must_use]
    pub fn get_player(&self, world: &World) -> Option<Arc<Player>> {
        let uuid = self.uuid()?;
        world
            .get_player_by_uuid(uuid)
            .or_else(|| world.server.upgrade()?.get_player_by_uuid(uuid))
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound, key: &str) {
        if let Some(uuid) = self.uuid() {
            nbt.put_uuid(key, uuid);
        }
    }

    /// Updates only when `key` is present
    /// an invalid UUID clears it like vanilla.
    pub fn read_nbt(&self, nbt: &NbtCompound, key: &str) {
        if nbt.has(key) {
            self.set_uuid(nbt.get_uuid(key));
        }
    }

    /// Forgets the entity, e.g. `data remove`.
    pub fn clear(&self) {
        self.set_uuid(None);
    }

    fn set_uuid(&self, uuid: Option<Uuid>) {
        self.uuid.store(uuid);
        *self.resolved.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}

/// Vanilla `Projectile` owner state: who shot it and whether it has cleared them.
#[derive(Default)]
pub struct ProjectileOwner {
    owner: EntityReference,
    left_owner: AtomicBool,
}

impl ProjectileOwner {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            owner: EntityReference::none(),
            left_owner: AtomicBool::new(false),
        }
    }

    #[must_use]
    pub fn new(owner: Option<&Entity>) -> Self {
        Self {
            owner: EntityReference::new(owner),
            left_owner: AtomicBool::new(false),
        }
    }

    /// Vanilla `checkLeftOwner`, once per tick before moving.
    pub fn check_left_owner(&self, projectile: &Entity) {
        if self.left_owner.load(Relaxed) {
            return;
        }
        let outside = self
            .owner
            .get(&projectile.world.load())
            .is_none_or(|owner| Self::outside_owner_collision_range(projectile, &*owner));
        self.left_owner.store(outside, Relaxed);
    }

    /// Vanilla `isOutsideOwnerCollisionRange`: nothing of the owner's vehicle near the path.
    fn outside_owner_collision_range(projectile: &Entity, owner: &dyn EntityBase) -> bool {
        let velocity = projectile.velocity.load();
        let area = projectile
            .bounding_box
            .load()
            .expand_towards(velocity.x, velocity.y, velocity.z)
            .expand_all(1.0);
        let root = owner.get_entity().get_root_vehicle();
        !Self::any_self_or_passenger(root.as_deref().unwrap_or(owner), &|entity| {
            !entity.is_spectator() && entity.get_entity().bounding_box.load().intersects(&area)
        })
    }

    fn any_self_or_passenger(
        entity: &dyn EntityBase,
        predicate: &dyn Fn(&dyn EntityBase) -> bool,
    ) -> bool {
        predicate(entity)
            || entity
                .get_entity()
                .passengers
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .iter()
                .any(|passenger| Self::any_self_or_passenger(&**passenger, predicate))
    }

    /// Vanilla `canHitEntity` owner rule: the owner's vehicle group only once it has left.
    #[must_use]
    pub fn can_hit(&self, world: &World, entity: &Entity) -> bool {
        self.left_owner.load(Relaxed)
            || self
                .owner
                .get(world)
                .is_none_or(|owner| !owner.get_entity().is_passenger_of_same_vehicle(entity))
    }

    pub fn write_nbt(&self, nbt: &mut NbtCompound) {
        self.owner.write_nbt(nbt, "Owner");
        if self.left_owner.load(Relaxed) {
            nbt.put_bool("LeftOwner", true);
        }
    }

    /// Updates only the keys present, so a partial read keeps the rest.
    pub fn read_nbt(&self, nbt: &NbtCompound) {
        self.owner.read_nbt(nbt, "Owner");
        if nbt.has("LeftOwner") {
            self.left_owner
                .store(nbt.get_bool("LeftOwner").unwrap_or(false), Relaxed);
        }
    }

    /// Resets the removed keys to vanilla's defaults.
    pub fn remove_nbt(&self, removed: &[Box<str>]) {
        for key in removed {
            match &**key {
                "Owner" => self.owner.clear(),
                "LeftOwner" => self.left_owner.store(false, Relaxed),
                _ => {}
            }
        }
    }
}

impl std::ops::Deref for ProjectileOwner {
    type Target = EntityReference;

    fn deref(&self) -> &EntityReference {
        &self.owner
    }
}
