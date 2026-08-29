use pumpkin_data::{Block, BlockId, BlockState};

use pumpkin_data::BlockStateId;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::{RandomGenerator, get_seed, xoroshiro128::Xoroshiro};

use crate::entity::experience_orb::ExperienceOrbEntity;
use crate::entity::player::Player;
use crate::world::World;
use crate::world::loot::{LootContextParameters, LootTableExt};
use std::sync::Arc;

pub mod blocks;
pub mod entities;
pub mod fluid;
pub mod registry;
pub mod viewer;

use crate::block::registry::BlockActionResult;
use crate::entity::EntityBase;
use crate::server::Server;
use pumpkin_data::BlockDirection;
use pumpkin_data::block_rotation::{Mirror, Rotation};
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::{BlockAccessor, BlockFlags};

pub trait BlockMetadata {
    fn ids() -> Box<[BlockId]>;
}

pub trait FluidMetadata {
    fn ids() -> Box<[u16]>;
}

pub(crate) fn stop_vertical_movement_after_fall(entity: &dyn EntityBase) {
    let entity = entity.get_entity();
    let mut velocity = entity.velocity.load();
    velocity.y = 0.0;
    entity.velocity.store(velocity);
}

pub(crate) fn bounce_entity_after_fall(entity: &dyn EntityBase, bounce_multiplier: f64) {
    let base_entity = entity.get_entity();
    let mut velocity = base_entity.velocity.load();

    if base_entity.is_sneaking() {
        velocity.y = 0.0;
    } else if velocity.y < 0.0 {
        let entity_factor = if entity.get_living_entity().is_some() {
            1.0
        } else {
            0.8
        };
        velocity.y = -velocity.y * bounce_multiplier * entity_factor;
    }

    base_entity.velocity.store(velocity);
}

pub trait BlockBehaviour: Send + Sync {
    fn is_valid_bonemeal_target(&self, _args: BonemealArgs<'_>) -> bool {
        false
    }

    fn is_bonemeal_success(&self, _args: BonemealArgs<'_>) -> bool {
        true
    }

    fn perform_bonemeal(&self, _args: BonemealArgs<'_>) {}

    fn normal_use(&self, _args: NormalUseArgs<'_>) -> BlockActionResult {
        BlockActionResult::Pass
    }

    fn use_with_item(&self, _args: UseWithItemArgs<'_>) -> BlockActionResult {
        BlockActionResult::PassToDefaultBlockAction
    }

    fn on_entity_collision(&self, _args: OnEntityCollisionArgs<'_>) {}

    /// Called when an entity is standing on / walking over the top face of this block.
    fn on_entity_step(&self, _args: OnEntityStepArgs<'_>) {}

    fn should_drop_items_on_explosion(&self) -> bool {
        true
    }

    fn explode(&self, _args: ExplodeArgs<'_>) {}

    /// Handles the block event, which is an event specific to a block with an integer ID and data.
    ///
    /// returns whether the event was handled successfully
    fn on_synced_block_event(&self, _args: OnSyncedBlockEventArgs<'_>) -> bool {
        false
    }

    /// getPlacementState in source code
    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        args.block.default_state.id
    }

    fn random_tick(&self, _args: RandomTickArgs<'_>) {}

    fn can_place_at(&self, _args: CanPlaceAtArgs<'_>) -> bool {
        true
    }

    fn can_update_at(&self, _args: CanUpdateAtArgs<'_>) -> bool {
        false
    }

    /// onBlockAdded in source code
    fn placed(&self, _args: PlacedArgs<'_>) {}

    fn player_placed(&self, _args: PlayerPlacedArgs<'_>) {}

    fn on_landed_upon(&self, args: OnLandedUponArgs<'_>) {
        if let Some(living) = args.entity.get_living_entity() {
            living.handle_fall_damage(args.entity, args.fall_distance, 1.0);
        }
    }

    fn update_entity_movement_after_fall_on(&self, args: UpdateEntityMovementAfterFallOnArgs<'_>) {
        stop_vertical_movement_after_fall(args.entity);
    }

    fn broken(&self, _args: BrokenArgs<'_>) {}

    fn on_neighbor_update(&self, _args: OnNeighborUpdateArgs<'_>) {}

    /// Called if a block state is replaced or it replaces another state
    fn prepare(&self, _args: PrepareArgs<'_>) {}

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        args.state_id
    }

    fn on_scheduled_tick(&self, _args: OnScheduledTickArgs<'_>) {}

    fn on_state_replaced(&self, _args: OnStateReplacedArgs<'_>) {}

    // --- Redstone/Comparator Methods ---

    /// Sides where redstone connects to
    fn emits_redstone_power(&self, _args: EmitsRedstonePowerArgs<'_>) -> bool {
        false
    }

    /// Weak redstone power, aka. block that should be powered needs to be directly next to the source block
    fn get_weak_redstone_power(&self, _args: GetRedstonePowerArgs<'_>) -> u8 {
        0
    }

    /// Strong redstone power. this can power a block that then gives power
    fn get_strong_redstone_power(&self, _args: GetRedstonePowerArgs<'_>) -> u8 {
        0
    }

    fn get_comparator_output(&self, _args: GetComparatorOutputArgs<'_>) -> Option<u8> {
        None
    }

    fn get_inside_collision_shape(&self, _args: GetInsideCollisionShapeArgs<'_>) -> BoundingBox {
        BoundingBox::full_block()
    }

    fn mirror(&self, block: &Block, state_id: BlockStateId, mirror: Mirror) -> &'static BlockState {
        block.mirror(state_id, mirror)
    }

    fn rotate(
        &self,
        block: &Block,
        state_id: BlockStateId,
        rotation: Rotation,
    ) -> &'static BlockState {
        block.rotate(state_id, rotation)
    }
}

#[derive(Clone, Copy)]
pub struct BonemealArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub position: &'a BlockPos,
    pub state_id: BlockStateId,
}

pub struct NormalUseArgs<'a> {
    pub server: &'a Server,
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub position: &'a BlockPos,
    pub player: &'a Arc<Player>,
    pub hit: &'a BlockHitResult<'a>,
}

pub struct UseWithItemArgs<'a> {
    pub server: &'a Server,
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub position: &'a BlockPos,
    pub player: &'a Arc<Player>,
    pub hit: &'a BlockHitResult<'a>,
    pub item_stack: &'a mut ItemStack,
    pub equipment_slot: &'a EquipmentSlot,
}

pub struct BlockHitResult<'a> {
    pub face: &'a BlockDirection,
    pub cursor_pos: &'a Vector3<f32>,
}

pub struct OnEntityCollisionArgs<'a> {
    pub server: &'a Server,
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub state: &'a BlockState,
    pub position: &'a BlockPos,
    pub entity: &'a dyn EntityBase,
}

pub struct OnEntityStepArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub state: &'a BlockState,
    pub position: &'a BlockPos,
    pub entity: &'a dyn EntityBase,
    pub below_supporting_block: bool,
}

pub struct ExplodeArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub position: &'a BlockPos,
}

pub struct OnSyncedBlockEventArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub position: &'a BlockPos,
    pub r#type: u8,
    pub data: u8,
}

pub struct OnPlaceArgs<'a> {
    pub server: &'a Server,
    pub world: &'a World,
    pub block: &'a Block,
    pub position: &'a BlockPos,
    pub direction: BlockDirection,
    pub player: &'a Player,
    pub replacing: BlockIsReplacing,
    pub use_item_on: &'a SUseItemOn,
}

pub struct RandomTickArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub position: &'a BlockPos,
}

pub struct CanPlaceAtArgs<'a> {
    pub server: Option<&'a Server>,
    pub world: Option<&'a World>,
    pub block_accessor: &'a dyn BlockAccessor,
    pub block: &'a Block,
    pub state: &'a BlockState,
    pub position: &'a BlockPos,
    pub direction: Option<BlockDirection>,
    pub player: Option<&'a Player>,
    pub use_item_on: Option<&'a SUseItemOn>,
}

pub struct CanUpdateAtArgs<'a> {
    pub world: &'a World,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub position: &'a BlockPos,
    pub direction: BlockDirection,
    pub player: &'a Player,
    pub use_item_on: &'a SUseItemOn,
}

pub struct PlacedArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub old_state_id: BlockStateId,
    pub position: &'a BlockPos,
    pub notify: bool,
}

pub struct PlayerPlacedArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub position: &'a BlockPos,
    pub direction: BlockDirection,
    pub player: &'a Player,
}

pub struct OnLandedUponArgs<'a> {
    pub world: &'a Arc<World>,
    pub fall_distance: f32,
    pub entity: &'a dyn EntityBase,
}

pub struct UpdateEntityMovementAfterFallOnArgs<'a> {
    pub entity: &'a dyn EntityBase,
}

pub struct BrokenArgs<'a> {
    pub block: &'a Block,
    pub player: &'a Arc<Player>,
    pub position: &'a BlockPos,
    pub server: &'a Server,
    pub world: &'a Arc<World>,
    pub state: &'a BlockState,
}

pub struct OnNeighborUpdateArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub position: &'a BlockPos,
    pub source_block: &'a Block,
    pub notify: bool,
}

pub struct PrepareArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub position: &'a BlockPos,
    pub flags: BlockFlags,
}

pub struct GetStateForNeighborUpdateArgs<'a> {
    pub world: &'a World,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub position: &'a BlockPos,
    pub direction: BlockDirection,
    pub neighbor_position: &'a BlockPos,
    pub neighbor_state_id: BlockStateId,
}

pub struct OnScheduledTickArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub position: &'a BlockPos,
}

pub struct OnStateReplacedArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub old_state_id: BlockStateId,
    pub position: &'a BlockPos,
    pub moved: bool,
}

pub struct EmitsRedstonePowerArgs<'a> {
    pub block: &'a Block,
    pub state: &'a BlockState,
    pub direction: BlockDirection,
}

pub struct GetRedstonePowerArgs<'a> {
    pub world: &'a World,
    pub block: &'a Block,
    pub state: &'a BlockState,
    pub position: &'a BlockPos,
    pub direction: BlockDirection,
}

pub struct GetComparatorOutputArgs<'a> {
    pub world: &'a World,
    pub block: &'a Block,
    pub state: &'a BlockState,
    pub position: &'a BlockPos,
}

pub struct GetInsideCollisionShapeArgs<'a> {
    pub world: &'a World,
    pub block: &'a Block,
    pub state: &'a BlockState,
    pub position: &'a BlockPos,
}

#[derive(Clone)]
pub struct BlockEvent {
    pub pos: BlockPos,
    pub r#type: u8,
    pub data: u8,
}

pub fn drop_loot(
    world: &Arc<World>,
    block: &Block,
    pos: &BlockPos,
    experience: bool,
    params: LootContextParameters,
) {
    if let Some(loot_table) = &block.loot_table {
        let items = loot_table.get_loot(params);
        if !items.is_empty() {
            let mut event = crate::plugin::block::block_drop_item::BlockDropItemEvent {
                block_pos: *pos,
                world: world.clone(),
                player: None,
                items,
                cancelled: false,
            };
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            if !event.cancelled {
                for stack in event.items {
                    world.drop_stack(pos, stack);
                }
            }
        }
    }

    if experience && let Some(experience) = &block.experience {
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(get_seed()));
        let amount = experience.experience.get(&mut random);
        // TODO: Silk touch gives no exp
        if amount > 0 {
            let mut event = crate::plugin::block::block_exp::BlockExpEvent {
                block_pos: *pos,
                world: world.clone(),
                exp: amount,
            };
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            if event.exp > 0 {
                ExperienceOrbEntity::spawn(world, pos.to_f64(), event.exp as u32);
            }
        }
    }
}

pub fn calc_block_breaking(player: &Player, state: &BlockState, block: &'static Block) -> f32 {
    let hardness = state.hardness;
    #[expect(clippy::float_cmp)]
    if hardness == -1.0 {
        // unbreakable
        return 0.0;
    }
    let i = if player.can_harvest(state, block) {
        30.0
    } else {
        100.0
    };

    player.get_mining_speed(block) / hardness / i
}

#[derive(PartialEq, Eq, Debug)]
pub enum BlockIsReplacing {
    Itself(BlockStateId),
    Water(u8),
    Other,
    None,
}

impl BlockIsReplacing {
    #[must_use]
    /// Returns true if the block was a water source block.
    pub const fn water_source(&self) -> bool {
        match self {
            // Level 0 means the water is a source block
            Self::Water(level) => *level == 0,
            _ => false,
        }
    }
}

pub fn calculate_comparator_output(inventory: &dyn pumpkin_world::inventory::Inventory) -> u8 {
    let size = inventory.size();
    if size == 0 {
        return 0;
    }
    let mut fill_sum = 0.0;
    let mut non_empty_count = 0;
    for i in 0..size {
        let stack = inventory.get_stack(i);
        if !stack.is_empty() {
            let max_stack = stack.get_max_stack_size() as f32;
            let count = stack.item_count as f32;
            fill_sum += count / max_stack;
            non_empty_count += 1;
        }
    }
    if non_empty_count == 0 {
        return 0;
    }
    let percentage = fill_sum / (size as f32);
    let output = 1.0 + percentage * 14.0;
    output.floor() as u8
}
