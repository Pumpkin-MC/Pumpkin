use std::cell::RefCell;
use std::sync::Arc;

use crate::block::entities::BlockEntity;
use crate::block::entities::chest::ChestBlockEntity;
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::{ChestLikeProperties, ChestType, HorizontalFacing};
use pumpkin_data::entity::EntityPose;
use pumpkin_data::sound::Sound;
use pumpkin_data::{Block, BlockDirection, translation};
use pumpkin_inventory::Inventory;
use pumpkin_inventory::double::DoubleInventory;
use pumpkin_inventory::generic_container_screen_handler::{create_generic_9x3, create_generic_9x6};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_inventory::screen_handler::{
    InventoryPlayer, ScreenHandlerFactory, SharedScreenHandler,
};
use pumpkin_macros::{pumpkin_block, pumpkin_block_from_tag};
use pumpkin_util::GameMode;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::TextComponent;
use pumpkin_world::world::BlockFlags;
use std::sync::Mutex;

use crate::block::{
    BlockBehaviour, BrokenArgs, EmitsRedstonePowerArgs, GetComparatorOutputArgs,
    GetRedstonePowerArgs, GetScreenHandlerFactoryArgs, NormalUseArgs, OnPlaceArgs,
    OnSyncedBlockEventArgs, PathComputationType, PlacedArgs, RandomTickArgs,
    registry::BlockActionResult,
};
use crate::entity::EntityBase;
use crate::entity::passive::cat::CatEntity;
use crate::entity::passive::tamable::TamableAnimal;
use crate::entity::player::Player;
use crate::plugin::api::events::world::loot_generate::LootGenerateEvent;
use crate::world::World;
use crate::world::loot::{LootContextParameters, fill_chest_inventory_handle};
use pumpkin_data::BlockState;

// TODO vanilla `BaseContainerBlockEntity.canOpen`: the `lock` component, with `container.isLocked` and the
// `CHEST_LOCKED` sound on a failed open.
struct ChestScreenFactory {
    inventory: Arc<dyn Inventory>,
    loot_pending: bool,
}

impl ScreenHandlerFactory for ChestScreenFactory {
    fn create_screen_handler(
        &self,
        sync_id: u8,
        player_inventory: &Arc<PlayerInventory>,
        player: &dyn InventoryPlayer,
    ) -> Option<SharedScreenHandler> {
        if self.loot_pending && player.is_spectator() {
            return None;
        }

        let concrete_handler = if self.inventory.size() > 27 {
            create_generic_9x6(sync_id, player_inventory, self.inventory.clone(), player)
        } else {
            create_generic_9x3(sync_id, player_inventory, self.inventory.clone(), player)
        };

        let concrete_arc = Arc::new(Mutex::new(concrete_handler));

        Some(concrete_arc as SharedScreenHandler)
    }

    fn get_display_name(&self) -> TextComponent {
        if self.inventory.size() > 27 {
            pumpkin_macros::translate_cross!(
                translation::java::CONTAINER_CHESTDOUBLE,
                translation::bedrock::CONTAINER_CHESTDOUBLE
            )
        } else {
            pumpkin_macros::translate_cross!(
                translation::java::CONTAINER_CHEST,
                translation::bedrock::CONTAINER_CHEST
            )
        }
    }
}

// Shared chest behavior implementations
const LID_ANIMATION_EVENT_TYPE: u8 = 1;

fn on_place_chest_impl(args: &OnPlaceArgs<'_>) -> BlockStateId {
    let mut chest_props = ChestLikeProperties::default(args.block);
    chest_props.waterlogged = args.replacing.water_source();

    let (r#type, facing) = compute_chest_props(
        args.world,
        args.player,
        args.block,
        args.position,
        args.direction,
    );
    chest_props.facing = facing;
    chest_props.r#type = r#type;

    chest_props.to_state_id(args.block)
}

fn placed_chest_impl<E: BlockEntity + 'static>(
    args: &PlacedArgs<'_>,
    create_entity: impl FnOnce(BlockPos) -> E,
) {
    if args.world.get_block_entity(args.position).is_none() {
        let chest = create_entity(*args.position);
        args.world.add_block_entity(Arc::new(chest));
    }

    let chest_props = ChestLikeProperties::from_state_id(args.state_id);
    let connected_towards = match chest_props.r#type {
        ChestType::Single => return,
        ChestType::Left => chest_props.facing.rotate_clockwise(),
        ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
    };

    if let Some(mut neighbor_props) = get_chest_properties_if_can_connect(
        args.world,
        args.block,
        args.position,
        chest_props.facing,
        connected_towards,
        ChestType::Single,
    ) {
        neighbor_props.r#type = chest_props.r#type.opposite();

        args.world.set_block_state(
            &args.position.offset(connected_towards.to_offset()),
            neighbor_props.to_state_id(args.block),
            BlockFlags::NOTIFY_LISTENERS,
        );
    }
}

/// Direction to the other half of a double chest, `None` for a single chest.
const fn connected_direction(props: ChestLikeProperties) -> Option<HorizontalFacing> {
    match props.r#type {
        ChestType::Single => None,
        ChestType::Left => Some(props.facing.rotate_clockwise()),
        ChestType::Right => Some(props.facing.rotate_counter_clockwise()),
    }
}

/// Vanilla `DoubleBlockCombiner.combineWithNeigbour`: the other half counts only while it is the
/// same block, faces the same way and is the opposite half.
fn connected_half(
    world: &World,
    block: &Block,
    position: &BlockPos,
    props: ChestLikeProperties,
) -> Option<BlockPos> {
    let direction = connected_direction(props)?;
    get_chest_properties_if_can_connect(
        world,
        block,
        position,
        props.facing,
        direction,
        props.r#type.opposite(),
    )?;
    Some(position.offset(direction.to_offset()))
}

/// Vanilla `CompoundContainer` order: the `RIGHT` half comes first.
fn combine_halves(
    chest_type: ChestType,
    this: Arc<dyn Inventory>,
    other: Arc<dyn Inventory>,
) -> Arc<dyn Inventory> {
    if chest_type == ChestType::Right {
        DoubleInventory::new(this, other)
    } else {
        DoubleInventory::new(other, this)
    }
}

thread_local! {
    /// Chests (world address, position) whose `LootGenerateEvent` is being handled on this thread.
    static LOOT_EVENT_DISPATCH: RefCell<Vec<(usize, BlockPos)>> = const { RefCell::new(Vec::new()) };
}

/// Keeps a chest in [`LOOT_EVENT_DISPATCH`] while its event fires, also if a handler panics.
struct LootEventDispatch;

impl LootEventDispatch {
    fn enter(chest: (usize, BlockPos)) -> Self {
        LOOT_EVENT_DISPATCH.with_borrow_mut(|chests| chests.push(chest));
        Self
    }
}

impl Drop for LootEventDispatch {
    fn drop(&mut self) {
        LOOT_EVENT_DISPATCH.with_borrow_mut(Vec::pop);
    }
}

/// Vanilla `RandomizableContainer.unpackLootTable`: fills the chest from its pending loot table once.
fn unpack_loot_table(world: &World, entity: &Arc<dyn BlockEntity>, player: Option<&Player>) {
    // Fired before the loot lock -> a handler reading this chest would unpack it again.
    // A cancel still clears the key, the chest just stays empty.
    // TODO let `LootGenerateEvent` carry the container, context and generated items so a plugin can edit the loot.
    // TODO concurrent first unpacks on different threads each fire the event, only the one that takes the
    // key decides. One event per unpack needs an "unpacking" loot state that other threads wait on.
    let chest = (std::ptr::from_ref(world) as usize, entity.get_position());
    // A handler reading this chest lands here again: it stays packed, the outer call decides.
    if LOOT_EVENT_DISPATCH.with_borrow(|chests| chests.contains(&chest)) {
        return;
    }
    let cancelled_key = entity.loot_table_key().filter(|key| {
        let _dispatch = LootEventDispatch::enter(chest);
        world.cancelled_by_plugin(|| LootGenerateEvent::new(key.clone()))
    });
    let mut unpacked = None;
    entity.unpack_loot_table_with(&mut |loot_key, seed| {
        // The cancel only covers the key the event saw.
        if cancelled_key.as_deref() != Some(loot_key.as_str()) {
            fill_loot_table(world, entity, player, &loot_key, seed);
        }
        unpacked = Some(loot_key);
    });
    // Outside the loot lock: advancement handlers must not run while it is held.
    if let (Some(player), Some(loot_key)) = (player, unpacked) {
        player.trigger_advancement(
            crate::entity::player::advancement::trigger::AdvancementTrigger::PlayerGeneratesContainerLoot {
                loot_table_id: loot_key,
            },
        );
    }
}

fn fill_loot_table(
    world: &World,
    entity: &Arc<dyn BlockEntity>,
    player: Option<&Player>,
    loot_key: &str,
    seed: i64,
) {
    // Vanilla resolves an unknown key to `LootTable.EMPTY`: the key is cleared, nothing is placed.
    let (Some(table), Some(inventory)) = (
        world.get_loot_table(loot_key),
        entity.clone().get_inventory(),
    ) else {
        return;
    };
    fill_chest_inventory_handle(
        &inventory,
        &table,
        seed,
        &LootContextParameters {
            world: Some(world),
            // Vanilla `ORIGIN` is `Vec3.atCenterOf` of the half being unpacked.
            position: Some(entity.get_position().to_centered_f64()),
            this_entity: player.map(|player| player as &dyn EntityBase),
            luck: player.map_or(0.0, |player| {
                player
                    .living_entity
                    .get_attribute_value(&pumpkin_data::attributes::Attributes::LUCK)
                    as f32
            }),
            ..Default::default()
        },
    );
    inventory.mark_dirty();
}

/// Vanilla `ChestBlock.getContainer` with `ignoreBeingBlocked = true`: a double chest is one container.
///
/// Vanilla unpacks a half on its first slot access. every caller reads slots right away, so both
/// halves unpack here.
pub fn chest_container(
    world: &World,
    block: &Block,
    position: &BlockPos,
    state_id: BlockStateId,
    entity: &Arc<dyn BlockEntity>,
) -> Option<Arc<dyn Inventory>> {
    unpack_loot_table(world, entity, None);
    let this = entity.clone().get_container()?;
    let props = ChestLikeProperties::from_state_id(state_id);
    let other = connected_half(world, block, position, props)
        .and_then(|other_pos| world.get_block_entity(&other_pos))
        .and_then(|other| {
            unpack_loot_table(world, &other, None);
            other.get_container()
        });
    Some(match other {
        Some(other) => combine_halves(props.r#type, this, other),
        None => this,
    })
}

/// Computes the comparator output for a chest, combining both halves for double chests.
fn get_chest_comparator_output(args: &GetComparatorOutputArgs<'_>) -> Option<u8> {
    let state = args.world.get_block_state_id(args.position);
    let first_chest = args.world.get_block_entity(args.position)?;

    let chest_props = ChestLikeProperties::from_state_id(state);
    let other_pos = connected_half(args.world, args.block, args.position, chest_props);

    // Vanilla passes `ignoreBeingBlocked = false`, so a blocked half reads zero.
    if is_chest_blocked(args.world, args.position)
        || other_pos.is_some_and(|other_pos| is_chest_blocked(args.world, &other_pos))
    {
        return Some(0);
    }

    // Vanilla reads every slot through `getItem`, which unpacks pending loot first.
    let second_chest = other_pos.and_then(|other_pos| args.world.get_block_entity(&other_pos));
    for entity in std::iter::once(&first_chest).chain(&second_chest) {
        unpack_loot_table(args.world, entity, None);
    }

    let first_inventory = first_chest.get_inventory()?;
    let inventory = match second_chest.and_then(BlockEntity::get_inventory) {
        Some(second_inventory) => {
            combine_halves(chest_props.r#type, first_inventory, second_inventory)
        }
        None => first_inventory,
    };
    Some(crate::block::calculate_comparator_output(
        inventory.as_ref(),
    ))
}

/// Returns a screen handler factory for opening a chest or double chest inventory.
///
/// Unpacks deferred loot tables on first open (for non-spectator players) and combines
/// connected inventories for double chests if neither chest half is obstructed.
fn get_chest_screen_handler_factory(
    args: GetScreenHandlerFactoryArgs<'_>,
) -> Option<Box<dyn ScreenHandlerFactory>> {
    let state = args.world.get_block_state_id(args.position);
    let first_chest = args.world.get_block_entity(args.position);

    let chest_props = ChestLikeProperties::from_state_id(state);
    let other_pos = connected_half(args.world, args.block, args.position, chest_props);

    // Vanilla gives a blocked chest no menu, so it stays packed too.
    if is_chest_blocked(args.world, args.position)
        || other_pos.is_some_and(|other_pos| is_chest_blocked(args.world, &other_pos))
    {
        return None;
    }

    let second = other_pos.and_then(|other_pos| args.world.get_block_entity(&other_pos));
    // Both halves of a double chest are unpacked at once, like vanilla's CompoundContainer.
    if args.player.gamemode.load() != GameMode::Spectator {
        for entity in first_chest.iter().chain(&second) {
            unpack_loot_table(args.world, entity, Some(args.player));
        }
    }

    let loot_pending = first_chest
        .iter()
        .chain(&second)
        .any(|entity| entity.has_loot_table());
    let first_inventory = first_chest.and_then(BlockEntity::get_inventory)?;

    let inventory = match second.and_then(BlockEntity::get_inventory) {
        Some(second_inventory) => {
            combine_halves(chest_props.r#type, first_inventory, second_inventory)
        }
        None => first_inventory,
    };

    Some(Box::new(ChestScreenFactory {
        inventory,
        loot_pending,
    }))
}

fn normal_use_chest_impl(args: &NormalUseArgs<'_>) -> BlockActionResult {
    // Vanilla awards the stat only when there is a menu, so a blocked chest doesn't count.
    if let Some(factory) = get_chest_screen_handler_factory(GetScreenHandlerFactoryArgs {
        server: args.server,
        world: args.world,
        block: args.block,
        position: args.position,
        player: args.player,
    }) {
        args.player
            .open_handled_screen(factory.as_ref(), Some(*args.position));
        let stat = if args.block.id == Block::TRAPPED_CHEST.id {
            pumpkin_data::statistic::CustomStatistic::TriggerTrappedChest
        } else {
            pumpkin_data::statistic::CustomStatistic::OpenChest
        };
        args.player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Custom,
            stat as i32,
            1,
        );
        // TODO vanilla `PiglinAi.angerNearbyPiglins(level, player, true)`.
    }

    BlockActionResult::Success
}

fn broken_chest_impl(args: &BrokenArgs<'_>) {
    let chest_props = ChestLikeProperties::from_state_id(args.state.id);
    let connected_towards = match chest_props.r#type {
        ChestType::Single => return,
        ChestType::Left => chest_props.facing.rotate_clockwise(),
        ChestType::Right => chest_props.facing.rotate_counter_clockwise(),
    };

    if let Some(mut neighbor_props) = get_chest_properties_if_can_connect(
        args.world,
        args.block,
        args.position,
        chest_props.facing,
        connected_towards,
        chest_props.r#type.opposite(),
    ) {
        neighbor_props.r#type = ChestType::Single;

        args.world.set_block_state(
            &args.position.offset(connected_towards.to_offset()),
            neighbor_props.to_state_id(args.block),
            BlockFlags::NOTIFY_LISTENERS,
        );
    }
}

#[pumpkin_block_from_tag("c:chests/wooden")]
pub struct ChestBlock;

impl BlockBehaviour for ChestBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        on_place_chest_impl(&args)
    }

    fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        args.r#type == LID_ANIMATION_EVENT_TYPE
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        placed_chest_impl(&args, ChestBlockEntity::new);
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        normal_use_chest_impl(&args)
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        get_chest_screen_handler_factory(args)
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        broken_chest_impl(&args);
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        get_chest_comparator_output(&args)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

/// Copper chests have the same behavior as wooden chests but also oxidize over time.
#[pumpkin_block_from_tag("minecraft:copper_chests")]
pub struct CopperChestBlock;

impl CopperChestBlock {
    pub(crate) fn get_hinge_sound(block: &Block, open: bool) -> Sound {
        let (open_sound, close_sound) = if block == &Block::WEATHERED_COPPER_CHEST
            || block == &Block::WAXED_WEATHERED_COPPER_CHEST
        {
            (
                Sound::BlockCopperChestWeatheredOpen,
                Sound::BlockCopperChestWeatheredClose,
            )
        } else if block == &Block::OXIDIZED_COPPER_CHEST
            || block == &Block::WAXED_OXIDIZED_COPPER_CHEST
        {
            (
                Sound::BlockCopperChestOxidizedOpen,
                Sound::BlockCopperChestOxidizedClose,
            )
        } else {
            (Sound::BlockCopperChestOpen, Sound::BlockCopperChestClose)
        };

        if open { open_sound } else { close_sound }
    }
}

impl
    crate::block::blocks::weathering_copper::ChangeOverTimeBlock<
        crate::block::blocks::weathering_copper::WeatherState,
    > for CopperChestBlock
{
    fn get_age(
        &self,
        block: &Block,
    ) -> Option<crate::block::blocks::weathering_copper::WeatherState> {
        crate::block::blocks::weathering_copper::get_weather_state(block)
    }

    fn get_chance_modifier(
        &self,
        age: crate::block::blocks::weathering_copper::WeatherState,
    ) -> f32 {
        crate::block::blocks::weathering_copper::get_chance_modifier(age)
    }

    fn get_next(&self, block: &Block) -> Option<&'static Block> {
        crate::block::blocks::weathering_copper::get_next(block)
    }

    fn get_previous(&self, block: &Block) -> Option<&'static Block> {
        crate::block::blocks::weathering_copper::get_previous(block)
    }

    fn get_first(&self, block: &Block) -> Option<&'static Block> {
        crate::block::blocks::weathering_copper::get_first(block)
    }
}

impl crate::block::blocks::weathering_copper::WeatheringCopper for CopperChestBlock {}

impl BlockBehaviour for CopperChestBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        on_place_chest_impl(&args)
    }

    fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        args.r#type == LID_ANIMATION_EVENT_TYPE
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        placed_chest_impl(&args, ChestBlockEntity::new);
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        normal_use_chest_impl(&args)
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        get_chest_screen_handler_factory(args)
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        broken_chest_impl(&args);
    }

    fn random_tick(&self, args: RandomTickArgs<'_>) {
        let current_state_id = args.world.get_block_state_id(args.position);
        let chest_props = ChestLikeProperties::from_state_id(current_state_id);

        // Only oxidize LEFT or SINGLE chests (not RIGHT) to prevent double oxidation
        if chest_props.r#type == ChestType::Right {
            return;
        }

        // Only oxidize if no players are viewing the chest
        if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(chest_entity) = block_entity.as_any().downcast_ref::<ChestBlockEntity>()
            && chest_entity.get_viewer_count() > 0
        {
            return;
        }

        crate::block::blocks::weathering_copper::change_over_time(
            args.world,
            args.position,
            args.block,
        );
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        get_chest_comparator_output(&args)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

/// Trapped chests have the same behavior as wooden chests but also emit redstone power based on viewer count.
#[pumpkin_block("minecraft:trapped_chest")]
pub struct TrappedChestBlock;

impl BlockBehaviour for TrappedChestBlock {
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        on_place_chest_impl(&args)
    }

    fn on_synced_block_event(&self, args: OnSyncedBlockEventArgs<'_>) -> bool {
        args.r#type == LID_ANIMATION_EVENT_TYPE
    }

    fn placed(&self, args: PlacedArgs<'_>) {
        use crate::block::entities::trapped_chest::TrappedChestBlockEntity;
        placed_chest_impl(&args, TrappedChestBlockEntity::new);
    }

    fn normal_use(&self, args: NormalUseArgs<'_>) -> BlockActionResult {
        normal_use_chest_impl(&args)
    }

    fn get_screen_handler_factory(
        &self,
        args: GetScreenHandlerFactoryArgs<'_>,
    ) -> Option<Box<dyn ScreenHandlerFactory>> {
        get_chest_screen_handler_factory(args)
    }

    fn broken(&self, args: BrokenArgs<'_>) {
        broken_chest_impl(&args);
    }

    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        true
    }

    fn get_weak_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        use crate::block::entities::trapped_chest::TrappedChestBlockEntity;

        // Get viewer count from this chest
        let viewer_count = if let Some(block_entity) = args.world.get_block_entity(args.position)
            && let Some(trapped_chest) = block_entity
                .as_any()
                .downcast_ref::<TrappedChestBlockEntity>()
        {
            trapped_chest.get_viewer_count()
        } else {
            0
        };

        viewer_count.min(15) as u8
    }

    fn get_strong_redstone_power(&self, args: GetRedstonePowerArgs<'_>) -> u8 {
        // Strong power emitted to the block beneath the trapped chest
        // The block below queries with direction Up (from below looking up at the chest)
        if args.direction == BlockDirection::Up {
            self.get_weak_redstone_power(args)
        } else {
            0
        }
    }

    fn get_comparator_output(&self, args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        get_chest_comparator_output(&args)
    }

    fn is_pathfindable(&self, _state: &BlockState, _computation_type: PathComputationType) -> bool {
        false
    }
}

fn compute_chest_props(
    world: &World,
    player: &Player,
    block: &Block,
    block_pos: &BlockPos,
    face: BlockDirection,
) -> (ChestType, HorizontalFacing) {
    let player_facing = player.get_entity().get_horizontal_facing();
    let chest_facing = player_facing.opposite();

    if player.get_entity().pose.load() == EntityPose::Crouching {
        let Some(face) = face.to_horizontal_facing() else {
            return (ChestType::Single, chest_facing);
        };

        let (clicked_block, clicked_block_state) =
            world.get_block_and_state_id(&block_pos.offset(face.to_offset()));

        if clicked_block == block {
            let clicked_props = ChestLikeProperties::from_state_id(clicked_block_state);

            if clicked_props.r#type != ChestType::Single {
                return (ChestType::Single, chest_facing);
            }

            if clicked_props.facing.rotate_clockwise() == face {
                return (ChestType::Left, clicked_props.facing);
            } else if clicked_props.facing.rotate_counter_clockwise() == face {
                return (ChestType::Right, clicked_props.facing);
            }
        }

        return (ChestType::Single, chest_facing);
    }

    if get_chest_properties_if_can_connect(
        world,
        block,
        block_pos,
        chest_facing,
        chest_facing.rotate_clockwise(),
        ChestType::Single,
    )
    .is_some()
    {
        (ChestType::Left, chest_facing)
    } else if get_chest_properties_if_can_connect(
        world,
        block,
        block_pos,
        chest_facing,
        chest_facing.rotate_counter_clockwise(),
        ChestType::Single,
    )
    .is_some()
    {
        (ChestType::Right, chest_facing)
    } else {
        (ChestType::Single, chest_facing)
    }
}

fn get_chest_properties_if_can_connect(
    world: &World,
    block: &Block,
    block_pos: &BlockPos,
    facing: HorizontalFacing,
    direction: HorizontalFacing,
    wanted_type: ChestType,
) -> Option<ChestLikeProperties> {
    let (neighbor_block, neighbor_block_state) =
        world.get_block_and_state_id(&block_pos.offset(direction.to_offset()));

    if neighbor_block != block {
        return None;
    }

    let neighbor_props = ChestLikeProperties::from_state_id(neighbor_block_state);
    if neighbor_props.facing == facing && neighbor_props.r#type == wanted_type {
        return Some(neighbor_props);
    }

    None
}

fn is_chest_blocked(world: &World, block_pos: &BlockPos) -> bool {
    has_block_on_top(world, block_pos) || is_cat_sitting_on_chest(world, block_pos)
}
fn has_block_on_top(world: &World, block_pos: &BlockPos) -> bool {
    let above_pos = block_pos.up();
    let above_state = world.get_block_state(&above_pos);
    above_state.is_solid_block()
}

fn is_cat_sitting_on_chest(world: &World, block_pos: &BlockPos) -> bool {
    let search_box = BoundingBox::from_block(&block_pos.up());
    world.get_entities_at_box(&search_box).iter().any(|entity| {
        entity
            .cast_any()
            .downcast_ref::<CatEntity>()
            .is_some_and(TamableAnimal::is_in_sitting_pose)
    })
}

trait ChestTypeExt {
    fn opposite(&self) -> ChestType;
}

impl ChestTypeExt for ChestType {
    fn opposite(&self) -> Self {
        match self {
            Self::Single => Self::Single,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }
}
