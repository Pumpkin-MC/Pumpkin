use crate::Arc;
use crate::block::blocks::abstract_wall_mounting::WallMountedBlock;
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockFuture, BlockHitResult, BrokenArgs, CanPlaceAtArgs, NormalUseArgs,
    OnNeighborUpdateArgs, OnPlaceArgs, PlacedArgs,
};
use crate::world::World;
use pumpkin_data::block_properties::BellLikeProperties;
use pumpkin_data::block_properties::BlockFace;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::block_properties::{Attachment, Axis};
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_data::{HorizontalFacingExt, tag};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::block::entities::bell::BellBlockEntity;
use pumpkin_world::world::BlockFlags;

async fn ring_bell(
    position: BlockPos,
    world: &Arc<World>,
    direction2: Option<HorizontalFacing>,
) -> () {
    let state = world.get_block_state(&position).await;

    let props = BellLikeProperties::from_state_id(state.id, world.get_block(&position).await);
    let direction = direction2.map_or(props.facing, |direction3| direction3);

    if let Some(block_entity) = world.get_block_entity(&position).await
        && let Some(be) = block_entity.as_any().downcast_ref::<BellBlockEntity>()
    {
        be.activate(direction);
    }

    world
        .play_sound_fine(
            Sound::BlockBellUse,
            SoundCategory::Blocks,
            &position.to_centered_f64(),
            1.0,
            2.0,
        )
        .await;

    //TODO Emit game event: BLOCK_CHANGE -> Send block update Packet
}

fn is_point_on_bell(
    hit: &BlockHitResult,
    attachment: Attachment,
    block_face: HorizontalFacing,
) -> bool {
    if hit.face == &BlockDirection::Up || hit.face == &BlockDirection::Down {
        return false;
    }
    if hit.face.to_axis() != Axis::Y && hit.cursor_pos.y <= 0.8124f32 {
        match attachment {
            Attachment::Floor => hit.face.to_axis() == block_face.to_block_direction().to_axis(),
            Attachment::SingleWall | Attachment::DoubleWall => {
                hit.face.to_axis() != block_face.to_block_direction().to_axis()
            }
            Attachment::Ceiling => true,
        }
    } else {
        false
    }
}

async fn is_single_wall(position: BlockPos, facing: HorizontalFacing, world: &World) -> bool {
    !world
        .get_block(&match facing {
            HorizontalFacing::North => position.add(0, 0, -1),
            HorizontalFacing::East => position.add(1, 0, 0),
            HorizontalFacing::South => position.add(0, 0, 1),
            HorizontalFacing::West => position.add(-1, 0, 0),
        })
        .await
        .is_solid()
}

#[pumpkin_block("minecraft:bell")]
pub struct BellBlock;

impl WallMountedBlock for BellBlock {
    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection {
        let props = BellLikeProperties::from_state_id(state_id, block);
        match props.attachment {
            Attachment::Ceiling => BlockDirection::Down,
            Attachment::Floor => BlockDirection::Up,
            Attachment::SingleWall | Attachment::DoubleWall => {
                props.facing.opposite().to_block_direction()
            }
        }
    }
}

impl BlockBehaviour for BellBlock {
    fn can_place_at<'a>(&'a self, args: CanPlaceAtArgs<'a>) -> BlockFuture<'a, bool> {
        Box::pin(async move {
            if let Some(direction) = args.direction
                && let Some(world) = args.world
            {
                if direction == BlockDirection::Up {
                    let block: &Block = world.get_block(args.position).await;

                    if block.has_tag(&tag::Block::MINECRAFT_UNSTABLE_BOTTOM_CENTER) {
                        false
                    } else {
                        let block_pos = args.position.offset(direction.to_offset());
                        let block_state = world.get_block_state(&block_pos).await;
                        block_state.is_center_solid(direction)
                    }
                } else {
                    WallMountedBlock::can_place_at(self, world, args.position, direction).await
                }
            } else {
                false
            }
        })
    }
    fn broken<'a>(&'a self, args: BrokenArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let world: &World = args.world;
            world.remove_block_entity(args.position).await;
        })
    }

    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world
                .add_block_entity(Arc::new(BellBlockEntity::new(*args.position)))
                .await;
        })
    }
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {
        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;

            let props = BellLikeProperties::from_state_id(state.id, args.block);

            if !is_point_on_bell(args.hit, props.attachment, props.facing) {
                return BlockActionResult::Pass; // Pass if Crosshair wasn't correctly positioned
            }
            ring_bell(
                *args.position,
                args.world,
                args.hit.face.to_horizontal_facing(),
            )
            .await;

            BlockActionResult::Success
        })
    }

    fn on_place<'a>(&'a self, args: OnPlaceArgs<'a>) -> BlockFuture<'a, BlockStateId> {
        Box::pin(async move {
            let mut props = BellLikeProperties::default(args.block);

            let block_face;
            let facing;
            (block_face, facing) =
                WallMountedBlock::get_placement_face(self, args.player, args.direction);

            props.facing = match block_face {
                BlockFace::Floor | BlockFace::Ceiling => facing,
                BlockFace::Wall => facing.opposite(),
            };

            props.attachment = match block_face {
                BlockFace::Wall => {
                    if is_single_wall(*args.position, props.facing.opposite(), args.world).await {
                        Attachment::SingleWall
                    } else {
                        Attachment::DoubleWall
                    }
                }
                BlockFace::Floor => Attachment::Floor,
                BlockFace::Ceiling => Attachment::Ceiling,
            };

            props.to_state_id(args.block)
        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let world: &World = args.world;

            let is_receiving_power = block_receives_redstone_power(world, args.position).await;
            let state = args.world.get_block_state(args.position).await;

            let mut props = BellLikeProperties::from_state_id(state.id, args.block);

            if props.powered != is_receiving_power {
                props.powered = is_receiving_power;

                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;

                if is_receiving_power {
                    ring_bell(*args.position, args.world, None).await;
                }
            }
        })
    }
}
