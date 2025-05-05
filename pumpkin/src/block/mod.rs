use blocks::cactus::CactusBlock;
use blocks::dirt_path::DirtPathBlock;
use blocks::doors::DoorBlock;
use blocks::farmland::FarmLandBlock;
use blocks::fence_gates::FenceGateBlock;
use blocks::fences::FenceBlock;
use blocks::fire::fire::FireBlock;
use blocks::fire::soul_fire::SoulFireBlock;
use blocks::glass_panes::GlassPaneBlock;
use blocks::iron_bars::IronBarsBlock;
use blocks::logs::LogBlock;
use blocks::nether_portal::NetherPortalBlock;
use blocks::redstone::buttons::ButtonBlock;
use blocks::redstone::observer::ObserverBlock;
use blocks::redstone::piston::PistonBlock;
use blocks::redstone::rails::activator_rail::ActivatorRailBlock;
use blocks::redstone::rails::detector_rail::DetectorRailBlock;
use blocks::redstone::rails::powered_rail::PoweredRailBlock;
use blocks::redstone::rails::rail::RailBlock;
use blocks::redstone::redstone_block::RedstoneBlock;
use blocks::redstone::redstone_lamp::RedstoneLamp;
use blocks::redstone::redstone_torch::RedstoneTorchBlock;
use blocks::redstone::redstone_wire::RedstoneWireBlock;
use blocks::redstone::repeater::RepeaterBlock;
use blocks::redstone::target_block::TargetBlock;
use blocks::signs::SignBlock;
use blocks::slabs::SlabBlock;
use blocks::stairs::StairBlock;
use blocks::sugar_cane::SugarCaneBlock;
use blocks::torches::TorchBlock;
use blocks::walls::WallBlock;
use blocks::{
    chest::ChestBlock, furnace::FurnaceBlock, redstone::lever::LeverBlock, tnt::TNTBlock,
};
use fluids::lava::FlowingLava;
use fluids::water::FlowingWater;
use pumpkin_data::block_properties::Integer0To15;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::tag::Tagable;
use pumpkin_data::{Block, BlockState};
use pumpkin_util::loot_table::{
    AlternativeEntry, ItemEntry, LootCondition, LootPool, LootPoolEntryTypes, LootTable,
};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::BlockStateId;
use pumpkin_world::item::ItemStack;
use rand::Rng;

use crate::block::registry::BlockRegistry;
use crate::entity::item::ItemEntity;
use crate::world::World;
use crate::{block::blocks::crafting_table::CraftingTableBlock, entity::player::Player};
use crate::{block::blocks::jukebox::JukeboxBlock, entity::experience_orb::ExperienceOrbEntity};
use std::sync::Arc;

pub(crate) mod blocks;
mod fluids;
pub mod pumpkin_block;
pub mod pumpkin_fluid;
pub mod registry;

#[must_use]
pub fn default_registry() -> Arc<BlockRegistry> {
    let mut manager = BlockRegistry::default();

    // Blocks
    manager.register(CactusBlock);
    manager.register(ChestBlock);
    manager.register(CraftingTableBlock);
    manager.register(DirtPathBlock);
    manager.register(DoorBlock);
    manager.register(FarmLandBlock);
    manager.register(FenceGateBlock);
    manager.register(FenceBlock);
    manager.register(FurnaceBlock);
    manager.register(GlassPaneBlock);
    manager.register(IronBarsBlock);
    manager.register(JukeboxBlock);
    manager.register(LogBlock);
    manager.register(SignBlock);
    manager.register(SlabBlock);
    manager.register(StairBlock);
    manager.register(SugarCaneBlock);
    manager.register(TNTBlock);
    manager.register(TorchBlock);
    manager.register(WallBlock);
    manager.register(NetherPortalBlock);

    // Fire
    manager.register(SoulFireBlock);
    manager.register(FireBlock);

    // Redstone
    manager.register(ButtonBlock);
    manager.register(LeverBlock);
    manager.register(ObserverBlock);
    manager.register(PistonBlock);
    manager.register(RedstoneBlock);
    manager.register(RedstoneLamp);
    manager.register(RedstoneTorchBlock);
    manager.register(RedstoneWireBlock);
    manager.register(RepeaterBlock);
    manager.register(TargetBlock);

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

pub async fn drop_loot(
    world: &Arc<World>,
    block: &Block,
    pos: &BlockPos,
    experience: bool,
    state_id: BlockStateId,
) {
    if let Some(table) = &block.loot_table {
        let props =
            Block::properties(block, state_id).map_or_else(Vec::new, |props| props.to_props());
        let loot = table.get_loot(
            &props
                .iter()
                .map(|(key, value)| (key.as_str(), value.as_str()))
                .collect::<Vec<_>>(),
        );

        if block.is_tagged_with("#minecraft:slabs").unwrap()
            && SlabBlock::drop_double_loot(block, state_id)
        {
            for mut stack in loot {
                stack.item_count *= 2;
                drop_stack(world, pos, stack).await;
            }
        } else {
            for stack in loot {
                drop_stack(world, pos, stack).await;
            }
        }
    }

    if experience {
        if let Some(experience) = &block.experience {
            let amount = experience.experience.get();
            // TODO: Silk touch gives no exp
            if amount > 0 {
                ExperienceOrbEntity::spawn(world, pos.to_f64(), amount as u32).await;
            }
        }
    }
}

#[allow(dead_code)]
async fn drop_stack(world: &Arc<World>, pos: &BlockPos, stack: ItemStack) {
    let height = EntityType::ITEM.dimension[1] / 2.0;
    let pos = Vector3::new(
        f64::from(pos.0.x) + 0.5 + rand::thread_rng().gen_range(-0.25..0.25),
        f64::from(pos.0.y) + 0.5 + rand::thread_rng().gen_range(-0.25..0.25) - f64::from(height),
        f64::from(pos.0.z) + 0.5 + rand::thread_rng().gen_range(-0.25..0.25),
    );

    let entity = world.create_entity(pos, EntityType::ITEM);
    let item_entity =
        Arc::new(ItemEntity::new(entity, stack.item.id, u32::from(stack.item_count)).await);
    world.spawn_entity(item_entity.clone()).await;
    item_entity.send_meta_packet().await;
}

pub async fn calc_block_breaking(player: &Player, state: &BlockState, block_name: &str) -> f32 {
    let hardness = state.hardness;
    #[expect(clippy::float_cmp)]
    if hardness == -1.0 {
        // unbreakable
        return 0.0;
    }
    let i = if player.can_harvest(state, block_name).await {
        30
    } else {
        100
    };

    player.get_mining_speed(block_name).await / hardness / i as f32
}

// These traits need to be implemented here so they have access to pumpkin_data

trait LootTableExt {
    fn get_loot(&self, block_props: &[(&str, &str)]) -> Vec<ItemStack>;
}

impl LootTableExt for LootTable {
    fn get_loot(&self, block_props: &[(&str, &str)]) -> Vec<ItemStack> {
        let mut items = vec![];
        if let Some(pools) = &self.pools {
            for i in 0..pools.len() {
                let pool = &pools[i];
                items.extend_from_slice(&pool.get_loot(block_props));
            }
        }
        items
    }
}

trait LootPoolExt {
    fn get_loot(&self, block_props: &[(&str, &str)]) -> Vec<ItemStack>;
}

impl LootPoolExt for LootPool {
    fn get_loot(&self, block_props: &[(&str, &str)]) -> Vec<ItemStack> {
        let i = self.rolls.round() as i32 + self.bonus_rolls.floor() as i32; // TODO: mul by luck
        let mut items = vec![];
        for _ in 0..i {
            for entry_idx in 0..self.entries.len() {
                let entry = &self.entries[entry_idx];
                if let Some(conditions) = &entry.conditions {
                    if !conditions.iter().all(|c| c.test(block_props)) {
                        continue;
                    }
                }
                items.extend_from_slice(&entry.content.get_items(block_props));
            }
        }
        items
    }
}

trait ItemEntryExt {
    fn get_items(&self) -> Vec<ItemStack>;
}

impl ItemEntryExt for ItemEntry {
    fn get_items(&self) -> Vec<ItemStack> {
        let item = &self.name.replace("minecraft:", "");
        vec![ItemStack::new(1, Item::from_registry_key(item).unwrap())]
    }
}

trait AlternativeEntryExt {
    fn get_items(&self, block_props: &[(&str, &str)]) -> Vec<ItemStack>;
}

impl AlternativeEntryExt for AlternativeEntry {
    fn get_items(&self, block_props: &[(&str, &str)]) -> Vec<ItemStack> {
        let mut items = vec![];
        for i in 0..self.children.len() {
            let child = &self.children[i];
            if let Some(conditions) = &child.conditions {
                if !conditions.iter().all(|c| c.test(block_props)) {
                    continue;
                }
            }
            items.extend_from_slice(&child.content.get_items(block_props));
        }
        items
    }
}

trait LootPoolEntryTypesExt {
    fn get_items(&self, block_props: &[(&str, &str)]) -> Vec<ItemStack>;
}

impl LootPoolEntryTypesExt for LootPoolEntryTypes {
    fn get_items(&self, block_props: &[(&str, &str)]) -> Vec<ItemStack> {
        match self {
            Self::Empty => todo!(),
            Self::Item(item_entry) => item_entry.get_items(),
            Self::LootTable => todo!(),
            Self::Dynamic => todo!(),
            Self::Tag => todo!(),
            Self::Alternatives(alternative) => alternative.get_items(block_props),
            Self::Sequence => todo!(),
            Self::Group => todo!(),
        }
    }
}

trait LootConditionExt {
    fn test(&self, block_props: &[(&str, &str)]) -> bool;
}

impl LootConditionExt for LootCondition {
    // TODO: This is trash. Make this right
    fn test(&self, block_props: &[(&str, &str)]) -> bool {
        match self {
            Self::SurvivesExplosion => true,
            Self::BlockStateProperty { properties } => properties
                .iter()
                .all(|(key, value)| block_props.iter().any(|(k, v)| k == key && v == value)),
            _ => false,
        }
    }
}

#[derive(PartialEq)]
pub enum BlockIsReplacing {
    Itself(BlockStateId),
    Water(Integer0To15),
    Other,
}

impl BlockIsReplacing {
    #[must_use]
    /// Returns true if the block was a water source block.
    pub fn water_source(&self) -> bool {
        match self {
            // Level 0 means the water is a source block
            Self::Water(level) => *level == Integer0To15::L0,
            _ => false,
        }
    }
}
