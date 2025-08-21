use crate::block::blocks::anvil::AnvilBlock;
use crate::block::blocks::bamboo::BambooBlock;
use crate::block::blocks::barrel::BarrelBlock;
use crate::block::blocks::bed::BedBlock;
use crate::block::blocks::cactus::CactusBlock;
use crate::block::blocks::carpet::{CarpetBlock, MossCarpetBlock, PaleMossCarpetBlock};
use crate::block::blocks::carved_pumpkin::CarvedPumpkinBlock;
use crate::block::blocks::chests::ChestBlock;
use crate::block::blocks::command::CommandBlock;
use crate::block::blocks::composter::ComposterBlock;
use crate::block::blocks::dirt_path::DirtPathBlock;
use crate::block::blocks::doors::DoorBlock;
use crate::block::blocks::end_portal::EndPortalBlock;
use crate::block::blocks::end_portal_frame::EndPortalFrameBlock;
use crate::block::blocks::falling::FallingBlock;
use crate::block::blocks::farmland::FarmlandBlock;
use crate::block::blocks::fence_gates::FenceGateBlock;
use crate::block::blocks::fences::FenceBlock;
use crate::block::blocks::fire::fire::FireBlock;
use crate::block::blocks::fire::soul_fire::SoulFireBlock;
use crate::block::blocks::furnace::FurnaceBlock;
use crate::block::blocks::glass_panes::GlassPaneBlock;
use crate::block::blocks::grindstone::GrindstoneBlock;
use crate::block::blocks::iron_bars::IronBarsBlock;
use crate::block::blocks::logs::LogBlock;
use crate::block::blocks::nether_portal::NetherPortalBlock;
use crate::block::blocks::note::NoteBlock;
use crate::block::blocks::piston::piston::PistonBlock;
use crate::block::blocks::piston::piston_extension::PistonExtensionBlock;
use crate::block::blocks::piston::piston_head::PistonHeadBlock;
use crate::block::blocks::plant::bush::BushBlock;
use crate::block::blocks::plant::dry_vegetation::DryVegetationBlock;
use crate::block::blocks::plant::flower::FlowerBlock;
use crate::block::blocks::plant::flowerbed::FlowerbedBlock;
use crate::block::blocks::plant::leaf_litter::LeafLitterBlock;
use crate::block::blocks::plant::lily_pad::LilyPadBlock;
use crate::block::blocks::plant::mushroom_plant::MushroomPlantBlock;
use crate::block::blocks::plant::sapling::SaplingBlock;
use crate::block::blocks::plant::short_plant::ShortPlantBlock;
use crate::block::blocks::plant::tall_plant::TallPlantBlock;
use crate::block::blocks::pumpkin::PumpkinBlock;
use crate::block::blocks::redstone::buttons::ButtonBlock;
use crate::block::blocks::redstone::comparator::ComparatorBlock;
use crate::block::blocks::redstone::copper_bulb::CopperBulbBlock;
use crate::block::blocks::redstone::lever::LeverBlock;
use crate::block::blocks::redstone::observer::ObserverBlock;
use crate::block::blocks::redstone::pressure_plate::plate::PressurePlateBlock;
use crate::block::blocks::redstone::pressure_plate::weighted::WeightedPressurePlateBlock;
use crate::block::blocks::redstone::rails::activator_rail::ActivatorRailBlock;
use crate::block::blocks::redstone::rails::detector_rail::DetectorRailBlock;
use crate::block::blocks::redstone::rails::powered_rail::PoweredRailBlock;
use crate::block::blocks::redstone::rails::rail::RailBlock;
use crate::block::blocks::redstone::redstone_block::RedstoneBlock;
use crate::block::blocks::redstone::redstone_lamp::RedstoneLamp;
use crate::block::blocks::redstone::redstone_torch::RedstoneTorchBlock;
use crate::block::blocks::redstone::redstone_wire::RedstoneWireBlock;
use crate::block::blocks::redstone::repeater::RepeaterBlock;
use crate::block::blocks::redstone::target_block::TargetBlock;
use crate::block::blocks::redstone::tripwire::TripwireBlock;
use crate::block::blocks::redstone::tripwire_hook::TripwireHookBlock;
use crate::block::blocks::signs::SignBlock;
use crate::block::blocks::slabs::SlabBlock;
use crate::block::blocks::stairs::StairBlock;
use crate::block::blocks::sugar_cane::SugarCaneBlock;
use crate::block::blocks::tnt::TNTBlock;
use crate::block::blocks::torches::TorchBlock;
use crate::block::blocks::trapdoor::TrapDoorBlock;
use crate::block::blocks::vine::VineBlock;
use crate::block::blocks::walls::WallBlock;
use crate::block::fluid::lava::FlowingLava;
use crate::block::fluid::water::FlowingWater;
use crate::block::{BlockBehaviour, BlockHitResult, BlockMetadata, OnEntityCollisionArgs};
use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::server::Server;
use crate::world::World;
use async_trait::async_trait;
use pumpkin_data::fluid;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_protocol::java::server::play::SUseItemOn;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;
use pumpkin_world::item::ItemStack;
use pumpkin_world::world::{BlockAccessor, BlockFlags, BlockRegistryExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::block::blocks::banners::BannerBlock;
use crate::block::blocks::cake::CakeBlock;
use crate::block::blocks::campfire::CampfireBlock;
use crate::block::blocks::candle_cakes::CandleCakeBlock;
use crate::block::blocks::candles::CandleBlock;
use crate::block::blocks::chiseled_bookshelf::ChiseledBookshelfBlock;
use crate::block::blocks::flower_pots::FlowerPotBlock;
use crate::block::blocks::glazed_terracotta::GlazedTerracottaBlock;
use crate::block::blocks::plant::crop::beetroot::BeetrootBlock;
use crate::block::blocks::plant::crop::carrot::CarrotBlock;
use crate::block::blocks::plant::crop::potatoes::PotatoBlock;
use crate::block::blocks::plant::crop::torch_flower::TorchFlowerBlock;
use crate::block::blocks::plant::crop::wheat::WheatBlock;
use crate::block::blocks::plant::nether_wart::NetherWartBlock;
use crate::block::blocks::plant::roots::RootsBlock;
use crate::block::blocks::plant::sea_grass::SeaGrassBlock;
use crate::block::blocks::plant::sea_pickles::SeaPickleBlock;
use crate::block::blocks::redstone::dispenser::DispenserBlock;
use crate::block::blocks::redstone::dropper::DropperBlock;

use super::BlockIsReplacing;
use super::blocks::plant::crop::gourds::attached_stem::AttachedStemBlock;
use super::blocks::plant::crop::gourds::stem::StemBlock;
use super::fluid::FluidBehaviour;
use super::{
    BrokenArgs, CanPlaceAtArgs, CanUpdateAtArgs, EmitsRedstonePowerArgs, ExplodeArgs,
    GetRedstonePowerArgs, GetStateForNeighborUpdateArgs, NormalUseArgs, OnNeighborUpdateArgs,
    OnPlaceArgs, OnStateReplacedArgs, OnSyncedBlockEventArgs, PlacedArgs, PlayerPlacedArgs,
    PrepareArgs, UseWithItemArgs,
};
use crate::block::blocks::blast_furnace::BlastFurnaceBlock;
use crate::block::blocks::chain::ChainBlock;
use crate::block::blocks::crafting_table::CraftingTableBlock;
use crate::block::blocks::end_rod::EndRodBlock;
use crate::block::blocks::ender_chest::EnderChestBlock;
use crate::block::blocks::hopper::HopperBlock;
use crate::block::blocks::jukebox::JukeboxBlock;
use crate::block::blocks::ladder::LadderBlock;
use crate::block::blocks::lanterns::LanternBlock;
use crate::block::blocks::lectern::LecternBlock;
use crate::block::blocks::shulker_box::ShulkerBoxBlock;
use crate::block::blocks::skull_block::SkullBlock;
use crate::block::blocks::smoker::SmokerBlock;

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn default_registry() -> Arc<BlockRegistry> {
    let mut manager = BlockRegistry::default();

    // Blocks
    manager.register(AnvilBlock);
    manager.register(BedBlock);
    manager.register(SaplingBlock);
    manager.register(CactusBlock);
    manager.register(CarpetBlock);
    manager.register(CarvedPumpkinBlock);
    manager.register(CampfireBlock);
    manager.register(MossCarpetBlock);
    manager.register(PaleMossCarpetBlock);
    manager.register(ChestBlock);
    manager.register(EnderChestBlock);
    manager.register(CraftingTableBlock);
    manager.register(DirtPathBlock);
    manager.register(DoorBlock);
    manager.register(FarmlandBlock);
    manager.register(FenceGateBlock);
    manager.register(FenceBlock);
    manager.register(FlowerPotBlock);
    manager.register(FurnaceBlock);
    manager.register(BlastFurnaceBlock);
    manager.register(SmokerBlock);
    manager.register(GlassPaneBlock);
    manager.register(GlazedTerracottaBlock);
    manager.register(GrindstoneBlock);
    manager.register(IronBarsBlock);
    manager.register(JukeboxBlock);
    manager.register(LogBlock);
    manager.register(BambooBlock);
    manager.register(BannerBlock);
    manager.register(SignBlock);
    manager.register(SlabBlock);
    manager.register(StairBlock);
    manager.register(ShortPlantBlock);
    manager.register(DryVegetationBlock);
    manager.register(LilyPadBlock);
    manager.register(SugarCaneBlock);
    manager.register(VineBlock);
    manager.register(TNTBlock);
    manager.register(BushBlock);
    manager.register(FlowerBlock);
    manager.register(PotatoBlock);
    manager.register(BeetrootBlock);
    manager.register(TorchFlowerBlock);
    manager.register(CarrotBlock);
    manager.register(SeaGrassBlock);
    manager.register(NetherWartBlock);
    manager.register(WheatBlock);
    manager.register(TorchBlock);
    manager.register(TrapDoorBlock);
    manager.register(MushroomPlantBlock);
    manager.register(FlowerbedBlock);
    manager.register(LeafLitterBlock);
    manager.register(WallBlock);
    manager.register(RootsBlock);
    manager.register(NetherPortalBlock);
    manager.register(TallPlantBlock);
    manager.register(NoteBlock);
    manager.register(PumpkinBlock);
    manager.register(CommandBlock);
    manager.register(ComposterBlock);
    manager.register(PressurePlateBlock);
    manager.register(WeightedPressurePlateBlock);
    manager.register(EndPortalBlock);
    manager.register(EndPortalFrameBlock);
    manager.register(CandleBlock);
    manager.register(SeaPickleBlock);
    manager.register(CakeBlock);
    manager.register(CandleCakeBlock);
    manager.register(SkullBlock);
    manager.register(ChiseledBookshelfBlock);
    manager.register(LecternBlock);
    manager.register(StemBlock);
    manager.register(AttachedStemBlock);
    manager.register(ChainBlock);
    manager.register(LanternBlock);
    manager.register(EndRodBlock);

    manager.register(FallingBlock);

    // Fire
    manager.register(SoulFireBlock);
    manager.register(FireBlock);

    // Redstone
    manager.register(ButtonBlock);
    manager.register(LeverBlock);
    manager.register(ObserverBlock);
    manager.register(TripwireBlock);
    manager.register(TripwireHookBlock);

    // Piston
    manager.register(PistonBlock);
    manager.register(PistonExtensionBlock);
    manager.register(PistonHeadBlock);

    manager.register(RedstoneBlock);
    manager.register(RedstoneLamp);
    manager.register(CopperBulbBlock);
    manager.register(RedstoneTorchBlock);
    manager.register(RedstoneWireBlock);
    manager.register(RepeaterBlock);
    manager.register(ComparatorBlock);
    manager.register(TargetBlock);
    manager.register(BarrelBlock);
    manager.register(HopperBlock);
    manager.register(ShulkerBoxBlock);
    manager.register(DropperBlock);
    manager.register(DispenserBlock);
    manager.register(LadderBlock);

    // Rails
    manager.register(RailBlock);
    manager.register(ActivatorRailBlock);
    manager.register(DetectorRailBlock);
    manager.register(PoweredRailBlock);

    // Fluids
    manager.register_fluid(FlowingWater);
    manager.register_fluid(FlowingLava);
    Arc::new(manager)
}

// ActionResult.java
#[derive(PartialEq)]
pub enum BlockActionResult {
    /// Action was successful | Same as SUCCESS in vanilla
    Success,
    /// Action was successful and we should swing the hand for the server | Same as `SUCCESS_SERVER` in vanilla
    SuccessServer,
    /// Block other actions from being executed | Same as CONSUME in vanilla
    Consume,
    /// Allow other actions to be executed, but indicate it failed | Same as FAIL in vanilla
    Fail,
    /// Allow other actions to be executed | Same as PASS in vanilla
    Pass,
    /// Use default action for the block: `normal_use` | Same as `PASS_TO_DEFAULT_BLOCK_ACTION` in vanilla
    PassToDefaultBlockAction,
}

impl BlockActionResult {
    #[must_use]
    pub fn consumes_action(&self) -> bool {
        matches!(self, Self::Consume | Self::Success | Self::SuccessServer)
    }
}

#[derive(Default)]
pub struct BlockRegistry {
    blocks: HashMap<&'static Block, Arc<dyn BlockBehaviour>>,
    fluids: HashMap<&'static Fluid, Arc<dyn FluidBehaviour>>,
}

#[async_trait]
impl BlockRegistryExt for BlockRegistry {
    fn can_place_at(
        &self,
        block: &pumpkin_data::Block,
        block_accessor: &dyn BlockAccessor,
        block_pos: &BlockPos,
        face: BlockDirection,
    ) -> bool {
        futures::executor::block_on(async move {
            self.can_place_at(
                None,
                None,
                block_accessor,
                None,
                block,
                block_pos,
                face,
                None,
            )
            .await
        })
    }
}

impl BlockRegistry {
    pub fn register<T: BlockBehaviour + BlockMetadata + 'static>(&mut self, block: T) {
        let names = block.names();
        let val = Arc::new(block);
        self.blocks.reserve(names.len());
        for i in names {
            self.blocks
                .insert(Block::from_name(i.as_str()).unwrap(), val.clone());
        }
    }

    pub fn register_fluid<T: FluidBehaviour + BlockMetadata + 'static>(&mut self, fluid: T) {
        let names = fluid.names();
        let val = Arc::new(fluid);
        self.fluids.reserve(names.len());
        for i in names {
            self.fluids
                .insert(fluid::get_fluid(i.as_str()).unwrap(), val.clone());
        }
    }

    pub async fn on_synced_block_event(
        &self,
        block: &Block,
        world: &Arc<World>,
        position: &BlockPos,
        r#type: u8,
        data: u8,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .on_synced_block_event(OnSyncedBlockEventArgs {
                    world,
                    block,
                    position,
                    r#type,
                    data,
                })
                .await;
        }
        false
    }

    pub async fn on_entity_collision(
        &self,
        block: &Block,
        world: &Arc<World>,
        entity: &dyn EntityBase,
        position: &BlockPos,
        state: &BlockState,
        server: &Server,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .on_entity_collision(OnEntityCollisionArgs {
                    server,
                    world,
                    block,
                    state,
                    position,
                    entity,
                })
                .await;
        }
    }

    pub async fn on_entity_collision_fluid(&self, fluid: &Fluid, entity: &dyn EntityBase) {
        let pumpkin_fluid = self.get_pumpkin_fluid(fluid);
        if let Some(pumpkin_fluid) = pumpkin_fluid {
            pumpkin_fluid.on_entity_collision(entity).await;
        }
    }

    pub async fn on_use(
        &self,
        block: &Block,
        player: &Player,
        position: &BlockPos,
        hit: &BlockHitResult<'_>,
        server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .normal_use(NormalUseArgs {
                    server,
                    world,
                    block,
                    position,
                    player,
                    hit,
                })
                .await;
        }
        BlockActionResult::Pass
    }

    pub async fn explode(&self, block: &Block, world: &Arc<World>, position: &BlockPos) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .explode(ExplodeArgs {
                    world,
                    block,
                    position,
                })
                .await;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn use_with_item(
        &self,
        block: &Block,
        player: &Player,
        position: &BlockPos,
        hit: &BlockHitResult<'_>,
        item_stack: &Arc<Mutex<ItemStack>>,
        server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .use_with_item(UseWithItemArgs {
                    server,
                    world,
                    block,
                    position,
                    player,
                    hit,
                    item_stack,
                })
                .await;
        }
        BlockActionResult::Pass
    }

    pub async fn use_with_item_fluid(
        &self,
        fluid: &Fluid,
        player: &Player,
        position: BlockPos,
        item: &Item,
        server: &Server,
        world: &Arc<World>,
    ) -> BlockActionResult {
        let pumpkin_fluid = self.get_pumpkin_fluid(fluid);
        if let Some(pumpkin_fluid) = pumpkin_fluid {
            return pumpkin_fluid
                .use_with_item(fluid, player, position, item, server, world)
                .await;
        }
        BlockActionResult::Pass
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn can_place_at(
        &self,
        server: Option<&Server>,
        world: Option<&World>,
        block_accessor: &dyn BlockAccessor,
        player: Option<&Player>,
        block: &Block,
        position: &BlockPos,
        direction: BlockDirection,
        use_item_on: Option<&SUseItemOn>,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .can_place_at(CanPlaceAtArgs {
                    server,
                    world,
                    block_accessor,
                    block,
                    position,
                    direction,
                    player,
                    use_item_on,
                })
                .await;
        }
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn can_update_at(
        &self,
        world: &World,
        block: &Block,
        state_id: BlockStateId,
        position: &BlockPos,
        direction: BlockDirection,
        use_item_on: &SUseItemOn,
        player: &Player,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .can_update_at(CanUpdateAtArgs {
                    world,
                    block,
                    state_id,
                    position,
                    direction,
                    player,
                    use_item_on,
                })
                .await;
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn on_place(
        &self,
        server: &Server,
        world: &World,
        player: &Player,
        block: &Block,
        position: &BlockPos,
        direction: BlockDirection,
        replacing: BlockIsReplacing,
        use_item_on: &SUseItemOn,
    ) -> BlockStateId {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .on_place(OnPlaceArgs {
                    server,
                    world,
                    block,
                    position,
                    direction,
                    player,
                    replacing,
                    use_item_on,
                })
                .await;
        }
        block.default_state.id
    }

    pub async fn player_placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: u16,
        position: &BlockPos,
        direction: BlockDirection,
        player: &Player,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .player_placed(PlayerPlacedArgs {
                    world,
                    block,
                    state_id,
                    position,
                    direction,
                    player,
                })
                .await;
        }
    }

    pub async fn on_placed(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        position: &BlockPos,
        old_state_id: BlockStateId,
        notify: bool,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .placed(PlacedArgs {
                    world,
                    block,
                    state_id,
                    old_state_id,
                    position,
                    notify,
                })
                .await;
        }
    }

    pub async fn on_placed_fluid(
        &self,
        world: &Arc<World>,
        fluid: &Fluid,
        state_id: BlockStateId,
        position: &BlockPos,
        old_state_id: BlockStateId,
        notify: bool,
    ) {
        let pumpkin_fluid = self.get_pumpkin_fluid(fluid);
        if let Some(pumpkin_fluid) = pumpkin_fluid {
            pumpkin_fluid
                .placed(world, fluid, state_id, position, old_state_id, notify)
                .await;
        }
    }

    pub async fn broken(
        &self,
        world: &Arc<World>,
        block: &Block,
        player: &Arc<Player>,
        position: &BlockPos,
        server: &Server,
        state: &BlockState,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .broken(BrokenArgs {
                    block,
                    player,
                    position,
                    server,
                    world,
                    state,
                })
                .await;
        }
    }

    pub async fn on_state_replaced(
        &self,
        world: &Arc<World>,
        block: &Block,
        position: &BlockPos,
        old_state_id: BlockStateId,
        moved: bool,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .on_state_replaced(OnStateReplacedArgs {
                    world,
                    block,
                    old_state_id,
                    position,
                    moved,
                })
                .await;
        }
    }

    /// Updates state of all neighbors of the block
    pub async fn post_process_state(
        &self,
        world: &Arc<World>,
        position: &BlockPos,
        block: &Block,
        flags: BlockFlags,
    ) {
        let state = world.get_block_state(position).await;
        for direction in BlockDirection::all() {
            let neighbor_pos = position.offset(direction.to_offset());
            let neighbor_state = world.get_block_state(&neighbor_pos).await;
            let pumpkin_block = self.get_pumpkin_block(block);
            if let Some(pumpkin_block) = pumpkin_block {
                let new_state = pumpkin_block
                    .get_state_for_neighbor_update(GetStateForNeighborUpdateArgs {
                        world,
                        block,
                        state_id: state.id,
                        position,
                        direction: direction.opposite(),
                        neighbor_position: &neighbor_pos,
                        neighbor_state_id: neighbor_state.id,
                    })
                    .await;
                world.set_block_state(&neighbor_pos, new_state, flags).await;
            }
        }
    }

    pub async fn prepare(
        &self,
        world: &Arc<World>,
        position: &BlockPos,
        block: &Block,
        state_id: BlockStateId,
        flags: BlockFlags,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .prepare(PrepareArgs {
                    world,
                    block,
                    state_id,
                    position,
                    flags,
                })
                .await;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn get_state_for_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        state_id: BlockStateId,
        position: &BlockPos,
        direction: BlockDirection,
        neighbor_location: &BlockPos,
        neighbor_state_id: BlockStateId,
    ) -> BlockStateId {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .get_state_for_neighbor_update(GetStateForNeighborUpdateArgs {
                    world,
                    block,
                    state_id,
                    position,
                    direction,
                    neighbor_position: neighbor_location,
                    neighbor_state_id,
                })
                .await;
        }
        state_id
    }

    pub async fn update_neighbors(
        &self,
        world: &Arc<World>,
        position: &BlockPos,
        _block: &Block,
        flags: BlockFlags,
    ) {
        for direction in BlockDirection::abstract_block_update_order() {
            let pos = position.offset(direction.to_offset());

            Box::pin(world.replace_with_state_for_neighbor_update(
                &pos,
                direction.opposite(),
                flags,
            ))
            .await;
        }
    }

    pub async fn on_neighbor_update(
        &self,
        world: &Arc<World>,
        block: &Block,
        position: &BlockPos,
        source_block: &Block,
        notify: bool,
    ) {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            pumpkin_block
                .on_neighbor_update(OnNeighborUpdateArgs {
                    world,
                    block,
                    position,
                    source_block,
                    notify,
                })
                .await;
        }
    }

    #[must_use]
    pub fn get_pumpkin_block(&self, block: &Block) -> Option<&Arc<dyn BlockBehaviour>> {
        self.blocks.get(block)
    }

    #[must_use]
    pub fn get_pumpkin_fluid(&self, fluid: &Fluid) -> Option<&Arc<dyn FluidBehaviour>> {
        self.fluids.get(fluid)
    }

    pub async fn emits_redstone_power(
        &self,
        block: &Block,
        state: &BlockState,
        direction: BlockDirection,
    ) -> bool {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .emits_redstone_power(EmitsRedstonePowerArgs {
                    block,
                    state,
                    direction,
                })
                .await;
        }
        false
    }

    pub async fn get_weak_redstone_power(
        &self,
        block: &Block,
        world: &World,
        position: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .get_weak_redstone_power(GetRedstonePowerArgs {
                    world,
                    block,
                    state,
                    position,
                    direction,
                })
                .await;
        }
        0
    }

    pub async fn get_strong_redstone_power(
        &self,
        block: &Block,
        world: &World,
        position: &BlockPos,
        state: &BlockState,
        direction: BlockDirection,
    ) -> u8 {
        let pumpkin_block = self.get_pumpkin_block(block);
        if let Some(pumpkin_block) = pumpkin_block {
            return pumpkin_block
                .get_strong_redstone_power(GetRedstonePowerArgs {
                    world,
                    block,
                    state,
                    position,
                    direction,
                })
                .await;
        }
        0
    }
}
