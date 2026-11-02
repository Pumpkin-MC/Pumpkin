use crate::block::{
    BlockBehaviour, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs, OnPlaceArgs,
    PlayerPlacedArgs, is_exception_for_connection,
};
use pumpkin_data::block_properties::{
    EastWall, HorizontalFacing, NorthWall, PaleMossCarpetLikeProperties, SouthWall, WestWall,
    is_air,
};
use pumpkin_data::{Block, BlockDirection, BlockStateId, HorizontalFacingExt, Mirror, Rotation};
use pumpkin_macros::{pumpkin_block, pumpkin_block_from_tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

#[pumpkin_block_from_tag("minecraft:wool_carpets")]
pub struct CarpetBlock;

impl BlockBehaviour for CarpetBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !can_place_at(args.world, args.position) {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }
}

#[pumpkin_block("minecraft:moss_carpet")]
pub struct MossCarpetBlock;

impl BlockBehaviour for MossCarpetBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        can_place_at(args.block_accessor, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !can_place_at(args.world, args.position) {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }
}

#[pumpkin_block("minecraft:pale_moss_carpet")]
pub struct PaleMossCarpetBlock;

impl PaleMossCarpetBlock {
    fn can_survive(
        state_id: BlockStateId,
        block_accessor: &dyn BlockAccessor,
        pos: &BlockPos,
    ) -> bool {
        let props = PaleMossCarpetLikeProperties::from_state_id(state_id);
        let below_pos = pos.down();
        let (below_block, below_state) = block_accessor.get_block_and_state(&below_pos);
        if props.bottom {
            !below_state.is_air()
        } else {
            below_block == &Block::PALE_MOSS_CARPET
                && PaleMossCarpetLikeProperties::from_state_id(below_state.id).bottom
        }
    }

    const fn has_faces(props: PaleMossCarpetLikeProperties) -> bool {
        props.bottom
            || matches!(props.north, NorthWall::Low | NorthWall::Tall)
            || matches!(props.east, EastWall::Low | EastWall::Tall)
            || matches!(props.south, SouthWall::Low | SouthWall::Tall)
            || matches!(props.west, WestWall::Low | WestWall::Tall)
    }

    fn can_support_at_face(
        block_accessor: &dyn BlockAccessor,
        pos: &BlockPos,
        direction: BlockDirection,
    ) -> bool {
        if direction == BlockDirection::Up {
            return false;
        }
        let neighbor_pos = pos.offset(direction.to_offset());
        let (neighbor_block, neighbor_state) = block_accessor.get_block_and_state(&neighbor_pos);
        neighbor_state.is_side_solid(direction.opposite())
            && !is_exception_for_connection(neighbor_block)
    }

    const fn get_side(props: PaleMossCarpetLikeProperties, dir: HorizontalFacing) -> u8 {
        match dir {
            HorizontalFacing::North => match props.north {
                NorthWall::None => 0,
                NorthWall::Low => 1,
                NorthWall::Tall => 2,
            },
            HorizontalFacing::East => match props.east {
                EastWall::None => 0,
                EastWall::Low => 1,
                EastWall::Tall => 2,
            },
            HorizontalFacing::South => match props.south {
                SouthWall::None => 0,
                SouthWall::Low => 1,
                SouthWall::Tall => 2,
            },
            HorizontalFacing::West => match props.west {
                WestWall::None => 0,
                WestWall::Low => 1,
                WestWall::Tall => 2,
            },
        }
    }

    const fn set_side(props: &mut PaleMossCarpetLikeProperties, dir: HorizontalFacing, side: u8) {
        match dir {
            HorizontalFacing::North => {
                props.north = match side {
                    1 => NorthWall::Low,
                    2 => NorthWall::Tall,
                    _ => NorthWall::None,
                };
            }
            HorizontalFacing::East => {
                props.east = match side {
                    1 => EastWall::Low,
                    2 => EastWall::Tall,
                    _ => EastWall::None,
                };
            }
            HorizontalFacing::South => {
                props.south = match side {
                    1 => SouthWall::Low,
                    2 => SouthWall::Tall,
                    _ => SouthWall::None,
                };
            }
            HorizontalFacing::West => {
                props.west = match side {
                    1 => WestWall::Low,
                    2 => WestWall::Tall,
                    _ => WestWall::None,
                };
            }
        }
    }

    fn get_updated_state(
        mut props: PaleMossCarpetLikeProperties,
        block_accessor: &dyn BlockAccessor,
        pos: &BlockPos,
        mut create_sides: bool,
    ) -> PaleMossCarpetLikeProperties {
        let mut above_props: Option<PaleMossCarpetLikeProperties> = None;
        let mut below_props: Option<PaleMossCarpetLikeProperties> = None;
        create_sides |= props.bottom;

        for dir in HorizontalFacing::all() {
            let block_dir = dir.to_block_direction();
            let mut side = if Self::can_support_at_face(block_accessor, pos, block_dir) {
                if create_sides {
                    1
                } else {
                    Self::get_side(props, dir)
                }
            } else {
                0
            };

            if side == 1 {
                if above_props.is_none() {
                    let (above_block, above_state) = block_accessor.get_block_and_state(&pos.up());
                    if above_block == &Block::PALE_MOSS_CARPET {
                        above_props =
                            Some(PaleMossCarpetLikeProperties::from_state_id(above_state.id));
                    }
                }
                if let Some(above) = above_props
                    && Self::get_side(above, dir) != 0
                    && !above.bottom
                {
                    side = 2;
                }

                if !props.bottom {
                    if below_props.is_none() {
                        let (below_block, below_state) =
                            block_accessor.get_block_and_state(&pos.down());
                        if below_block == &Block::PALE_MOSS_CARPET {
                            below_props =
                                Some(PaleMossCarpetLikeProperties::from_state_id(below_state.id));
                        }
                    }
                    if let Some(below) = below_props
                        && Self::get_side(below, dir) == 0
                    {
                        side = 0;
                    }
                }
            }

            Self::set_side(&mut props, dir, side);
        }

        props
    }

    fn create_topper_with_side_chance(
        block_accessor: &dyn BlockAccessor,
        pos: &BlockPos,
        mut side_survival_test: impl FnMut() -> bool,
    ) -> BlockStateId {
        let above = pos.up();
        let (above_block, above_state) = block_accessor.get_block_and_state(&above);
        let is_mossy_carpet_above = above_block == &Block::PALE_MOSS_CARPET;
        let above_has_base = is_mossy_carpet_above
            && PaleMossCarpetLikeProperties::from_state_id(above_state.id).bottom;

        if (!is_mossy_carpet_above || !above_has_base)
            && (is_mossy_carpet_above || above_state.replaceable())
        {
            let mut no_base = PaleMossCarpetLikeProperties::default(&Block::PALE_MOSS_CARPET);
            no_base.bottom = false;
            let mut above_props = Self::get_updated_state(no_base, block_accessor, &above, true);

            for dir in HorizontalFacing::all() {
                if Self::get_side(above_props, dir) != 0 && !side_survival_test() {
                    Self::set_side(&mut above_props, dir, 0);
                }
            }

            let new_state_id = above_props.to_state_id(&Block::PALE_MOSS_CARPET);
            if Self::has_faces(above_props) && new_state_id != above_state.id {
                new_state_id
            } else {
                Block::AIR.default_state.id
            }
        } else {
            Block::AIR.default_state.id
        }
    }
}

impl BlockBehaviour for PaleMossCarpetBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::can_survive(args.state.id, args.block_accessor, args.position)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let default_props = PaleMossCarpetLikeProperties::default(args.block);
        Self::get_updated_state(default_props, args.world, args.position, true)
            .to_state_id(args.block)
    }

    fn player_placed(&self, args: PlayerPlacedArgs<'_>) {
        let mut rng = rand::rng();
        let topper =
            Self::create_topper_with_side_chance(args.world.as_ref(), args.position, || {
                rng.random::<bool>()
            });
        if topper != Block::AIR.default_state.id {
            args.world
                .set_block_state(&args.position.up(), topper, BlockFlags::NOTIFY_ALL);
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !Self::can_survive(args.state_id, args.world, args.position) {
            return Block::AIR.default_state.id;
        }
        let props = PaleMossCarpetLikeProperties::from_state_id(args.state_id);
        let updated = Self::get_updated_state(props, args.world, args.position, false);
        if Self::has_faces(updated) {
            updated.to_state_id(args.block)
        } else {
            Block::AIR.default_state.id
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        let props = PaleMossCarpetLikeProperties::from_state_id(args.state_id);
        props.bottom
            && Self::create_topper_with_side_chance(args.world.as_ref(), args.position, || true)
                != Block::AIR.default_state.id
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        let topper =
            Self::create_topper_with_side_chance(args.world.as_ref(), args.position, || true);
        if topper != Block::AIR.default_state.id {
            args.world
                .set_block_state(&args.position.up(), topper, BlockFlags::NOTIFY_ALL);
        }
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static pumpkin_data::BlockState {
        let mut props = PaleMossCarpetLikeProperties::from_state_id(state_id);
        let n = Self::get_side(props, HorizontalFacing::North);
        let e = Self::get_side(props, HorizontalFacing::East);
        let s = Self::get_side(props, HorizontalFacing::South);
        let w = Self::get_side(props, HorizontalFacing::West);
        match rotation {
            Rotation::Rotate180 => {
                Self::set_side(&mut props, HorizontalFacing::North, s);
                Self::set_side(&mut props, HorizontalFacing::East, w);
                Self::set_side(&mut props, HorizontalFacing::South, n);
                Self::set_side(&mut props, HorizontalFacing::West, e);
            }
            Rotation::CounterClockwise90 => {
                Self::set_side(&mut props, HorizontalFacing::North, e);
                Self::set_side(&mut props, HorizontalFacing::East, s);
                Self::set_side(&mut props, HorizontalFacing::South, w);
                Self::set_side(&mut props, HorizontalFacing::West, n);
            }
            Rotation::Clockwise90 => {
                Self::set_side(&mut props, HorizontalFacing::North, w);
                Self::set_side(&mut props, HorizontalFacing::East, n);
                Self::set_side(&mut props, HorizontalFacing::South, e);
                Self::set_side(&mut props, HorizontalFacing::West, s);
            }
            Rotation::None => {}
        }
        pumpkin_data::BlockState::from_id(props.to_state_id(block))
    }

    fn mirror(
        &self,
        block: &Block,
        state_id: BlockStateId,
        mirror: Mirror,
    ) -> &'static pumpkin_data::BlockState {
        let mut props = PaleMossCarpetLikeProperties::from_state_id(state_id);
        match mirror {
            Mirror::LeftRight => {
                let n = Self::get_side(props, HorizontalFacing::North);
                let s = Self::get_side(props, HorizontalFacing::South);
                Self::set_side(&mut props, HorizontalFacing::North, s);
                Self::set_side(&mut props, HorizontalFacing::South, n);
            }
            Mirror::FrontBack => {
                let e = Self::get_side(props, HorizontalFacing::East);
                let w = Self::get_side(props, HorizontalFacing::West);
                Self::set_side(&mut props, HorizontalFacing::East, w);
                Self::set_side(&mut props, HorizontalFacing::West, e);
            }
            Mirror::None => {}
        }
        pumpkin_data::BlockState::from_id(props.to_state_id(block))
    }
}

fn can_place_at(block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
    !is_air(block_accessor.get_block_state_id(&block_pos.down()))
}
