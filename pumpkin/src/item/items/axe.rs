use std::pin::Pin;

use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::block_properties::BlockProperties;
use pumpkin_data::block_properties::{
    OakDoorLikeProperties, OakTrapdoorLikeProperties, PaleOakWoodLikeProperties,
};
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, tag};
use pumpkin_data::{BlockDirection, BlockId};
use pumpkin_util::GameMode;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

pub struct AxeItem;

impl ItemMetadata for AxeItem {
    fn ids() -> Box<[u16]> {
        tag::Item::MINECRAFT_AXES.1.into()
    }
}

impl ItemBehaviour for AxeItem {
    fn use_on_block<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // I tried to follow mojang order of doing things.
            let world = player.world();
            let replacement_block = try_use_axe(block.id);
            // First we try to strip the block. by getting his equivalent and applying it the axis.

            // If there is a strip equivalent.
            let changed = if let Some((replacement, action)) = replacement_block {
                let new_block = replacement.to_block();
                // Bamboo blocks are pillars too, but they are not part of the logs tag.
                let new_state_id = if block.has_tag(&tag::Block::MINECRAFT_LOGS)
                    || block == &Block::BAMBOO_BLOCK
                {
                    let log_information = world.get_block_state_id(&location);
                    let log_props =
                        PaleOakWoodLikeProperties::from_state_id(log_information, block);
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
                    let door_information = world.get_block_state_id(&location);
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
                } else if block.has_tag(&tag::Block::MINECRAFT_TRAPDOORS) {
                    let trapdoor_information = world.get_block_state_id(&location);
                    axe_trapdoor_state(trapdoor_information, block, new_block)
                } else {
                    let old_state_id = world.get_block_state_id(&location);
                    crate::item::items::state_with_properties_of(block, old_state_id, new_block)
                };
                world
                    .set_block_state(&location, new_state_id, BlockFlags::NOTIFY_ALL)
                    .await;

                // Vanilla AxeItem#evaluateNewBlockState / spawnSoundAndParticle: strip only
                // plays a sound; scrape and wax-off additionally emit a level event (particle).
                let (sound, level_event) = match action {
                    AxeAction::Strip => (Sound::ItemAxeStrip, None),
                    AxeAction::Deoxidize => {
                        (Sound::ItemAxeScrape, Some(WorldEvent::ParticlesScrape))
                    }
                    AxeAction::Unwax => (Sound::ItemAxeWaxOff, Some(WorldEvent::ParticlesWaxOff)),
                };
                // Vanilla `AxeItem#useOn` uses `level.playSound(player, pos, ...)`, which plays
                // the sound at the block centre for every nearby player except the one acting.
                world.play_block_sound_expect(player, sound, SoundCategory::Blocks, location);
                if let Some(event) = level_event {
                    world.sync_world_event(event, location, 0);
                }

                true
            } else {
                false
            };

            if changed && player.gamemode.load() != GameMode::Creative {
                player.damage_held_item(1).await;
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn axe_trapdoor_state(
    state_id: pumpkin_data::BlockStateId,
    source: &Block,
    target: &Block,
) -> pumpkin_data::BlockStateId {
    let source_properties = OakTrapdoorLikeProperties::from_state_id(state_id, source);
    let mut target_properties = OakTrapdoorLikeProperties::default(target);
    target_properties.facing = source_properties.facing;
    target_properties.half = source_properties.half;
    target_properties.open = source_properties.open;
    target_properties.powered = source_properties.powered;
    target_properties.waterlogged = source_properties.waterlogged;
    target_properties.to_state_id(target)
}
#[derive(Clone, Copy)]
enum AxeAction {
    Strip,
    Deoxidize,
    Unwax,
}

const fn try_use_axe(id: BlockId) -> Option<(BlockId, AxeAction)> {
    // Trying to get the strip equivalent
    if let Some(block) = get_stripped_equivalent(id) {
        return Some((block, AxeAction::Strip));
    }
    // Else decrease the level of oxidation
    if let Some(block) = get_deoxidized_equivalent(id) {
        return Some((block, AxeAction::Deoxidize));
    }
    // Else unwax the block
    match get_unwaxed_equivalent(id) {
        Some(block) => Some((block, AxeAction::Unwax)),
        None => None,
    }
}

const fn get_stripped_equivalent(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::OAK_LOG => Some(BlockId::STRIPPED_OAK_LOG),
        BlockId::SPRUCE_LOG => Some(BlockId::STRIPPED_SPRUCE_LOG),
        BlockId::BIRCH_LOG => Some(BlockId::STRIPPED_BIRCH_LOG),
        BlockId::JUNGLE_LOG => Some(BlockId::STRIPPED_JUNGLE_LOG),
        BlockId::ACACIA_LOG => Some(BlockId::STRIPPED_ACACIA_LOG),
        BlockId::DARK_OAK_LOG => Some(BlockId::STRIPPED_DARK_OAK_LOG),
        BlockId::MANGROVE_LOG => Some(BlockId::STRIPPED_MANGROVE_LOG),
        BlockId::CHERRY_LOG => Some(BlockId::STRIPPED_CHERRY_LOG),
        BlockId::PALE_OAK_LOG => Some(BlockId::STRIPPED_PALE_OAK_LOG),
        BlockId::OAK_WOOD => Some(BlockId::STRIPPED_OAK_WOOD),
        BlockId::SPRUCE_WOOD => Some(BlockId::STRIPPED_SPRUCE_WOOD),
        BlockId::BIRCH_WOOD => Some(BlockId::STRIPPED_BIRCH_WOOD),
        BlockId::JUNGLE_WOOD => Some(BlockId::STRIPPED_JUNGLE_WOOD),
        BlockId::ACACIA_WOOD => Some(BlockId::STRIPPED_ACACIA_WOOD),
        BlockId::DARK_OAK_WOOD => Some(BlockId::STRIPPED_DARK_OAK_WOOD),
        BlockId::MANGROVE_WOOD => Some(BlockId::STRIPPED_MANGROVE_WOOD),
        BlockId::CHERRY_WOOD => Some(BlockId::STRIPPED_CHERRY_WOOD),
        BlockId::PALE_OAK_WOOD => Some(BlockId::STRIPPED_PALE_OAK_WOOD),
        BlockId::CRIMSON_STEM => Some(BlockId::STRIPPED_CRIMSON_STEM),
        BlockId::WARPED_STEM => Some(BlockId::STRIPPED_WARPED_STEM),
        BlockId::CRIMSON_HYPHAE => Some(BlockId::STRIPPED_CRIMSON_HYPHAE),
        BlockId::WARPED_HYPHAE => Some(BlockId::STRIPPED_WARPED_HYPHAE),
        BlockId::BAMBOO_BLOCK => Some(BlockId::STRIPPED_BAMBOO_BLOCK),
        _ => None,
    }
}

const fn get_deoxidized_equivalent(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::OXIDIZED_COPPER => Some(BlockId::WEATHERED_COPPER),
        BlockId::WEATHERED_COPPER => Some(BlockId::EXPOSED_COPPER),
        BlockId::EXPOSED_COPPER => Some(BlockId::COPPER_BLOCK),
        BlockId::OXIDIZED_CHISELED_COPPER => Some(BlockId::WEATHERED_CHISELED_COPPER),
        BlockId::WEATHERED_CHISELED_COPPER => Some(BlockId::EXPOSED_CHISELED_COPPER),
        BlockId::EXPOSED_CHISELED_COPPER => Some(BlockId::CHISELED_COPPER),
        BlockId::OXIDIZED_COPPER_GRATE => Some(BlockId::WEATHERED_COPPER_GRATE),
        BlockId::WEATHERED_COPPER_GRATE => Some(BlockId::EXPOSED_COPPER_GRATE),
        BlockId::EXPOSED_COPPER_GRATE => Some(BlockId::COPPER_GRATE),
        BlockId::OXIDIZED_CUT_COPPER => Some(BlockId::WEATHERED_CUT_COPPER),
        BlockId::WEATHERED_CUT_COPPER => Some(BlockId::EXPOSED_CUT_COPPER),
        BlockId::EXPOSED_CUT_COPPER => Some(BlockId::CUT_COPPER),
        BlockId::OXIDIZED_CUT_COPPER_STAIRS => Some(BlockId::WEATHERED_CUT_COPPER_STAIRS),
        BlockId::WEATHERED_CUT_COPPER_STAIRS => Some(BlockId::EXPOSED_CUT_COPPER_STAIRS),
        BlockId::EXPOSED_CUT_COPPER_STAIRS => Some(BlockId::CUT_COPPER_STAIRS),
        BlockId::OXIDIZED_CUT_COPPER_SLAB => Some(BlockId::WEATHERED_CUT_COPPER_SLAB),
        BlockId::WEATHERED_CUT_COPPER_SLAB => Some(BlockId::EXPOSED_CUT_COPPER_SLAB),
        BlockId::EXPOSED_CUT_COPPER_SLAB => Some(BlockId::CUT_COPPER_SLAB),
        BlockId::OXIDIZED_COPPER_BULB => Some(BlockId::WEATHERED_COPPER_BULB),
        BlockId::WEATHERED_COPPER_BULB => Some(BlockId::EXPOSED_COPPER_BULB),
        BlockId::EXPOSED_COPPER_BULB => Some(BlockId::COPPER_BULB),
        BlockId::OXIDIZED_COPPER_DOOR => Some(BlockId::WEATHERED_COPPER_DOOR),
        BlockId::WEATHERED_COPPER_DOOR => Some(BlockId::EXPOSED_COPPER_DOOR),
        BlockId::EXPOSED_COPPER_DOOR => Some(BlockId::COPPER_DOOR),
        BlockId::OXIDIZED_COPPER_TRAPDOOR => Some(BlockId::WEATHERED_COPPER_TRAPDOOR),
        BlockId::WEATHERED_COPPER_TRAPDOOR => Some(BlockId::EXPOSED_COPPER_TRAPDOOR),
        BlockId::EXPOSED_COPPER_TRAPDOOR => Some(BlockId::COPPER_TRAPDOOR),
        BlockId::OXIDIZED_COPPER_BARS => Some(BlockId::WEATHERED_COPPER_BARS),
        BlockId::WEATHERED_COPPER_BARS => Some(BlockId::EXPOSED_COPPER_BARS),
        BlockId::EXPOSED_COPPER_BARS => Some(BlockId::COPPER_BARS),
        BlockId::OXIDIZED_COPPER_CHAIN => Some(BlockId::WEATHERED_COPPER_CHAIN),
        BlockId::WEATHERED_COPPER_CHAIN => Some(BlockId::EXPOSED_COPPER_CHAIN),
        BlockId::EXPOSED_COPPER_CHAIN => Some(BlockId::COPPER_CHAIN),
        BlockId::OXIDIZED_COPPER_LANTERN => Some(BlockId::WEATHERED_COPPER_LANTERN),
        BlockId::WEATHERED_COPPER_LANTERN => Some(BlockId::EXPOSED_COPPER_LANTERN),
        BlockId::EXPOSED_COPPER_LANTERN => Some(BlockId::COPPER_LANTERN),
        BlockId::OXIDIZED_COPPER_CHEST => Some(BlockId::WEATHERED_COPPER_CHEST),
        BlockId::WEATHERED_COPPER_CHEST => Some(BlockId::EXPOSED_COPPER_CHEST),
        BlockId::EXPOSED_COPPER_CHEST => Some(BlockId::COPPER_CHEST),
        BlockId::OXIDIZED_COPPER_GOLEM_STATUE => Some(BlockId::WEATHERED_COPPER_GOLEM_STATUE),
        BlockId::WEATHERED_COPPER_GOLEM_STATUE => Some(BlockId::EXPOSED_COPPER_GOLEM_STATUE),
        BlockId::EXPOSED_COPPER_GOLEM_STATUE => Some(BlockId::COPPER_GOLEM_STATUE),
        BlockId::OXIDIZED_LIGHTNING_ROD => Some(BlockId::WEATHERED_LIGHTNING_ROD),
        BlockId::WEATHERED_LIGHTNING_ROD => Some(BlockId::EXPOSED_LIGHTNING_ROD),
        BlockId::EXPOSED_LIGHTNING_ROD => Some(BlockId::LIGHTNING_ROD),
        _ => None,
    }
}

const fn get_unwaxed_equivalent(id: BlockId) -> Option<BlockId> {
    match id {
        BlockId::WAXED_OXIDIZED_COPPER => Some(BlockId::OXIDIZED_COPPER),
        BlockId::WAXED_WEATHERED_COPPER => Some(BlockId::WEATHERED_COPPER),
        BlockId::WAXED_EXPOSED_COPPER => Some(BlockId::EXPOSED_COPPER),
        BlockId::WAXED_COPPER_BLOCK => Some(BlockId::COPPER_BLOCK),
        BlockId::WAXED_OXIDIZED_CHISELED_COPPER => Some(BlockId::OXIDIZED_CHISELED_COPPER),
        BlockId::WAXED_WEATHERED_CHISELED_COPPER => Some(BlockId::WEATHERED_CHISELED_COPPER),
        BlockId::WAXED_EXPOSED_CHISELED_COPPER => Some(BlockId::EXPOSED_CHISELED_COPPER),
        BlockId::WAXED_CHISELED_COPPER => Some(BlockId::CHISELED_COPPER),
        BlockId::WAXED_COPPER_GRATE => Some(BlockId::COPPER_GRATE),
        BlockId::WAXED_OXIDIZED_COPPER_GRATE => Some(BlockId::OXIDIZED_COPPER_GRATE),
        BlockId::WAXED_WEATHERED_COPPER_GRATE => Some(BlockId::WEATHERED_COPPER_GRATE),
        BlockId::WAXED_EXPOSED_COPPER_GRATE => Some(BlockId::EXPOSED_COPPER_GRATE),
        BlockId::WAXED_OXIDIZED_CUT_COPPER => Some(BlockId::OXIDIZED_CUT_COPPER),
        BlockId::WAXED_WEATHERED_CUT_COPPER => Some(BlockId::WEATHERED_CUT_COPPER),
        BlockId::WAXED_EXPOSED_CUT_COPPER => Some(BlockId::EXPOSED_CUT_COPPER),
        BlockId::WAXED_CUT_COPPER => Some(BlockId::CUT_COPPER),
        BlockId::WAXED_OXIDIZED_CUT_COPPER_STAIRS => Some(BlockId::OXIDIZED_CUT_COPPER_STAIRS),
        BlockId::WAXED_WEATHERED_CUT_COPPER_STAIRS => Some(BlockId::WEATHERED_CUT_COPPER_STAIRS),
        BlockId::WAXED_EXPOSED_CUT_COPPER_STAIRS => Some(BlockId::EXPOSED_CUT_COPPER_STAIRS),
        BlockId::WAXED_CUT_COPPER_STAIRS => Some(BlockId::CUT_COPPER_STAIRS),
        BlockId::WAXED_OXIDIZED_CUT_COPPER_SLAB => Some(BlockId::OXIDIZED_CUT_COPPER_SLAB),
        BlockId::WAXED_WEATHERED_CUT_COPPER_SLAB => Some(BlockId::WEATHERED_CUT_COPPER_SLAB),
        BlockId::WAXED_EXPOSED_CUT_COPPER_SLAB => Some(BlockId::EXPOSED_CUT_COPPER_SLAB),
        BlockId::WAXED_CUT_COPPER_SLAB => Some(BlockId::CUT_COPPER_SLAB),
        BlockId::WAXED_OXIDIZED_COPPER_BULB => Some(BlockId::OXIDIZED_COPPER_BULB),
        BlockId::WAXED_WEATHERED_COPPER_BULB => Some(BlockId::WEATHERED_COPPER_BULB),
        BlockId::WAXED_EXPOSED_COPPER_BULB => Some(BlockId::EXPOSED_COPPER_BULB),
        BlockId::WAXED_COPPER_BULB => Some(BlockId::COPPER_BULB),
        BlockId::WAXED_OXIDIZED_COPPER_DOOR => Some(BlockId::OXIDIZED_COPPER_DOOR),
        BlockId::WAXED_WEATHERED_COPPER_DOOR => Some(BlockId::WEATHERED_COPPER_DOOR),
        BlockId::WAXED_EXPOSED_COPPER_DOOR => Some(BlockId::EXPOSED_COPPER_DOOR),
        BlockId::WAXED_COPPER_DOOR => Some(BlockId::COPPER_DOOR),
        BlockId::WAXED_OXIDIZED_COPPER_TRAPDOOR => Some(BlockId::OXIDIZED_COPPER_TRAPDOOR),
        BlockId::WAXED_WEATHERED_COPPER_TRAPDOOR => Some(BlockId::WEATHERED_COPPER_TRAPDOOR),
        BlockId::WAXED_EXPOSED_COPPER_TRAPDOOR => Some(BlockId::EXPOSED_COPPER_TRAPDOOR),
        BlockId::WAXED_COPPER_TRAPDOOR => Some(BlockId::COPPER_TRAPDOOR),
        BlockId::WAXED_OXIDIZED_COPPER_BARS => Some(BlockId::OXIDIZED_COPPER_BARS),
        BlockId::WAXED_WEATHERED_COPPER_BARS => Some(BlockId::WEATHERED_COPPER_BARS),
        BlockId::WAXED_EXPOSED_COPPER_BARS => Some(BlockId::EXPOSED_COPPER_BARS),
        BlockId::WAXED_COPPER_BARS => Some(BlockId::COPPER_BARS),
        BlockId::WAXED_OXIDIZED_COPPER_CHAIN => Some(BlockId::OXIDIZED_COPPER_CHAIN),
        BlockId::WAXED_WEATHERED_COPPER_CHAIN => Some(BlockId::WEATHERED_COPPER_CHAIN),
        BlockId::WAXED_EXPOSED_COPPER_CHAIN => Some(BlockId::EXPOSED_COPPER_CHAIN),
        BlockId::WAXED_COPPER_CHAIN => Some(BlockId::COPPER_CHAIN),
        BlockId::WAXED_OXIDIZED_COPPER_LANTERN => Some(BlockId::OXIDIZED_COPPER_LANTERN),
        BlockId::WAXED_WEATHERED_COPPER_LANTERN => Some(BlockId::WEATHERED_COPPER_LANTERN),
        BlockId::WAXED_EXPOSED_COPPER_LANTERN => Some(BlockId::EXPOSED_COPPER_LANTERN),
        BlockId::WAXED_COPPER_LANTERN => Some(BlockId::COPPER_LANTERN),
        BlockId::WAXED_OXIDIZED_COPPER_CHEST => Some(BlockId::OXIDIZED_COPPER_CHEST),
        BlockId::WAXED_WEATHERED_COPPER_CHEST => Some(BlockId::WEATHERED_COPPER_CHEST),
        BlockId::WAXED_EXPOSED_COPPER_CHEST => Some(BlockId::EXPOSED_COPPER_CHEST),
        BlockId::WAXED_COPPER_CHEST => Some(BlockId::COPPER_CHEST),
        BlockId::WAXED_OXIDIZED_COPPER_GOLEM_STATUE => Some(BlockId::OXIDIZED_COPPER_GOLEM_STATUE),
        BlockId::WAXED_WEATHERED_COPPER_GOLEM_STATUE => {
            Some(BlockId::WEATHERED_COPPER_GOLEM_STATUE)
        }
        BlockId::WAXED_EXPOSED_COPPER_GOLEM_STATUE => Some(BlockId::EXPOSED_COPPER_GOLEM_STATUE),
        BlockId::WAXED_COPPER_GOLEM_STATUE => Some(BlockId::COPPER_GOLEM_STATUE),
        BlockId::WAXED_OXIDIZED_LIGHTNING_ROD => Some(BlockId::OXIDIZED_LIGHTNING_ROD),
        BlockId::WAXED_WEATHERED_LIGHTNING_ROD => Some(BlockId::WEATHERED_LIGHTNING_ROD),
        BlockId::WAXED_EXPOSED_LIGHTNING_ROD => Some(BlockId::EXPOSED_LIGHTNING_ROD),
        BlockId::WAXED_LIGHTNING_ROD => Some(BlockId::LIGHTNING_ROD),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{axe_trapdoor_state, get_deoxidized_equivalent, get_unwaxed_equivalent};
    use pumpkin_data::Block;
    use pumpkin_data::BlockState;
    use pumpkin_data::block_properties::{BlockProperties, OakTrapdoorLikeProperties};

    #[test]
    fn axe_trapdoor_replacement_preserves_state_properties() {
        let source = &Block::WAXED_COPPER_TRAPDOOR;
        let target = &Block::COPPER_TRAPDOOR;
        assert_eq!(
            get_unwaxed_equivalent(source.id),
            Some(target.id),
            "waxed copper trapdoors should be removable with an axe"
        );

        let source_state = source
            .states
            .iter()
            .find(|state| {
                let properties = OakTrapdoorLikeProperties::from_state_id(state.id, source);
                properties.open && properties.waterlogged
            })
            .expect("waxed copper trapdoor should expose an open waterlogged state");
        let source_properties = OakTrapdoorLikeProperties::from_state_id(source_state.id, source);
        let target_state = BlockState::from_id(axe_trapdoor_state(source_state.id, source, target));
        let target_properties = OakTrapdoorLikeProperties::from_state_id(target_state.id, target);

        assert_eq!(source_properties.facing, target_properties.facing);
        assert_eq!(source_properties.half, target_properties.half);
        assert_eq!(source_properties.open, target_properties.open);
        assert_eq!(source_properties.powered, target_properties.powered);
        assert_eq!(source_properties.waterlogged, target_properties.waterlogged);
    }

    /// Vanilla `WeatheringCopper.NEXT_BY_BLOCK` also weathers copper bars, chains, lanterns,
    /// chests, golem statues and lightning rods, and `HoneycombItem.WAXABLES` waxes all of them,
    /// so an axe has to be able to scrape and de-wax them too.
    #[test]
    fn axe_scrapes_and_dewaxes_every_waxable_copper_family() {
        for family in [
            "copper_bars",
            "copper_chain",
            "copper_lantern",
            "copper_chest",
            "copper_golem_statue",
            "lightning_rod",
        ] {
            let stages = [
                format!("oxidized_{family}"),
                format!("weathered_{family}"),
                format!("exposed_{family}"),
                family.to_string(),
            ];

            for pair in stages.windows(2) {
                let from = Block::from_registry_key(&pair[0]).expect("block should exist");
                let to = Block::from_registry_key(&pair[1]).expect("block should exist");
                assert_eq!(
                    get_deoxidized_equivalent(from.id),
                    Some(to.id),
                    "{} should scrape to {}",
                    pair[0],
                    pair[1]
                );
            }

            for stage in &stages {
                let waxed = Block::from_registry_key(&format!("waxed_{stage}"))
                    .expect("block should exist");
                let unwaxed = Block::from_registry_key(stage).expect("block should exist");
                assert_eq!(
                    get_unwaxed_equivalent(waxed.id),
                    Some(unwaxed.id),
                    "waxed_{stage} should de-wax to {stage}"
                );
            }
        }
    }
}
