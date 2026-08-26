use pumpkin_data::block_properties::NoteblockInstrument as InternalNoteblockInstrument;
use pumpkin_data::block_state::PistonBehavior;
use pumpkin_data::{BlockDirection as InternalBlockDirection, BlockId, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::chunk::ChunkHeightmapType;
use pumpkin_world::chunk::io::Dirtiable;
use pumpkin_world::world::BlockFlags;
use std::sync::Arc;
use wasmtime::component::Resource;

use crate::block::entities::banner::BannerBlockEntity as InternalBannerBlockEntity;
use crate::block::entities::barrel::BarrelBlockEntity as InternalBarrelBlockEntity;
use crate::block::entities::beacon::BeaconBlockEntity as InternalBeaconBlockEntity;
use crate::block::entities::bed::BedBlockEntity as InternalBedBlockEntity;
use crate::block::entities::beehive::BeehiveBlockEntity as InternalBeehiveBlockEntity;
use crate::block::entities::bell::BellBlockEntity as InternalBellBlockEntity;
use crate::block::entities::blasting_furnace::BlastingFurnaceBlockEntity as InternalBlastingFurnaceBlockEntity;
use crate::block::entities::brewing_stand::BrewingStandBlockEntity as InternalBrewingStandBlockEntity;
use crate::block::entities::brushable_block::BrushableBlockBlockEntity as InternalBrushableBlockBlockEntity;
use crate::block::entities::calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity as InternalCalibratedSculkSensorBlockEntity;
use crate::block::entities::campfire::CampfireBlockEntity as InternalCampfireBlockEntity;
use crate::block::entities::chest::ChestBlockEntity as InternalChestBlockEntity;
use crate::block::entities::chiseled_bookshelf::ChiseledBookshelfBlockEntity as InternalChiseledBookshelfBlockEntity;
use crate::block::entities::command_block::CommandBlockEntity as InternalCommandBlockEntity;
use crate::block::entities::comparator::ComparatorBlockEntity as InternalComparatorBlockEntity;
use crate::block::entities::conduit::ConduitBlockEntity as InternalConduitBlockEntity;
use crate::block::entities::copper_golem_statue::CopperGolemStatueBlockEntity as InternalCopperGolemStatueBlockEntity;
use crate::block::entities::crafter::CrafterBlockEntity as InternalCrafterBlockEntity;
use crate::block::entities::creaking_heart::CreakingHeartBlockEntity as InternalCreakingHeartBlockEntity;
use crate::block::entities::daylight_detector::DaylightDetectorBlockEntity as InternalDaylightDetectorBlockEntity;
use crate::block::entities::decorated_pot::DecoratedPotBlockEntity as InternalDecoratedPotBlockEntity;
use crate::block::entities::dispenser::DispenserBlockEntity as InternalDispenserBlockEntity;
use crate::block::entities::dropper::DropperBlockEntity as InternalDropperBlockEntity;
use crate::block::entities::enchanting_table::EnchantingTableBlockEntity as InternalEnchantingTableBlockEntity;
use crate::block::entities::end_gateway::EndGatewayBlockEntity as InternalEndGatewayBlockEntity;
use crate::block::entities::end_portal::EndPortalBlockEntity as InternalEndPortalBlockEntity;
use crate::block::entities::ender_chest::EnderChestBlockEntity as InternalEnderChestBlockEntity;
use crate::block::entities::furnace::FurnaceBlockEntity as InternalFurnaceBlockEntity;
use crate::block::entities::hanging_sign::HangingSignBlockEntity as InternalHangingSignBlockEntity;
use crate::block::entities::hopper::HopperBlockEntity as InternalHopperBlockEntity;
use crate::block::entities::jigsaw_block::JigsawBlockEntity as InternalJigsawBlockEntity;
use crate::block::entities::jukebox::JukeboxBlockEntity as InternalJukeboxBlockEntity;
use crate::block::entities::lectern::LecternBlockEntity as InternalLecternBlockEntity;
use crate::block::entities::map::MapBlockEntity as InternalMapBlockEntity;
use crate::block::entities::mob_spawner::MobSpawnerBlockEntity as InternalMobSpawnerBlockEntity;
use crate::block::entities::piston::PistonBlockEntity as InternalPistonBlockEntity;
use crate::block::entities::potent_sulfur::PotentSulfurBlockEntity as InternalPotentSulfurBlockEntity;
use crate::block::entities::sculk_catalyst::SculkCatalystBlockEntity as InternalSculkCatalystBlockEntity;
use crate::block::entities::sculk_sensor::SculkSensorBlockEntity as InternalSculkSensorBlockEntity;
use crate::block::entities::sculk_shrieker::SculkShriekerBlockEntity as InternalSculkShriekerBlockEntity;
use crate::block::entities::shelf::ShelfBlockEntity as InternalShelfBlockEntity;
use crate::block::entities::shulker_box::ShulkerBoxBlockEntity as InternalShulkerBoxBlockEntity;
use crate::block::entities::sign::SignBlockEntity as InternalSignBlockEntity;
use crate::block::entities::skull::SkullBlockEntity as InternalSkullBlockEntity;
use crate::block::entities::smoker::SmokerBlockEntity as InternalSmokerBlockEntity;
use crate::block::entities::structure_block::StructureBlockBlockEntity as InternalStructureBlockBlockEntity;
use crate::block::entities::test_block::TestBlockBlockEntity as InternalTestBlockBlockEntity;
use crate::block::entities::test_instance_block::TestInstanceBlockBlockEntity as InternalTestInstanceBlockBlockEntity;
use crate::block::entities::trapped_chest::TrappedChestBlockEntity as InternalTrappedChestBlockEntity;
use crate::block::entities::trial_spawner::TrialSpawnerBlockEntity as InternalTrialSpawnerBlockEntity;
use crate::block::entities::vault::VaultBlockEntity as InternalVaultBlockEntity;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::common::Position as WitPosition;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::game_rules::{
    GameRule as WitGameRule, GameRuleValue as WitGameRuleValue,
};
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::world::{
    Block as WitBlock, BlockDirection as WitBlockDirection, BlockEntity, BlockEntityType,
    BlockFlags as WitBlockFlags, BlockPos as WitBlockPos, BlockState as WitBlockState,
    BlockStateInfo as WitBlockStateInfo, BoundingBox as WitBoundingBox, Chunk as WitChunk,
    Flammable as WitFlammable, NoteblockInstrument as WitNoteblockInstrument,
    PistonBehavior as WitPistonBehavior, RayTraceBlockResult as WitRayTraceBlockResult,
    RayTraceEntityResult as WitRayTraceEntityResult, WorldBorder as WitWorldBorder,
    WorldSpawnLocation as WitWorldSpawnLocation,
};
use crate::plugin::loader::wasm::wasm_host::{
    state::{
        ChunkResource, PluginHostState, TextComponentResource, WorldBorderResource, WorldResource,
    },
    wit::v0_1::pumpkin::{self, plugin::world::World},
};
use crate::world::explosion::ExplosionInteraction;
use pumpkin_data::game_rules::{GameRule, GameRuleValue};

pub(crate) fn from_wit_game_rule(rule: WitGameRule) -> GameRule {
    // SAFETY: WIT GameRule and pumpkin_data::game_rules::GameRule have identical variant order
    unsafe { std::mem::transmute::<u8, GameRule>(rule as u8) }
}

pub(crate) fn to_wit_game_rule_value(value: &GameRuleValue<i64, bool>) -> WitGameRuleValue {
    match *value {
        GameRuleValue::Int(v) => {
            WitGameRuleValue::Int(v.clamp(i32::MIN as i64, i32::MAX as i64) as i32)
        }
        GameRuleValue::Bool(v) => WitGameRuleValue::Bool(v),
    }
}

pub(crate) const fn from_wit_game_rule_value(value: WitGameRuleValue) -> GameRuleValue<i64, bool> {
    match value {
        WitGameRuleValue::Int(v) => GameRuleValue::Int(v as i64),
        WitGameRuleValue::Bool(v) => GameRuleValue::Bool(v),
    }
}

pub(crate) const fn to_wasm_block_direction(dir: InternalBlockDirection) -> WitBlockDirection {
    match dir {
        InternalBlockDirection::Down => WitBlockDirection::Down,
        InternalBlockDirection::Up => WitBlockDirection::Up,
        InternalBlockDirection::North => WitBlockDirection::North,
        InternalBlockDirection::South => WitBlockDirection::South,
        InternalBlockDirection::West => WitBlockDirection::West,
        InternalBlockDirection::East => WitBlockDirection::East,
    }
}

pub(crate) const fn to_wit_noteblock_instrument(
    instr: InternalNoteblockInstrument,
) -> WitNoteblockInstrument {
    match instr {
        InternalNoteblockInstrument::Harp => WitNoteblockInstrument::Harp,
        InternalNoteblockInstrument::Basedrum => WitNoteblockInstrument::Basedrum,
        InternalNoteblockInstrument::Snare => WitNoteblockInstrument::Snare,
        InternalNoteblockInstrument::Hat => WitNoteblockInstrument::Hat,
        InternalNoteblockInstrument::Bass => WitNoteblockInstrument::Bass,
        InternalNoteblockInstrument::Flute => WitNoteblockInstrument::Flute,
        InternalNoteblockInstrument::Bell => WitNoteblockInstrument::Bell,
        InternalNoteblockInstrument::Guitar => WitNoteblockInstrument::Guitar,
        InternalNoteblockInstrument::Chime => WitNoteblockInstrument::Chime,
        InternalNoteblockInstrument::Xylophone => WitNoteblockInstrument::Xylophone,
        InternalNoteblockInstrument::IronXylophone => WitNoteblockInstrument::IronXylophone,
        InternalNoteblockInstrument::CowBell => WitNoteblockInstrument::CowBell,
        InternalNoteblockInstrument::Didgeridoo => WitNoteblockInstrument::Didgeridoo,
        InternalNoteblockInstrument::Bit => WitNoteblockInstrument::Bit,
        InternalNoteblockInstrument::Banjo => WitNoteblockInstrument::Banjo,
        InternalNoteblockInstrument::Pling => WitNoteblockInstrument::Pling,
        InternalNoteblockInstrument::Trumpet => WitNoteblockInstrument::Trumpet,
        InternalNoteblockInstrument::TrumpetExposed => WitNoteblockInstrument::TrumpetExposed,
        InternalNoteblockInstrument::TrumpetOxidized => WitNoteblockInstrument::TrumpetOxidized,
        InternalNoteblockInstrument::TrumpetWeathered => WitNoteblockInstrument::TrumpetWeathered,
        InternalNoteblockInstrument::Zombie => WitNoteblockInstrument::Zombie,
        InternalNoteblockInstrument::Skeleton => WitNoteblockInstrument::Skeleton,
        InternalNoteblockInstrument::Creeper => WitNoteblockInstrument::Creeper,
        InternalNoteblockInstrument::Dragon => WitNoteblockInstrument::Dragon,
        InternalNoteblockInstrument::WitherSkeleton => WitNoteblockInstrument::WitherSkeleton,
        InternalNoteblockInstrument::Piglin => WitNoteblockInstrument::Piglin,
        InternalNoteblockInstrument::CustomHead => WitNoteblockInstrument::CustomHead,
    }
}

pub(crate) const fn to_wit_bounding_box(
    bb: pumpkin_util::math::boundingbox::BoundingBox,
) -> WitBoundingBox {
    WitBoundingBox {
        min: (bb.min.x, bb.min.y, bb.min.z),
        max: (bb.max.x, bb.max.y, bb.max.z),
    }
}

pub(crate) fn to_wit_block(block: &pumpkin_data::Block) -> WitBlock {
    WitBlock {
        id: block.id.as_u16(),
        name: block.name.to_string(),
        hardness: block.hardness,
        blast_resistance: block.blast_resistance,
        map_color: block.map_color,
        slipperiness: block.slipperiness,
        velocity_multiplier: block.velocity_multiplier,
        jump_velocity_multiplier: block.jump_velocity_multiplier,
        item_id: block.item_id,
        default_state_id: block.default_state.id.as_u16(),
        state_ids: block.states.iter().map(|s| s.id.as_u16()).collect(),
        is_solid: block.is_solid(),
        is_air: block.is_air(),
        is_flammable: block.flammable.is_some(),
        flammable: block.flammable.as_ref().map(|f| WitFlammable {
            spread_chance: f.spread_chance,
            burn_chance: f.burn_chance,
        }),
    }
}

pub(crate) fn to_wit_block_state(
    state: &pumpkin_data::BlockState,
    pos: Option<&BlockPos>,
) -> WitBlockState {
    let dummy_pos = BlockPos::new(0, 0, 0);
    let internal_pos = pos.unwrap_or(&dummy_pos);
    let block = pumpkin_data::Block::from_state_id(state.id);
    let properties = block
        .properties(state.id)
        .map(|p| {
            p.to_props()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect()
        })
        .unwrap_or_default();

    WitBlockState {
        id: state.id.as_u16(),
        block_id: block.id.as_u16(),
        block_name: block.name.to_string(),
        luminance: state.luminance,
        opacity: state.opacity,
        hardness: state.hardness,
        is_air: state.is_air(),
        is_liquid: state.is_liquid(),
        is_solid: state.is_solid(),
        is_full_cube: state.is_full_cube(),
        has_random_ticks: state.has_random_ticks(),
        piston_behavior: match state.piston_behavior {
            PistonBehavior::Normal => WitPistonBehavior::Normal,
            PistonBehavior::Destroy => WitPistonBehavior::Destroy,
            PistonBehavior::Block => WitPistonBehavior::Block,
            PistonBehavior::Ignore => WitPistonBehavior::Ignore,
            PistonBehavior::PushOnly => WitPistonBehavior::PushOnly,
        },
        burnable: state.burnable(),
        tool_required: state.tool_required(),
        sided_transparency: state.sided_transparency(),
        replaceable: state.replaceable(),
        is_solid_block: state.is_solid_block(),
        block_entity_type: state.block_entity_type,
        instrument: to_wit_noteblock_instrument(state.instrument),
        collision_shapes: state
            .get_block_collision_shapes_at(internal_pos)
            .map(to_wit_bounding_box)
            .collect(),
        outline_shapes: state
            .get_block_outline_shapes_at(internal_pos)
            .map(to_wit_bounding_box)
            .collect(),
        down_side_solid: state.is_side_solid(InternalBlockDirection::Down),
        up_side_solid: state.is_side_solid(InternalBlockDirection::Up),
        north_side_solid: state.is_side_solid(InternalBlockDirection::North),
        south_side_solid: state.is_side_solid(InternalBlockDirection::South),
        west_side_solid: state.is_side_solid(InternalBlockDirection::West),
        east_side_solid: state.is_side_solid(InternalBlockDirection::East),
        down_center_solid: state.is_center_solid(InternalBlockDirection::Down),
        up_center_solid: state.is_center_solid(InternalBlockDirection::Up),
        map_color: block.map_color,
        properties,
    }
}

// --- Trapping Helpers ---
impl PluginHostState {
    pub(crate) fn get_world_res(&self, res: &Resource<World>) -> wasmtime::Result<&WorldResource> {
        self.resource_table
            .get::<WorldResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }

    fn get_chunk_res(&self, res: &Resource<WitChunk>) -> wasmtime::Result<&ChunkResource> {
        self.resource_table
            .get::<ChunkResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }

    fn get_world_border_res(
        &self,
        res: &Resource<WitWorldBorder>,
    ) -> wasmtime::Result<&WorldBorderResource> {
        self.resource_table
            .get::<WorldBorderResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }

    pub(crate) fn get_text_provider(
        &self,
        res: &Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<pumpkin_util::text::TextComponent> {
        Ok(self
            .resource_table
            .get::<TextComponentResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)?
            .provider
            .clone())
    }

    fn get_wit_biome(
        biome: &pumpkin_data::biome::Biome,
    ) -> wasmtime::Result<pumpkin::plugin::biomes::Biome> {
        let mut names: Vec<String> = serde_json::from_str::<
            std::collections::BTreeMap<String, serde_json::Value>,
        >(&std::fs::read_to_string("assets/biome.json")?)?
        .keys()
        .cloned()
        .collect();
        names.sort();

        let index = names
            .iter()
            .position(|n| n.strip_prefix("minecraft:").unwrap_or(n) == biome.registry_id)
            .ok_or_else(|| wasmtime::Error::msg(format!("Unknown biome: {}", biome.registry_id)))?;

        // SAFETY: The WIT enum is generated from the sorted keys of assets/biome.json.
        Ok(unsafe { std::mem::transmute::<u8, pumpkin::plugin::biomes::Biome>(index as u8) })
    }

    fn get_wit_block_entity(
        &mut self,
        block_entity: Arc<dyn crate::block::entities::BlockEntity>,
    ) -> wasmtime::Result<Option<BlockEntityType>> {
        let be = block_entity;
        macro_rules! match_be {
            ($( $internal_type:ty => $variant:ident ),* $(,)?) => {
                $(
                    if be.as_any().downcast_ref::<$internal_type>().is_some() {
                        let res: Resource<BlockEntity> = self.add_block_entity(be)?;
                        return Ok(Some(BlockEntityType::$variant(Resource::new_own(res.rep()))));
                    }
                )*
            };
        }

        match_be! {
            InternalCommandBlockEntity => CommandBlockEntity,
            InternalSignBlockEntity => SignBlockEntity,
            InternalHangingSignBlockEntity => HangingSignBlockEntity,
            InternalJukeboxBlockEntity => JukeboxBlockEntity,
            InternalChestBlockEntity => ChestBlockEntity,
            InternalTrappedChestBlockEntity => TrappedChestBlockEntity,
            InternalMobSpawnerBlockEntity => MobSpawnerBlockEntity,
            InternalMapBlockEntity => MapBlockEntity,
            InternalBannerBlockEntity => BannerBlockEntity,
            InternalBarrelBlockEntity => BarrelBlockEntity,
            InternalBeaconBlockEntity => BeaconBlockEntity,
            InternalBedBlockEntity => BedBlockEntity,
            InternalBeehiveBlockEntity => BeehiveBlockEntity,
            InternalBellBlockEntity => BellBlockEntity,
            InternalBlastingFurnaceBlockEntity => BlastingFurnaceBlockEntity,
            InternalBrewingStandBlockEntity => BrewingStandBlockEntity,
            InternalBrushableBlockBlockEntity => BrushableBlockBlockEntity,
            InternalCalibratedSculkSensorBlockEntity => CalibratedSculkSensorBlockEntity,
            InternalCampfireBlockEntity => CampfireBlockEntity,
            InternalChiseledBookshelfBlockEntity => ChiseledBookshelfBlockEntity,
            InternalComparatorBlockEntity => ComparatorBlockEntity,
            InternalConduitBlockEntity => ConduitBlockEntity,
            InternalCopperGolemStatueBlockEntity => CopperGolemStatueBlockEntity,
            InternalCrafterBlockEntity => CrafterBlockEntity,
            InternalCreakingHeartBlockEntity => CreakingHeartBlockEntity,
            InternalDaylightDetectorBlockEntity => DaylightDetectorBlockEntity,
            InternalDecoratedPotBlockEntity => DecoratedPotBlockEntity,
            InternalDispenserBlockEntity => DispenserBlockEntity,
            InternalDropperBlockEntity => DropperBlockEntity,
            InternalEnchantingTableBlockEntity => EnchantingTableBlockEntity,
            InternalEndGatewayBlockEntity => EndGatewayBlockEntity,
            InternalEndPortalBlockEntity => EndPortalBlockEntity,
            InternalEnderChestBlockEntity => EnderChestBlockEntity,
            InternalFurnaceBlockEntity => FurnaceBlockEntity,
            InternalHopperBlockEntity => HopperBlockEntity,
            InternalJigsawBlockEntity => JigsawBlockEntity,
            InternalLecternBlockEntity => LecternBlockEntity,
            InternalPistonBlockEntity => PistonBlockEntity,
            InternalPotentSulfurBlockEntity => PotentSulfurBlockEntity,
            InternalSculkCatalystBlockEntity => SculkCatalystBlockEntity,
            InternalSculkSensorBlockEntity => SculkSensorBlockEntity,
            InternalSculkShriekerBlockEntity => SculkShriekerBlockEntity,
            InternalShelfBlockEntity => ShelfBlockEntity,
            InternalShulkerBoxBlockEntity => ShulkerBoxBlockEntity,
            InternalSkullBlockEntity => SkullBlockEntity,
            InternalSmokerBlockEntity => SmokerBlockEntity,
            InternalStructureBlockBlockEntity => StructureBlockBlockEntity,
            InternalTestBlockBlockEntity => TestBlockBlockEntity,
            InternalTestInstanceBlockBlockEntity => TestInstanceBlockBlockEntity,
            InternalTrialSpawnerBlockEntity => TrialSpawnerBlockEntity,
            InternalVaultBlockEntity => VaultBlockEntity,
        }

        Ok(None)
    }
}

impl pumpkin::plugin::world::Host for PluginHostState {
    async fn resolve_block_state(
        &mut self,
        name: String,
        properties: Vec<(String, String)>,
    ) -> wasmtime::Result<Option<u16>> {
        let Some(block) = pumpkin_data::Block::from_name(&name) else {
            return Ok(None);
        };

        if properties.is_empty() {
            return Ok(Some(block.default_state.id.as_u16()));
        }

        let props: Vec<(&str, &str)> = properties
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        // from_properties/from_value panics on unknown property values,
        // so catch panics to avoid crashing on schematics from other MC versions.
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let block_props = block.from_properties(&props);
            block_props.to_state_id(block).as_u16()
        }));
        Ok(result.ok())
    }

    async fn block_state_to_info(
        &mut self,
        state_id: u16,
    ) -> wasmtime::Result<Option<WitBlockStateInfo>> {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let bsid = BlockStateId::new_or_air(state_id);
            let block = pumpkin_data::Block::from_state_id(bsid);
            let name = format!("minecraft:{}", block.name);
            let properties = block
                .properties(bsid)
                .map(|p| {
                    p.to_props()
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect()
                })
                .unwrap_or_default();
            WitBlockStateInfo { name, properties }
        }));
        Ok(result.ok())
    }

    async fn get_block_by_id(&mut self, id: u16) -> wasmtime::Result<Option<WitBlock>> {
        let block_id = BlockId::new(id);
        Ok(block_id.map(|id| to_wit_block(pumpkin_data::Block::from_id(id))))
    }

    async fn get_block_by_name(&mut self, name: String) -> wasmtime::Result<Option<WitBlock>> {
        Ok(pumpkin_data::Block::from_name(&name).map(to_wit_block))
    }

    async fn get_all_blocks(&mut self) -> wasmtime::Result<Vec<WitBlock>> {
        let mut blocks = Vec::with_capacity(BlockId::COUNT as usize);
        for raw_id in 0..BlockId::COUNT {
            if let Some(id) = BlockId::new(raw_id) {
                blocks.push(to_wit_block(pumpkin_data::Block::from_id(id)));
            }
        }
        Ok(blocks)
    }

    async fn get_all_block_names(&mut self) -> wasmtime::Result<Vec<String>> {
        let mut names = Vec::with_capacity(BlockId::COUNT as usize);
        for raw_id in 0..BlockId::COUNT {
            if let Some(id) = BlockId::new(raw_id) {
                names.push(pumpkin_data::Block::from_id(id).name.to_string());
            }
        }
        Ok(names)
    }

    async fn get_block_count(&mut self) -> wasmtime::Result<u32> {
        Ok(BlockId::COUNT as u32)
    }

    async fn get_block_state_count(&mut self) -> wasmtime::Result<u32> {
        Ok(BlockStateId::COUNT as u32)
    }

    async fn get_states_for_block(
        &mut self,
        block: WitBlock,
    ) -> wasmtime::Result<Vec<WitBlockState>> {
        let block_id = BlockId::new_or_air(block.id);
        let block_ref = pumpkin_data::Block::from_id(block_id);
        Ok(block_ref
            .states
            .iter()
            .map(|s| to_wit_block_state(s, None))
            .collect())
    }

    async fn get_states_for_block_id(
        &mut self,
        block_id: u16,
    ) -> wasmtime::Result<Vec<WitBlockState>> {
        let Some(id) = BlockId::new(block_id) else {
            return Ok(Vec::new());
        };
        let block_ref = pumpkin_data::Block::from_id(id);
        Ok(block_ref
            .states
            .iter()
            .map(|s| to_wit_block_state(s, None))
            .collect())
    }

    async fn get_state_ids_for_block_id(&mut self, block_id: u16) -> wasmtime::Result<Vec<u16>> {
        let Some(id) = BlockId::new(block_id) else {
            return Ok(Vec::new());
        };
        let block_ref = pumpkin_data::Block::from_id(id);
        Ok(block_ref.states.iter().map(|s| s.id.as_u16()).collect())
    }

    async fn get_block_properties(
        &mut self,
        state_id: u16,
    ) -> wasmtime::Result<Vec<(String, String)>> {
        let bsid = BlockStateId::new_or_air(state_id);
        let block = pumpkin_data::Block::from_state_id(bsid);
        let props = block
            .properties(bsid)
            .map(|p| {
                p.to_props()
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        Ok(props)
    }

    async fn get_block_from_state_id(
        &mut self,
        state_id: u16,
    ) -> wasmtime::Result<Option<WitBlock>> {
        let bsid = BlockStateId::new(state_id);
        Ok(bsid.map(|id| to_wit_block(pumpkin_data::Block::from_state_id(id))))
    }

    async fn get_block_from_state(&mut self, state: WitBlockState) -> wasmtime::Result<WitBlock> {
        let bsid = BlockStateId::new_or_air(state.id);
        Ok(to_wit_block(pumpkin_data::Block::from_state_id(bsid)))
    }

    async fn get_default_state_from_block(
        &mut self,
        block: WitBlock,
    ) -> wasmtime::Result<WitBlockState> {
        let block_id = BlockId::new_or_air(block.id);
        let block_ref = pumpkin_data::Block::from_id(block_id);
        Ok(to_wit_block_state(block_ref.default_state, None))
    }

    async fn get_default_state_from_block_id(
        &mut self,
        block_id: u16,
    ) -> wasmtime::Result<Option<WitBlockState>> {
        let block_id = BlockId::new(block_id);
        Ok(block_id.map(|id| {
            let block = pumpkin_data::Block::from_id(id);
            to_wit_block_state(block.default_state, None)
        }))
    }

    async fn get_block_state_by_id(
        &mut self,
        state_id: u16,
    ) -> wasmtime::Result<Option<WitBlockState>> {
        let bsid = BlockStateId::new(state_id);
        Ok(bsid.map(|id| {
            let state = pumpkin_data::BlockState::from_id(id);
            to_wit_block_state(state, None)
        }))
    }
}
impl pumpkin::plugin::particles::Host for PluginHostState {}
impl pumpkin::plugin::sounds::Host for PluginHostState {}

impl pumpkin::plugin::world::HostWorld for PluginHostState {
    async fn get_id(&mut self, world: Resource<World>) -> wasmtime::Result<String> {
        Ok(self
            .get_world_res(&world)?
            .provider
            .get_world_name()
            .to_string())
    }

    async fn get_border(
        &mut self,
        world: Resource<World>,
    ) -> wasmtime::Result<Resource<WitWorldBorder>> {
        self.get_world_border(world).await
    }

    async fn get_world_border(
        &mut self,
        world: Resource<World>,
    ) -> wasmtime::Result<Resource<WitWorldBorder>> {
        let world_res = self.get_world_res(&world)?;
        self.add_world_border(world_res.provider.clone())
    }

    async fn get_spawn_location(
        &mut self,
        world: Resource<World>,
    ) -> wasmtime::Result<WitWorldSpawnLocation> {
        let (pos, yaw, pitch) = self.get_world_res(&world)?.provider.get_spawn_location();
        Ok(WitWorldSpawnLocation {
            pos: WitBlockPos {
                x: pos.0.x,
                y: pos.0.y,
                z: pos.0.z,
            },
            yaw,
            pitch,
        })
    }

    async fn get_chunk(
        &mut self,
        world: Resource<World>,
        x: i32,
        z: i32,
    ) -> wasmtime::Result<Option<Resource<WitChunk>>> {
        let world_res = self.get_world_res(&world)?;
        let world_provider = world_res.provider.clone();
        let pos = pumpkin_util::math::vector2::Vector2::new(x, z);

        let chunk = world_provider
            .level
            .loaded_chunks
            .get(&pos)
            .map(|c| c.value().clone());
        if let Some(chunk) = chunk {
            let res = self.add_chunk(world_provider, std::sync::Arc::downgrade(&chunk))?;
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }

    async fn get_block_state_id(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u16> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        Ok(world_ref
            .provider
            .get_block_state_id(&internal_pos)
            .as_u16())
    }

    async fn get_block_state(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<WitBlockState> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        let state = world_ref.provider.get_block_state(&internal_pos);
        Ok(to_wit_block_state(state, Some(&internal_pos)))
    }

    async fn get_block(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<WitBlock> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        let state = world_ref.provider.get_block_state(&internal_pos);
        let block = pumpkin_data::Block::from_state_id(state.id);
        Ok(to_wit_block(block))
    }

    async fn get_block_id(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u16> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        let state = world_ref.provider.get_block_state(&internal_pos);
        Ok(pumpkin_data::BlockId::from_state_id(state.id).as_u16())
    }

    async fn set_block(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        block: WitBlock,
        update_flags: WitBlockFlags,
    ) -> wasmtime::Result<()> {
        let block_id = BlockId::new_or_air(block.id);
        let default_state_id = pumpkin_data::Block::from_id(block_id)
            .default_state
            .id
            .as_u16();
        self.set_block_state(world, pos, default_state_id, update_flags)
            .await
    }

    async fn set_block_by_id(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        block_id: u16,
        update_flags: WitBlockFlags,
    ) -> wasmtime::Result<()> {
        let Some(id) = BlockId::new(block_id) else {
            return Err(wasmtime::Error::msg("Invalid BlockId"));
        };
        let default_state_id = pumpkin_data::Block::from_id(id).default_state.id.as_u16();
        self.set_block_state(world, pos, default_state_id, update_flags)
            .await
    }

    async fn set_block_by_name(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        name: String,
        update_flags: WitBlockFlags,
    ) -> wasmtime::Result<bool> {
        if let Some(block) = pumpkin_data::Block::from_name(&name) {
            let default_state_id = block.default_state.id.as_u16();
            self.set_block_state(world, pos, default_state_id, update_flags)
                .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn set_block_state(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        state: u16,
        update_flags: WitBlockFlags,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);

        let mut internal_flags = BlockFlags::empty();
        if update_flags.contains(WitBlockFlags::NOTIFY_NEIGHBORS) {
            internal_flags |= BlockFlags::NOTIFY_NEIGHBORS;
        }
        if update_flags.contains(WitBlockFlags::NOTIFY_LISTENERS) {
            internal_flags |= BlockFlags::NOTIFY_LISTENERS;
        }
        if update_flags.contains(WitBlockFlags::FORCE_STATE) {
            internal_flags |= BlockFlags::FORCE_STATE;
        }
        if update_flags.contains(WitBlockFlags::SKIP_DROPS) {
            internal_flags |= BlockFlags::SKIP_DROPS;
        }
        if update_flags.contains(WitBlockFlags::MOVED) {
            internal_flags |= BlockFlags::MOVED;
        }
        if update_flags.contains(WitBlockFlags::SKIP_REDSTONE_WIRE_STATE_REPLACEMENT) {
            internal_flags |= BlockFlags::SKIP_REDSTONE_WIRE_STATE_REPLACEMENT;
        }
        if update_flags.contains(WitBlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK) {
            internal_flags |= BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK;
        }
        if update_flags.contains(WitBlockFlags::SKIP_BLOCK_ADDED_CALLBACK) {
            internal_flags |= BlockFlags::SKIP_BLOCK_ADDED_CALLBACK;
        }
        let Some(state) = BlockStateId::new(state) else {
            return Err(wasmtime::Error::msg("Invalid BlockStateId"));
        };
        world_ref
            .provider
            .clone()
            .set_block_state(&internal_pos, state, internal_flags);
        Ok(())
    }

    async fn get_time_of_day(&mut self, world: Resource<World>) -> wasmtime::Result<u64> {
        Ok(self.get_world_res(&world)?.provider.get_time_of_day() as u64)
    }

    async fn set_time_of_day(&mut self, world: Resource<World>, time: u64) -> wasmtime::Result<()> {
        self.get_world_res(&world)?
            .provider
            .set_time_of_day(time as i64);
        Ok(())
    }

    async fn get_world_age(&mut self, world: Resource<World>) -> wasmtime::Result<u64> {
        Ok(self.get_world_res(&world)?.provider.get_world_age() as u64)
    }

    async fn get_dimension(&mut self, world: Resource<World>) -> wasmtime::Result<String> {
        Ok(self
            .get_world_res(&world)?
            .provider
            .dimension
            .minecraft_name
            .to_string())
    }

    async fn get_top_block_y(
        &mut self,
        world: Resource<World>,
        x: i32,
        z: i32,
    ) -> wasmtime::Result<i32> {
        Ok(self
            .get_world_res(&world)?
            .provider
            .get_top_block(pumpkin_util::math::vector2::Vector2::new(x, z)))
    }

    async fn get_motion_blocking_height(
        &mut self,
        world: Resource<World>,
        x: i32,
        z: i32,
    ) -> wasmtime::Result<i32> {
        Ok(self.get_world_res(&world)?.provider.get_heightmap_height(
            ChunkHeightmapType::MotionBlocking,
            x,
            z,
        ))
    }

    async fn is_raining(&mut self, world: Resource<World>) -> wasmtime::Result<bool> {
        Ok(self.get_world_res(&world)?.provider.is_raining())
    }

    async fn set_raining(&mut self, world: Resource<World>, raining: bool) -> wasmtime::Result<()> {
        self.get_world_res(&world)?.provider.set_raining(raining);
        Ok(())
    }

    async fn is_thundering(&mut self, world: Resource<World>) -> wasmtime::Result<bool> {
        Ok(self.get_world_res(&world)?.provider.is_thundering())
    }

    async fn set_thundering(
        &mut self,
        world: Resource<World>,
        thundering: bool,
    ) -> wasmtime::Result<()> {
        self.get_world_res(&world)?
            .provider
            .set_thundering(thundering);
        Ok(())
    }

    async fn broadcast_system_message(
        &mut self,
        world: Resource<World>,
        message: Resource<pumpkin::plugin::text::TextComponent>,
        overlay: bool,
    ) -> wasmtime::Result<()> {
        let msg = self.get_text_provider(&message)?;
        self.get_world_res(&world)?
            .provider
            .broadcast_system_message(&msg, overlay);
        Ok(())
    }

    async fn get_scoreboard(
        &mut self,
        world: Resource<World>,
    ) -> wasmtime::Result<Resource<pumpkin::plugin::scoreboard::Scoreboard>> {
        let world_provider = self.get_world_res(&world)?.provider.clone();
        self.add_scoreboard(
            crate::plugin::loader::wasm::wasm_host::state::ScoreboardProvider::World(
                world_provider,
            ),
        )
    }

    async fn play_sound(
        &mut self,
        world: Resource<World>,
        sound: pumpkin::plugin::sounds::Sound,
        category: pumpkin::plugin::world::SoundCategory,
        pos: pumpkin::plugin::common::Position,
        volume: f32,
        pitch: f32,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let sound_name = format!("{sound:?}").to_lowercase().replace('_', ".");
        let sound_data = pumpkin_data::sound::Sound::from_name(&sound_name)
            .ok_or_else(|| wasmtime::Error::msg(format!("Unknown sound: {sound_name}")))?;

        let internal_category = match category {
            pumpkin::plugin::world::SoundCategory::Master => {
                pumpkin_data::sound::SoundCategory::Master
            }
            pumpkin::plugin::world::SoundCategory::Music => {
                pumpkin_data::sound::SoundCategory::Music
            }
            pumpkin::plugin::world::SoundCategory::Records => {
                pumpkin_data::sound::SoundCategory::Records
            }
            pumpkin::plugin::world::SoundCategory::Weather => {
                pumpkin_data::sound::SoundCategory::Weather
            }
            pumpkin::plugin::world::SoundCategory::Blocks => {
                pumpkin_data::sound::SoundCategory::Blocks
            }
            pumpkin::plugin::world::SoundCategory::Hostile => {
                pumpkin_data::sound::SoundCategory::Hostile
            }
            pumpkin::plugin::world::SoundCategory::Neutral => {
                pumpkin_data::sound::SoundCategory::Neutral
            }
            pumpkin::plugin::world::SoundCategory::Players => {
                pumpkin_data::sound::SoundCategory::Players
            }
            pumpkin::plugin::world::SoundCategory::Ambient => {
                pumpkin_data::sound::SoundCategory::Ambient
            }
            pumpkin::plugin::world::SoundCategory::Voice => {
                pumpkin_data::sound::SoundCategory::Voice
            }
        };

        world_ref.provider.play_sound_raw(
            sound_data as u16,
            internal_category,
            &pumpkin_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2),
            volume,
            pitch,
        );
        Ok(())
    }

    async fn spawn_particle(
        &mut self,
        world: Resource<World>,
        particle: pumpkin::plugin::particles::Particle,
        pos: pumpkin::plugin::common::Position,
        offset: pumpkin::plugin::common::Position,
        max_speed: f32,
        count: i32,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let particle_data =
            pumpkin_data::particle::Particle::from_id(particle as u16).ok_or_else(|| {
                wasmtime::Error::msg(format!("Unknown particle ID: {}", particle as u16))
            })?;

        world_ref.provider.spawn_particle(
            pumpkin_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2),
            pumpkin_util::math::vector3::Vector3::new(
                offset.0 as f32,
                offset.1 as f32,
                offset.2 as f32,
            ),
            max_speed,
            count,
            particle_data,
        );
        Ok(())
    }

    async fn create_explosion(
        &mut self,
        world: Resource<World>,
        pos: pumpkin::plugin::common::Position,
        power: f32,
        _create_fire: bool,
        interaction: pumpkin::plugin::world::ExplosionInteraction,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let interaction = match interaction {
            pumpkin::plugin::world::ExplosionInteraction::None => ExplosionInteraction::None,
            pumpkin::plugin::world::ExplosionInteraction::Block => ExplosionInteraction::Block,
            pumpkin::plugin::world::ExplosionInteraction::Mob => ExplosionInteraction::Mob,
            pumpkin::plugin::world::ExplosionInteraction::Tnt => ExplosionInteraction::Tnt,
            pumpkin::plugin::world::ExplosionInteraction::Trigger => ExplosionInteraction::Trigger,
        };
        world_ref.provider.explode(
            pumpkin_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2),
            power,
            interaction,
        );
        Ok(())
    }

    async fn get_sea_level(&mut self, world: Resource<World>) -> wasmtime::Result<i32> {
        Ok(self.get_world_res(&world)?.provider.sea_level)
    }

    async fn get_min_y(&mut self, world: Resource<World>) -> wasmtime::Result<i32> {
        Ok(self.get_world_res(&world)?.provider.min_y)
    }

    async fn get_sky_light(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u8> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        Ok(world_ref.provider.get_sky_light_level(&internal_pos))
    }

    async fn set_sky_light(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        level: u8,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        world_ref.provider.set_sky_light_level(&internal_pos, level);
        Ok(())
    }

    async fn get_block_light(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u8> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        Ok(world_ref
            .provider
            .get_block_light_level(&internal_pos)
            .unwrap_or(0))
    }

    async fn set_block_light(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        level: u8,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        world_ref
            .provider
            .set_block_light_level(&internal_pos, level);
        Ok(())
    }

    async fn get_biome(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<pumpkin::plugin::biomes::Biome> {
        let world_ref = self.get_world_res(&world)?;
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        let biome = world_ref.provider.get_biome(&internal_pos);

        Self::get_wit_biome(biome)
    }

    async fn spawn_entity(
        &mut self,
        world: Resource<World>,
        entity_type: pumpkin::plugin::entity_types::EntityType,
        pos: pumpkin::plugin::common::Position,
    ) -> wasmtime::Result<Resource<pumpkin::plugin::world::Entity>> {
        let world_ref = self.get_world_res(&world)?;
        let world_provider = world_ref.provider.clone();

        let mut names: Vec<String> = serde_json::from_str::<
            std::collections::BTreeMap<String, serde_json::Value>,
        >(&std::fs::read_to_string("assets/entities.json")?)?
        .keys()
        .cloned()
        .collect();
        names.sort();

        let type_name = names.get(entity_type as usize).ok_or_else(|| {
            wasmtime::Error::msg(format!("Invalid entity type index: {}", entity_type as u8))
        })?;

        let internal_type = pumpkin_data::entity::EntityType::from_name(type_name)
            .ok_or_else(|| wasmtime::Error::msg(format!("Invalid entity type: {type_name}")))?;

        let internal_pos = pumpkin_util::math::vector3::Vector3::new(pos.0, pos.1, pos.2);
        let entity = crate::entity::r#type::from_type(
            internal_type,
            internal_pos,
            &world_provider,
            uuid::Uuid::new_v4(),
        );

        world_provider.spawn_entity(entity.clone());

        self.add_entity(entity)
    }

    async fn get_entities(
        &mut self,
        world: Resource<World>,
    ) -> wasmtime::Result<Vec<Resource<pumpkin::plugin::entity::Entity>>> {
        let world_provider = self.get_world_res(&world)?.provider.clone();
        let mut entities = Vec::new();

        // Add players as entities
        for player in world_provider.players.load().iter() {
            entities.push(self.add_entity(player.clone() as Arc<dyn crate::entity::EntityBase>)?);
        }

        // Add other entities
        for entity in world_provider.entities.load().iter() {
            entities.push(self.add_entity(entity.clone())?);
        }

        Ok(entities)
    }

    async fn strike_lightning(
        &mut self,
        world: Resource<World>,
        pos: WitPosition,
        effect_only: bool,
    ) -> wasmtime::Result<()> {
        let world_provider = self.get_world_res(&world)?.provider.clone();
        let internal_pos = super::events::from_wasm_position(pos);
        world_provider.strike_lightning(internal_pos, effect_only);
        Ok(())
    }

    async fn ray_trace_blocks(
        &mut self,
        world: Resource<World>,
        start: WitPosition,
        end: WitPosition,
    ) -> wasmtime::Result<Option<WitPosition>> {
        let world_provider = self.get_world_res(&world)?.provider.clone();
        let start_pos = super::events::from_wasm_position(start);
        let end_pos = super::events::from_wasm_position(end);
        let res = world_provider.raycast(start_pos, end_pos, |pos, w| {
            !w.get_block_state(pos).is_air()
        });
        Ok(res.map(|(p, _)| {
            super::events::to_wasm_position(pumpkin_util::math::vector3::Vector3::new(
                f64::from(p.0.x),
                f64::from(p.0.y),
                f64::from(p.0.z),
            ))
        }))
    }

    async fn ray_trace_block(
        &mut self,
        world: Resource<World>,
        start: WitPosition,
        end: WitPosition,
        include_fluids: bool,
    ) -> wasmtime::Result<Option<WitRayTraceBlockResult>> {
        let world_provider = self.get_world_res(&world)?.provider.clone();
        let start_pos = super::events::from_wasm_position(start);
        let end_pos = super::events::from_wasm_position(end);
        let res = world_provider.ray_trace_block(start_pos, end_pos, include_fluids);
        Ok(res.map(|(pos, face, hit_pos)| WitRayTraceBlockResult {
            pos: WitBlockPos {
                x: pos.0.x,
                y: pos.0.y,
                z: pos.0.z,
            },
            face: to_wasm_block_direction(face),
            hit_pos: super::events::to_wasm_position(hit_pos),
        }))
    }

    async fn ray_trace_entity(
        &mut self,
        world: Resource<World>,
        start: WitPosition,
        end: WitPosition,
    ) -> wasmtime::Result<Option<WitRayTraceEntityResult>> {
        let world_provider = self.get_world_res(&world)?.provider.clone();
        let start_pos = super::events::from_wasm_position(start);
        let end_pos = super::events::from_wasm_position(end);
        if let Some((entity, hit_pos, distance)) =
            world_provider.ray_trace_entity(start_pos, end_pos)
        {
            let entity_res = self
                .add_entity(entity)
                .map_err(|_| wasmtime::Error::msg("failed to add entity resource"))?;
            Ok(Some(WitRayTraceEntityResult {
                entity: entity_res,
                hit_pos: super::events::to_wasm_position(hit_pos),
                distance,
            }))
        } else {
            Ok(None)
        }
    }

    async fn ray_trace_entities(
        &mut self,
        world: Resource<World>,
        start: WitPosition,
        end: WitPosition,
    ) -> wasmtime::Result<Vec<WitRayTraceEntityResult>> {
        let world_provider = self.get_world_res(&world)?.provider.clone();
        let start_pos = super::events::from_wasm_position(start);
        let end_pos = super::events::from_wasm_position(end);
        let hits = world_provider.ray_trace_entities(start_pos, end_pos);
        let mut results = Vec::with_capacity(hits.len());
        for (entity, hit_pos, distance) in hits {
            let entity_res = self
                .add_entity(entity)
                .map_err(|_| wasmtime::Error::msg("failed to add entity resource"))?;
            results.push(WitRayTraceEntityResult {
                entity: entity_res,
                hit_pos: super::events::to_wasm_position(hit_pos),
                distance,
            });
        }
        Ok(results)
    }

    async fn get_block_entity(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<Option<BlockEntityType>> {
        let world_provider = self.get_world_res(&world)?.provider.clone();
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);
        let block_entity = world_provider.get_block_entity(&internal_pos);

        block_entity.map_or_else(|| Ok(None), |be| self.get_wit_block_entity(be))
    }

    async fn get_block_entity_nbt(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<Option<Vec<u8>>> {
        let world_ref = self.get_world_res(&world)?.provider.clone();
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);

        let Some(entity) = world_ref.get_block_entity(&internal_pos) else {
            return Ok(None);
        };

        let mut nbt = pumpkin_nbt::NbtCompound::new();
        entity.write_internal(&mut nbt);

        let bytes = pumpkin_nbt::Nbt::from(nbt).write_unnamed();
        Ok(Some(bytes.to_vec()))
    }

    async fn set_block_entity_nbt(
        &mut self,
        world: Resource<World>,
        pos: WitBlockPos,
        nbt_data: Vec<u8>,
    ) -> wasmtime::Result<Result<(), String>> {
        let world_ref = self.get_world_res(&world)?.provider.clone();
        let internal_pos = BlockPos::new(pos.x, pos.y, pos.z);

        let mut cursor = std::io::Cursor::new(&nbt_data[..]);
        let mut reader = pumpkin_nbt::deserializer::NbtReadHelperJava::new(
            pumpkin_nbt::deserializer::NbtStreamReader(&mut cursor),
        );
        let mut nbt: pumpkin_nbt::NbtCompound = pumpkin_nbt::Nbt::read_unnamed(&mut reader)
            .map_err(|e| wasmtime::Error::msg(format!("Invalid NBT: {e}")))?
            .root_tag;

        // Override NBT position with the caller-provided position so tile
        // entities land at the correct coordinates during schematic pastes.
        nbt.put_int("x", pos.x);
        nbt.put_int("y", pos.y);
        nbt.put_int("z", pos.z);

        // Use add_block_entity_nbt for lazy loading — avoids broadcasting
        // a packet per tile entity during bulk operations.
        world_ref.add_block_entity_nbt(internal_pos, &nbt);
        Ok(Ok(()))
    }

    async fn set_chunk_generator(
        &mut self,
        world: Resource<World>,
        generator_id: u32,
    ) -> wasmtime::Result<()> {
        let world_ref = self.get_world_res(&world)?.provider.clone();
        let Some(plugin_weak) = self.plugin.as_ref() else {
            return Ok(());
        };
        let Some(plugin) = plugin_weak.upgrade() else {
            return Ok(());
        };

        let wasm_gen = Arc::new(WasmChunkGenerator {
            generator_id,
            plugin,
            dimension: world_ref.dimension.clone(),
            seed: world_ref.level.seed.0,
        });

        world_ref.level.set_world_gen(Arc::new(
            pumpkin_world::generation::generator::WorldGenerator::Custom(wasm_gen),
        ));
        Ok(())
    }

    async fn get_name(&mut self, world: Resource<World>) -> wasmtime::Result<String> {
        Ok(self
            .get_world_res(&world)?
            .provider
            .get_world_name()
            .to_string())
    }

    async fn save(&mut self, world: Resource<World>) -> wasmtime::Result<Result<(), String>> {
        let world_res = self.get_world_res(&world)?;
        world_res.provider.save().await;
        Ok(Ok(()))
    }

    async fn set_custom_data(
        &mut self,
        world: Resource<World>,
        namespace: String,
        key: String,
        value: super::common::WitNbtTree,
    ) -> wasmtime::Result<()> {
        let world_res = self.get_world_res(&world)?;
        let tag = super::common::from_wit_nbt_tree(&value).map_err(wasmtime::Error::msg)?;
        world_res.provider.set_custom_data(&namespace, &key, tag);
        Ok(())
    }

    async fn get_custom_data(
        &mut self,
        world: Resource<World>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<Option<super::common::WitNbtTree>> {
        let world_res = self.get_world_res(&world)?;
        let tag = world_res.provider.get_custom_data(&namespace, &key);
        Ok(tag.map(super::common::to_wit_nbt_tree))
    }

    async fn remove_custom_data(
        &mut self,
        world: Resource<World>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<()> {
        let world_res = self.get_world_res(&world)?;
        world_res.provider.remove_custom_data(&namespace, &key);
        Ok(())
    }

    async fn has_custom_data(
        &mut self,
        world: Resource<World>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<bool> {
        let world_res = self.get_world_res(&world)?;
        Ok(world_res.provider.has_custom_data(&namespace, &key))
    }

    async fn get_game_rule(
        &mut self,
        world: Resource<World>,
        rule: WitGameRule,
    ) -> wasmtime::Result<WitGameRuleValue> {
        let world_res = self.get_world_res(&world)?;
        let internal_rule = from_wit_game_rule(rule);
        let value = world_res.provider.get_game_rule(&internal_rule);
        Ok(to_wit_game_rule_value(&value))
    }

    async fn set_game_rule(
        &mut self,
        world: Resource<World>,
        rule: WitGameRule,
        value: WitGameRuleValue,
    ) -> wasmtime::Result<()> {
        let world_res = self.get_world_res(&world)?;
        let internal_rule = from_wit_game_rule(rule);
        let internal_value = from_wit_game_rule_value(value);
        world_res
            .provider
            .set_game_rule(&internal_rule, internal_value);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<World>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<WorldResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

impl pumpkin::plugin::world::HostChunk for PluginHostState {
    async fn get_x(&mut self, chunk: Resource<WitChunk>) -> wasmtime::Result<i32> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        Ok(chunk_data.x)
    }

    async fn get_z(&mut self, chunk: Resource<WitChunk>) -> wasmtime::Result<i32> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        Ok(chunk_data.z)
    }

    async fn get_block_state_id(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u16> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        Ok(chunk_data
            .section
            .get_block_absolute_y(pos.x as usize, pos.y, pos.z as usize)
            .unwrap_or(BlockStateId::AIR)
            .as_u16())
    }

    async fn get_block_state(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<WitBlockState> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let id = chunk_data
            .section
            .get_block_absolute_y(pos.x as usize, pos.y, pos.z as usize)
            .unwrap_or(BlockStateId::AIR);
        let state = id.to_state();
        let world_pos = BlockPos::new(chunk_data.x * 16 + pos.x, pos.y, chunk_data.z * 16 + pos.z);
        Ok(to_wit_block_state(state, Some(&world_pos)))
    }

    async fn get_block(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<WitBlock> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let id = chunk_data
            .section
            .get_block_absolute_y(pos.x as usize, pos.y, pos.z as usize)
            .unwrap_or(BlockStateId::AIR);
        let block = pumpkin_data::Block::from_state_id(id);
        Ok(to_wit_block(block))
    }

    async fn set_block(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
        block: WitBlock,
    ) -> wasmtime::Result<()> {
        let block_id = BlockId::new_or_air(block.id);
        let default_state_id = pumpkin_data::Block::from_id(block_id)
            .default_state
            .id
            .as_u16();
        self.set_block_state(chunk, pos, default_state_id).await
    }

    async fn set_block_by_id(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
        block_id: u16,
    ) -> wasmtime::Result<()> {
        let Some(id) = BlockId::new(block_id) else {
            return Err(wasmtime::Error::msg("Invalid BlockId"));
        };
        let default_state_id = pumpkin_data::Block::from_id(id).default_state.id.as_u16();
        self.set_block_state(chunk, pos, default_state_id).await
    }

    async fn set_block_state(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
        state: u16,
    ) -> wasmtime::Result<()> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (world, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };

        let Some(state) = BlockStateId::new(state) else {
            return Err(wasmtime::Error::msg("Invalid BlockStateId"));
        };

        let replaced =
            chunk_data.set_block_absolute_y(pos.x as usize, pos.y, pos.z as usize, state);

        if replaced != state {
            chunk_data.mark_dirty(true);
            let absolute_pos =
                BlockPos::new(chunk_data.x * 16 + pos.x, pos.y, chunk_data.z * 16 + pos.z);
            world.register_block_change(absolute_pos, state);
        }

        Ok(())
    }

    async fn get_biome(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<pumpkin::plugin::biomes::Biome> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let id = chunk_data
            .section
            .get_rough_biome_absolute_y(pos.x as usize, pos.y, pos.z as usize)
            .unwrap_or(0);
        let biome = pumpkin_data::biome::Biome::from_id(id)
            .unwrap_or(&pumpkin_data::biome::Biome::THE_VOID);

        Self::get_wit_biome(biome)
    }

    async fn get_block_entity(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<Option<BlockEntityType>> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (world, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let absolute_pos =
            BlockPos::new(chunk_data.x * 16 + pos.x, pos.y, chunk_data.z * 16 + pos.z);
        let block_entity = world.get_block_entity(&absolute_pos);

        block_entity.map_or_else(|| Ok(None), |be| self.get_wit_block_entity(be))
    }

    async fn get_top_block_y(
        &mut self,
        chunk: Resource<WitChunk>,
        x: i32,
        z: i32,
    ) -> wasmtime::Result<i32> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        Ok(chunk_data
            .heightmap
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(
                ChunkHeightmapType::WorldSurface,
                x,
                z,
                chunk_data.section.min_y,
            ))
    }

    async fn get_sky_light(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u8> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let section_index = (pos.y - chunk_data.section.min_y) as usize / 16;
        Ok(chunk_data
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .sky_light
            .get(section_index)
            .map_or(0, |c| {
                c.get(pos.x as usize, pos.y as usize % 16, pos.z as usize)
            }))
    }

    async fn get_block_light(
        &mut self,
        chunk: Resource<WitChunk>,
        pos: WitBlockPos,
    ) -> wasmtime::Result<u8> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let section_index = (pos.y - chunk_data.section.min_y) as usize / 16;
        Ok(chunk_data
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .block_light
            .get(section_index)
            .map_or(0, |c| {
                c.get(pos.x as usize, pos.y as usize % 16, pos.z as usize)
            }))
    }

    async fn set_custom_data(
        &mut self,
        chunk: Resource<WitChunk>,
        namespace: String,
        key: String,
        value: super::common::WitNbtTree,
    ) -> wasmtime::Result<()> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let tag = super::common::from_wit_nbt_tree(&value).map_err(wasmtime::Error::msg)?;
        chunk_data.set_custom_data(&namespace, &key, tag);
        Ok(())
    }

    async fn get_custom_data(
        &mut self,
        chunk: Resource<WitChunk>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<Option<super::common::WitNbtTree>> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        let tag = chunk_data.get_custom_data(&namespace, &key);
        Ok(tag.map(super::common::to_wit_nbt_tree))
    }

    async fn remove_custom_data(
        &mut self,
        chunk: Resource<WitChunk>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<()> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        chunk_data.remove_custom_data(&namespace, &key);
        Ok(())
    }

    async fn has_custom_data(
        &mut self,
        chunk: Resource<WitChunk>,
        namespace: String,
        key: String,
    ) -> wasmtime::Result<bool> {
        let chunk_res = self.get_chunk_res(&chunk)?;
        let (_, chunk_data) = &chunk_res.provider;
        let Some(chunk_data) = chunk_data.upgrade() else {
            return Err(wasmtime::Error::msg("Chunk unloaded"));
        };
        Ok(chunk_data.has_custom_data(&namespace, &key))
    }

    async fn drop(&mut self, rep: Resource<WitChunk>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<ChunkResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

impl pumpkin::plugin::world::HostWorldBorder for PluginHostState {
    async fn get_center_x(&mut self, border: Resource<WitWorldBorder>) -> wasmtime::Result<f64> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .center_x)
    }

    async fn get_center_z(&mut self, border: Resource<WitWorldBorder>) -> wasmtime::Result<f64> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .center_z)
    }

    async fn get_center(
        &mut self,
        border: Resource<WitWorldBorder>,
    ) -> wasmtime::Result<WitPosition> {
        let border_res = self.get_world_border_res(&border)?;
        let guard = border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(super::events::to_wasm_position(
            pumpkin_util::math::vector3::Vector3::new(guard.center_x, 0.0, guard.center_z),
        ))
    }

    async fn set_center(
        &mut self,
        border: Resource<WitWorldBorder>,
        x: f64,
        z: f64,
    ) -> wasmtime::Result<()> {
        let border_res = self.get_world_border_res(&border)?;
        let world = border_res.provider.clone();
        world
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_center(&world, x, z);
        Ok(())
    }

    async fn get_diameter(&mut self, border: Resource<WitWorldBorder>) -> wasmtime::Result<f64> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .new_diameter)
    }

    async fn get_size(&mut self, border: Resource<WitWorldBorder>) -> wasmtime::Result<f64> {
        self.get_diameter(border).await
    }

    async fn set_diameter(
        &mut self,
        border: Resource<WitWorldBorder>,
        diameter: f64,
        speed: Option<u64>,
    ) -> wasmtime::Result<()> {
        let border_res = self.get_world_border_res(&border)?;
        let world = border_res.provider.clone();
        world
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_diameter(&world, diameter, speed.map(|s| s as i64));
        Ok(())
    }

    async fn set_size(
        &mut self,
        border: Resource<WitWorldBorder>,
        size: f64,
    ) -> wasmtime::Result<()> {
        self.set_diameter(border, size, None).await
    }

    async fn set_size_transition(
        &mut self,
        border: Resource<WitWorldBorder>,
        new_size: f64,
        time_seconds: u64,
    ) -> wasmtime::Result<()> {
        let speed_millis = time_seconds.saturating_mul(1000);
        self.set_diameter(border, new_size, Some(speed_millis))
            .await
    }

    async fn get_target_diameter(
        &mut self,
        border: Resource<WitWorldBorder>,
    ) -> wasmtime::Result<f64> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .new_diameter)
    }

    async fn get_target_speed(
        &mut self,
        border: Resource<WitWorldBorder>,
    ) -> wasmtime::Result<i64> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .speed)
    }

    async fn get_warning_distance(
        &mut self,
        border: Resource<WitWorldBorder>,
    ) -> wasmtime::Result<i32> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .warning_blocks)
    }

    async fn set_warning_distance(
        &mut self,
        border: Resource<WitWorldBorder>,
        distance: i32,
    ) -> wasmtime::Result<()> {
        let border_res = self.get_world_border_res(&border)?;
        let world = border_res.provider.clone();
        world
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_warning_distance(&world, distance);
        Ok(())
    }

    async fn get_warning_delay(
        &mut self,
        border: Resource<WitWorldBorder>,
    ) -> wasmtime::Result<i32> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .warning_time)
    }

    async fn set_warning_delay(
        &mut self,
        border: Resource<WitWorldBorder>,
        delay: i32,
    ) -> wasmtime::Result<()> {
        let border_res = self.get_world_border_res(&border)?;
        let world = border_res.provider.clone();
        world
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_warning_delay(&world, delay);
        Ok(())
    }

    async fn get_warning_time(
        &mut self,
        border: Resource<WitWorldBorder>,
    ) -> wasmtime::Result<i32> {
        self.get_warning_delay(border).await
    }

    async fn set_warning_time(
        &mut self,
        border: Resource<WitWorldBorder>,
        time: i32,
    ) -> wasmtime::Result<()> {
        self.set_warning_delay(border, time).await
    }

    async fn get_damage_buffer(
        &mut self,
        border: Resource<WitWorldBorder>,
    ) -> wasmtime::Result<f64> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(f64::from(
            border_res
                .provider
                .worldborder
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .buffer,
        ))
    }

    async fn set_damage_buffer(
        &mut self,
        border: Resource<WitWorldBorder>,
        buffer: f64,
    ) -> wasmtime::Result<()> {
        let border_res = self.get_world_border_res(&border)?;
        border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_damage_buffer(buffer as f32);
        Ok(())
    }

    async fn get_damage_amount(
        &mut self,
        border: Resource<WitWorldBorder>,
    ) -> wasmtime::Result<f64> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(f64::from(
            border_res
                .provider
                .worldborder
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .damage_per_block,
        ))
    }

    async fn set_damage_amount(
        &mut self,
        border: Resource<WitWorldBorder>,
        damage: f64,
    ) -> wasmtime::Result<()> {
        let border_res = self.get_world_border_res(&border)?;
        border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_damage_per_block(damage as f32);
        Ok(())
    }

    async fn contains(
        &mut self,
        border: Resource<WitWorldBorder>,
        x: f64,
        z: f64,
    ) -> wasmtime::Result<bool> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(x, z))
    }

    async fn contains_pos(
        &mut self,
        border: Resource<WitWorldBorder>,
        pos: WitPosition,
    ) -> wasmtime::Result<bool> {
        let border_res = self.get_world_border_res(&border)?;
        Ok(border_res
            .provider
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(pos.0, pos.2))
    }

    async fn reset(&mut self, border: Resource<WitWorldBorder>) -> wasmtime::Result<()> {
        let border_res = self.get_world_border_res(&border)?;
        let world = border_res.provider.clone();
        world
            .worldborder
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .reset(&world);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<WitWorldBorder>) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<WorldBorderResource>(Resource::new_own(rep.rep()))
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

impl pumpkin::plugin::world::HostChunkBuffer for PluginHostState {
    async fn get_x(
        &mut self,
        this: Resource<pumpkin::plugin::world::ChunkBuffer>,
    ) -> wasmtime::Result<i32> {
        let res = self.get_chunk_buffer_res(&this)?;
        Ok(res.provider.x)
    }

    async fn get_z(
        &mut self,
        this: Resource<pumpkin::plugin::world::ChunkBuffer>,
    ) -> wasmtime::Result<i32> {
        let res = self.get_chunk_buffer_res(&this)?;
        Ok(res.provider.z)
    }

    async fn get_min_y(
        &mut self,
        this: Resource<pumpkin::plugin::world::ChunkBuffer>,
    ) -> wasmtime::Result<i32> {
        let res = self.get_chunk_buffer_res(&this)?;
        Ok(res.provider.min_y)
    }

    async fn get_height(
        &mut self,
        this: Resource<pumpkin::plugin::world::ChunkBuffer>,
    ) -> wasmtime::Result<u32> {
        let res = self.get_chunk_buffer_res(&this)?;
        Ok(res.provider.height)
    }

    async fn set_block_state_id(
        &mut self,
        this: Resource<pumpkin::plugin::world::ChunkBuffer>,
        x: u8,
        y: i32,
        z: u8,
        state_id: u16,
    ) -> wasmtime::Result<()> {
        let res = self.get_chunk_buffer_res(&this)?;
        if x < 16 && z < 16 {
            let world_x =
                pumpkin_world::generation::positions::chunk_pos::start_block_x(res.provider.x)
                    + x as i32;
            let world_z =
                pumpkin_world::generation::positions::chunk_pos::start_block_z(res.provider.z)
                    + z as i32;
            // SAFETY: `proto_chunk` points to a valid proto chunk allocated for world generation and is not aliased across threads.
            let proto = unsafe { &mut *res.provider.proto_chunk };
            let block_state = pumpkin_data::BlockState::from_id(
                pumpkin_data::BlockStateId::new(state_id)
                    .unwrap_or(pumpkin_data::BlockStateId::AIR),
            );
            proto.set_block_state(world_x, y, world_z, block_state);
        }
        Ok(())
    }

    async fn get_block_state_id(
        &mut self,
        this: Resource<pumpkin::plugin::world::ChunkBuffer>,
        x: u8,
        y: i32,
        z: u8,
    ) -> wasmtime::Result<u16> {
        let res = self.get_chunk_buffer_res(&this)?;
        if x < 16 && z < 16 {
            // SAFETY: `proto_chunk` points to a valid proto chunk allocated for world generation and is not aliased across threads.
            let proto = unsafe { &*res.provider.proto_chunk };
            let local_y = y - proto.bottom_y() as i32;
            if local_y >= 0 && local_y < proto.height() as i32 {
                Ok(proto
                    .get_block_state_raw(x as i32, local_y, z as i32)
                    .as_u16())
            } else {
                Ok(0)
            }
        } else {
            Ok(0)
        }
    }

    async fn fill_layer(
        &mut self,
        this: Resource<pumpkin::plugin::world::ChunkBuffer>,
        y: i32,
        state_id: u16,
    ) -> wasmtime::Result<()> {
        let res = self.get_chunk_buffer_res(&this)?;
        let start_x =
            pumpkin_world::generation::positions::chunk_pos::start_block_x(res.provider.x);
        let start_z =
            pumpkin_world::generation::positions::chunk_pos::start_block_z(res.provider.z);
        let block_state = pumpkin_data::BlockState::from_id(
            pumpkin_data::BlockStateId::new(state_id).unwrap_or(pumpkin_data::BlockStateId::AIR),
        );
        // SAFETY: `proto_chunk` points to a valid proto chunk allocated for world generation and is not aliased across threads.
        let proto = unsafe { &mut *res.provider.proto_chunk };
        for x in 0..16 {
            for z in 0..16 {
                proto.set_block_state(start_x + x, y, start_z + z, block_state);
            }
        }
        Ok(())
    }

    async fn fill_range(
        &mut self,
        this: Resource<pumpkin::plugin::world::ChunkBuffer>,
        x: u8,
        min_y: i32,
        max_y: i32,
        z: u8,
        state_id: u16,
    ) -> wasmtime::Result<()> {
        let res = self.get_chunk_buffer_res(&this)?;
        if x < 16 && z < 16 {
            let world_x =
                pumpkin_world::generation::positions::chunk_pos::start_block_x(res.provider.x)
                    + x as i32;
            let world_z =
                pumpkin_world::generation::positions::chunk_pos::start_block_z(res.provider.z)
                    + z as i32;
            let block_state = pumpkin_data::BlockState::from_id(
                pumpkin_data::BlockStateId::new(state_id)
                    .unwrap_or(pumpkin_data::BlockStateId::AIR),
            );
            // SAFETY: `proto_chunk` points to a valid proto chunk allocated for world generation and is not aliased across threads.
            let proto = unsafe { &mut *res.provider.proto_chunk };
            for y in min_y..=max_y {
                proto.set_block_state(world_x, y, world_z, block_state);
            }
        }
        Ok(())
    }

    async fn fill_cuboid(
        &mut self,
        this: Resource<pumpkin::plugin::world::ChunkBuffer>,
        min_x: u8,
        min_y: i32,
        min_z: u8,
        max_x: u8,
        max_y: i32,
        max_z: u8,
        state_id: u16,
    ) -> wasmtime::Result<()> {
        let res = self.get_chunk_buffer_res(&this)?;
        let start_x =
            pumpkin_world::generation::positions::chunk_pos::start_block_x(res.provider.x);
        let start_z =
            pumpkin_world::generation::positions::chunk_pos::start_block_z(res.provider.z);
        let block_state = pumpkin_data::BlockState::from_id(
            pumpkin_data::BlockStateId::new(state_id).unwrap_or(pumpkin_data::BlockStateId::AIR),
        );
        // SAFETY: `proto_chunk` points to a valid proto chunk allocated for world generation and is not aliased across threads.
        let proto = unsafe { &mut *res.provider.proto_chunk };
        let max_x = max_x.min(15);
        let max_z = max_z.min(15);
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    proto.set_block_state(start_x + x as i32, y, start_z + z as i32, block_state);
                }
            }
        }
        Ok(())
    }

    async fn set_biome(
        &mut self,
        this: Resource<pumpkin::plugin::world::ChunkBuffer>,
        x: u8,
        y: i32,
        z: u8,
        biome: pumpkin::plugin::biomes::Biome,
    ) -> wasmtime::Result<()> {
        let res = self.get_chunk_buffer_res(&this)?;
        if x < 16 && z < 16 {
            let biome_id = biome as u8;
            // SAFETY: `proto_chunk` points to a valid proto chunk allocated for world generation and is not aliased across threads.
            let proto = unsafe { &mut *res.provider.proto_chunk };
            let biome_x = x as i32 / 4;
            let biome_z = z as i32 / 4;
            let biome_y = (y - proto.bottom_y() as i32) / 4;
            if biome_y >= 0 && (biome_y as usize) < (proto.height() as usize / 4) {
                let index = proto.local_biome_pos_to_biome_index(biome_x, biome_y, biome_z);
                if index < proto.flat_biome_map.len() {
                    proto.flat_biome_map[index] = biome_id;
                }
            }
        }
        Ok(())
    }

    async fn fill_biome(
        &mut self,
        this: Resource<pumpkin::plugin::world::ChunkBuffer>,
        biome: pumpkin::plugin::biomes::Biome,
    ) -> wasmtime::Result<()> {
        let res = self.get_chunk_buffer_res(&this)?;
        let biome_id = biome as u8;
        // SAFETY: `proto_chunk` points to a valid proto chunk allocated for world generation and is not aliased across threads.
        let proto = unsafe { &mut *res.provider.proto_chunk };
        proto.flat_biome_map.fill(biome_id);
        Ok(())
    }

    async fn drop(
        &mut self,
        rep: Resource<pumpkin::plugin::world::ChunkBuffer>,
    ) -> wasmtime::Result<()> {
        self.resource_table
            .delete::<crate::plugin::loader::wasm::wasm_host::state::ChunkBufferResource>(
                Resource::new_own(rep.rep()),
            )
            .map_err(wasmtime::Error::from)?;
        Ok(())
    }
}

pub struct WasmChunkGenerator {
    pub generator_id: u32,
    pub plugin: Arc<crate::plugin::loader::wasm::wasm_host::WasmPlugin>,
    pub dimension: pumpkin_data::dimension::Dimension,
    pub seed: u64,
}

impl WasmChunkGenerator {
    fn invoke_phase(
        &self,
        phase: pumpkin::plugin::world::GenerationPhase,
        proto_chunk: &mut pumpkin_world::ProtoChunk,
    ) {
        let chunk_buffer = crate::plugin::loader::wasm::wasm_host::state::ChunkBuffer {
            x: proto_chunk.x,
            z: proto_chunk.z,
            min_y: proto_chunk.bottom_y() as i32,
            height: proto_chunk.height() as u32,
            proto_chunk,
        };

        futures::executor::block_on(async {
            let mut store = self.plugin.store.lock().await;
            let Ok(buffer_res) = store.data_mut().add_chunk_buffer(chunk_buffer) else {
                return;
            };
            let buffer_rep = buffer_res.rep();

            match self.plugin.plugin_instance {
                crate::plugin::loader::wasm::wasm_host::PluginInstance::V0_1(ref plugin) => {
                    let _ = plugin
                        .call_handle_generate_phase(
                            &mut *store,
                            self.generator_id,
                            phase,
                            buffer_res,
                        )
                        .await;

                    let _ = store
                        .data_mut()
                        .resource_table
                        .delete::<crate::plugin::loader::wasm::wasm_host::state::ChunkBufferResource>(
                            wasmtime::component::Resource::new_own(buffer_rep),
                        );
                }
            }
        });
    }
}

impl pumpkin_world::generation::generator::CustomChunkGenerator for WasmChunkGenerator {
    fn dimension(&self) -> &pumpkin_data::dimension::Dimension {
        &self.dimension
    }

    fn seed(&self) -> u64 {
        self.seed
    }

    fn step_to_biomes(&self, chunk: &mut pumpkin_world::ProtoChunk) {
        self.invoke_phase(pumpkin::plugin::world::GenerationPhase::Biomes, chunk);
        chunk.stage = pumpkin_world::chunk_system::StagedChunkEnum::Biomes;
    }

    fn step_to_noise(&self, chunk: &mut pumpkin_world::ProtoChunk) {
        self.invoke_phase(pumpkin::plugin::world::GenerationPhase::Noise, chunk);
        chunk.stage = pumpkin_world::chunk_system::StagedChunkEnum::Noise;
    }

    fn step_to_surface(&self, chunk: &mut pumpkin_world::ProtoChunk) {
        self.invoke_phase(pumpkin::plugin::world::GenerationPhase::Surface, chunk);
        chunk.stage = pumpkin_world::chunk_system::StagedChunkEnum::Surface;
    }

    fn step_to_carvers(&self, chunk: &mut pumpkin_world::ProtoChunk) {
        chunk.stage = pumpkin_world::chunk_system::StagedChunkEnum::Carvers;
    }

    fn step_to_features(
        &self,
        cache: &mut pumpkin_world::chunk_system::generation_cache::Cache,
        _block_registry: &dyn pumpkin_world::world::WorldPortalExt,
    ) {
        let mid = ((cache.size * cache.size) >> 1) as usize;
        let chunk = cache.chunks[mid].get_proto_chunk_mut();
        self.invoke_phase(pumpkin::plugin::world::GenerationPhase::Features, chunk);
        chunk.stage = pumpkin_world::chunk_system::StagedChunkEnum::Features;
    }
}

#[cfg(test)]
mod tests {
    use super::pumpkin;

    #[test]
    fn wit_particle_ids_match_internal_particle_ids() {
        let cases = [
            (
                pumpkin::plugin::particles::Particle::AngryVillager,
                pumpkin_data::particle::Particle::AngryVillager,
            ),
            (
                pumpkin::plugin::particles::Particle::HappyVillager,
                pumpkin_data::particle::Particle::HappyVillager,
            ),
            (
                pumpkin::plugin::particles::Particle::SulfurCubeGoo,
                pumpkin_data::particle::Particle::SulfurCubeGoo,
            ),
        ];

        for (wit, internal) in cases {
            assert_eq!(
                pumpkin_data::particle::Particle::from_id(wit as u16),
                Some(internal)
            );
        }
    }
}
