use pumpkin_data::BlockStateId;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::plugin::{
    block::{
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
    },
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{
                ToFromWasmEvent, cleanup_event, consume_player, consume_world,
                from_wasm_block_name, from_wasm_block_position, to_wasm_block_name,
                to_wasm_block_position,
            },
            pumpkin::plugin::event::{
                BellResonateEventData, BellRingEventData, BlockBreakEventData, BlockBrushEventData,
                BlockBurnEventData, BlockCanBuildEventData, BlockCookEventData,
                BlockDamageAbortEventData, BlockDamageEventData, BlockDispenseArmorEventData,
                BlockDispenseEventData, BlockDispenseLootEventData, BlockDropItemEventData,
                BlockExpEventData, BlockExplodeEventData, BlockFadeEventData,
                BlockFertilizeEventData, BlockFormEventData, BlockFromToEventData,
                BlockGrowEventData, BlockIgniteEventData, BlockMultiPlaceEventData,
                BlockPhysicsEventData, BlockPistonExtendEventData, BlockPistonRetractEventData,
                BlockPlaceEventData, BlockReceiveGameEventData, BlockRedstoneEventData,
                BlockShearEntityEventData, BlockSpreadEventData, BrewingStartEventData,
                CampfireStartEventData, CauldronLevelChangeEventData, CrafterCraftEventData,
                EntityBlockFormEventData, Event, FluidLevelChangeEventData,
                InventoryBlockStartEventData, LeavesDecayEventData, MoistureChangeEventData,
                NotePlayEventData, SculkBloomEventData, SignChangeEventData, SpongeAbsorbEventData,
                TntPrimeEventData, VaultDisplayItemEventData,
            },
        },
    },
};

impl ToFromWasmEvent for BlockRedstoneEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        Event::BlockRedstoneEvent(BlockRedstoneEventData {
            target_world,
            state_id: self.block_state_id.as_u16(),
            block_pos: to_wasm_block_position(self.block_pos),
            old_current: self.old_current,
            new_current: self.new_current,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockRedstoneEvent(data) => Self {
                world: consume_world(state, &data.target_world),
                block_state_id: BlockStateId::new_or_air(data.state_id),
                block_pos: from_wasm_block_position(data.block_pos),
                old_current: data.old_current,
                new_current: data.new_current,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockBreakEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = self.player.as_ref().map(|player| {
            state
                .add_player(player.clone())
                .expect("failed to add player resource")
        });

        Event::BlockBreakEvent(BlockBreakEventData {
            player,
            block: to_wasm_block_name(self.block),
            block_pos: to_wasm_block_position(self.block_position),
            exp: self.exp,
            should_drop: self.drop,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockBreakEvent(data) => Self {
                player: data.player.map(|player| consume_player(state, &player)),
                block: from_wasm_block_name(&data.block),
                block_position: from_wasm_block_position(data.block_pos),
                exp: data.exp,
                drop: data.should_drop,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockBurnEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BlockBurnEvent(BlockBurnEventData {
            igniting_block: to_wasm_block_name(self.igniting_block),
            block: to_wasm_block_name(self.block),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockBurnEvent(data) => Self {
                igniting_block: from_wasm_block_name(&data.igniting_block),
                block: from_wasm_block_name(&data.block),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockCanBuildEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::BlockCanBuildEvent(BlockCanBuildEventData {
            block_to_build: to_wasm_block_name(self.block_to_build),
            buildable: self.buildable,
            player,
            block: to_wasm_block_name(self.block),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockCanBuildEvent(data) => Self {
                block_to_build: from_wasm_block_name(&data.block_to_build),
                buildable: data.buildable,
                player: consume_player(state, &data.player),
                block: from_wasm_block_name(&data.block),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockGrowEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        Event::BlockGrowEvent(BlockGrowEventData {
            target_world,
            old_block: to_wasm_block_name(self.old_block),
            old_state_id: self.old_state_id.as_u16(),
            new_block: to_wasm_block_name(self.new_block),
            new_state_id: self.new_state_id.as_u16(),
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockGrowEvent(data) => Self {
                world: consume_world(state, &data.target_world),
                old_block: from_wasm_block_name(&data.old_block),
                old_state_id: BlockStateId::new_or_air(data.old_state_id),
                new_block: from_wasm_block_name(&data.new_block),
                new_state_id: BlockStateId::new_or_air(data.new_state_id),
                block_pos: from_wasm_block_position(data.block_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockPlaceEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::BlockPlaceEvent(BlockPlaceEventData {
            player,
            block_placed: to_wasm_block_name(self.block_placed),
            block_placed_against: to_wasm_block_name(self.block_placed_against),
            block_pos: to_wasm_block_position(self.block_position),
            can_build: self.can_build,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockPlaceEvent(data) => Self {
                player: consume_player(state, &data.player),
                block_placed: from_wasm_block_name(&data.block_placed),
                block_placed_against: from_wasm_block_name(&data.block_placed_against),
                block_position: from_wasm_block_position(data.block_pos),
                can_build: data.can_build,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockDamageEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::BlockDamageEvent(BlockDamageEventData {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            insta_break: self.insta_break,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockDamageEvent(data) => Self {
                player: consume_player(state, &data.player),
                block: &pumpkin_data::Block::AIR,
                block_pos: from_wasm_block_position(data.block_pos),
                insta_break: data.insta_break,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockIgniteEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BlockIgniteEvent(BlockIgniteEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockIgniteEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                igniting_block: &pumpkin_data::Block::FIRE,
                player: None,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockFromToEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BlockFromToEvent(BlockFromToEventData {
            from_pos: to_wasm_block_position(self.from_pos),
            to_pos: to_wasm_block_position(self.to_pos),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockFromToEvent(data) => Self {
                from_pos: from_wasm_block_position(data.from_pos),
                to_pos: from_wasm_block_position(data.to_pos),
                block: &pumpkin_data::Block::WATER,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockFormEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BlockFormEvent(BlockFormEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockFormEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                block: &pumpkin_data::Block::SNOW,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockFadeEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BlockFadeEvent(BlockFadeEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockFadeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                block: &pumpkin_data::Block::ICE,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockDispenseEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BlockDispenseEvent(BlockDispenseEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            item_name: self.item_name.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockDispenseEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                item_name: data.item_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockExplodeEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BlockExplodeEvent(BlockExplodeEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            yield_rate: self.yield_rate,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockExplodeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                yield_rate: data.yield_rate,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockPhysicsEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BlockPhysicsEvent(BlockPhysicsEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            changed_pos: to_wasm_block_position(self.changed_pos),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockPhysicsEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                changed_pos: from_wasm_block_position(data.changed_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockPistonExtendEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BlockPistonExtendEvent(BlockPistonExtendEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            direction: self.direction.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockPistonExtendEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                direction: data.direction,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockPistonRetractEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::BlockPistonRetractEvent(BlockPistonRetractEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            direction: self.direction.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockPistonRetractEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                direction: data.direction,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for NotePlayEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::NotePlayEvent(NotePlayEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            instrument: self.instrument.clone(),
            note: self.note,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::NotePlayEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                instrument: data.instrument,
                note: data.note,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for SignChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");
        Event::SignChangeEvent(SignChangeEventData {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            lines: self.lines.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::SignChangeEvent(data) => Self {
                player: consume_player(state, &data.player),
                block_pos: from_wasm_block_position(data.block_pos),
                lines: data.lines,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for SpongeAbsorbEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::SpongeAbsorbEvent(SpongeAbsorbEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::SpongeAbsorbEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for TNTPrimeEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::TntPrimeEvent(TntPrimeEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            prime_reason: self.prime_reason.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::TntPrimeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                prime_reason: data.prime_reason,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BellResonateEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::BellResonateEvent(BellResonateEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BellResonateEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::BellResonateEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                world: consume_world(state, &data.target_world),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BellRingEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::BellRingEvent(BellRingEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            entity_id: self.entity.as_ref().map(|e| e.get_entity().entity_id),
            direction: self.direction.map(|d| format!("{d:?}")),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BellRingEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BellRingEvent(_) => panic!("Cannot construct BellRingEvent from WASM"),
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockBrushEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");
        let item = state
            .add_item_stack(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");
        Event::BlockBrushEvent(BlockBrushEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            player,
            item,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BlockBrushEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockBrushEvent(_) => panic!("Cannot construct BlockBrushEvent from WASM"),
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockCookEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let source = state
            .add_item_stack(Arc::new(Mutex::new(self.source.clone())))
            .expect("failed to add item stack resource");
        let result = state
            .add_item_stack(Arc::new(Mutex::new(self.result.clone())))
            .expect("failed to add item stack resource");
        Event::BlockCookEvent(BlockCookEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            source,
            result,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BlockCookEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockCookEvent(_) => panic!("Cannot construct BlockCookEvent from WASM"),
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockDamageAbortEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let item_in_hand = state
            .add_item_stack(Arc::new(Mutex::new(self.item_in_hand.clone())))
            .expect("failed to add item stack resource");
        Event::BlockDamageAbortEvent(BlockDamageAbortEventData {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            item_in_hand,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockDamageAbortEvent(_) => {
                panic!("Cannot construct BlockDamageAbortEvent from WASM")
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockDispenseArmorEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let item = state
            .add_item_stack(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");
        Event::BlockDispenseArmorEvent(BlockDispenseArmorEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            target_entity_id: self.target.get_entity().entity_id,
            item,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BlockDispenseArmorEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockDispenseArmorEvent(_) => {
                panic!("Cannot construct BlockDispenseArmorEvent from WASM")
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockDispenseLootEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let items = self
            .items
            .iter()
            .map(|i| {
                state
                    .add_item_stack(Arc::new(Mutex::new(i.clone())))
                    .expect("failed to add item stack resource")
            })
            .collect();
        Event::BlockDispenseLootEvent(BlockDispenseLootEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            items,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BlockDispenseLootEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockDispenseLootEvent(_) => {
                panic!("Cannot construct BlockDispenseLootEvent from WASM")
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockDropItemEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let player = self.player.as_ref().map(|p| {
            state
                .add_player(p.clone())
                .expect("failed to add player resource")
        });
        let items = self
            .items
            .iter()
            .map(|i| {
                state
                    .add_item_stack(Arc::new(Mutex::new(i.clone())))
                    .expect("failed to add item stack resource")
            })
            .collect();
        Event::BlockDropItemEvent(BlockDropItemEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            player,
            items,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BlockDropItemEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockDropItemEvent(_) => panic!("Cannot construct BlockDropItemEvent from WASM"),
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockExpEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::BlockExpEvent(BlockExpEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            exp: self.exp,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BlockExpEvent(data) = event {
            self.exp = data.exp;
        }
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockExpEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                world: consume_world(state, &data.target_world),
                exp: data.exp,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockFertilizeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let player = self.player.as_ref().map(|p| {
            state
                .add_player(p.clone())
                .expect("failed to add player resource")
        });
        let changed_blocks = self
            .changed_blocks
            .iter()
            .map(|(pos, id)| (to_wasm_block_position(*pos), id.as_u16()))
            .collect();
        Event::BlockFertilizeEvent(BlockFertilizeEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            player,
            changed_blocks,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BlockFertilizeEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockFertilizeEvent(_) => {
                panic!("Cannot construct BlockFertilizeEvent from WASM")
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockMultiPlaceEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let placed_blocks = self
            .placed_blocks
            .iter()
            .map(|(pos, id)| (to_wasm_block_position(*pos), id.as_u16()))
            .collect();
        Event::BlockMultiPlaceEvent(BlockMultiPlaceEventData {
            player,
            target_world,
            placed_blocks,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BlockMultiPlaceEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockMultiPlaceEvent(_) => {
                panic!("Cannot construct BlockMultiPlaceEvent from WASM")
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockReceiveGameEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::BlockReceiveGameEvent(BlockReceiveGameEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            game_event: self.game_event.clone(),
            source_entity_id: self
                .source_entity
                .as_ref()
                .map(|e| e.get_entity().entity_id),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BlockReceiveGameEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockReceiveGameEvent(_) => {
                panic!("Cannot construct BlockReceiveGameEvent from WASM")
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockShearEntityEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let item = state
            .add_item_stack(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");
        Event::BlockShearEntityEvent(BlockShearEntityEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            target_entity_id: self.target.get_entity().entity_id,
            item,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BlockShearEntityEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockShearEntityEvent(_) => {
                panic!("Cannot construct BlockShearEntityEvent from WASM")
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BlockSpreadEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::BlockSpreadEvent(BlockSpreadEventData {
            source_pos: to_wasm_block_position(self.source_pos),
            target_pos: to_wasm_block_position(self.target_pos),
            target_world,
            new_state_id: self.new_state_id.as_u16(),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BlockSpreadEvent(data) = event {
            self.new_state_id = BlockStateId::new_or_air(data.new_state_id);
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::BlockSpreadEvent(data) => Self {
                source_pos: from_wasm_block_position(data.source_pos),
                target_pos: from_wasm_block_position(data.target_pos),
                world: consume_world(state, &data.target_world),
                new_state_id: BlockStateId::new_or_air(data.new_state_id),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for BrewingStartEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::BrewingStartEvent(BrewingStartEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            brewing_time: self.brewing_time,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::BrewingStartEvent(data) = event {
            self.brewing_time = data.brewing_time;
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::BrewingStartEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                world: consume_world(state, &data.target_world),
                brewing_time: data.brewing_time,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for CampfireStartEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let item = state
            .add_item_stack(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");
        Event::CampfireStartEvent(CampfireStartEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            item,
            slot: self.slot,
            cooking_time: self.cooking_time,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::CampfireStartEvent(data) = event {
            self.cooking_time = data.cooking_time;
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::CampfireStartEvent(_) => panic!("Cannot construct CampfireStartEvent from WASM"),
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for CauldronLevelChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::CauldronLevelChangeEvent(CauldronLevelChangeEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            old_level: self.old_level,
            new_level: self.new_level,
            reason: format!("{:?}", self.reason),
            entity_id: self.entity.as_ref().map(|e| e.get_entity().entity_id),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::CauldronLevelChangeEvent(data) = event {
            self.new_level = data.new_level;
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::CauldronLevelChangeEvent(_) => {
                panic!("Cannot construct CauldronLevelChangeEvent from WASM")
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for CrafterCraftEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let result = state
            .add_item_stack(Arc::new(Mutex::new(self.result.clone())))
            .expect("failed to add item stack resource");
        Event::CrafterCraftEvent(CrafterCraftEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            result,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::CrafterCraftEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::CrafterCraftEvent(_) => panic!("Cannot construct CrafterCraftEvent from WASM"),
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for EntityBlockFormEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::EntityBlockFormEvent(EntityBlockFormEventData {
            entity_id: self.entity.get_entity().entity_id,
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            new_state_id: self.new_state_id.as_u16(),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::EntityBlockFormEvent(data) = event {
            self.new_state_id = BlockStateId::new_or_air(data.new_state_id);
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityBlockFormEvent(_) => {
                panic!("Cannot construct EntityBlockFormEvent from WASM")
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for FluidLevelChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::FluidLevelChangeEvent(FluidLevelChangeEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            new_state_id: self.new_state_id.as_u16(),
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::FluidLevelChangeEvent(data) = event {
            self.new_state_id = BlockStateId::new_or_air(data.new_state_id);
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::FluidLevelChangeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                world: consume_world(state, &data.target_world),
                new_state_id: BlockStateId::new_or_air(data.new_state_id),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for InventoryBlockStartEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::InventoryBlockStartEvent(InventoryBlockStartEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::InventoryBlockStartEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                world: consume_world(state, &data.target_world),
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for LeavesDecayEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::LeavesDecayEvent(LeavesDecayEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::LeavesDecayEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::LeavesDecayEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                world: consume_world(state, &data.target_world),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for MoistureChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::MoistureChangeEvent(MoistureChangeEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            new_moisture: self.new_moisture,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::MoistureChangeEvent(data) = event {
            self.new_moisture = data.new_moisture;
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::MoistureChangeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                world: consume_world(state, &data.target_world),
                new_moisture: data.new_moisture,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for SculkBloomEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        Event::SculkBloomEvent(SculkBloomEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            charge: self.charge,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::SculkBloomEvent(data) = event {
            self.charge = data.charge;
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::SculkBloomEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                world: consume_world(state, &data.target_world),
                charge: data.charge,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for VaultDisplayItemEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");
        let item = state
            .add_item_stack(Arc::new(Mutex::new(self.item.clone())))
            .expect("failed to add item stack resource");
        Event::VaultDisplayItemEvent(VaultDisplayItemEventData {
            block_pos: to_wasm_block_position(self.block_pos),
            target_world,
            item,
            cancelled: self.cancelled,
        })
    }

    fn apply_wasm_event(&mut self, event: Event, state: &mut PluginHostState) {
        cleanup_event(&event, state);
        if let Event::VaultDisplayItemEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::VaultDisplayItemEvent(_) => {
                panic!("Cannot construct VaultDisplayItemEvent from WASM")
            }
            _ => panic!("unexpected event type"),
        }
    }
}
