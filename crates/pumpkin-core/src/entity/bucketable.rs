use std::sync::{Arc, atomic::Ordering::Relaxed};

use pumpkin_data::{
    data_component_impl::{BucketEntityDataImpl, CustomNameImpl},
    item::Item,
    item_stack::ItemStack,
    sound::Sound,
};
use pumpkin_nbt::compound::NbtCompound;

use super::{mob::Mob, player::Player};

pub trait Bucketable: Mob {
    fn is_from_bucket(&self) -> bool;
    fn set_from_bucket(&self, from_bucket: bool);
    fn bucket_item(&self) -> &'static Item;
    fn pickup_sound(&self) -> Sound;

    fn save_to_bucket_tag(&self, bucket: &mut ItemStack) {
        let entity = self.get_entity();
        let mob = self.get_mob_entity();
        if let Some(name) = entity.custom_name.load().as_ref() {
            bucket.set_data_component(CustomNameImpl { name: name.clone() });
        }
        let mut nbt = NbtCompound::new();
        for (key, value) in [
            ("NoAI", mob.is_no_ai()),
            ("Silent", entity.is_silent()),
            ("NoGravity", entity.has_no_gravity()),
            ("Glowing", entity.glowing.load(Relaxed)),
            ("Invulnerable", entity.invulnerable.load(Relaxed)),
            (
                "PersistenceRequired",
                mob.persistence_required.load(Relaxed),
            ),
        ] {
            if value {
                nbt.put_bool(key, true);
            }
        }
        nbt.put_float("Health", mob.living_entity.health.load());
        bucket.set_data_component(BucketEntityDataImpl { nbt: Some(nbt) });
    }

    fn load_from_bucket_tag(&self, nbt: &NbtCompound) {
        let entity = self.get_entity();
        let mob = self.get_mob_entity();
        if let Some(value) = nbt.get_bool("NoAI") {
            mob.set_no_ai(value);
        }
        if let Some(value) = nbt.get_bool("Silent") {
            entity.set_silent(value);
        }
        if let Some(value) = nbt.get_bool("NoGravity") {
            entity.set_has_no_gravity(value);
        }
        if let Some(value) = nbt.get_bool("Glowing") {
            entity.set_glowing(value);
        }
        if let Some(value) = nbt.get_bool("Invulnerable") {
            entity.set_invulnerable(value);
        }
        if nbt.get_bool("PersistenceRequired") == Some(true) {
            mob.persistence_required.store(true, Relaxed);
        }
        if let Some(health) = nbt.get_float("Health") {
            mob.living_entity.set_health(health);
        }
    }
}

pub fn bucket_mob_pickup(
    player: &Arc<Player>,
    stack: &mut ItemStack,
    mob: &dyn Bucketable,
) -> Option<bool> {
    let entity = mob.get_entity();
    if stack.item != &Item::WATER_BUCKET
        || !entity.is_alive()
        || mob.get_mob_entity().living_entity.is_dead_or_dying()
    {
        return None;
    }
    let world = entity.world.load();
    if let Some(server) = world.server.upgrade() {
        let mut event =
            crate::plugin::api::events::player::player_bucket_entity::PlayerBucketEntityEvent {
                player: player.clone(),
                entity_id: entity.entity_id,
                bucket_item: mob.bucket_item().registry_key.to_string(),
                cancelled: false,
            };
        server.plugin_manager.fire_blocking(&server, &mut event);
        if event.cancelled {
            return Some(true);
        }
    }
    if !entity.is_silent() {
        world.play_sound(
            mob.pickup_sound(),
            pumpkin_data::sound::SoundCategory::Neutral,
            &entity.pos.load(),
        );
    }
    let mut bucket = ItemStack::new(1, mob.bucket_item());
    mob.save_to_bucket_tag(&mut bucket);
    crate::item::items::bucket::give_filled_mob_bucket(stack, player, bucket);
    let leashed = entity
        .leashed_to
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .is_some();
    if leashed {
        entity.unleash();
        if entity
            .leashed_to
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none()
        {
            world.drop_stack(&entity.block_pos.load(), ItemStack::new(1, &Item::LEAD));
        }
    }
    entity.remove();
    Some(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{ai::test_support::world, r#type::from_type};
    use pumpkin_data::entity::EntityType;
    use pumpkin_util::math::vector3::Vector3;
    use uuid::Uuid;

    #[tokio::test]
    async fn fish_buckets_restore_only_bucketable_data() -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let source = from_type(
            &EntityType::COD,
            Vector3::new(0.0, 64.0, 0.0),
            &world,
            Uuid::new_v4(),
        );
        let original = source.get_entity();
        original.fire_ticks.store(80, Relaxed);
        original.set_air_supply(19);
        original.set_silent(true);
        let mut bucket = ItemStack::new(1, &Item::COD_BUCKET);
        let source_bucketable = source
            .get_mob()
            .and_then(|mob| mob.as_bucketable())
            .ok_or("missing Bucketable")?;
        source_bucketable
            .get_mob_entity()
            .living_entity
            .set_health(1.5);
        source_bucketable.save_to_bucket_tag(&mut bucket);
        let data = bucket
            .get_data_component::<BucketEntityDataImpl>()
            .ok_or("missing bucket data")?;
        let mut tag = data.nbt.clone().ok_or("missing NBT")?;
        assert_eq!(tag.get_float("Health"), Some(1.5));
        assert_eq!(tag.get_bool("Silent"), Some(true));
        assert!(tag.get("Air").is_none() && tag.get("Fire").is_none() && tag.get("Pos").is_none());
        tag.put_short("Air", 2);
        tag.put_short("Fire", 100);
        tag.put_bool("PersistenceRequired", false);
        let restored = from_type(
            &EntityType::COD,
            Vector3::new(2.0, 64.0, 0.0),
            &world,
            Uuid::new_v4(),
        );
        let initial_fire = restored.get_entity().fire_ticks.load(Relaxed);
        let restored_bucketable = restored
            .get_mob()
            .and_then(|mob| mob.as_bucketable())
            .ok_or("missing restored Bucketable")?;
        restored_bucketable
            .get_mob_entity()
            .persistence_required
            .store(true, Relaxed);
        restored_bucketable.load_from_bucket_tag(&tag);
        assert!(restored.get_entity().is_silent());
        assert_eq!(
            restored_bucketable
                .get_mob_entity()
                .living_entity
                .health
                .load(),
            1.5
        );
        assert_eq!(restored.get_entity().fire_ticks.load(Relaxed), initial_fire);
        assert_eq!(
            restored.get_entity().air_supply.load(Relaxed),
            crate::entity::breath::MAX_AIR
        );
        assert!(
            restored_bucketable
                .get_mob_entity()
                .persistence_required
                .load(Relaxed)
        );
        world.level.shutdown().await;
        Ok(())
    }
}
