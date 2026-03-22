use std::sync::Arc;

use pumpkin_data::dimension::Dimension;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::particle::Particle;
use pumpkin_data::potion::Effect;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::{Block, tag};
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::xoroshiro128::Xoroshiro;
use pumpkin_util::random::{RandomGenerator, RandomImpl, get_seed};
use pumpkin_world::BlockStateId;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::blocks::plant::PlantBlockBase;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    OnEntityCollisionArgs, OnScheduledTickArgs,
};

use crate::block::RandomTickArgs;
use crate::entity::EntityBase;
use crate::world::World;

pub struct FlowerBlock;

impl BlockMetadata for FlowerBlock {
    fn ids() -> Box<[u16]> {
        tag::Block::C_FLOWERS_SMALL.1.into()
    }
}

impl BlockBehaviour for FlowerBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            <Self as PlantBlockBase>::can_place_at(self, args.block_accessor, args.position).await
        })
    }

    fn get_state_for_neighbor_update<'a>(
        &'a self,
        args: GetStateForNeighborUpdateArgs<'a>,
    ) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            <Self as PlantBlockBase>::get_state_for_neighbor_update(
                self,
                args.world,
                args.position,
                args.state_id,
            )
            .await
        })
    }

    fn random_tick<'a>(&'a self, args: RandomTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(eyeblossom) = EyeblossomType::from_block(args.block) {
                eyeblossom
                    .try_change_state(args.world, args.position, true, false)
                    .await;
            }
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(eyeblossom) = EyeblossomType::from_block(args.block) {
                eyeblossom
                    .try_change_state(args.world, args.position, false, false)
                    .await;
            }
        })
    }

    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(eyeblossom) = EyeblossomType::from_block(args.block)
                && args.entity.get_entity().entity_type == &EntityType::BEE
            {
                eyeblossom.on_bee_collision(args.world, args.entity).await;
            }
        })
    }
}

impl PlantBlockBase for FlowerBlock {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EyeblossomType {
    Open,
    Closed,
}

impl EyeblossomType {
    #[must_use]
    pub fn from_block(block: &Block) -> Option<Self> {
        if block.eq(&Block::OPEN_EYEBLOSSOM) || block.eq(&Block::POTTED_OPEN_EYEBLOSSOM) {
            Some(Self::Open)
        } else if block.eq(&Block::CLOSED_EYEBLOSSOM) || block.eq(&Block::POTTED_CLOSED_EYEBLOSSOM)
        {
            Some(Self::Closed)
        } else {
            None
        }
    }

    pub async fn on_bee_collision(self, world: &Arc<World>, entity: &dyn EntityBase) {
        if self != Self::Open || world.level_info.load().difficulty == Difficulty::Peaceful {
            return;
        }

        if let Some(bee) = entity.get_living_entity()
            && !bee.has_effect(&StatusEffect::POISON).await
        {
            bee.add_effect(Effect {
                effect_type: &StatusEffect::POISON,
                duration: 25,
                amplifier: 0,
                ambient: false,
                show_particles: true,
                show_icon: true,
                blend: true,
            })
            .await;
        }
    }

    pub async fn try_change_state(
        mut self,
        world: &Arc<World>,
        position: &BlockPos,
        is_long_sound: bool,
        is_potted: bool,
    ) {
        if !(world.dimension.eq(&Dimension::OVERWORLD)
            || world.dimension.eq(&Dimension::OVERWORLD_CAVES))
        {
            return;
        }

        let time = world.level_time.lock().await.time_of_day % 24000;
        if ((12600..23400).contains(&time)) == self.is_open() {
            return;
        }

        let old = self.to_block(is_potted);
        self.transform();

        world
            .set_block_state(
                position,
                self.to_block(is_potted).default_state.id,
                BlockFlags::NOTIFY_ALL,
            )
            .await;

        let sound = if is_long_sound {
            self.long_sound()
        } else {
            self.sound()
        };

        world
            .play_block_sound(sound, SoundCategory::Blocks, *position)
            .await;

        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        self.spawn_particle(world, position, &mut random).await;

        if !is_potted {
            let start = position.add(-3, -2, -3);
            let end = position.add(3, 2, 3);
            for pos in BlockPos::iterate(start, end) {
                if pos == *position {
                    continue;
                }

                let (block, _) = world.get_block_and_state(&pos).await;
                if block.eq(&old) {
                    let dist = (position.0.squared_distance_to_vec(&pos.0) as f64).sqrt();
                    let delay =
                        random.next_inbetween_i32((dist * 5.0) as i32, (dist * 10.0) as i32);

                    world
                        .schedule_block_tick(block, pos, delay as u8, TickPriority::Normal)
                        .await;
                }
            }
        }
    }

    async fn spawn_particle(
        self,
        world: &std::sync::Arc<crate::world::World>,
        position: &pumpkin_util::math::position::BlockPos,
        random: &mut RandomGenerator,
    ) {
        let center = position.to_centered_f64();
        let color = self.particle_color();

        let d0 = 0.5 + random.next_f64();
        let duration = VarInt::from((20.0 * d0) as i32);
        let pos = Vector3::new(
            center.x + (random.next_f64() - 0.5) * d0,
            center.y + (random.next_f64() + 1.0) * d0,
            center.z + (random.next_f64() - 0.5) * d0,
        );

        let mut data = Vec::with_capacity(29);
        data.extend_from_slice(&pos.x.to_be_bytes());
        data.extend_from_slice(&pos.y.to_be_bytes());
        data.extend_from_slice(&pos.z.to_be_bytes());
        data.extend_from_slice(&color.to_be_bytes());
        duration.encode(&mut data).expect("Failed to encode VarInt");

        world
            .spawn_particle_with_data(
                center,
                Vector3::new(0.0, 0.0, 0.0),
                0.0,
                1,
                Particle::Trail,
                &data,
            )
            .await;
    }

    const fn transform(&mut self) {
        *self = match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        };
    }

    const fn to_block(self, is_potted: bool) -> Block {
        match self {
            Self::Open if is_potted => Block::POTTED_OPEN_EYEBLOSSOM,
            Self::Closed if is_potted => Block::POTTED_CLOSED_EYEBLOSSOM,
            Self::Open => Block::OPEN_EYEBLOSSOM,
            Self::Closed => Block::CLOSED_EYEBLOSSOM,
        }
    }

    fn is_open(self) -> bool {
        self == Self::Open
    }

    const fn sound(self) -> Sound {
        match self {
            Self::Open => Sound::BlockEyeblossomOpen,
            Self::Closed => Sound::BlockEyeblossomClose,
        }
    }

    const fn long_sound(self) -> Sound {
        match self {
            Self::Open => Sound::BlockEyeblossomOpenLong,
            Self::Closed => Sound::BlockEyeblossomCloseLong,
        }
    }

    const fn particle_color(self) -> i32 {
        match self {
            Self::Open => 16545810,
            Self::Closed => 6250335,
        }
    }
}
