use std::sync::Arc;

use pumpkin_data::block_properties::{BlockProperties, BrownMushroomBlockLikeProperties};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockId, BlockState, BlockStateId, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::{BlockAccessor, BlockFlags};
use rand::RngExt;

use crate::block::{
    BlockBehaviour, BlockMetadata, BonemealArgs, CanPlaceAtArgs, GetStateForNeighborUpdateArgs,
    RandomTickArgs, blocks::plant::PlantBlockBase,
};
use crate::plugin::api::events::world::structure_grow::{StructureGrowEvent, TreeType};
use crate::world::World;

pub struct MushroomPlantBlock;

impl BlockMetadata for MushroomPlantBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::BROWN_MUSHROOM, BlockId::RED_MUSHROOM].into()
    }
}

fn mushroom_tree_height(rng: &mut impl rand::Rng) -> i32 {
    let mut height = rng.random_range(0..3) + 4;
    if rng.random_range(0..12) == 0 {
        height *= 2;
    }
    height
}

impl MushroomPlantBlock {
    #[must_use]
    pub const fn may_place_on(state: &BlockState) -> bool {
        state.is_solid() && (state.is_full_cube() || state.is_solid_block())
    }

    pub fn can_survive(
        block_accessor: &dyn BlockAccessor,
        world: Option<&World>,
        pos: &BlockPos,
    ) -> bool {
        let below_pos = pos.down();
        let below_block = block_accessor.get_block(&below_pos);
        if below_block.has_tag(&tag::Block::MINECRAFT_OVERRIDES_MUSHROOM_LIGHT_REQUIREMENT) {
            return true;
        }

        let is_dark_enough = world.is_none_or(|world| world.get_max_local_raw_brightness(pos) < 13);

        is_dark_enough && Self::may_place_on(block_accessor.get_block_state(&below_pos))
    }

    pub fn grow_mushroom(
        world: &Arc<World>,
        pos: &BlockPos,
        block: &Block,
        _state_id: BlockStateId,
    ) -> bool {
        let species = if block == &Block::BROWN_MUSHROOM {
            TreeType::BrownMushroom
        } else if block == &Block::RED_MUSHROOM {
            TreeType::RedMushroom
        } else {
            TreeType::Custom
        };

        let mut event = StructureGrowEvent::new(*pos, species, true);
        if let Some(server) = world.server.upgrade() {
            server.plugin_manager.fire_blocking(&server, &mut event);
        }
        if event.cancelled {
            return false;
        }

        let tree_height = mushroom_tree_height(&mut rand::rng());

        if !world.is_in_height_limit(pos.0.y + tree_height + 1) {
            return false;
        }

        let foliage_radius = if block == &Block::BROWN_MUSHROOM {
            3
        } else {
            2
        };
        for dy in 0..=tree_height + 1 {
            let radius = if dy <= 3 { 0 } else { foliage_radius };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let check_pos = pos.add(dx, dy, dz);
                    if check_pos == *pos {
                        continue;
                    }
                    if !world.is_loaded(&check_pos) {
                        return false;
                    }
                    let check_state = world.get_block_state(&check_pos);
                    let check_block = world.get_block(&check_pos);
                    let can_replace = check_state.is_air()
                        || check_block.has_tag(&tag::Block::MINECRAFT_LEAVES)
                        || check_block.has_tag(&tag::Block::MINECRAFT_REPLACEABLE_BY_MUSHROOMS)
                        || check_state.replaceable();
                    if !can_replace {
                        return false;
                    }
                }
            }
        }

        world.set_block_state(pos, BlockStateId::AIR, BlockFlags::NOTIFY_ALL);

        if block == &Block::BROWN_MUSHROOM {
            place_huge_brown_mushroom(world, pos, tree_height);
        } else if block == &Block::RED_MUSHROOM {
            place_huge_red_mushroom(world, pos, tree_height);
        }

        true
    }
}

fn place_huge_brown_mushroom(world: &Arc<World>, pos: &BlockPos, tree_height: i32) {
    let radius = 3;
    let cap_y = pos.0.y + tree_height;
    for j in -radius..=radius {
        for k in -radius..=radius {
            let on_x_edge = j == -radius || j == radius;
            let on_z_edge = k == -radius || k == radius;

            if on_x_edge && on_z_edge {
                continue;
            }

            let props = BrownMushroomBlockLikeProperties {
                up: true,
                down: false,
                west: j == -radius || (on_z_edge && j == 1 - radius),
                east: j == radius || (on_z_edge && j == radius - 1),
                north: k == -radius || (on_x_edge && k == 1 - radius),
                south: k == radius || (on_x_edge && k == radius - 1),
            };
            let state_id = props.to_state_id(&Block::BROWN_MUSHROOM_BLOCK);
            let cap_pos = BlockPos::new(pos.0.x + j, cap_y, pos.0.z + k);
            world.set_block_state(&cap_pos, state_id, BlockFlags::NOTIFY_ALL);
        }
    }

    let stem_props = BrownMushroomBlockLikeProperties {
        up: false,
        down: false,
        north: true,
        east: true,
        south: true,
        west: true,
    };
    let stem_state = stem_props.to_state_id(&Block::MUSHROOM_STEM);
    for i in 0..tree_height {
        let stem_pos = BlockPos::new(pos.0.x, pos.0.y + i, pos.0.z);
        world.set_block_state(&stem_pos, stem_state, BlockFlags::NOTIFY_ALL);
    }
}

fn place_huge_red_mushroom(world: &Arc<World>, pos: &BlockPos, tree_height: i32) {
    let radius = 2;
    for i in (tree_height - 3)..=tree_height {
        let j = if i < tree_height { radius } else { radius - 1 };
        let k = radius - 2;

        for l in -j..=j {
            for m in -j..=j {
                let on_x_edge = l == -j || l == j;
                let on_z_edge = m == -j || m == j;

                if i < tree_height && on_x_edge == on_z_edge {
                    continue;
                }

                let props = BrownMushroomBlockLikeProperties {
                    up: i >= tree_height - 1,
                    down: false,
                    west: l < -k,
                    east: l > k,
                    north: m < -k,
                    south: m > k,
                };
                let state_id = props.to_state_id(&Block::RED_MUSHROOM_BLOCK);
                let cap_pos = BlockPos::new(pos.0.x + l, pos.0.y + i, pos.0.z + m);
                world.set_block_state(&cap_pos, state_id, BlockFlags::NOTIFY_ALL);
            }
        }
    }

    let stem_props = BrownMushroomBlockLikeProperties {
        up: false,
        down: false,
        north: true,
        east: true,
        south: true,
        west: true,
    };
    let stem_state = stem_props.to_state_id(&Block::MUSHROOM_STEM);
    for i in 0..tree_height {
        let stem_pos = BlockPos::new(pos.0.x, pos.0.y + i, pos.0.z);
        world.set_block_state(&stem_pos, stem_state, BlockFlags::NOTIFY_ALL);
    }
}

impl BlockBehaviour for MushroomPlantBlock {
    fn can_place_at(&self, args: CanPlaceAtArgs<'_>) -> bool {
        Self::can_survive(args.block_accessor, args.world, args.position)
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if !Self::can_survive(args.world, Some(args.world), args.position) {
            return Block::AIR.default_state.id;
        }
        args.state_id
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        if rand::rng().random_range(0..25) != 0 {
            return;
        }
        let pos = *args.position;
        let world = args.world;
        let this_block = args.block;
        let state_id = world.get_block_state_id(&pos);

        let mut max = 5;
        for dx in -4..=4 {
            for dy in -1..=1 {
                for dz in -4..=4 {
                    let check_pos = pos.add(dx, dy, dz);
                    if world.is_loaded(&check_pos) && world.get_block(&check_pos) == this_block {
                        max -= 1;
                        if max <= 0 {
                            return;
                        }
                    }
                }
            }
        }

        let mut current_pos = pos;
        let mut offset = current_pos.add(
            rand::rng().random_range(0..3) - 1,
            rand::rng().random_range(0..2) - rand::rng().random_range(0..2),
            rand::rng().random_range(0..3) - 1,
        );

        for _ in 0..4 {
            if world.is_loaded(&offset)
                && world.get_block_state(&offset).is_air()
                && Self::can_survive(world.as_ref(), Some(world.as_ref()), &offset)
            {
                current_pos = offset;
            }
            offset = current_pos.add(
                rand::rng().random_range(0..3) - 1,
                rand::rng().random_range(0..2) - rand::rng().random_range(0..2),
                rand::rng().random_range(0..3) - 1,
            );
        }

        if world.is_loaded(&offset)
            && world.get_block_state(&offset).is_air()
            && Self::can_survive(world.as_ref(), Some(world.as_ref()), &offset)
        {
            world.set_block_state(&offset, state_id, BlockFlags::NOTIFY_LISTENERS);
        }
    }

    fn is_valid_bonemeal_target(&self, args: BonemealArgs<'_>) -> bool {
        let foliage_radius = if args.block == &Block::BROWN_MUSHROOM {
            3
        } else {
            2
        };
        let min_height = 4 + foliage_radius;
        args.world
            .is_in_height_limit(args.position.0.y + min_height)
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        rand::rng().random::<f32>() < 0.4
    }

    fn perform_bonemeal(&self, args: BonemealArgs<'_>) {
        Self::grow_mushroom(args.world, args.position, args.block, args.state_id);
    }
}

impl PlantBlockBase for MushroomPlantBlock {
    fn can_plant_on_top(&self, block_accessor: &dyn BlockAccessor, pos: &BlockPos) -> bool {
        let state = block_accessor.get_block_state(pos);
        Self::may_place_on(state)
    }

    fn can_place_at(&self, block_accessor: &dyn BlockAccessor, block_pos: &BlockPos) -> bool {
        Self::can_survive(block_accessor, None, block_pos)
    }
}
