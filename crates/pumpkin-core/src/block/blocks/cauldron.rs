use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::block::blocks::dripstone::{CauldronDripFluid, cauldron_drip_fluid};
use crate::block::registry::BlockActionResult;
use crate::block::{
    BlockBehaviour, BlockMetadata, GetComparatorOutputArgs, GetInsideCollisionShapeArgs,
    OnEntityCollisionArgs, OnScheduledTickArgs, PathComputationType, UseWithItemArgs,
};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::items::bucket::create_filled_result;
use crate::item::items::glass_bottle::water_bottle;
use crate::plugin::block::cauldron_level_change::{CauldronChangeReason, CauldronLevelChangeEvent};
use crate::world::World;
use pumpkin_data::block_properties::WaterCauldronLikeProperties;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{BannerPatternsImpl, DyedColorImpl, PotionContentsImpl};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::potion::Potion;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::statistic::{CustomStatistic, StatisticCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockId, BlockState, BlockStateId};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

const RAIN_FILL_CHANCE: f32 = 0.05;
const POWDER_SNOW_FILL_CHANCE: f32 = 0.1;

pub struct CauldronBlock;

impl BlockMetadata for CauldronBlock {
    fn ids() -> Box<[BlockId]> {
        [
            BlockId::CAULDRON,
            BlockId::WATER_CAULDRON,
            BlockId::LAVA_CAULDRON,
            BlockId::POWDER_SNOW_CAULDRON,
        ]
        .into()
    }
}

fn fire_cauldron_change(
    world: &Arc<World>,
    pos: BlockPos,
    old_level: i32,
    new_level: i32,
    reason: CauldronChangeReason,
    entity: Option<Arc<dyn EntityBase>>,
) -> bool {
    let mut event =
        CauldronLevelChangeEvent::new(pos, world.clone(), old_level, new_level, reason, entity);
    if let Some(server) = world.server.upgrade() {
        server.plugin_manager.fire_blocking(&server, &mut event);
    }
    !event.cancelled
}

fn cauldron_level(block_id: BlockId, state_id: BlockStateId) -> i32 {
    match block_id {
        BlockId::WATER_CAULDRON | BlockId::POWDER_SNOW_CAULDRON => {
            i32::from(WaterCauldronLikeProperties::from_state_id(state_id).level)
        }
        BlockId::LAVA_CAULDRON => 3,
        _ => 0,
    }
}

fn layered_state_id(block: &Block, level: u8) -> BlockStateId {
    let mut props = WaterCauldronLikeProperties::default(block);
    props.level = level;
    props.to_state_id(block)
}

fn is_under_water(world: &World, pos: &BlockPos) -> bool {
    world
        .get_fluid(&pos.up())
        .has_tag(&tag::Fluid::MINECRAFT_WATER)
}

pub(crate) fn is_water_potion(stack: &ItemStack) -> bool {
    stack.item.id == Item::POTION.id
        && stack
            .get_data_component::<PotionContentsImpl>()
            .is_some_and(|contents| contents.potion_id == Some(i32::from(Potion::WATER.id)))
}

fn lower_fill_level(world: &Arc<World>, pos: &BlockPos, block: &Block, current_level: u8) {
    let new_state_id = if current_level <= 1 {
        Block::CAULDRON.default_state.id
    } else {
        layered_state_id(block, current_level - 1)
    };
    world.set_block_state(pos, new_state_id, BlockFlags::NOTIFY_ALL);
    world.emit_game_event(GameEvent::BlockChange.name(), pos.to_centered_f64());
}

fn content_collision_box(block_id: BlockId, state_id: BlockStateId) -> BoundingBox {
    let max_y = match block_id {
        BlockId::WATER_CAULDRON | BlockId::POWDER_SNOW_CAULDRON => {
            let level = WaterCauldronLikeProperties::from_state_id(state_id).level;
            (6.0 + 3.0 * f64::from(level)) / 16.0
        }
        BlockId::LAVA_CAULDRON => 15.0 / 16.0,
        _ => 4.0 / 16.0,
    };
    BoundingBox::new_array([0.125, 0.25, 0.125], [0.875, max_y, 0.875])
}

pub(crate) fn clean_shulker_box_copy(stack: &ItemStack) -> ItemStack {
    let mut cleaned = stack.copy_with_count(1);
    cleaned.item = &Item::SHULKER_BOX;
    cleaned
}

pub(crate) fn clean_banner_single_layer(stack: &ItemStack) -> Option<ItemStack> {
    if !stack.item.has_tag(&tag::Item::MINECRAFT_BANNERS) {
        return None;
    }
    let patterns = stack.get_data_component::<BannerPatternsImpl>()?;
    if patterns.layers.is_empty() {
        return None;
    }
    let mut remaining_layers = patterns.layers.clone();
    remaining_layers.pop();
    let mut cleaned = stack.copy_with_count(1);
    if remaining_layers.is_empty() {
        cleaned
            .patch
            .retain(|(id, _)| *id != DataComponent::BannerPatterns);
    } else {
        cleaned.set_data_component(BannerPatternsImpl {
            layers: remaining_layers,
        });
    }
    Some(cleaned)
}

pub(crate) fn remove_cauldron_dye(stack: &mut ItemStack) -> bool {
    if !stack
        .item
        .has_tag(&tag::Item::MINECRAFT_CAULDRON_CAN_REMOVE_DYE)
        || stack.get_data_component::<DyedColorImpl>().is_none()
    {
        return false;
    }
    if stack
        .item
        .components
        .iter()
        .any(|(id, _)| *id == DataComponent::DyedColor)
    {
        stack.remove_data_component(DataComponent::DyedColor);
    } else {
        stack
            .patch
            .retain(|(id, _)| *id != DataComponent::DyedColor);
    }
    true
}

impl CauldronBlock {
    pub fn handle_precipitation(
        world: &Arc<World>,
        pos: &BlockPos,
        block_id: BlockId,
        state_id: BlockStateId,
        is_rain: bool,
    ) {
        let chance = if is_rain {
            RAIN_FILL_CHANCE
        } else {
            POWDER_SNOW_FILL_CHANCE
        };
        if rand::rng().random::<f32>() >= chance {
            return;
        }
        match block_id {
            BlockId::CAULDRON => {
                if !fire_cauldron_change(world, *pos, 0, 1, CauldronChangeReason::NaturalFill, None)
                {
                    return;
                }
                let target_block = if is_rain {
                    &Block::WATER_CAULDRON
                } else {
                    &Block::POWDER_SNOW_CAULDRON
                };
                world.set_block_state(
                    pos,
                    layered_state_id(target_block, 1),
                    BlockFlags::NOTIFY_ALL,
                );
                world.emit_game_event(GameEvent::BlockChange.name(), pos.to_centered_f64());
            }
            BlockId::WATER_CAULDRON if is_rain => {
                let level = WaterCauldronLikeProperties::from_state_id(state_id).level;
                if level < 3
                    && fire_cauldron_change(
                        world,
                        *pos,
                        i32::from(level),
                        i32::from(level + 1),
                        CauldronChangeReason::NaturalFill,
                        None,
                    )
                {
                    world.set_block_state(
                        pos,
                        layered_state_id(&Block::WATER_CAULDRON, level + 1),
                        BlockFlags::NOTIFY_ALL,
                    );
                    world.emit_game_event(GameEvent::BlockChange.name(), pos.to_centered_f64());
                }
            }
            BlockId::POWDER_SNOW_CAULDRON if !is_rain => {
                let level = WaterCauldronLikeProperties::from_state_id(state_id).level;
                if level < 3
                    && fire_cauldron_change(
                        world,
                        *pos,
                        i32::from(level),
                        i32::from(level + 1),
                        CauldronChangeReason::NaturalFill,
                        None,
                    )
                {
                    world.set_block_state(
                        pos,
                        layered_state_id(&Block::POWDER_SNOW_CAULDRON, level + 1),
                        BlockFlags::NOTIFY_ALL,
                    );
                    world.emit_game_event(GameEvent::BlockChange.name(), pos.to_centered_f64());
                }
            }
            _ => {}
        }
    }

    fn try_empty_bucket(
        world: &Arc<World>,
        pos: &BlockPos,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        old_level: i32,
        new_state_id: BlockStateId,
        sound: Sound,
    ) -> BlockActionResult {
        let entity: Arc<dyn EntityBase> = Arc::clone(player) as Arc<dyn EntityBase>;
        if !fire_cauldron_change(
            world,
            *pos,
            old_level,
            3,
            CauldronChangeReason::BucketEmpty,
            Some(entity),
        ) {
            return BlockActionResult::Pass;
        }
        let used_item_id = item_stack.item.id;
        create_filled_result(item_stack, player, ItemStack::new(1, &Item::BUCKET), true);
        player.increment_stat(
            StatisticCategory::Custom,
            CustomStatistic::FillCauldron as i32,
            1,
        );
        player.increment_stat(StatisticCategory::Used, i32::from(used_item_id), 1);
        world.set_block_state(pos, new_state_id, BlockFlags::NOTIFY_ALL);
        world.play_sound(sound, SoundCategory::Blocks, &pos.to_f64());
        world.emit_game_event(GameEvent::FluidPlace.name(), pos.to_centered_f64());
        BlockActionResult::Success
    }

    fn try_fill_bucket(
        world: &Arc<World>,
        pos: &BlockPos,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        block_id: BlockId,
        state_id: BlockStateId,
    ) -> BlockActionResult {
        let (filled_item, sound) = match block_id {
            BlockId::WATER_CAULDRON
                if WaterCauldronLikeProperties::from_state_id(state_id).level == 3 =>
            {
                (&Item::WATER_BUCKET, Sound::ItemBucketFill)
            }
            BlockId::LAVA_CAULDRON => (&Item::LAVA_BUCKET, Sound::ItemBucketFillLava),
            BlockId::POWDER_SNOW_CAULDRON
                if WaterCauldronLikeProperties::from_state_id(state_id).level == 3 =>
            {
                (&Item::POWDER_SNOW_BUCKET, Sound::ItemBucketFillPowderSnow)
            }
            _ => return BlockActionResult::PassToDefaultBlockAction,
        };
        let entity: Arc<dyn EntityBase> = Arc::clone(player) as Arc<dyn EntityBase>;
        if !fire_cauldron_change(
            world,
            *pos,
            3,
            0,
            CauldronChangeReason::BucketFill,
            Some(entity),
        ) {
            return BlockActionResult::Pass;
        }
        create_filled_result(item_stack, player, ItemStack::new(1, filled_item), true);
        player.increment_stat(
            StatisticCategory::Custom,
            CustomStatistic::UseCauldron as i32,
            1,
        );
        player.increment_stat(StatisticCategory::Used, i32::from(Item::BUCKET.id), 1);
        world.set_block_state(
            pos,
            Block::CAULDRON.default_state.id,
            BlockFlags::NOTIFY_ALL,
        );
        world.play_sound(sound, SoundCategory::Blocks, &pos.to_f64());
        world.emit_game_event(GameEvent::FluidPickup.name(), pos.to_centered_f64());
        BlockActionResult::Success
    }

    fn interact_empty_cauldron(
        world: &Arc<World>,
        pos: &BlockPos,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
    ) -> BlockActionResult {
        if !is_water_potion(item_stack) {
            return BlockActionResult::PassToDefaultBlockAction;
        }
        let entity: Arc<dyn EntityBase> = Arc::clone(player) as Arc<dyn EntityBase>;
        if !fire_cauldron_change(
            world,
            *pos,
            0,
            1,
            CauldronChangeReason::BottleEmpty,
            Some(entity),
        ) {
            return BlockActionResult::Pass;
        }
        create_filled_result(
            item_stack,
            player,
            ItemStack::new(1, &Item::GLASS_BOTTLE),
            true,
        );
        player.increment_stat(
            StatisticCategory::Custom,
            CustomStatistic::UseCauldron as i32,
            1,
        );
        player.increment_stat(StatisticCategory::Used, i32::from(Item::POTION.id), 1);
        world.set_block_state(
            pos,
            layered_state_id(&Block::WATER_CAULDRON, 1),
            BlockFlags::NOTIFY_ALL,
        );
        world.play_sound(Sound::ItemBottleEmpty, SoundCategory::Blocks, &pos.to_f64());
        world.emit_game_event(GameEvent::FluidPlace.name(), pos.to_centered_f64());
        BlockActionResult::Success
    }

    fn interact_water_cauldron(
        world: &Arc<World>,
        pos: &BlockPos,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        level: u8,
    ) -> BlockActionResult {
        let item_id = item_stack.item.id;
        let entity = || -> Arc<dyn EntityBase> { Arc::clone(player) as Arc<dyn EntityBase> };

        if item_id == Item::GLASS_BOTTLE.id {
            if !fire_cauldron_change(
                world,
                *pos,
                i32::from(level),
                i32::from(level - 1),
                CauldronChangeReason::BottleFill,
                Some(entity()),
            ) {
                return BlockActionResult::Pass;
            }
            create_filled_result(item_stack, player, water_bottle(), true);
            player.increment_stat(
                StatisticCategory::Custom,
                CustomStatistic::UseCauldron as i32,
                1,
            );
            player.increment_stat(StatisticCategory::Used, i32::from(Item::GLASS_BOTTLE.id), 1);
            lower_fill_level(world, pos, &Block::WATER_CAULDRON, level);
            world.play_sound(Sound::ItemBottleFill, SoundCategory::Blocks, &pos.to_f64());
            world.emit_game_event(GameEvent::FluidPickup.name(), pos.to_centered_f64());
            return BlockActionResult::Success;
        }

        if item_id == Item::POTION.id {
            if level >= 3 || !is_water_potion(item_stack) {
                return BlockActionResult::PassToDefaultBlockAction;
            }
            if !fire_cauldron_change(
                world,
                *pos,
                i32::from(level),
                i32::from(level + 1),
                CauldronChangeReason::BottleEmpty,
                Some(entity()),
            ) {
                return BlockActionResult::Pass;
            }
            create_filled_result(
                item_stack,
                player,
                ItemStack::new(1, &Item::GLASS_BOTTLE),
                true,
            );
            player.increment_stat(
                StatisticCategory::Custom,
                CustomStatistic::UseCauldron as i32,
                1,
            );
            player.increment_stat(StatisticCategory::Used, i32::from(Item::POTION.id), 1);
            world.set_block_state(
                pos,
                layered_state_id(&Block::WATER_CAULDRON, level + 1),
                BlockFlags::NOTIFY_ALL,
            );
            world.play_sound(Sound::ItemBottleEmpty, SoundCategory::Blocks, &pos.to_f64());
            world.emit_game_event(GameEvent::FluidPlace.name(), pos.to_centered_f64());
            return BlockActionResult::Success;
        }

        Self::try_clean_water_cauldron_item(world, pos, player, item_stack, level)
    }

    fn try_clean_water_cauldron_item(
        world: &Arc<World>,
        pos: &BlockPos,
        player: &Arc<Player>,
        item_stack: &mut ItemStack,
        level: u8,
    ) -> BlockActionResult {
        let entity = || -> Arc<dyn EntityBase> { Arc::clone(player) as Arc<dyn EntityBase> };

        if item_stack.item.has_tag(&tag::Item::MINECRAFT_SHULKER_BOXES)
            && item_stack.item.id != Item::SHULKER_BOX.id
        {
            if !fire_cauldron_change(
                world,
                *pos,
                i32::from(level),
                i32::from(level - 1),
                CauldronChangeReason::Unknown,
                Some(entity()),
            ) {
                return BlockActionResult::Pass;
            }
            let cleaned = clean_shulker_box_copy(item_stack);
            create_filled_result(item_stack, player, cleaned, false);
            player.increment_stat(
                StatisticCategory::Custom,
                CustomStatistic::CleanShulkerBox as i32,
                1,
            );
            lower_fill_level(world, pos, &Block::WATER_CAULDRON, level);
            return BlockActionResult::Success;
        }

        if let Some(cleaned_banner) = clean_banner_single_layer(item_stack) {
            if !fire_cauldron_change(
                world,
                *pos,
                i32::from(level),
                i32::from(level - 1),
                CauldronChangeReason::Unknown,
                Some(entity()),
            ) {
                return BlockActionResult::Pass;
            }
            create_filled_result(item_stack, player, cleaned_banner, false);
            player.increment_stat(
                StatisticCategory::Custom,
                CustomStatistic::CleanBanner as i32,
                1,
            );
            lower_fill_level(world, pos, &Block::WATER_CAULDRON, level);
            return BlockActionResult::Success;
        }

        if item_stack
            .item
            .has_tag(&tag::Item::MINECRAFT_CAULDRON_CAN_REMOVE_DYE)
            && item_stack.get_data_component::<DyedColorImpl>().is_some()
        {
            if !fire_cauldron_change(
                world,
                *pos,
                i32::from(level),
                i32::from(level - 1),
                CauldronChangeReason::Unknown,
                Some(entity()),
            ) {
                return BlockActionResult::Pass;
            }
            remove_cauldron_dye(item_stack);
            player.increment_stat(
                StatisticCategory::Custom,
                CustomStatistic::CleanArmor as i32,
                1,
            );
            lower_fill_level(world, pos, &Block::WATER_CAULDRON, level);
            return BlockActionResult::Success;
        }

        BlockActionResult::PassToDefaultBlockAction
    }
}

impl BlockBehaviour for CauldronBlock {
    fn use_with_item(&self, args: UseWithItemArgs<'_>) -> BlockActionResult {
        let item_id = args.item_stack.item.id;
        let block_id = args.block.id;
        let state_id = args.world.get_block_state_id(args.position);
        let old_level = cauldron_level(block_id, state_id);

        if item_id == Item::WATER_BUCKET.id {
            return Self::try_empty_bucket(
                args.world,
                args.position,
                args.player,
                args.item_stack,
                old_level,
                layered_state_id(&Block::WATER_CAULDRON, 3),
                Sound::ItemBucketEmpty,
            );
        }
        if item_id == Item::LAVA_BUCKET.id {
            if is_under_water(args.world, args.position) {
                return BlockActionResult::Consume;
            }
            return Self::try_empty_bucket(
                args.world,
                args.position,
                args.player,
                args.item_stack,
                old_level,
                Block::LAVA_CAULDRON.default_state.id,
                Sound::ItemBucketEmptyLava,
            );
        }
        if item_id == Item::POWDER_SNOW_BUCKET.id {
            if is_under_water(args.world, args.position) {
                return BlockActionResult::Consume;
            }
            return Self::try_empty_bucket(
                args.world,
                args.position,
                args.player,
                args.item_stack,
                old_level,
                layered_state_id(&Block::POWDER_SNOW_CAULDRON, 3),
                Sound::ItemBucketEmptyPowderSnow,
            );
        }
        if item_id == Item::BUCKET.id {
            return Self::try_fill_bucket(
                args.world,
                args.position,
                args.player,
                args.item_stack,
                block_id,
                state_id,
            );
        }

        if block_id == BlockId::CAULDRON {
            return Self::interact_empty_cauldron(
                args.world,
                args.position,
                args.player,
                args.item_stack,
            );
        }

        if block_id == BlockId::WATER_CAULDRON {
            let level = WaterCauldronLikeProperties::from_state_id(state_id).level;
            if level > 0 {
                return Self::interact_water_cauldron(
                    args.world,
                    args.position,
                    args.player,
                    args.item_stack,
                    level,
                );
            }
        }

        BlockActionResult::PassToDefaultBlockAction
    }

    fn get_inside_collision_shape(&self, args: GetInsideCollisionShapeArgs<'_>) -> BoundingBox {
        content_collision_box(args.block.id, args.state.id)
    }

    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        let content_box =
            content_collision_box(args.block.id, args.state.id).at_pos(*args.position);
        let base_entity = args.entity.get_entity();
        if !base_entity.bounding_box.load().intersects(&content_box) {
            return;
        }

        match args.block.id {
            BlockId::WATER_CAULDRON | BlockId::POWDER_SNOW_CAULDRON => {
                if base_entity.fire_ticks.load(Ordering::Relaxed) > 0 {
                    let may_interact = args.entity.get_player().is_some()
                        || args.world.level_info.load().game_rules.mob_griefing;
                    if may_interact {
                        let level = WaterCauldronLikeProperties::from_state_id(args.state.id).level;
                        let entity_arc = args
                            .world
                            .get_entity_by_id(base_entity.entity_id)
                            .or_else(|| {
                                args.entity
                                    .get_player()
                                    .and_then(|p| args.world.get_player_by_id(p.entity_id()))
                                    .map(|p| p as Arc<dyn EntityBase>)
                            });
                        if fire_cauldron_change(
                            args.world,
                            *args.position,
                            i32::from(level),
                            i32::from(level.saturating_sub(1)),
                            CauldronChangeReason::Extinguish,
                            entity_arc,
                        ) {
                            // Powder snow cauldrons melt into a lowered water cauldron when extinguishing fire.
                            lower_fill_level(
                                args.world,
                                args.position,
                                &Block::WATER_CAULDRON,
                                level,
                            );
                        }
                    }
                }
                base_entity.extinguish();
            }
            BlockId::LAVA_CAULDRON => {
                base_entity.set_frozen_ticks(0);
                if !base_entity.is_fire_immune() {
                    args.entity.set_on_fire_for(15.0);
                    if args.entity.damage(args.entity, 4.0, DamageType::LAVA) {
                        let pitch = 2.0 + rand::rng().random::<f32>() * 0.4;
                        args.entity
                            .play_entity_sound(Sound::EntityGenericBurn, 0.4, pitch);
                    }
                }
            }
            _ => {}
        }
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        let Some(fluid) = cauldron_drip_fluid(args.world, args.position) else {
            return;
        };
        let state_id = args.world.get_block_state_id(args.position);
        match (args.block.id, fluid) {
            (BlockId::CAULDRON, CauldronDripFluid::Water) => {
                if !fire_cauldron_change(
                    args.world,
                    *args.position,
                    0,
                    1,
                    CauldronChangeReason::NaturalFill,
                    None,
                ) {
                    return;
                }
                args.world.set_block_state(
                    args.position,
                    layered_state_id(&Block::WATER_CAULDRON, 1),
                    BlockFlags::NOTIFY_ALL,
                );
                args.world.emit_game_event(
                    GameEvent::BlockChange.name(),
                    args.position.to_centered_f64(),
                );
                args.world.sync_world_event(
                    WorldEvent::SoundDripWaterIntoCauldron,
                    *args.position,
                    0,
                );
            }
            (BlockId::CAULDRON, CauldronDripFluid::Lava) => {
                if !fire_cauldron_change(
                    args.world,
                    *args.position,
                    0,
                    3,
                    CauldronChangeReason::NaturalFill,
                    None,
                ) {
                    return;
                }
                args.world.set_block_state(
                    args.position,
                    Block::LAVA_CAULDRON.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
                args.world.emit_game_event(
                    GameEvent::BlockChange.name(),
                    args.position.to_centered_f64(),
                );
                args.world.sync_world_event(
                    WorldEvent::SoundDripLavaIntoCauldron,
                    *args.position,
                    0,
                );
            }
            (BlockId::WATER_CAULDRON, CauldronDripFluid::Water) => {
                let level = WaterCauldronLikeProperties::from_state_id(state_id).level;
                if level >= 3
                    || !fire_cauldron_change(
                        args.world,
                        *args.position,
                        i32::from(level),
                        i32::from(level + 1),
                        CauldronChangeReason::NaturalFill,
                        None,
                    )
                {
                    return;
                }
                args.world.set_block_state(
                    args.position,
                    layered_state_id(&Block::WATER_CAULDRON, level + 1),
                    BlockFlags::NOTIFY_ALL,
                );
                args.world.emit_game_event(
                    GameEvent::BlockChange.name(),
                    args.position.to_centered_f64(),
                );
                args.world.sync_world_event(
                    WorldEvent::SoundDripWaterIntoCauldron,
                    *args.position,
                    0,
                );
            }
            _ => {}
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        match args.block.id {
            BlockId::WATER_CAULDRON | BlockId::POWDER_SNOW_CAULDRON => {
                let state_id = args.world.get_block_state_id(args.position);
                let props = WaterCauldronLikeProperties::from_state_id(state_id);
                Some(props.level)
            }
            BlockId::LAVA_CAULDRON => Some(3),
            _ => Some(0),
        }
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::data_component_impl::{BannerPatternLayer, CustomNameImpl};
    use pumpkin_data::dye_color::DyeColor;
    use pumpkin_util::text::TextComponent;

    #[test]
    fn water_potion_detection_rejects_non_water_potions() {
        let water = water_bottle();
        assert!(is_water_potion(&water));

        let mut healing = ItemStack::new(1, &Item::POTION);
        healing.set_data_component(PotionContentsImpl {
            potion_id: Some(i32::from(Potion::HEALING.id)),
            custom_color: None,
            custom_effects: Vec::new(),
            custom_name: None,
        });
        assert!(!is_water_potion(&healing));
    }

    #[test]
    fn banner_cleaning_ignores_blank_banners_and_pops_one_layer_at_a_time() {
        let blank = ItemStack::new(1, &Item::WHITE_BANNER);
        assert!(clean_banner_single_layer(&blank).is_none());

        let mut patterned = ItemStack::new(2, &Item::WHITE_BANNER);
        patterned.set_data_component(BannerPatternsImpl {
            layers: vec![
                BannerPatternLayer {
                    pattern: "minecraft:stripe_bottom".to_string(),
                    color: DyeColor::Red,
                },
                BannerPatternLayer {
                    pattern: "minecraft:cross".to_string(),
                    color: DyeColor::Blue,
                },
            ],
        });

        let after_first = clean_banner_single_layer(&patterned).expect("two-layer banner cleans");
        assert_eq!(after_first.item_count, 1);
        let first_layers = &after_first
            .get_data_component::<BannerPatternsImpl>()
            .expect("patterns component present")
            .layers;
        assert_eq!(first_layers.len(), 1);
        assert_eq!(first_layers[0].pattern, "minecraft:stripe_bottom");

        let after_second =
            clean_banner_single_layer(&after_first).expect("one-layer banner cleans");
        assert!(
            after_second.are_items_and_components_equal(&blank),
            "fully cleaned banner should stack with blank banners"
        );
    }

    #[test]
    fn shulker_box_cleaning_preserves_components_and_single_count() {
        let mut dyed = ItemStack::new(2, &Item::RED_SHULKER_BOX);
        dyed.set_data_component(CustomNameImpl {
            name: TextComponent::text("Loot"),
        });

        let cleaned = clean_shulker_box_copy(&dyed);
        assert_eq!(cleaned.item.id, Item::SHULKER_BOX.id);
        assert_eq!(cleaned.item_count, 1);
        assert!(cleaned.get_data_component::<CustomNameImpl>().is_some());
    }

    #[test]
    fn remove_dye_checks_tag_and_restores_clean_patch() {
        let clean_helmet = ItemStack::new(1, &Item::LEATHER_HELMET);
        let mut dyed_helmet = ItemStack::new(1, &Item::LEATHER_HELMET);
        dyed_helmet.set_data_component(DyedColorImpl { rgb: 0xFF0000 });

        assert!(remove_cauldron_dye(&mut dyed_helmet));
        assert!(dyed_helmet.are_items_and_components_equal(&clean_helmet));
        assert!(!remove_cauldron_dye(&mut dyed_helmet));
    }
}
