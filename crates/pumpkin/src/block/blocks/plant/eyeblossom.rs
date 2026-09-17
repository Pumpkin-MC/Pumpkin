use std::sync::Arc;

use pumpkin_data::{
    Block, BlockId, BlockStateId,
    effect::StatusEffect,
    entity::EntityType,
    particle::Particle,
    sound::{Sound, SoundCategory},
};
use pumpkin_protocol::java::client::play::CParticle;
use pumpkin_util::{
    Difficulty,
    math::{position::BlockPos, vector3::Vector3},
    version::JavaMinecraftVersion,
};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};
use rand::RngExt;

use crate::{
    block::{
        BlockBehaviour, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
        OnEntityCollisionArgs, OnScheduledTickArgs, RandomTickArgs, blocks::plant::PlantBlockBase,
    },
    net::ClientPlatform,
    world::World,
};

const EYEBLOSSOM_XZ_RANGE: i32 = 3;
const EYEBLOSSOM_Y_RANGE: i32 = 2;
const OPEN_EYEBLOSSOM_PARTICLE_COLOR: i32 = 16_545_810;
const CLOSED_EYEBLOSSOM_PARTICLE_COLOR: i32 = 6_250_335;

pub struct EyeblossomBlock;

impl BlockMetadata for EyeblossomBlock {
    fn ids() -> Box<[BlockId]> {
        Box::new([BlockId::OPEN_EYEBLOSSOM, BlockId::CLOSED_EYEBLOSSOM])
    }
}

impl BlockBehaviour for EyeblossomBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        <Self as PlantBlockBase>::get_state_for_neighbor_update(
            self,
            args.world,
            args.position,
            args.state_id,
        )
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if !<Self as PlantBlockBase>::can_place_at(self, args.world.as_ref(), args.position) {
            args.world
                .break_block(args.position, None, BlockFlags::empty());
            return;
        }

        let was_open = args.block == &Block::OPEN_EYEBLOSSOM;
        if try_changing_state(args.world, args.block, args.position) {
            let sound = if was_open {
                Sound::BlockEyeblossomClose
            } else {
                Sound::BlockEyeblossomOpen
            };
            args.world.play_sound(
                sound,
                SoundCategory::Blocks,
                &args.position.to_centered_f64(),
            );
        }
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let was_open = args.block == &Block::OPEN_EYEBLOSSOM;
        if try_changing_state(args.world, args.block, args.position) {
            let sound = if was_open {
                Sound::BlockEyeblossomCloseLong
            } else {
                Sound::BlockEyeblossomOpenLong
            };
            args.world.play_sound(
                sound,
                SoundCategory::Blocks,
                &args.position.to_centered_f64(),
            );
        }
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        {
            if args.world.level_info.load().difficulty == Difficulty::Peaceful {
                return;
            }

            if args.entity.get_entity().entity_type == &EntityType::BEE
                && let Some(living_entity) = args.entity.get_living_entity()
            {
                let effect = pumpkin_data::potion::Effect {
                    effect_type: &StatusEffect::POISON,
                    duration: 25,
                    amplifier: 0,
                    ambient: false,
                    show_particles: true,
                    show_icon: true,
                    blend: true,
                };
                living_entity.add_effect(effect);
            }
        }
    }
}

impl PlantBlockBase for EyeblossomBlock {}

fn encode_trail_particle_data(target: Vector3<f64>, color: i32, duration: Option<u8>) -> Vec<u8> {
    let mut data = Vec::with_capacity(if duration.is_some() { 29 } else { 28 });
    data.extend_from_slice(&target.x.to_be_bytes());
    data.extend_from_slice(&target.y.to_be_bytes());
    data.extend_from_slice(&target.z.to_be_bytes());
    data.extend_from_slice(&color.to_be_bytes());
    // Duration was added to Trail in 1.21.4. Eyeblossom durations are 10..30 ticks,
    // so their VarInt encoding is always one byte.
    if let Some(duration) = duration {
        data.push(duration);
    }
    data
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TrailParticleDelivery {
    LegacyFallback,
    PreDurationPacket,
    DurationPacket,
}

fn trail_particle_delivery_for_version(version: JavaMinecraftVersion) -> TrailParticleDelivery {
    if version == JavaMinecraftVersion::Unknown || version < JavaMinecraftVersion::V_1_21_2 {
        TrailParticleDelivery::LegacyFallback
    } else if version < JavaMinecraftVersion::V_1_21_4 {
        TrailParticleDelivery::PreDurationPacket
    } else {
        TrailParticleDelivery::DurationPacket
    }
}

pub fn try_changing_state(world: &Arc<World>, current_block: &Block, pos: &BlockPos) -> bool {
    let is_open = current_block == &Block::OPEN_EYEBLOSSOM;
    let should_be_open = world.eyeblossom_open(pos).unwrap_or(is_open);

    if should_be_open == is_open {
        return false;
    }

    let new_block = if is_open {
        &Block::CLOSED_EYEBLOSSOM
    } else {
        &Block::OPEN_EYEBLOSSOM
    };

    world.set_block_state(pos, new_block.default_state.id, BlockFlags::NOTIFY_ALL);

    let center = pos.to_centered_f64();
    let mut rng = rand::rng();
    let distance = 0.5 + rng.random::<f64>();
    let target = Vector3::new(
        center.x + (rng.random::<f64>() - 0.5) * distance,
        center.y + (rng.random::<f64>() + 1.0) * distance,
        center.z + (rng.random::<f64>() - 0.5) * distance,
    );
    let color = if is_open {
        CLOSED_EYEBLOSSOM_PARTICLE_COLOR
    } else {
        OPEN_EYEBLOSSOM_PARTICLE_COLOR
    };
    let duration = (20.0 * distance) as u8;
    let data_1_21_2 = encode_trail_particle_data(target, color, None);
    let data_1_21_4 = encode_trail_particle_data(target, color, Some(duration));
    let particle_1_21_2 = CParticle::new(
        false,
        false,
        center,
        Vector3::new(0.0, 0.0, 0.0),
        0.0,
        1,
        (Particle::Trail as i32).into(),
        &data_1_21_2,
    );
    let particle_1_21_4 = CParticle::new(
        false,
        false,
        center,
        Vector3::new(0.0, 0.0, 0.0),
        0.0,
        1,
        (Particle::Trail as i32).into(),
        &data_1_21_4,
    );

    for player in world.players.load().iter() {
        let ClientPlatform::Java(client) = player.client.as_ref() else {
            continue;
        };
        match trail_particle_delivery_for_version(client.version.load()) {
            TrailParticleDelivery::LegacyFallback => {
                player.spawn_particle(center, Vector3::new(0.0, 0.0, 0.0), 0.0, 1, Particle::Trail);
            }
            TrailParticleDelivery::PreDurationPacket => {
                player.try_send_client_packet(&particle_1_21_2);
            }
            TrailParticleDelivery::DurationPacket => {
                player.try_send_client_packet(&particle_1_21_4);
            }
        }
    }

    for dx in -EYEBLOSSOM_XZ_RANGE..=EYEBLOSSOM_XZ_RANGE {
        for dy in -EYEBLOSSOM_Y_RANGE..=EYEBLOSSOM_Y_RANGE {
            for dz in -EYEBLOSSOM_XZ_RANGE..=EYEBLOSSOM_XZ_RANGE {
                if dx == 0 && dy == 0 && dz == 0 {
                    continue;
                }
                let nearby_pos = pos.offset(Vector3::new(dx, dy, dz));
                let nearby_block = world.get_block(&nearby_pos);
                if nearby_block == current_block {
                    let dist_sqr = (dx * dx + dy * dy + dz * dz) as f64;
                    let distance = dist_sqr.sqrt();
                    let min_delay = (distance * 5.0) as u8;
                    let max_delay = (distance * 10.0) as u8;
                    let delay = if min_delay >= max_delay {
                        min_delay
                    } else {
                        rng.random_range(min_delay..=max_delay)
                    };
                    world.schedule_block_tick(
                        current_block,
                        nearby_pos,
                        delay.max(1),
                        TickPriority::Normal,
                    );
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::{
        TrailParticleDelivery, encode_trail_particle_data, trail_particle_delivery_for_version,
    };
    use pumpkin_util::{math::vector3::Vector3, version::JavaMinecraftVersion};

    #[test]
    fn trail_particle_data_matches_versioned_protocol_layouts() {
        let target = Vector3::new(1.25, 64.5, -3.75);
        let color = 0x12_34_56;

        let pre_duration = encode_trail_particle_data(target, color, None);
        assert_eq!(pre_duration.len(), 28);
        assert_eq!(&pre_duration[0..8], &target.x.to_be_bytes());
        assert_eq!(&pre_duration[8..16], &target.y.to_be_bytes());
        assert_eq!(&pre_duration[16..24], &target.z.to_be_bytes());
        assert_eq!(&pre_duration[24..28], &color.to_be_bytes());

        let with_duration = encode_trail_particle_data(target, color, Some(20));
        assert_eq!(with_duration.len(), 29);
        assert_eq!(&with_duration[..28], &pre_duration);
        assert_eq!(with_duration[28], 20);
    }

    #[test]
    fn trail_particle_delivery_matches_protocol_boundaries() {
        assert_eq!(
            trail_particle_delivery_for_version(JavaMinecraftVersion::Unknown),
            TrailParticleDelivery::LegacyFallback
        );
        assert_eq!(
            trail_particle_delivery_for_version(JavaMinecraftVersion::V_1_21),
            TrailParticleDelivery::LegacyFallback
        );
        assert_eq!(
            trail_particle_delivery_for_version(JavaMinecraftVersion::V_1_21_2),
            TrailParticleDelivery::PreDurationPacket
        );
        assert_eq!(
            trail_particle_delivery_for_version(JavaMinecraftVersion::V_1_21_4),
            TrailParticleDelivery::DurationPacket
        );
    }
}
