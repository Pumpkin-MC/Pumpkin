use crate::Arc;
use crate::block::blocks::abstract_wall_mounting::WallMountedBlock;
use crate::block::blocks::redstone::block_receives_redstone_power;
use crate::block::entities::bell::BellBlockEntity;
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockHitResult, BrokenArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    NormalUseArgs, OnNeighborUpdateArgs, OnPlaceArgs, OnProjectileHitArgs, PathComputationType,
    PlacedArgs,
};
use crate::entity::EntityBase;
use crate::world::World;
use pumpkin_data::HorizontalFacingExt;
use pumpkin_data::block_properties::{Axis, BellAttachment, BellLikeProperties, HorizontalFacing};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::sound::Sound;
use pumpkin_data::sound::SoundCategory;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

fn ring_bell(
    position: BlockPos,
    world: &Arc<World>,
    hit_direction: Option<HorizontalFacing>,
    entity: Option<Arc<dyn crate::entity::EntityBase>>,
) -> bool {
    let mut event = crate::plugin::block::bell_ring::BellRingEvent {
        block_pos: position,
        world: world.clone(),
        direction: hit_direction.map(|d| d.to_block_direction()),
        entity,
        cancelled: false,
    };
    if let Some(server) = world.server.upgrade() {
        server.plugin_manager.fire_blocking(&server, &mut event);
    }
    if event.cancelled {
        return false;
    }

    let (_block, state_id) = world.get_block_and_state_id(&position);

    let props = BellLikeProperties::from_state_id(state_id);
    let direction = hit_direction.unwrap_or(props.facing);

    if let Some(block_entity) = world.get_block_entity(&position)
        && let Some(be) = block_entity.as_any().downcast_ref::<BellBlockEntity>()
    {
        be.activate(direction);
    }

    world.play_sound_fine(
        Sound::BlockBellUse,
        SoundCategory::Blocks,
        &position.to_centered_f64(),
        2.0,
        1.0,
    );
    world.emit_game_event(GameEvent::BlockChange.name(), position.to_centered_f64());

    true
}

fn is_point_on_bell(
    hit: &BlockHitResult,
    attachment: BellAttachment,
    block_face: HorizontalFacing,
) -> bool {
    if hit.face == &BlockDirection::Up || hit.face == &BlockDirection::Down {
        return false;
    }
    if hit.cursor_pos.y <= 0.8124f32 {
        match attachment {
            BellAttachment::Floor => {
                hit.face.to_axis() == block_face.to_block_direction().to_axis()
            }
            BellAttachment::SingleWall | BellAttachment::DoubleWall => {
                hit.face.to_axis() != block_face.to_block_direction().to_axis()
            }
            BellAttachment::Ceiling => true,
        }
    } else {
        false
    }
}

#[pumpkin_block("minecraft:bell")]
pub struct BellBlock;

impl BellBlock {
    fn can_survive_dir(
        &self,
        block_accessor: &dyn BlockAccessor,
        position: &BlockPos,
        direction: BlockDirection,
    ) -> bool {
        if direction == BlockDirection::Up {
            crate::block::can_support_center(block_accessor, &position.up(), BlockDirection::Down)
        } else {
            WallMountedBlock::can_place_at(self, block_accessor, position, direction)
        }
    }
}

impl WallMountedBlock for BellBlock {
    fn get_direction(&self, state_id: BlockStateId, _block: &Block) -> BlockDirection {
        let props = BellLikeProperties::from_state_id(state_id);
        match props.attachment {
            BellAttachment::Ceiling => BlockDirection::Down,
            BellAttachment::Floor => BlockDirection::Up,
            BellAttachment::SingleWall | BellAttachment::DoubleWall => {
                props.facing.opposite().to_block_direction()
            }
        }
    }
}

impl BlockBehaviour for BellBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        let direction = args
            .direction
            .unwrap_or_else(|| self.get_direction(args.state.id, args.block).opposite());
        self.can_survive_dir(args.block_accessor, args.position, direction)
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        let world: &World = args.world;
        world.remove_block_entity(args.position);
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        args.world
            .add_block_entity(Arc::new(BellBlockEntity::new(*args.position)));
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        let state = args.world.get_block_state(args.position);

        let props = BellLikeProperties::from_state_id(state.id);

        if !is_point_on_bell(args.hit, props.attachment, props.facing) {
            return BlockActionResult::Pass; // Pass if Crosshair wasn't correctly positioned
        }
        if !ring_bell(
            *args.position,
            args.world,
            args.hit.face.to_horizontal_facing(),
            Some(args.player.clone()),
        ) {
            return BlockActionResult::Pass;
        }

        args.player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            pumpkin_data::statistic::CustomStatistic::BellRing as i32,
            1,
        );

        BlockActionResult::Success
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = BellLikeProperties::default(args.block);
        let axis = args.direction.to_axis();

        if axis == Axis::Y {
            props.attachment = if args.direction == BlockDirection::Up {
                BellAttachment::Ceiling
            } else {
                BellAttachment::Floor
            };
            props.facing = args.player.get_entity().get_horizontal_facing();
        } else {
            let double_attached = (axis == Axis::X
                && args
                    .world
                    .get_block_state(&args.position.west())
                    .is_side_solid(BlockDirection::East)
                && args
                    .world
                    .get_block_state(&args.position.east())
                    .is_side_solid(BlockDirection::West))
                || (axis == Axis::Z
                    && args
                        .world
                        .get_block_state(&args.position.north())
                        .is_side_solid(BlockDirection::South)
                    && args
                        .world
                        .get_block_state(&args.position.south())
                        .is_side_solid(BlockDirection::North));
            props.facing = args.direction.to_cardinal_direction();
            props.attachment = if double_attached {
                BellAttachment::DoubleWall
            } else {
                BellAttachment::SingleWall
            };
            let state_id = props.to_state_id(args.block);
            if self.can_survive_dir(
                args.world,
                args.position,
                self.get_direction(state_id, args.block).opposite(),
            ) {
                return state_id;
            }

            let can_attach_below = args
                .world
                .get_block_state(&args.position.down())
                .is_side_solid(BlockDirection::Up);
            props.attachment = if can_attach_below {
                BellAttachment::Floor
            } else {
                BellAttachment::Ceiling
            };
        }

        let state_id = props.to_state_id(args.block);
        if self.can_survive_dir(
            args.world,
            args.position,
            self.get_direction(state_id, args.block).opposite(),
        ) {
            return state_id;
        }

        Block::AIR.default_state.id
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        let mut props = BellLikeProperties::from_state_id(args.state_id);
        let connected_dir = self.get_direction(args.state_id, args.block).opposite();
        if connected_dir == args.direction
            && !self.can_survive_dir(args.world, args.position, connected_dir)
            && props.attachment != BellAttachment::DoubleWall
        {
            return Block::AIR.default_state.id;
        }
        if args.direction.to_axis() == props.facing.to_block_direction().to_axis() {
            let neighbor_state = BlockState::from_id(args.neighbor_state_id);
            if props.attachment == BellAttachment::DoubleWall
                && !neighbor_state.is_side_solid(args.direction.opposite())
            {
                props.attachment = BellAttachment::SingleWall;
                props.facing = args.direction.opposite().to_cardinal_direction();
                return props.to_state_id(args.block);
            }
            if props.attachment == BellAttachment::SingleWall
                && connected_dir.opposite() == args.direction
                && neighbor_state.is_side_solid(props.facing.to_block_direction())
            {
                props.attachment = BellAttachment::DoubleWall;
                return props.to_state_id(args.block);
            }
        }
        args.state_id
    }

    fn on_neighbor_update(&self, args: OnNeighborUpdateArgs<'_>) {
        let world: &World = args.world;

        let is_receiving_power = block_receives_redstone_power(world, args.position);
        let state = args.world.get_block_state(args.position);

        let mut props = BellLikeProperties::from_state_id(state.id);

        if props.powered != is_receiving_power {
            props.powered = is_receiving_power;

            args.world.set_block_state(
                args.position,
                props.to_state_id(args.block),
                BlockFlags::NOTIFY_ALL,
            );

            if is_receiving_power {
                ring_bell(*args.position, args.world, None, None);
            }
        }
    }

    fn on_projectile_hit(&self, args: OnProjectileHitArgs<'_>) {
        let player = args
            .projectile
            .get_owner_id()
            .and_then(|id| args.world.get_player_by_id(id));
        if ring_bell(
            *args.position,
            args.world,
            None,
            player
                .as_ref()
                .map(|p| p.clone() as Arc<dyn crate::entity::EntityBase>),
        ) && let Some(player) = player
        {
            player.increment_stat(
                pumpkin_data::statistic::StatisticCategory::Custom,
                pumpkin_data::statistic::CustomStatistic::BellRing as i32,
                1,
            );
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}
