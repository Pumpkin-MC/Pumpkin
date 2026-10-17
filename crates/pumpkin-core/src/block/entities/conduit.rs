use super::BlockEntity;
use crate::{entity::EntityBase, world::World};
use crossbeam::atomic::AtomicCell;
use pumpkin_data::{
    damage::DamageType,
    effect::StatusEffect,
    potion::Effect,
    sound::{Sound, SoundCategory},
    tag::{self, Taggable},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3};
use rand::{RngExt, seq::IndexedRandom};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI64, Ordering},
};
use uuid::Uuid;

pub struct ConduitBlockEntity {
    pub position: BlockPos,
    active: AtomicBool,
    destroy_target: AtomicCell<Option<Uuid>>,
    next_ambient_sound_activation: AtomicI64,
}

impl ConduitBlockEntity {
    pub const ID: &'static str = "minecraft:conduit";
    const BLOCK_REFRESH_RATE: i64 = 2;
    const EFFECT_DURATION: i32 = 13;
    const MIN_ACTIVE_SIZE: usize = 16;
    const MIN_KILL_SIZE: usize = 42;
    const KILL_RANGE: f64 = 8.0;

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            active: AtomicBool::new(false),
            destroy_target: AtomicCell::new(None),
            next_ambient_sound_activation: AtomicI64::new(0),
        }
    }

    fn ensure_shape_position_loaded(world: &Arc<World>, pos: BlockPos) -> bool {
        if !world.is_in_build_limit(pos) || world.is_loaded(&pos) {
            return true;
        }
        let world = Arc::clone(world);
        tokio::runtime::Handle::current().spawn(async move {
            world.get_block_state_id_async(&pos).await;
        });
        false
    }

    fn update_shape(&self, world: &Arc<World>) -> Option<Vec<BlockPos>> {
        let mut effect_blocks = Vec::new();
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let pos = self.position.offset(Vector3::new(x, y, z));
                    if !Self::ensure_shape_position_loaded(world, pos) {
                        return None;
                    }
                    if !world.get_fluid(&pos).has_tag(&tag::Fluid::MINECRAFT_WATER) {
                        return Some(effect_blocks);
                    }
                }
            }
        }
        for x in -2i32..=2 {
            for y in -2i32..=2 {
                for z in -2i32..=2 {
                    let (ax, ay, az) = (x.abs(), y.abs(), z.abs());
                    if (ax > 1 || ay > 1 || az > 1)
                        && ((x == 0 && (ay == 2 || az == 2))
                            || (y == 0 && (ax == 2 || az == 2))
                            || (z == 0 && (ax == 2 || ay == 2)))
                    {
                        let pos = self.position.offset(Vector3::new(x, y, z));
                        if !Self::ensure_shape_position_loaded(world, pos) {
                            return None;
                        }
                        if world
                            .get_block(&pos)
                            .has_tag(&tag::Block::MINECRAFT_CONDUIT_EFFECT_BLOCK)
                        {
                            effect_blocks.push(pos);
                        }
                    }
                }
            }
        }
        Some(effect_blocks)
    }

    fn apply_effects(&self, world: &Arc<World>, effect_blocks: &[BlockPos]) {
        let range = (effect_blocks.len() / 7 * 16) as f64;
        let pos = self.position.0.to_f64();
        let bounds = BoundingBox::new_array(
            [pos.x, pos.y, pos.z],
            [pos.x + 1.0, pos.y + 1.0, pos.z + 1.0],
        )
        .expand_all(range)
        .expand_towards(0.0, f64::from(world.dimension.height), 0.0);
        for player in world.get_players_at_box(&bounds) {
            if player.is_spectator() {
                continue;
            }
            let entity = &player.living_entity.entity;
            if pos.squared_distance_to_vec(&entity.block_pos.load().0.to_f64()) < range * range
                && entity.is_in_water_or_rain()
            {
                player.add_effect(Effect {
                    effect_type: &StatusEffect::CONDUIT_POWER,
                    duration: Self::EFFECT_DURATION * 20,
                    amplifier: 0,
                    ambient: true,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                });
            }
        }
    }

    fn get_destroy_range_aabb(&self) -> BoundingBox {
        let pos = self.position.0.to_f64();
        BoundingBox::new_array(
            [pos.x, pos.y, pos.z],
            [pos.x + 1.0, pos.y + 1.0, pos.z + 1.0],
        )
        .expand_all(Self::KILL_RANGE)
    }

    fn select_new_target(&self, world: &Arc<World>) -> Option<Arc<dyn EntityBase>> {
        let candidates: Vec<_> = world
            .get_entities_at_box(&self.get_destroy_range_aabb())
            .into_iter()
            .filter(|candidate| {
                let entity = candidate.get_entity();
                candidate.get_living_entity().is_some()
                    && !entity.is_removed()
                    && entity.entity_type.is_enemy
                    && entity.is_in_water_or_rain()
            })
            .collect();
        candidates.choose(&mut rand::rng()).cloned()
    }

    fn update_destroy_target(
        &self,
        world: &Arc<World>,
        is_active: bool,
    ) -> Option<Arc<dyn EntityBase>> {
        if !is_active {
            return None;
        }
        let Some(uuid) = self.destroy_target.load() else {
            return self.select_new_target(world);
        };
        let target = world.get_entity_by_uuid(uuid)?;
        let living = target.get_living_entity()?;
        (!target.get_entity().is_removed()
            && !living.dead.load(Ordering::Relaxed)
            && living.health.load() > 0.0
            && self
                .position
                .0
                .to_f64()
                .squared_distance_to_vec(&target.get_entity().block_pos.load().0.to_f64())
                < Self::KILL_RANGE * Self::KILL_RANGE)
            .then_some(target)
    }

    fn update_and_attack_target(&self, world: &Arc<World>, is_active: bool) {
        let target = self.update_destroy_target(world, is_active);
        let uuid = target
            .as_ref()
            .map(|target| target.get_entity().entity_uuid);
        if let Some(target) = target {
            world.play_sound(
                Sound::BlockConduitAttackTarget,
                SoundCategory::Blocks,
                &target.get_entity().pos.load(),
            );
            target.damage(&*target, 4.0, DamageType::MAGIC);
        }
        if self.destroy_target.swap(uuid) != uuid
            && let Some(block_entity) = world.get_block_entity(&self.position)
        {
            world.update_block_entity(&block_entity);
        }
    }
}

impl BlockEntity for ConduitBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self {
        let entity = Self::new(position);
        entity.destroy_target.store(nbt.get_uuid("Target"));
        entity
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(uuid) = self.destroy_target.load() {
            nbt.put_uuid("Target", uuid);
        }
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_nbt(&mut nbt);
        Some(nbt)
    }

    fn tick(&self, world: &Arc<World>) {
        let game_time = world.get_world_age();
        if game_time % (Self::BLOCK_REFRESH_RATE * 20) == 0
            && let Some(effect_blocks) = self.update_shape(world)
        {
            let active = effect_blocks.len() >= Self::MIN_ACTIVE_SIZE;
            if self.active.swap(active, Ordering::Relaxed) != active {
                world.play_sound(
                    if active {
                        Sound::BlockConduitActivate
                    } else {
                        Sound::BlockConduitDeactivate
                    },
                    SoundCategory::Blocks,
                    &self.position.to_centered_f64(),
                );
            }
            if active {
                self.apply_effects(world, &effect_blocks);
                self.update_and_attack_target(world, effect_blocks.len() >= Self::MIN_KILL_SIZE);
            }
        }
        if self.active.load(Ordering::Relaxed) {
            if game_time % 80 == 0 {
                world.play_sound(
                    Sound::BlockConduitAmbient,
                    SoundCategory::Blocks,
                    &self.position.to_centered_f64(),
                );
            }
            if game_time > self.next_ambient_sound_activation.load(Ordering::Relaxed) {
                self.next_ambient_sound_activation.store(
                    game_time + 60 + rand::rng().random_range(0..40),
                    Ordering::Relaxed,
                );
                world.play_sound(
                    Sound::BlockConduitAmbientShort,
                    SoundCategory::Blocks,
                    &self.position.to_centered_f64(),
                );
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_nbt::tag::NbtTag;

    #[test]
    fn target_nbt_uses_vanilla_uuid_format() {
        let words = [0x0011_2233, 0x4455_6677, -2_003_195_205, -857_870_593];
        let mut input = NbtCompound::new();
        input.put("Target", NbtTag::IntArray(words.to_vec()));
        let entity = ConduitBlockEntity::from_nbt(&input, BlockPos::new(0, 64, 0));
        let mut output = NbtCompound::new();
        entity.write_nbt(&mut output);
        assert_eq!(output.get_int_array("Target"), Some(words.as_slice()));
        assert_eq!(
            output.get_uuid("Target"),
            Some(Uuid::from_u128(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff)),
        );

        input.put("Target", NbtTag::IntArray(vec![1, 2, 3]));
        let entity = ConduitBlockEntity::from_nbt(&input, BlockPos::new(0, 64, 0));
        let mut output = NbtCompound::new();
        entity.write_nbt(&mut output);
        assert!(output.get("Target").is_none());
    }
}
