use pumpkin_data::BlockStateId;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    events::{
        ToFromWasmEvent, cleanup_event, consume_player, consume_world, from_wasm_block_name,
        from_wasm_block_position, to_wasm_block_name, to_wasm_block_position,
    },
    pumpkin::plugin::event::{
        BellResonateEvent as BellResonateEventData, BellRingEvent as BellRingEventData,
        BlockBreakEvent as BlockBreakEventData, BlockBrushEvent as BlockBrushEventData,
        BlockBurnEvent as BlockBurnEventData, BlockCanBuildEvent as BlockCanBuildEventData,
        BlockCookEvent as BlockCookEventData, BlockDamageAbortEvent as BlockDamageAbortEventData,
        BlockDamageEvent as BlockDamageEventData,
        BlockDispenseArmorEvent as BlockDispenseArmorEventData,
        BlockDispenseEvent as BlockDispenseEventData,
        BlockDispenseLootEvent as BlockDispenseLootEventData,
        BlockDropItemEvent as BlockDropItemEventData, BlockExpEvent as BlockExpEventData,
        BlockExplodeEvent as BlockExplodeEventData, BlockFadeEvent as BlockFadeEventData,
        BlockFertilizeEvent as BlockFertilizeEventData, BlockFormEvent as BlockFormEventData,
        BlockFromToEvent as BlockFromToEventData, BlockGrowEvent as BlockGrowEventData,
        BlockIgniteEvent as BlockIgniteEventData, BlockMultiPlaceEvent as BlockMultiPlaceEventData,
        BlockPhysicsEvent as BlockPhysicsEventData,
        BlockPistonExtendEvent as BlockPistonExtendEventData,
        BlockPistonRetractEvent as BlockPistonRetractEventData,
        BlockPlaceEvent as BlockPlaceEventData, BlockReceiveGameEvent as BlockReceiveGameEventData,
        BlockRedstoneEvent as BlockRedstoneEventData,
        BlockShearEntityEvent as BlockShearEntityEventData,
        BlockSpreadEvent as BlockSpreadEventData, BrewingStartEvent as BrewingStartEventData,
        CampfireStartEvent as CampfireStartEventData,
        CauldronLevelChangeEvent as CauldronLevelChangeEventData,
        CrafterCraftEvent as CrafterCraftEventData,
        EntityBlockFormEvent as EntityBlockFormEventData, Event,
        FluidLevelChangeEvent as FluidLevelChangeEventData,
        InventoryBlockStartEvent as InventoryBlockStartEventData,
        LeavesDecayEvent as LeavesDecayEventData, MoistureChangeEvent as MoistureChangeEventData,
        NotePlayEvent as NotePlayEventData, SculkBloomEvent as SculkBloomEventData,
        SignChangeEvent as SignChangeEventData, SpongeAbsorbEvent as SpongeAbsorbEventData,
        TntPrimeEvent as TntPrimeEventData, VaultDisplayItemEvent as VaultDisplayItemEventData,
    },
};
use pumpkin_core::plugin::block::{
    bell_resonate::BellResonateEvent,
    bell_ring::BellRingEvent,
    block_break::BlockBreakEvent,
    block_brush::BlockBrushEvent,
    block_burn::BlockBurnEvent,
    block_can_build::BlockCanBuildEvent,
    block_cook::BlockCookEvent,
    block_damage::BlockDamageEvent,
    block_damage_abort::BlockDamageAbortEvent,
    block_dispense::BlockDispenseEvent,
    block_dispense_armor::BlockDispenseArmorEvent,
    block_dispense_loot::BlockDispenseLootEvent,
    block_drop_item::BlockDropItemEvent,
    block_exp::BlockExpEvent,
    block_explode::BlockExplodeEvent,
    block_fade::BlockFadeEvent,
    block_fertilize::BlockFertilizeEvent,
    block_form::BlockFormEvent,
    block_from_to::BlockFromToEvent,
    block_grow::BlockGrowEvent,
    block_ignite::BlockIgniteEvent,
    block_multi_place::BlockMultiPlaceEvent,
    block_physics::BlockPhysicsEvent,
    block_piston::{BlockPistonExtendEvent, BlockPistonRetractEvent},
    block_place::BlockPlaceEvent,
    block_receive_game::BlockReceiveGameEvent,
    block_redstone::BlockRedstoneEvent,
    block_shear_entity::BlockShearEntityEvent,
    block_spread::BlockSpreadEvent,
    brewing_start::BrewingStartEvent,
    campfire_start::CampfireStartEvent,
    cauldron_level_change::CauldronLevelChangeEvent,
    crafter_craft::CrafterCraftEvent,
    entity_block_form::EntityBlockFormEvent,
    fluid_level_change::FluidLevelChangeEvent,
    inventory_block_start::InventoryBlockStartEvent,
    leaves_decay::LeavesDecayEvent,
    moisture_change::MoistureChangeEvent,
    note_play::NotePlayEvent,
    sculk_bloom::SculkBloomEvent,
    sign_change::SignChangeEvent,
    sponge_absorb::SpongeAbsorbEvent,
    tnt_prime::TNTPrimeEvent,
    vault_display_item::VaultDisplayItemEvent,
};
use pumpkin_wasm_host_common::state::PluginHostState;

impl ToFromWasmEvent for BlockRedstoneEvent {
    type Wit = BlockRedstoneEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        Self::Wit {
            target_world,
            state_id: self.block_state_id.as_u16(),
            block_pos: to_wasm_block_position(self.block_pos),
            old_current: self.old_current,
            new_current: self.new_current,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            world: consume_world(state, &data.target_world),
            block_state_id: BlockStateId::new_or_air(data.state_id),
            block_pos: from_wasm_block_position(data.block_pos),
            old_current: data.old_current,
            new_current: data.new_current,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockBreakEvent {
    type Wit = BlockBreakEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = self.player.as_ref().map(|player| {
            state
                .add(player.clone())
                .expect("failed to add player resource")
        });

        Self::Wit {
            player,
            block: to_wasm_block_name(self.block),
            block_pos: to_wasm_block_position(self.block_position),
            exp: self.exp,
            should_drop: self.drop,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: data.player.map(|player| consume_player(state, &player)),
            block: from_wasm_block_name(&data.block),
            block_position: from_wasm_block_position(data.block_pos),
            exp: data.exp,
            drop: data.should_drop,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockBurnEvent {
    type Wit = BlockBurnEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            igniting_block: to_wasm_block_name(self.igniting_block),
            block: to_wasm_block_name(self.block),
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            world: consume_world(state, &data.target_world),
            igniting_block: from_wasm_block_name(&data.igniting_block),
            block: from_wasm_block_name(&data.block),
            block_pos: from_wasm_block_position(data.block_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockCanBuildEvent {
    type Wit = BlockCanBuildEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            block_to_build: to_wasm_block_name(self.block_to_build),
            buildable: self.buildable,
            player,
            block: to_wasm_block_name(self.block),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            block_to_build: from_wasm_block_name(&data.block_to_build),
            buildable: data.buildable,
            player: consume_player(state, &data.player),
            block: from_wasm_block_name(&data.block),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockGrowEvent {
    type Wit = BlockGrowEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        Self::Wit {
            target_world,
            old_block: to_wasm_block_name(self.old_block),
            old_state_id: self.old_state_id.as_u16(),
            new_block: to_wasm_block_name(self.new_block),
            new_state_id: self.new_state_id.as_u16(),
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            world: consume_world(state, &data.target_world),
            old_block: from_wasm_block_name(&data.old_block),
            old_state_id: BlockStateId::new_or_air(data.old_state_id),
            new_block: from_wasm_block_name(&data.new_block),
            new_state_id: BlockStateId::new_or_air(data.new_state_id),
            block_pos: from_wasm_block_position(data.block_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockPlaceEvent {
    type Wit = BlockPlaceEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            block_placed: to_wasm_block_name(self.block_placed),
            block_placed_against: to_wasm_block_name(self.block_placed_against),
            block_pos: to_wasm_block_position(self.block_position),
            can_build: self.can_build,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            block_placed: from_wasm_block_name(&data.block_placed),
            block_placed_against: from_wasm_block_name(&data.block_placed_against),
            block_position: from_wasm_block_position(data.block_pos),
            can_build: data.can_build,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockDamageEvent {
    type Wit = BlockDamageEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            insta_break: self.insta_break,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            block: &pumpkin_data::Block::AIR,
            block_pos: from_wasm_block_position(data.block_pos),
            insta_break: data.insta_break,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockIgniteEvent {
    type Wit = BlockIgniteEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            igniting_block: &pumpkin_data::Block::FIRE,
            player: None,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockFromToEvent {
    type Wit = BlockFromToEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            from_pos: to_wasm_block_position(self.from_pos),
            to_pos: to_wasm_block_position(self.to_pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            from_pos: from_wasm_block_position(data.from_pos),
            to_pos: from_wasm_block_position(data.to_pos),
            block: &pumpkin_data::Block::WATER,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockFormEvent {
    type Wit = BlockFormEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            block: &pumpkin_data::Block::SNOW,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockFadeEvent {
    type Wit = BlockFadeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            block: &pumpkin_data::Block::ICE,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockDispenseEvent {
    type Wit = BlockDispenseEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            item_name: self.item_name.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            item_name: data.item_name,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockExplodeEvent {
    type Wit = BlockExplodeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            yield_rate: self.yield_rate,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            yield_rate: data.yield_rate,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockPhysicsEvent {
    type Wit = BlockPhysicsEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            changed_pos: to_wasm_block_position(self.changed_pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            changed_pos: from_wasm_block_position(data.changed_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockPistonExtendEvent {
    type Wit = BlockPistonExtendEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            direction: self.direction.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            direction: data.direction,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BlockPistonRetractEvent {
    type Wit = BlockPistonRetractEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            direction: self.direction.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            direction: data.direction,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for NotePlayEvent {
    type Wit = NotePlayEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            instrument: self.instrument.clone(),
            note: self.note,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            instrument: data.instrument,
            note: data.note,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for SignChangeEvent {
    type Wit = SignChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            lines: self.lines.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            block_pos: from_wasm_block_position(data.block_pos),
            lines: data.lines,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for SpongeAbsorbEvent {
    type Wit = SpongeAbsorbEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for TNTPrimeEvent {
    type Wit = TntPrimeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            prime_reason: self.prime_reason.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            prime_reason: data.prime_reason,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BellResonateEvent {
    type Wit = BellResonateEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            world: consume_world(state, &data.target_world),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BellRingEvent {
    type Wit = BellRingEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            entity_id: self.entity.as_ref().map(|e| e.get_entity().entity_id),
            direction: self.direction.map(|d| format!("{d:?}")),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct BellRingEvent from WASM");
    }
}

impl ToFromWasmEvent for BlockBrushEvent {
    type Wit = BlockBrushEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let item = state
            .add(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            player,
            item,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct BlockBrushEvent from WASM");
    }
}

impl ToFromWasmEvent for BlockCookEvent {
    type Wit = BlockCookEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let source = state
            .add(Arc::new(Mutex::new(self.source.clone())))
            .expect("failed to add item stack resource");
        let result = state
            .add(Arc::new(Mutex::new(self.result.clone())))
            .expect("failed to add item stack resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            source,
            result,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct BlockCookEvent from WASM");
    }
}

impl ToFromWasmEvent for BlockDamageAbortEvent {
    type Wit = BlockDamageAbortEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let item_in_hand = state
            .add(Arc::new(Mutex::new(self.item_in_hand.clone())))
            .expect("failed to add item stack resource");
        Self::Wit {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            item_in_hand,
        }
    }

    fn apply_wasm_event(&mut self, _data: Self::Wit, _state: &mut PluginHostState) {}

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct BlockDamageAbortEvent from WASM");
    }
}

impl ToFromWasmEvent for BlockDispenseArmorEvent {
    type Wit = BlockDispenseArmorEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let item = state
            .add(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            target_entity_id: self.target.get_entity().entity_id,
            item,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct BlockDispenseArmorEvent from WASM");
    }
}

impl ToFromWasmEvent for BlockDispenseLootEvent {
    type Wit = BlockDispenseLootEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let items = self
            .items
            .iter()
            .map(|i| {
                state
                    .add(Arc::new(Mutex::new(i.clone())))
                    .expect("failed to add item stack resource")
            })
            .collect();
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            items,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct BlockDispenseLootEvent from WASM");
    }
}

impl ToFromWasmEvent for BlockDropItemEvent {
    type Wit = BlockDropItemEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let player = self
            .player
            .as_ref()
            .map(|p| state.add(p.clone()).expect("failed to add player resource"));
        let items = self
            .items
            .iter()
            .map(|i| {
                state
                    .add(Arc::new(Mutex::new(i.clone())))
                    .expect("failed to add item stack resource")
            })
            .collect();
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            player,
            items,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct BlockDropItemEvent from WASM");
    }
}

impl ToFromWasmEvent for BlockExpEvent {
    type Wit = BlockExpEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            exp: self.exp,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.exp = data.exp;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            world: consume_world(state, &data.target_world),
            exp: data.exp,
        }
    }
}

impl ToFromWasmEvent for BlockFertilizeEvent {
    type Wit = BlockFertilizeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let player = self
            .player
            .as_ref()
            .map(|p| state.add(p.clone()).expect("failed to add player resource"));
        let changed_blocks = self
            .changed_blocks
            .iter()
            .map(|(pos, id)| (to_wasm_block_position(*pos), id.as_u16()))
            .collect();
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            player,
            changed_blocks,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct BlockFertilizeEvent from WASM");
    }
}

impl ToFromWasmEvent for BlockMultiPlaceEvent {
    type Wit = BlockMultiPlaceEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let placed_blocks = self
            .placed_blocks
            .iter()
            .map(|(pos, id)| (to_wasm_block_position(*pos), id.as_u16()))
            .collect();
        Self::Wit {
            player,
            target_world,
            placed_blocks,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct BlockMultiPlaceEvent from WASM");
    }
}
impl ToFromWasmEvent for BlockReceiveGameEvent {
    type Wit = BlockReceiveGameEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            game_event: self.game_event.clone(),
            source_entity_id: self
                .source_entity
                .as_ref()
                .map(|e| e.get_entity().entity_id),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct BlockReceiveGameEvent from WASM");
    }
}

impl ToFromWasmEvent for BlockShearEntityEvent {
    type Wit = BlockShearEntityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let item = state
            .add(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            target_entity_id: self.target.get_entity().entity_id,
            item,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct BlockShearEntityEvent from WASM");
    }
}

impl ToFromWasmEvent for BlockSpreadEvent {
    type Wit = BlockSpreadEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            source_pos: to_wasm_block_position(self.source_pos),
            target_pos: to_wasm_block_position(self.target_pos),
            target_world,
            new_state_id: self.new_state_id.as_u16(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.new_state_id = BlockStateId::new_or_air(data.new_state_id);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            source_pos: from_wasm_block_position(data.source_pos),
            target_pos: from_wasm_block_position(data.target_pos),
            world: consume_world(state, &data.target_world),
            new_state_id: BlockStateId::new_or_air(data.new_state_id),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BrewingStartEvent {
    type Wit = BrewingStartEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            brewing_time: self.brewing_time,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.brewing_time = data.brewing_time;
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            world: consume_world(state, &data.target_world),
            brewing_time: data.brewing_time,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for CampfireStartEvent {
    type Wit = CampfireStartEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let item = state
            .add(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            item,
            slot: self.slot,
            cooking_time: self.cooking_time,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cooking_time = data.cooking_time;
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct CampfireStartEvent from WASM");
    }
}

impl ToFromWasmEvent for CauldronLevelChangeEvent {
    type Wit = CauldronLevelChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            old_level: self.old_level,
            new_level: self.new_level,
            reason: format!("{:?}", self.reason),
            entity_id: self.entity.as_ref().map(|e| e.get_entity().entity_id),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.new_level = data.new_level;
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct CauldronLevelChangeEvent from WASM");
    }
}

impl ToFromWasmEvent for CrafterCraftEvent {
    type Wit = CrafterCraftEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let result = state
            .add(Arc::new(Mutex::new(self.result.clone())))
            .expect("failed to add item stack resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            result,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct CrafterCraftEvent from WASM");
    }
}

impl ToFromWasmEvent for EntityBlockFormEvent {
    type Wit = EntityBlockFormEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            entity_id: self.entity.get_entity().entity_id,
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            new_state_id: self.new_state_id.as_u16(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.new_state_id = BlockStateId::new_or_air(data.new_state_id);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct EntityBlockFormEvent from WASM");
    }
}

impl ToFromWasmEvent for FluidLevelChangeEvent {
    type Wit = FluidLevelChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            new_state_id: self.new_state_id.as_u16(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.new_state_id = BlockStateId::new_or_air(data.new_state_id);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            world: consume_world(state, &data.target_world),
            new_state_id: BlockStateId::new_or_air(data.new_state_id),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for InventoryBlockStartEvent {
    type Wit = InventoryBlockStartEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            world: consume_world(state, &data.target_world),
        }
    }
}

impl ToFromWasmEvent for LeavesDecayEvent {
    type Wit = LeavesDecayEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            world: consume_world(state, &data.target_world),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for MoistureChangeEvent {
    type Wit = MoistureChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            new_moisture: self.new_moisture,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.new_moisture = data.new_moisture;
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            world: consume_world(state, &data.target_world),
            new_moisture: data.new_moisture,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for SculkBloomEvent {
    type Wit = SculkBloomEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            charge: self.charge,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.charge = data.charge;
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            block_pos: from_wasm_block_position(data.block_pos),
            world: consume_world(state, &data.target_world),
            charge: data.charge,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for VaultDisplayItemEvent {
    type Wit = VaultDisplayItemEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let item = state
            .add(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");
        Self::Wit {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            item,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct VaultDisplayItemEvent from WASM");
    }
}
