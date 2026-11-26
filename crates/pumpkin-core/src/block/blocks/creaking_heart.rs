use std::sync::Arc;

use pumpkin_data::block_properties::{
    Axis, CreakingHeartLikeProperties, CreakingHeartState, PaleOakWoodLikeProperties,
};
use pumpkin_data::damage::DamageType;
use pumpkin_data::environment_attribute::EnvironmentAttribute;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::{gamemode::GameMode, math::position::BlockPos};
use pumpkin_world::{tick::TickPriority, world::BlockFlags};
use rand::RngExt;

use crate::block::entities::creaking_heart::CreakingHeartBlockEntity;
use crate::block::{
    BeforeExplosionDestroyArgs, BlockBehaviour, GetComparatorOutputArgs,
    GetStateForNeighborUpdateArgs, OnPlaceArgs, OnScheduledTickArgs, OnStateReplacedArgs,
};
use crate::entity::{
    EntityBase, experience_orb::ExperienceOrbEntity, player::Player, tnt::TNTEntity,
};
use crate::world::World;

#[pumpkin_block("minecraft:creaking_heart")]
pub struct CreakingHeartBlock;

impl CreakingHeartBlock {
    pub fn active_state(world: &World, pos: &BlockPos) -> CreakingHeartState {
        if world
            .environment_attributes()
            .get_value_bool(EnvironmentAttribute::GameplayCreakingActive, pos)
        {
            CreakingHeartState::Awake
        } else {
            CreakingHeartState::Dormant
        }
    }

    fn is_aligned_pale_oak_log(state_id: BlockStateId, axis: Axis) -> bool {
        state_id
            .to_block()
            .has_tag(&tag::Block::MINECRAFT_PALE_OAK_LOGS)
            && PaleOakWoodLikeProperties::from_state_id(state_id).axis == axis
    }

    pub fn has_required_logs(world: &World, pos: &BlockPos, axis: Axis) -> bool {
        let (first, second) = match axis {
            Axis::X => (pos.west(), pos.east()),
            Axis::Y => (pos.down(), pos.up()),
            Axis::Z => (pos.north(), pos.south()),
        };
        Self::is_aligned_pale_oak_log(world.get_block_state_id(&first), axis)
            && Self::is_aligned_pale_oak_log(world.get_block_state_id(&second), axis)
    }

    fn update_state(
        world: &World,
        pos: &BlockPos,
        mut props: CreakingHeartLikeProperties,
    ) -> BlockStateId {
        if props.creaking_heart_state == CreakingHeartState::Uprooted
            && Self::has_required_logs(world, pos, props.axis)
        {
            props.creaking_heart_state = Self::active_state(world, pos);
        }
        props.to_state_id(&Block::CREAKING_HEART)
    }

    pub fn player_will_destroy(
        world: &Arc<World>,
        pos: &BlockPos,
        state: BlockStateId,
        player: &Player,
    ) {
        let Some(entity) = world.get_block_entity(pos) else {
            return;
        };
        if let Some(heart) = entity.as_any().downcast_ref::<CreakingHeartBlockEntity>()
            && heart.destroy_protector(
                world,
                Some(DamageType::PLAYER_ATTACK),
                Some(player),
                Some(player),
            )
        {
            Self::try_award_experience(world, pos, state, player);
        }
    }

    fn award_explosion_experience(
        world: &Arc<World>,
        pos: &BlockPos,
        state: BlockStateId,
        source: &dyn EntityBase,
    ) {
        // Vanilla resolves the explosion's indirect source from the exploding entity itself.
        // A damage source can credit a player even for a crystal or TNT minecart.
        if let Some(player) = source.get_player() {
            Self::try_award_experience(world, pos, state, player);
        } else if let Some(owner) = source.cast_any().downcast_ref::<TNTEntity>().map_or_else(
            || {
                source
                    .get_projectile_owner()
                    .and_then(|owner| owner.get(world))
            },
            |tnt| tnt.owner.get(world),
        ) && let Some(player) = owner.get_player()
        {
            Self::try_award_experience(world, pos, state, player);
        }
    }

    fn try_award_experience(
        world: &Arc<World>,
        pos: &BlockPos,
        state: BlockStateId,
        player: &Player,
    ) {
        if !matches!(
            player.gamemode.load(),
            GameMode::Creative | GameMode::Spectator
        ) && CreakingHeartLikeProperties::from_state_id(state).natural
            && world.level_info.load().game_rules.block_drops
        {
            ExperienceOrbEntity::spawn(
                world,
                pos.to_centered_f64(),
                rand::rng().random_range(20..=24),
            );
        }
    }
}

impl BlockBehaviour for CreakingHeartBlock {
    fn before_explosion_destroy(&self, args: BeforeExplosionDestroyArgs<'_>) {
        let Some(entity) = args.world.get_block_entity(args.position) else {
            return;
        };
        if let Some(heart) = entity.as_any().downcast_ref::<CreakingHeartBlockEntity>()
            && heart.destroy_protector(
                args.world,
                Some(args.explosion.damage_type()),
                args.explosion.source(),
                args.explosion.indirect_source(),
            )
            && let Some(source) = args.explosion.source()
        {
            Self::award_explosion_experience(args.world, args.position, args.state.id, source);
        }
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = CreakingHeartLikeProperties::from_state_id(args.block.default_state.id);
        props.axis = match args.direction {
            BlockDirection::North | BlockDirection::South => Axis::Z,
            BlockDirection::East | BlockDirection::West => Axis::X,
            BlockDirection::Up | BlockDirection::Down => Axis::Y,
        };
        Self::update_state(args.world, args.position, props)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        args.world
            .schedule_block_tick(args.block, *args.position, 1, TickPriority::Normal);
        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let current = args.world.get_block_state_id(args.position);
        if current.to_block() != &Block::CREAKING_HEART {
            return;
        }
        let next = Self::update_state(
            args.world,
            args.position,
            CreakingHeartLikeProperties::from_state_id(current),
        );
        if next != current {
            args.world
                .set_block_state_if(args.position, next, BlockFlags::NOTIFY_ALL, |state| {
                    state == current
                });
        }
    }

    fn on_state_replaced(&self, args: OnStateReplacedArgs<'_>) {
        args.world
            .update_neighbour_for_output_signal(args.position, args.block);
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        let props = CreakingHeartLikeProperties::from_state_id(args.state.id);
        if props.creaking_heart_state == CreakingHeartState::Uprooted {
            return Some(0);
        }
        Some(
            args.world
                .get_block_entity(args.position)
                .as_deref()
                .and_then(|entity| {
                    entity
                        .as_any()
                        .downcast_ref::<CreakingHeartBlockEntity>()
                        .map(CreakingHeartBlockEntity::output_signal)
                })
                .unwrap_or(0),
        )
    }
}
