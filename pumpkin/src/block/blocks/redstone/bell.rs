use crate::block::blocks::abstract_wall_mounting::WallMountedBlock;
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::registry::BlockActionResult;
use crate::block::{BlockBehaviour, BlockFuture, BlockHitResult, NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs, PlacedArgs};
use crate::world::World;
use crate::Arc;
use pumpkin_data::block_properties::{Attachment, Axis};
use pumpkin_data::block_properties::BellLikeProperties;
use pumpkin_data::block_properties::BlockFace;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::HorizontalFacing;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::block::entities::bell::BellBlockEntity;
use pumpkin_world::world::BlockFlags;
use pumpkin_world::BlockStateId;

fn reverse_horizontal_facing(direction: HorizontalFacing) -> HorizontalFacing{
    match direction {
        HorizontalFacing::North => HorizontalFacing::South,
        HorizontalFacing::South => HorizontalFacing::North,
        HorizontalFacing::East => HorizontalFacing::West,
        HorizontalFacing::West => HorizontalFacing::East
    }
}
async fn ring_bell<'a>(position: BlockPos, world:&Arc<World>, direction2: Option<HorizontalFacing>) -> (){
    let state = world.get_block_state(&position).await;

    let props = BellLikeProperties::from_state_id(state.id, world.get_block(&position).await);
    let direction;

    if let Some(direction3) = direction2 {
        direction=direction3;
    } else {
        direction=props.facing;
    }

    if let Some(block_entity)=world.get_block_entity(&position).await{

        block_entity.as_any().downcast_ref::<BellBlockEntity>().unwrap().activate(direction);

    }

    world
        .play_sound_fine(
            Sound::BlockBellUse,
            SoundCategory::Blocks,
            &position.to_centered_f64(),
            1.0,
            2.0
        )
        .await;

    //TODO Emit game event: BLOCK_CHANGE -> Send block update Packet
}

fn is_point_on_bell(hit: &BlockHitResult, attachment: Attachment, block_face: HorizontalFacing) -> bool {
    if hit.face==&BlockDirection::Up || hit.face==&BlockDirection::Down {
        return false;
    }
    if hit.face.to_axis() != Axis::Y && !(hit.cursor_pos.y > 0.8124f32) {
        match attachment {
            Attachment::Floor => hit.face.to_axis() == block_face.to_block_direction().to_axis(),
            Attachment::SingleWall | Attachment::DoubleWall => hit.face.to_axis() != block_face.to_block_direction().to_axis(),
            Attachment::Ceiling => true,
        }
    } else {
        false
    }
}

async fn is_single_wall(position: BlockPos, facing: HorizontalFacing, world: &World) -> bool{

    !world.get_block(&match facing{
        HorizontalFacing::North => position.add(0,0,-1),
        HorizontalFacing::East => position.add(1,0,0),
        HorizontalFacing::South => position.add(0,0,1),
        HorizontalFacing::West => position.add(-1,0,0),

    }).await.is_solid()

}

#[pumpkin_block("minecraft:bell")]
pub struct BellBlock;

impl WallMountedBlock for BellBlock {
    fn get_direction(&self, state_id: BlockStateId, block: &Block) -> BlockDirection {
        let props = BellLikeProperties::from_state_id(state_id, block);
        match props.attachment {
            Attachment::Ceiling => BlockDirection::Down,
            Attachment::Floor => BlockDirection::Up,
            Attachment::SingleWall => reverse_horizontal_facing(props.facing).to_block_direction(),
            Attachment::DoubleWall => reverse_horizontal_facing(props.facing).to_block_direction()
        }
    }
}

impl BlockBehaviour for BellBlock {
    fn placed<'a>(&'a self, args: PlacedArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            args.world.add_block_entity(Arc::new(BellBlockEntity::new(*args.position))).await;
        })
    }
    fn normal_use<'a>(&'a self, args: NormalUseArgs<'a>) -> BlockFuture<'a, BlockActionResult> {

        Box::pin(async move {
            let state = args.world.get_block_state(args.position).await;

            let props = BellLikeProperties::from_state_id(state.id, args.block);

            if !is_point_on_bell(args.hit, props.attachment, props.facing) {
                return BlockActionResult::Pass;  // Pass if Crosshair wasn't correctly positioned
            }
            ring_bell(*args.position,args.world,args.hit.face.to_horizontal_facing()).await;

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

            props.facing = match block_face{
                BlockFace::Floor => facing,
                BlockFace::Ceiling => facing,
                BlockFace::Wall => reverse_horizontal_facing(facing),
            };

            props.attachment=match block_face {
                BlockFace::Wall => match is_single_wall(*args.position,reverse_horizontal_facing(props.facing),args.world).await{
                    true => Attachment::SingleWall,
                    false => Attachment::DoubleWall
                }
                BlockFace::Floor => Attachment::Floor,
                BlockFace::Ceiling => Attachment::Ceiling
            };




            props.to_state_id(args.block)


        })
    }

    fn on_neighbor_update<'a>(&'a self, args: OnNeighborUpdateArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {

            let world: &World = &*args.world;

            let is_receiving_power = block_receives_redstone_power(world, args.position).await;
            let state = args.world.get_block_state(args.position).await;

            let mut props = BellLikeProperties::from_state_id(state.id, args.block);

            if props.powered!=is_receiving_power {
                props.powered=is_receiving_power;

                args.world
                    .set_block_state(
                        args.position,
                        props.to_state_id(args.block),
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await;

                if is_receiving_power {

                    ring_bell(*args.position,args.world, None).await;

                    args.world
                        .set_block_state(
                            args.position,
                            props.to_state_id(args.block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;

                }



            }

            ()
        })
    }

}