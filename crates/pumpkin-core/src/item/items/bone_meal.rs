use std::any::Any;
use std::sync::Arc;

use crate::block::registry::BlockActionResult;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use crate::world::World;
use pumpkin_data::block_properties::{
    BlockProperties, HorizontalFacing, LadderLikeProperties, WaterLikeProperties,
};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockDirection, BlockState, BlockStateId};
use pumpkin_util::gamemode::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

pub struct BoneMealItem;

fn is_full_water_source(block: &Block, state: &BlockState) -> bool {
    block == &Block::WATER && WaterLikeProperties::from_state_id(state.id).level == 0
}

fn pick_random_tagged_block(
    tag_keys: &[&'static str],
    rng: &mut impl RngExt,
) -> Option<&'static Block> {
    if tag_keys.is_empty() {
        return None;
    }
    Block::from_registry_key(tag_keys[rng.random_range(0..tag_keys.len())])
}

fn pick_coral_growth(
    attempt: usize,
    clicked_face: Option<BlockDirection>,
    rng: &mut impl RngExt,
) -> Option<(&'static Block, BlockStateId)> {
    if attempt == 0
        && let Some(face) = clicked_face
        && let Some(horizontal) = face.to_horizontal_facing()
        && let Some(wall_coral) = pick_random_tagged_block(tag::Block::MINECRAFT_WALL_CORALS.0, rng)
    {
        let mut state_id = wall_coral.default_state.id;
        if LadderLikeProperties::handles_block_id(wall_coral.id) {
            let mut props = LadderLikeProperties::from_state_id(state_id);
            props.facing = horizontal;
            props.waterlogged = true;
            state_id = props.to_state_id(wall_coral);
        }
        return Some((wall_coral, state_id));
    }

    if rng.random_range(0..4u8) == 0
        && let Some(underwater) =
            pick_random_tagged_block(tag::Block::MINECRAFT_UNDERWATER_BONEMEALS.0, rng)
    {
        let state_id = underwater
            .set_waterlogged(underwater.default_state.id, true)
            .unwrap_or(underwater.default_state.id);
        return Some((underwater, state_id));
    }
    None
}

impl BoneMealItem {
    pub fn grow_crop(stack: &mut ItemStack, world: &Arc<World>, pos: &BlockPos) -> bool {
        let (block, state) = world.get_block_and_state(pos);
        if world.block_registry.bone_meal(block, world, pos, state.id) {
            stack.decrement(1);
            if stack.item_count == 0 {
                stack.clear();
            }
            true
        } else {
            false
        }
    }

    pub fn grow_water_plant(
        stack: &mut ItemStack,
        world: &Arc<World>,
        pos: &BlockPos,
        clicked_face: Option<BlockDirection>,
    ) -> bool {
        let (initial_block, initial_state) = world.get_block_and_state(pos);
        if !is_full_water_source(initial_block, initial_state) {
            return false;
        }

        let server = world.server.upgrade();
        let mut rng = rand::rng();
        let horizontal_facings = [
            HorizontalFacing::North,
            HorizontalFacing::East,
            HorizontalFacing::South,
            HorizontalFacing::West,
        ];

        'attempts: for j in 0..128 {
            let mut test_pos = *pos;
            let mut block_to_grow: &'static Block = &Block::SEAGRASS;
            let mut state_id_to_grow = block_to_grow.default_state.id;

            for _ in 0..(j / 16) {
                let dx = rng.random_range(0..3i32) - 1;
                let dy = (rng.random_range(0..3i32) - 1) * rng.random_range(0..3i32) / 2;
                let dz = rng.random_range(0..3i32) - 1;
                test_pos = BlockPos::new(test_pos.0.x + dx, test_pos.0.y + dy, test_pos.0.z + dz);
                if world.get_block_state(&test_pos).is_full_cube() {
                    continue 'attempts;
                }
            }

            let biome = world.get_biome(&test_pos);
            if biome.has_tag(&tag::WorldgenBiome::MINECRAFT_PRODUCES_CORALS_FROM_BONEMEAL)
                && let Some((coral_block, coral_state_id)) =
                    pick_coral_growth(j, clicked_face, &mut rng)
            {
                block_to_grow = coral_block;
                state_id_to_grow = coral_state_id;
            }

            if block_to_grow.has_tag(&tag::Block::MINECRAFT_WALL_CORALS)
                && LadderLikeProperties::handles_block_id(block_to_grow.id)
            {
                for _ in 0..4 {
                    let state_obj = BlockState::from_id(state_id_to_grow);
                    if world.block_registry.can_place_at(
                        server.as_deref(),
                        Some(world.as_ref()),
                        world.as_ref(),
                        None,
                        block_to_grow,
                        state_obj,
                        &test_pos,
                        None,
                        None,
                    ) {
                        break;
                    }
                    let mut props = LadderLikeProperties::from_state_id(state_id_to_grow);
                    props.facing = horizontal_facings[rng.random_range(0..4)];
                    state_id_to_grow = props.to_state_id(block_to_grow);
                }
            }

            let state_obj = BlockState::from_id(state_id_to_grow);
            if !world.block_registry.can_place_at(
                server.as_deref(),
                Some(world.as_ref()),
                world.as_ref(),
                None,
                block_to_grow,
                state_obj,
                &test_pos,
                None,
                None,
            ) {
                continue;
            }

            let (test_block, test_state) = world.get_block_and_state(&test_pos);
            if is_full_water_source(test_block, test_state) {
                world.set_block_state(&test_pos, state_id_to_grow, BlockFlags::NOTIFY_ALL);
            } else if test_block == &Block::SEAGRASS && rng.random_range(0..10u8) == 0 {
                let _ = world
                    .block_registry
                    .bone_meal(test_block, world, &test_pos, test_state.id);
            }
        }

        stack.decrement(1);
        if stack.item_count == 0 {
            stack.clear();
        }
        true
    }
}

impl ItemMetadata for BoneMealItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::BONE_MEAL.id])
    }
}

impl ItemBehaviour for BoneMealItem {
    fn use_on_block(
        &self,
        item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        _block: &Block,
        _server: &Server,
    ) -> BlockActionResult {
        let world = player.world();
        let original = (player.gamemode.load() == GameMode::Creative).then(|| item.clone());
        if Self::grow_crop(item, &world, &location) {
            if let Some(original) = original {
                *item = original;
            }
            world.emit_game_event(GameEvent::ItemInteractFinish.name(), player.position());
            world.sync_world_event(WorldEvent::ParticlesAndSoundPlantGrowth, location, 15);
            return BlockActionResult::SuccessServer;
        }

        let clicked_state = world.get_block_state(&location);
        let relative = location.offset(face.to_offset());
        if clicked_state.is_side_solid(face)
            && Self::grow_water_plant(item, &world, &relative, Some(face))
        {
            if let Some(original) = original {
                *item = original;
            }
            world.emit_game_event(GameEvent::ItemInteractFinish.name(), player.position());
            world.sync_world_event(WorldEvent::ParticlesAndSoundPlantGrowth, relative, 15);
            BlockActionResult::Success
        } else {
            BlockActionResult::Pass
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
