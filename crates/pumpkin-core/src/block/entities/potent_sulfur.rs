use std::sync::{
    Arc,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::{
    Block, BlockDirection, BlockStateId,
    block_properties::{PotentSulfurLikeProperties, PotentSulfurState},
    effect::StatusEffect,
    potion::Effect,
    tag::{self, Taggable},
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::{
    math::{boundingbox::BoundingBox, position::BlockPos, vector3::Vector3},
    random::{RandomImpl, xoroshiro128::Xoroshiro},
};
use pumpkin_world::world::BlockFlags;

use crate::{
    block::blocks::potent_sulfur::PotentSulfurBlock, entity::living::LivingEntity, world::World,
};

use super::BlockEntity;

const ALLOWED_WATER_BLOCKS_ABOVE: i32 = 4;
const NOXIOUS_GAS_AFFECTED_RANGE: f64 = 3.0;
const NOXIOUS_GAS_EFFECT_DURATION: i32 = 80;
const GEYSER_MAX_VERTICAL_PUSH_FORCE_BASE: f32 = 0.3;
const GEYSER_VERTICAL_ACCELERATION: f32 = 0.2;
const COUNTDOWN_SALT: i64 = -904_011_478;

pub struct PotentSulfurBlockEntity {
    pub position: BlockPos,
    pub waiting_countdown: AtomicI32,
}

impl BlockEntity for PotentSulfurBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let countdown = nbt.get_int("countdown").unwrap_or(-1);
        Self {
            position,
            waiting_countdown: AtomicI32::new(countdown),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        let countdown = self.waiting_countdown.load(Ordering::Relaxed);
        if countdown != -1 {
            nbt.put_int("countdown", countdown);
        }
    }

    fn tick(&self, world: &Arc<World>) {
        let (block, state_id) = world.get_block_and_state_id(&self.position);
        if block != &Block::POTENT_SULFUR {
            return;
        }
        let props = PotentSulfurLikeProperties::from_state_id(state_id);
        match props.potent_sulfur_state {
            PotentSulfurState::Dry => {}
            PotentSulfurState::Wet => {
                Self::tick_wet(world, self.position);
            }
            PotentSulfurState::Dormant => {
                Self::tick_wet(world, self.position);
                self.tick_dormant_or_erupting(world, props);
            }
            PotentSulfurState::Erupting => {
                self.tick_dormant_or_erupting(world, props);
                Self::tick_erupting(world, self.position);
            }
            PotentSulfurState::Continuous => {
                Self::tick_erupting(world, self.position);
            }
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl PotentSulfurBlockEntity {
    pub const ID: &'static str = "minecraft:potent_sulfur";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            waiting_countdown: AtomicI32::new(-1),
        }
    }

    fn is_geyser_passable_block(world: &World, pos: &BlockPos) -> bool {
        let state = world.get_block_state(pos);
        !state.is_side_solid(BlockDirection::Up) && !state.is_side_solid(BlockDirection::Down)
    }

    #[must_use]
    pub fn find_noxious_gas_source_block(world: &World, origin: BlockPos) -> Option<BlockPos> {
        let max_y = origin.0.y + ALLOWED_WATER_BLOCKS_ABOVE + 1;
        for y in (origin.0.y + 1)..=max_y {
            let pos = BlockPos::new(origin.0.x, y, origin.0.z);
            let (block, state) = world.get_block_and_state(&pos);
            if !PotentSulfurBlock::is_water_source_block(world, &pos)
                || (block != &Block::WATER && !Self::is_geyser_passable_block(world, &pos))
            {
                if state.is_air() || Self::is_geyser_passable_block(world, &pos) {
                    return Some(pos);
                }
                return None;
            }
        }
        None
    }

    fn get_unobstructed_block_count(world: &World, origin: BlockPos, max_blocks: i32) -> i32 {
        for count in 0..max_blocks {
            let pos = BlockPos::new(origin.0.x, origin.0.y + count, origin.0.z);
            if !Self::is_geyser_passable_block(world, &pos) {
                return count;
            }
        }
        max_blocks
    }

    fn tick_wet(world: &Arc<World>, pos: BlockPos) {
        if world.get_world_age() % 10 != 0 {
            return;
        }
        let Some(source_block) = Self::find_noxious_gas_source_block(world, pos) else {
            return;
        };
        Self::apply_noxious_gas_effects(world, source_block);
    }

    fn apply_noxious_gas_effects(world: &Arc<World>, source_block: BlockPos) {
        let min = source_block.0.to_f64();
        let area = BoundingBox::new(
            Vector3::new(min.x - 2.5, min.y, min.z - 2.5),
            Vector3::new(min.x + 3.5, min.y + 1.0, min.z + 3.5),
        );
        let source_center = Vector3::new(min.x + 0.5, min.y + 0.5, min.z + 0.5);
        let max_dist_sq = NOXIOUS_GAS_AFFECTED_RANGE * NOXIOUS_GAS_AFFECTED_RANGE;

        for player in world.get_players_at_box(&area) {
            if !player.is_spectator()
                && Self::is_affected_by_noxious_gas(
                    world,
                    &player.living_entity,
                    source_center,
                    max_dist_sq,
                )
            {
                player.add_effect(Effect {
                    effect_type: &StatusEffect::NAUSEA,
                    duration: NOXIOUS_GAS_EFFECT_DURATION,
                    amplifier: 0,
                    ambient: true,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                });
            }
        }

        for entity in world.get_entities_at_box(&area) {
            if let Some(living) = entity.get_living_entity()
                && !entity.is_spectator()
                && Self::is_affected_by_noxious_gas(world, living, source_center, max_dist_sq)
            {
                living.add_effect(Effect {
                    effect_type: &StatusEffect::NAUSEA,
                    duration: NOXIOUS_GAS_EFFECT_DURATION,
                    amplifier: 0,
                    ambient: true,
                    show_particles: true,
                    show_icon: true,
                    blend: false,
                });
            }
        }
    }

    fn is_affected_by_noxious_gas(
        world: &World,
        living: &LivingEntity,
        source_center: Vector3<f64>,
        max_dist_sq: f64,
    ) -> bool {
        if living.entity.is_removed()
            || living.dead.load(Ordering::Relaxed)
            || living.health.load() <= 0.0
        {
            return false;
        }
        let eye_pos = living.entity.get_eye_pos();
        if eye_pos.squared_distance_to_vec(&source_center) > max_dist_sq {
            return false;
        }
        let eye_block_pos = BlockPos::floored(eye_pos.x, eye_pos.y, eye_pos.z);
        if !Self::is_geyser_passable_block(world, &eye_block_pos) {
            return false;
        }
        let below_eye_pos = BlockPos::floored(eye_pos.x, eye_pos.y - 1.0, eye_pos.z);
        PotentSulfurBlock::is_water_source_block(world, &below_eye_pos)
    }

    fn tick_dormant_or_erupting(&self, world: &Arc<World>, mut props: PotentSulfurLikeProperties) {
        if world.get_world_age() % 20 != 0 {
            return;
        }
        let mut countdown = self.waiting_countdown.load(Ordering::Relaxed);
        if countdown <= 0 {
            let water_blocks = Self::find_noxious_gas_source_block(world, self.position)
                .map_or(ALLOWED_WATER_BLOCKS_ABOVE, |source| {
                    source.0.y - self.position.0.y - 1
                })
                .max(1);
            let seed = world.level.seed.0 ^ (COUNTDOWN_SALT as u64);
            let mut rng = Xoroshiro::from_seed(seed).next_splitter().split_pos(
                self.position.0.x,
                self.position.0.y,
                self.position.0.z,
            );
            countdown = if props.potent_sulfur_state == PotentSulfurState::Dormant {
                10 * (water_blocks - 1) + rng.next_inbetween_i32(15, 30)
            } else {
                let _ = rng.next_i32();
                (water_blocks - 1) + rng.next_inbetween_i32(1, 2)
            };
        }

        countdown -= 1;
        self.waiting_countdown.store(countdown, Ordering::Relaxed);

        if countdown == 0 {
            let new_sulfur_state = if props.potent_sulfur_state == PotentSulfurState::Dormant {
                PotentSulfurState::Erupting
            } else {
                PotentSulfurState::Dormant
            };
            props.potent_sulfur_state = new_sulfur_state;
            let new_state_id: BlockStateId = props.to_state_id(&Block::POTENT_SULFUR);
            world.set_block_state(&self.position, new_state_id, BlockFlags::NOTIFY_ALL);
            PotentSulfurBlock::notify_eruption_started(world, &self.position, new_sulfur_state);
        }
    }

    fn tick_erupting(world: &Arc<World>, pos: BlockPos) {
        let water_blocks = Self::find_noxious_gas_source_block(world, pos)
            .map_or(ALLOWED_WATER_BLOCKS_ABOVE, |source| {
                source.0.y - pos.0.y - 1
            })
            .max(1);
        let max_height = 6 * water_blocks;
        let geyser_force_height = Self::get_unobstructed_block_count(world, pos.up(), max_height);
        if geyser_force_height <= 0 {
            return;
        }
        Self::launch_entities(world, pos.up(), geyser_force_height, water_blocks);
    }

    fn launch_entities(
        world: &Arc<World>,
        bottom: BlockPos,
        geyser_force_height: i32,
        water_blocks: i32,
    ) {
        let min = bottom.0.to_f64();
        let area = BoundingBox::new(
            min,
            Vector3::new(
                min.x + 1.0,
                min.y + f64::from(geyser_force_height),
                min.z + 1.0,
            ),
        );
        let max_vertical_push =
            f64::from(GEYSER_MAX_VERTICAL_PUSH_FORCE_BASE) + f64::from(water_blocks) * 0.1;

        for entity in world.get_entities_at_box(&area) {
            let base = entity.get_entity();
            if base.is_removed()
                || entity.is_spectator()
                || base.has_vehicle()
                || base
                    .entity_type
                    .has_tag(&tag::EntityType::MINECRAFT_NOT_AFFECTED_BY_GEYSERS)
            {
                continue;
            }
            if let Some(living) = entity.get_living_entity()
                && (living.dead.load(Ordering::Relaxed) || living.health.load() <= 0.0)
            {
                continue;
            }
            let mut vel = base.velocity.load();
            if vel.y < max_vertical_push {
                vel.y += f64::from(GEYSER_VERTICAL_ACCELERATION);
                base.velocity.store(vel);
                base.velocity_dirty.store(true, Ordering::Relaxed);
            }
        }
    }
}
