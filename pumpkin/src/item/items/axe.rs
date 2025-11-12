use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_properties::{
    BlockProperties, ChestLikeProperties, ChestType, CopperGolemStatueLikeProperties,
};
use pumpkin_data::block_properties::{
    LanternLikeProperties, LightningRodLikeProperties, OakDoorLikeProperties,
    OakFenceLikeProperties, OakTrapdoorLikeProperties, PaleOakWoodLikeProperties,
};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, tag};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::BlockFlags;

pub struct AxeItem;

impl ItemMetadata for AxeItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_AXES.1.to_vec().into_boxed_slice()
    }
}

#[async_trait]
impl ItemBehaviour for AxeItem {
    #[allow(clippy::too_many_lines)]
    async fn use_on_block(
        &self,
        _item: &mut ItemStack,
        player: &Player,
        location: BlockPos,
        _face: BlockDirection,
        block: &Block,
        _server: &Server,
    ) {
        // I tried to follow mojang order of doing things.
        let world = player.world();
        let replacement_block = try_use_axe(block);
        // First we try to strip the block. by getting his equivalent and applying it the axis.

        // If there is a strip equivalent.
        if replacement_block != 0 {
            let new_block = &Block::from_id(replacement_block);
            let new_state_id = if block.has_tag(&tag::Block::MINECRAFT_LOGS) {
                let log_information = world.get_block_state_id(&location).await;
                let log_props = PaleOakWoodLikeProperties::from_state_id(log_information, block);
                // create new properties for the new log.
                let mut new_log_properties = PaleOakWoodLikeProperties::default(new_block);
                new_log_properties.axis = log_props.axis;

                // create new properties for the new log.

                // Set old axis to the new log.
                new_log_properties.axis = log_props.axis;
                new_log_properties.to_state_id(new_block)
            }
            // Let's check if It's a door
            else if block.has_tag(&tag::Block::MINECRAFT_DOORS) {
                // get block state of the old log.
                let door_information = world.get_block_state_id(&location).await;
                // get the log properties
                let door_props = OakDoorLikeProperties::from_state_id(door_information, block);
                // create new properties for the new log.
                let mut new_door_properties = OakDoorLikeProperties::default(new_block);
                // Set old axis to the new log.
                new_door_properties.facing = door_props.facing;
                new_door_properties.open = door_props.open;
                new_door_properties.half = door_props.half;
                new_door_properties.hinge = door_props.hinge;
                new_door_properties.powered = door_props.powered;
                new_door_properties.to_state_id(new_block)
            } else if block.has_tag(&tag::Block::MINECRAFT_BARS) {
                let bar_information = world.get_block_state_id(&location).await;
                let bar_props = OakFenceLikeProperties::from_state_id(bar_information, block);
                let mut new_bars_props = OakFenceLikeProperties::default(new_block);
                new_bars_props.north = bar_props.north;
                new_bars_props.south = bar_props.south;
                new_bars_props.west = bar_props.west;
                new_bars_props.east = bar_props.east;
                new_bars_props.waterlogged = bar_props.waterlogged;
                new_bars_props.to_state_id(new_block)
            } else if block.has_tag(&tag::Block::MINECRAFT_LANTERNS) {
                let lantern_information = world.get_block_state_id(&location).await;
                let lantern_props =
                    LanternLikeProperties::from_state_id(lantern_information, block);
                let mut new_lantern_props = LanternLikeProperties::default(new_block);
                new_lantern_props.hanging = lantern_props.hanging;
                new_lantern_props.waterlogged = lantern_props.waterlogged;
                new_lantern_props.to_state_id(new_block)
            } else if block.has_tag(&tag::Block::MINECRAFT_TRAPDOORS) {
                let info = world.get_block_state_id(&location).await;
                let trapdoor_props = OakTrapdoorLikeProperties::from_state_id(info, block);
                let mut new_props = OakTrapdoorLikeProperties::default(new_block);
                new_props.powered = trapdoor_props.powered;
                new_props.open = trapdoor_props.open;
                new_props.facing = trapdoor_props.facing;
                new_props.half = trapdoor_props.half;
                new_props.waterlogged = trapdoor_props.waterlogged;
                new_props.to_state_id(new_block)
            } else if block.has_tag(&tag::Block::MINECRAFT_LIGHTNING_RODS) {
                let info = world.get_block_state_id(&location).await;
                let rod_props = LightningRodLikeProperties::from_state_id(info, block);
                let mut new_props = LightningRodLikeProperties::default(new_block);
                new_props.powered = rod_props.powered;
                new_props.facing = rod_props.facing;
                new_props.waterlogged = rod_props.waterlogged;
                new_props.to_state_id(new_block)
            } else if block.has_tag(&tag::Block::MINECRAFT_COPPER_CHESTS) {
                let info = world.get_block_state_id(&location).await;
                let chest_props = ChestLikeProperties::from_state_id(info, block);
                let mut new_props = ChestLikeProperties::default(new_block);
                new_props.r#type = chest_props.r#type;
                new_props.facing = chest_props.facing;
                new_props.waterlogged = chest_props.waterlogged;
                if new_props.r#type != ChestType::Single {
                    let connected_towards = match chest_props.r#type {
                        ChestType::Single => return,
                        ChestType::Left => chest_props.facing.rotate_clockwise(),
                        ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
                    };
                    let neighbor_location = location.offset(connected_towards.to_offset());
                    let neighbor_info = world.get_block_state_id(&neighbor_location).await;
                    let neighbor_chest_props =
                        ChestLikeProperties::from_state_id(neighbor_info, block);
                    let mut neighbor_props = ChestLikeProperties::default(new_block);
                    neighbor_props.r#type = neighbor_chest_props.r#type;
                    neighbor_props.facing = neighbor_chest_props.facing;
                    neighbor_props.waterlogged = neighbor_chest_props.waterlogged;
                    world
                        .set_block_state(
                            &neighbor_location,
                            neighbor_props.to_state_id(new_block),
                            BlockFlags::NOTIFY_ALL,
                        )
                        .await;
                }
                new_props.to_state_id(new_block)
            } else if block.has_tag(&tag::Block::MINECRAFT_COPPER_GOLEM_STATUES) {
                if block.id != new_block.id && new_block.id == Block::AIR.id {
                    todo!("Golem spawn")
                }
                let info = world.get_block_state_id(&location).await;
                let old_props = CopperGolemStatueLikeProperties::from_state_id(info, block);
                let mut new_props = CopperGolemStatueLikeProperties::default(new_block);
                new_props.copper_golem_pose = old_props.copper_golem_pose;
                new_props.facing = old_props.facing;
                new_props.waterlogged = old_props.waterlogged;
                new_props.to_state_id(new_block)
            } else {
                new_block.default_state.id
            };
            world
                .set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL)
                .await;
            return;
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
fn try_use_axe(block: &Block) -> u16 {
    // Trying to get the strip equivalent
    let block_id = get_stripped_equivalent(block);
    if block_id != 0 {
        return block_id;
    }
    // Else decrease the level of oxidation
    let block_id = get_deoxidized_equivalent(block);
    if block_id != 0 {
        return block_id;
    }
    // Else unwax the block
    get_unwaxed_equivalent(block)
}

fn get_stripped_equivalent(block: &Block) -> u16 {
    match block.id {
        id if id == Block::OAK_LOG.id => Block::STRIPPED_OAK_LOG.id,
        id if id == Block::SPRUCE_LOG.id => Block::STRIPPED_SPRUCE_LOG.id,
        id if id == Block::BIRCH_LOG.id => Block::STRIPPED_BIRCH_LOG.id,
        id if id == Block::JUNGLE_LOG.id => Block::STRIPPED_JUNGLE_LOG.id,
        id if id == Block::ACACIA_LOG.id => Block::STRIPPED_ACACIA_LOG.id,
        id if id == Block::DARK_OAK_LOG.id => Block::STRIPPED_DARK_OAK_LOG.id,
        id if id == Block::MANGROVE_LOG.id => Block::STRIPPED_MANGROVE_LOG.id,
        id if id == Block::CHERRY_LOG.id => Block::STRIPPED_CHERRY_LOG.id,
        id if id == Block::PALE_OAK_LOG.id => Block::STRIPPED_PALE_OAK_LOG.id,

        id if id == Block::OAK_WOOD.id => Block::STRIPPED_OAK_WOOD.id,
        id if id == Block::SPRUCE_WOOD.id => Block::STRIPPED_SPRUCE_WOOD.id,
        id if id == Block::BIRCH_WOOD.id => Block::STRIPPED_BIRCH_WOOD.id,
        id if id == Block::JUNGLE_WOOD.id => Block::STRIPPED_JUNGLE_WOOD.id,
        id if id == Block::ACACIA_WOOD.id => Block::STRIPPED_ACACIA_WOOD.id,
        id if id == Block::DARK_OAK_WOOD.id => Block::STRIPPED_DARK_OAK_WOOD.id,
        id if id == Block::MANGROVE_WOOD.id => Block::STRIPPED_MANGROVE_WOOD.id,
        id if id == Block::CHERRY_WOOD.id => Block::STRIPPED_CHERRY_WOOD.id,
        id if id == Block::PALE_OAK_WOOD.id => Block::STRIPPED_PALE_OAK_WOOD.id,
        _ => 0,
    }
}

#[allow(clippy::too_many_lines)]
fn get_deoxidized_equivalent(block: &Block) -> u16 {
    match block.id {
        id if id == Block::OXIDIZED_COPPER.id => Block::WEATHERED_COPPER.id,
        id if id == Block::WEATHERED_COPPER.id => Block::EXPOSED_COPPER.id,
        id if id == Block::EXPOSED_COPPER.id => Block::COPPER_BLOCK.id,
        id if id == Block::OXIDIZED_CHISELED_COPPER.id => Block::WEATHERED_CHISELED_COPPER.id,
        id if id == Block::WEATHERED_CHISELED_COPPER.id => Block::EXPOSED_CHISELED_COPPER.id,
        id if id == Block::EXPOSED_CHISELED_COPPER.id => Block::CHISELED_COPPER.id,
        id if id == Block::OXIDIZED_COPPER_GRATE.id => Block::WEATHERED_COPPER_GRATE.id,
        id if id == Block::WEATHERED_COPPER_GRATE.id => Block::EXPOSED_COPPER_GRATE.id,
        id if id == Block::EXPOSED_COPPER_GRATE.id => Block::COPPER_GRATE.id,
        id if id == Block::OXIDIZED_CUT_COPPER.id => Block::WEATHERED_CUT_COPPER.id,
        id if id == Block::WEATHERED_CUT_COPPER.id => Block::EXPOSED_CUT_COPPER.id,
        id if id == Block::EXPOSED_CUT_COPPER.id => Block::CUT_COPPER.id,
        id if id == Block::OXIDIZED_CUT_COPPER_STAIRS.id => Block::WEATHERED_CUT_COPPER_STAIRS.id,
        id if id == Block::WEATHERED_CUT_COPPER_STAIRS.id => Block::EXPOSED_CUT_COPPER_STAIRS.id,
        id if id == Block::EXPOSED_CUT_COPPER_STAIRS.id => Block::CUT_COPPER_STAIRS.id,
        id if id == Block::OXIDIZED_CUT_COPPER_SLAB.id => Block::WEATHERED_CUT_COPPER_SLAB.id,
        id if id == Block::WEATHERED_CUT_COPPER_SLAB.id => Block::EXPOSED_CUT_COPPER_SLAB.id,
        id if id == Block::EXPOSED_CUT_COPPER_SLAB.id => Block::CUT_COPPER_SLAB.id,
        id if id == Block::OXIDIZED_COPPER_BULB.id => Block::WEATHERED_COPPER_BULB.id,
        id if id == Block::WEATHERED_COPPER_BULB.id => Block::EXPOSED_COPPER_BULB.id,
        id if id == Block::EXPOSED_COPPER_BULB.id => Block::COPPER_BULB.id,
        id if id == Block::OXIDIZED_COPPER_DOOR.id => Block::WEATHERED_COPPER_DOOR.id,
        id if id == Block::WEATHERED_COPPER_DOOR.id => Block::EXPOSED_COPPER_DOOR.id,
        id if id == Block::EXPOSED_COPPER_DOOR.id => Block::COPPER_DOOR.id,
        id if id == Block::OXIDIZED_COPPER_TRAPDOOR.id => Block::WEATHERED_COPPER_TRAPDOOR.id,
        id if id == Block::WEATHERED_COPPER_TRAPDOOR.id => Block::EXPOSED_COPPER_TRAPDOOR.id,
        id if id == Block::EXPOSED_COPPER_TRAPDOOR.id => Block::COPPER_TRAPDOOR.id,
        id if id == Block::EXPOSED_COPPER_CHAIN.id => Block::COPPER_CHAIN.id,
        id if id == Block::WEATHERED_COPPER_CHAIN.id => Block::EXPOSED_COPPER_CHAIN.id,
        id if id == Block::OXIDIZED_COPPER_CHAIN.id => Block::WEATHERED_COPPER_CHAIN.id,
        id if id == Block::EXPOSED_COPPER_BARS.id => Block::COPPER_BARS.id,
        id if id == Block::WEATHERED_COPPER_BARS.id => Block::EXPOSED_COPPER_BARS.id,
        id if id == Block::OXIDIZED_COPPER_BARS.id => Block::WEATHERED_COPPER_BARS.id,
        id if id == Block::EXPOSED_COPPER_LANTERN.id => Block::COPPER_LANTERN.id,
        id if id == Block::WEATHERED_COPPER_LANTERN.id => Block::EXPOSED_COPPER_LANTERN.id,
        id if id == Block::OXIDIZED_COPPER_LANTERN.id => Block::WEATHERED_COPPER_LANTERN.id,
        id if id == Block::EXPOSED_LIGHTNING_ROD.id => Block::LIGHTNING_ROD.id,
        id if id == Block::WEATHERED_LIGHTNING_ROD.id => Block::EXPOSED_LIGHTNING_ROD.id,
        id if id == Block::OXIDIZED_LIGHTNING_ROD.id => Block::WEATHERED_LIGHTNING_ROD.id,
        id if id == Block::EXPOSED_COPPER_CHEST.id => Block::COPPER_CHEST.id,
        id if id == Block::WEATHERED_COPPER_CHEST.id => Block::EXPOSED_COPPER_CHEST.id,
        id if id == Block::OXIDIZED_COPPER_CHEST.id => Block::WEATHERED_COPPER_CHEST.id,
        // id if id == Block::COPPER_GOLEM_STATUE.id => Block::AIR.id, // To spawn golem //TODO rework the whole system
        id if id == Block::EXPOSED_COPPER_GOLEM_STATUE.id => Block::COPPER_GOLEM_STATUE.id,
        id if id == Block::WEATHERED_COPPER_GOLEM_STATUE.id => {
            Block::EXPOSED_COPPER_GOLEM_STATUE.id
        }
        id if id == Block::OXIDIZED_COPPER_GOLEM_STATUE.id => {
            Block::WEATHERED_COPPER_GOLEM_STATUE.id
        }
        _ => 0,
    }
}

#[allow(clippy::too_many_lines)]
fn get_unwaxed_equivalent(block: &Block) -> u16 {
    match block.id {
        id if id == Block::WAXED_OXIDIZED_COPPER.id => Block::OXIDIZED_COPPER.id,
        id if id == Block::WAXED_WEATHERED_COPPER.id => Block::WEATHERED_COPPER.id,
        id if id == Block::WAXED_EXPOSED_COPPER.id => Block::EXPOSED_COPPER.id,
        id if id == Block::WAXED_COPPER_BLOCK.id => Block::COPPER_BLOCK.id,
        id if id == Block::WAXED_OXIDIZED_CHISELED_COPPER.id => Block::OXIDIZED_CHISELED_COPPER.id,
        id if id == Block::WAXED_WEATHERED_CHISELED_COPPER.id => {
            Block::WEATHERED_CHISELED_COPPER.id
        }
        id if id == Block::WAXED_EXPOSED_CHISELED_COPPER.id => Block::EXPOSED_CHISELED_COPPER.id,
        id if id == Block::WAXED_CHISELED_COPPER.id => Block::CHISELED_COPPER.id,
        id if id == Block::WAXED_COPPER_GRATE.id => Block::COPPER_GRATE.id,
        id if id == Block::WAXED_OXIDIZED_COPPER_GRATE.id => Block::OXIDIZED_COPPER_GRATE.id,
        id if id == Block::WAXED_WEATHERED_COPPER_GRATE.id => Block::WEATHERED_COPPER_GRATE.id,
        id if id == Block::WAXED_EXPOSED_COPPER_GRATE.id => Block::EXPOSED_COPPER_GRATE.id,
        id if id == Block::WAXED_OXIDIZED_CUT_COPPER.id => Block::OXIDIZED_CUT_COPPER.id,
        id if id == Block::WAXED_WEATHERED_CUT_COPPER.id => Block::WEATHERED_CUT_COPPER.id,
        id if id == Block::WAXED_EXPOSED_CUT_COPPER.id => Block::EXPOSED_CUT_COPPER.id,
        id if id == Block::WAXED_CUT_COPPER.id => Block::CUT_COPPER.id,
        id if id == Block::WAXED_OXIDIZED_CUT_COPPER_STAIRS.id => {
            Block::OXIDIZED_CUT_COPPER_STAIRS.id
        }
        id if id == Block::WAXED_WEATHERED_CUT_COPPER_STAIRS.id => {
            Block::WEATHERED_CUT_COPPER_STAIRS.id
        }
        id if id == Block::WAXED_EXPOSED_CUT_COPPER_STAIRS.id => {
            Block::EXPOSED_CUT_COPPER_STAIRS.id
        }
        id if id == Block::WAXED_CUT_COPPER_STAIRS.id => Block::CUT_COPPER_STAIRS.id,
        id if id == Block::WAXED_OXIDIZED_CUT_COPPER_SLAB.id => Block::OXIDIZED_CUT_COPPER_SLAB.id,
        id if id == Block::WAXED_WEATHERED_CUT_COPPER_SLAB.id => {
            Block::WEATHERED_CUT_COPPER_SLAB.id
        }
        id if id == Block::WAXED_EXPOSED_CUT_COPPER_SLAB.id => Block::EXPOSED_CUT_COPPER_SLAB.id,
        id if id == Block::WAXED_CUT_COPPER_SLAB.id => Block::CUT_COPPER_SLAB.id,
        id if id == Block::WAXED_OXIDIZED_COPPER_BULB.id => Block::OXIDIZED_COPPER_BULB.id,
        id if id == Block::WAXED_WEATHERED_COPPER_BULB.id => Block::WEATHERED_COPPER_BULB.id,
        id if id == Block::WAXED_EXPOSED_COPPER_BULB.id => Block::EXPOSED_COPPER_BULB.id,
        id if id == Block::WAXED_COPPER_BULB.id => Block::COPPER_BULB.id,
        id if id == Block::WAXED_OXIDIZED_COPPER_DOOR.id => Block::OXIDIZED_COPPER_DOOR.id,
        id if id == Block::WAXED_WEATHERED_COPPER_DOOR.id => Block::WEATHERED_COPPER_DOOR.id,
        id if id == Block::WAXED_EXPOSED_COPPER_DOOR.id => Block::EXPOSED_COPPER_DOOR.id,
        id if id == Block::WAXED_COPPER_DOOR.id => Block::COPPER_DOOR.id,
        id if id == Block::WAXED_OXIDIZED_COPPER_TRAPDOOR.id => Block::OXIDIZED_COPPER_TRAPDOOR.id,
        id if id == Block::WAXED_WEATHERED_COPPER_TRAPDOOR.id => {
            Block::WEATHERED_COPPER_TRAPDOOR.id
        }
        id if id == Block::WAXED_EXPOSED_COPPER_TRAPDOOR.id => Block::EXPOSED_COPPER_TRAPDOOR.id,
        id if id == Block::WAXED_COPPER_TRAPDOOR.id => Block::COPPER_TRAPDOOR.id,
        id if id == Block::WAXED_COPPER_CHAIN.id => Block::COPPER_CHAIN.id,
        id if id == Block::WAXED_EXPOSED_COPPER_CHAIN.id => Block::EXPOSED_COPPER_CHAIN.id,
        id if id == Block::WAXED_WEATHERED_COPPER_CHAIN.id => Block::WEATHERED_COPPER_CHAIN.id,
        id if id == Block::WAXED_OXIDIZED_COPPER_CHAIN.id => Block::OXIDIZED_COPPER_CHAIN.id,
        id if id == Block::WAXED_COPPER_BARS.id => Block::COPPER_BARS.id,
        id if id == Block::WAXED_EXPOSED_COPPER_BARS.id => Block::EXPOSED_COPPER_BARS.id,
        id if id == Block::WAXED_WEATHERED_COPPER_BARS.id => Block::WEATHERED_COPPER_BARS.id,
        id if id == Block::WAXED_OXIDIZED_COPPER_BARS.id => Block::OXIDIZED_COPPER_BARS.id,
        id if id == Block::WAXED_COPPER_LANTERN.id => Block::COPPER_LANTERN.id,
        id if id == Block::WAXED_EXPOSED_COPPER_LANTERN.id => Block::EXPOSED_COPPER_LANTERN.id,
        id if id == Block::WAXED_WEATHERED_COPPER_LANTERN.id => Block::WEATHERED_COPPER_LANTERN.id,
        id if id == Block::WAXED_OXIDIZED_COPPER_LANTERN.id => Block::OXIDIZED_COPPER_LANTERN.id,
        id if id == Block::WAXED_LIGHTNING_ROD.id => Block::LIGHTNING_ROD.id,
        id if id == Block::WAXED_EXPOSED_LIGHTNING_ROD.id => Block::EXPOSED_LIGHTNING_ROD.id,
        id if id == Block::WAXED_WEATHERED_LIGHTNING_ROD.id => Block::WEATHERED_LIGHTNING_ROD.id,
        id if id == Block::WAXED_OXIDIZED_LIGHTNING_ROD.id => Block::OXIDIZED_LIGHTNING_ROD.id,
        id if id == Block::WAXED_COPPER_CHEST.id => Block::COPPER_CHEST.id,
        id if id == Block::WAXED_EXPOSED_COPPER_CHEST.id => Block::EXPOSED_COPPER_CHEST.id,
        id if id == Block::WAXED_WEATHERED_COPPER_CHEST.id => Block::WEATHERED_COPPER_CHEST.id,
        id if id == Block::WAXED_OXIDIZED_COPPER_CHEST.id => Block::OXIDIZED_COPPER_CHEST.id,
        id if id == Block::WAXED_COPPER_GOLEM_STATUE.id => Block::COPPER_GOLEM_STATUE.id,
        id if id == Block::WAXED_EXPOSED_COPPER_GOLEM_STATUE.id => {
            Block::EXPOSED_COPPER_GOLEM_STATUE.id
        }
        id if id == Block::WAXED_WEATHERED_COPPER_GOLEM_STATUE.id => {
            Block::WEATHERED_COPPER_GOLEM_STATUE.id
        }
        id if id == Block::WAXED_OXIDIZED_COPPER_GOLEM_STATUE.id => {
            Block::OXIDIZED_COPPER_GOLEM_STATUE.id
        }
        _ => 0,
    }
}
