use pumpkin_data::{
    Block, BlockDirection, BlockId, BlockState, BlockStateId,
    block_properties::BlockProperties,
    tag::{self},
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::{
    block::{
        BlockBehaviour, BlockFuture, BlockMetadata, CanPlaceAtArgs, EmitsRedstonePowerArgs,
        GetRedstonePowerArgs, OnEntityCollisionArgs, OnNeighborUpdateArgs, OnScheduledTickArgs,
        OnStateReplacedArgs,
    },
    entity::EntityBase,
    world::World,
};

use super::{PressurePlate, detection_box_at};

/// This is for Normal Pressure plates, so not Gold or Iron
pub struct PressurePlateBlock;

type PressurePlateProps = pumpkin_data::block_properties::StonePressurePlateLikeProperties;

fn detects_entity(block: &Block, is_living: bool, is_spectator: bool) -> bool {
    !is_spectator
        && (!block
            .id
            .has_tag(tag::Block::MINECRAFT_STONE_PRESSURE_PLATES)
            || is_living)
}

impl BlockMetadata for PressurePlateBlock {
    fn ids() -> Box<[BlockId]> {
        let mut combined = Vec::new();
        combined.extend_from_slice(tag::Block::MINECRAFT_WOODEN_PRESSURE_PLATES.1);
        combined.extend_from_slice(tag::Block::MINECRAFT_STONE_PRESSURE_PLATES.1);
        combined.iter().map(|v| BlockId::new_or_air(*v)).collect()
    }
}

impl BlockBehaviour for PressurePlateBlock {
    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.on_entity_collision_pp(args).await;
        })
    }

    fn on_scheduled_tick<'a>(&'a self, args: OnScheduledTickArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.on_scheduled_tick_pp(args).await;
        })
    }

    fn on_state_replaced<'a>(&'a self, args: OnStateReplacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            self.on_state_replaced_pp(args).await;
        })
    }

    fn get_weak_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move { self.get_redstone_output(args.block, args.state.id) })
    }

    fn get_strong_redstone_power<'a>(
        &'a self,
        args: GetRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, u8> {
        Box::pin(async move {
            if args.direction == BlockDirection::Up {
                return self.get_redstone_output(args.block, args.state.id);
            }
            0
        })
    }

    fn emits_redstone_power<'a>(
        &'a self,
        _args: EmitsRedstonePowerArgs<'a>,
    ) -> BlockFuture<'a, bool> {
        Box::pin(async move { true })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if !Self::can_pressure_plate_place_at(args.world, args.position) {
                args.world
                    .break_block(args.position, None, BlockFlags::NOTIFY_ALL)
                    .await;
            }
        })
    }

    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        args.world
            .is_some_and(|world| Self::can_pressure_plate_place_at(world, args.position))
    }
}

impl PressurePlate for PressurePlateBlock {
    fn get_redstone_output(&self, block: &Block, state: BlockStateId) -> u8 {
        let props = PressurePlateProps::from_state_id(state, block);
        if props.powered { 15 } else { 0 }
    }

    async fn calculate_redstone_output(&self, world: &World, block: &Block, pos: &BlockPos) -> u8 {
        let aabb = detection_box_at(pos);
        let entity_triggers = world.get_entities_at_box(&aabb).into_iter().any(|entity| {
            detects_entity(
                block,
                entity.get_living_entity().is_some(),
                entity.is_spectator(),
            )
        });
        let player_triggers = world
            .get_players_at_box(&aabb)
            .into_iter()
            .any(|player| detects_entity(block, true, player.is_spectator()));
        if entity_triggers || player_triggers {
            return 15;
        }
        0
    }

    fn set_redstone_output(&self, block: &Block, state: &BlockState, output: u8) -> BlockStateId {
        let mut props = PressurePlateProps::from_state_id(state.id, block);
        props.powered = output > 0;
        props.to_state_id(block)
    }
}

#[cfg(test)]
mod tests {
    use super::detects_entity;
    use pumpkin_data::Block;

    #[test]
    fn stone_pressure_plates_only_detect_living_entities() {
        assert!(detects_entity(&Block::STONE_PRESSURE_PLATE, true, false));
        assert!(!detects_entity(&Block::STONE_PRESSURE_PLATE, false, false));
        assert!(detects_entity(
            &Block::POLISHED_BLACKSTONE_PRESSURE_PLATE,
            true,
            false
        ));
        assert!(!detects_entity(
            &Block::POLISHED_BLACKSTONE_PRESSURE_PLATE,
            false,
            false
        ));
    }

    #[test]
    fn wooden_pressure_plates_detect_non_living_entities() {
        assert!(detects_entity(&Block::OAK_PRESSURE_PLATE, true, false));
        assert!(detects_entity(&Block::OAK_PRESSURE_PLATE, false, false));
    }

    #[test]
    fn pressure_plates_ignore_spectators() {
        assert!(!detects_entity(&Block::STONE_PRESSURE_PLATE, true, true));
        assert!(!detects_entity(&Block::OAK_PRESSURE_PLATE, true, true));
    }
}
