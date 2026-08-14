use pumpkin_data::BlockStateId;

use crate::plugin::{
    block::{
        block_break::BlockBreakEvent,
        block_burn::BlockBurnEvent,
        block_can_build::BlockCanBuildEvent,
        block_dispense::BlockDispenseEvent,
        block_explode::BlockExplodeEvent,
        block_grow::BlockGrowEvent,
        block_physics::BlockPhysicsEvent,
        block_piston::{BlockPistonExtendEvent, BlockPistonRetractEvent},
        block_place::BlockPlaceEvent,
        block_redstone::BlockRedstoneEvent,
        note_play::NotePlayEvent,
        sign_change::SignChangeEvent,
        sponge_absorb::SpongeAbsorbEvent,
        tnt_prime::TNTPrimeEvent,
    },
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{
                ToFromWasmEvent, consume_player, consume_world, from_wasm_block_name,
                from_wasm_block_position, to_wasm_block_name, to_wasm_block_position,
            },
            pumpkin::plugin::event::{
                BlockBreakEventData, BlockBurnEventData, BlockCanBuildEventData,
                BlockDamageEventData, BlockDispenseEventData, BlockExplodeEventData,
                BlockFadeEventData, BlockFormEventData, BlockFromToEventData, BlockGrowEventData,
                BlockIgniteEventData, BlockPhysicsEventData, BlockPistonExtendEventData,
                BlockPistonRetractEventData, BlockPlaceEventData, BlockRedstoneEventData, Event,
                NotePlayEventData, SignChangeEventData, SpongeAbsorbEventData, TntPrimeEventData,
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

impl ToFromWasmEvent for crate::plugin::api::events::block::block_damage::BlockDamageEvent {
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

impl ToFromWasmEvent for crate::plugin::api::events::block::block_ignite::BlockIgniteEvent {
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

impl ToFromWasmEvent for crate::plugin::api::events::block::block_from_to::BlockFromToEvent {
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

impl ToFromWasmEvent for crate::plugin::api::events::block::block_form::BlockFormEvent {
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

impl ToFromWasmEvent for crate::plugin::api::events::block::block_fade::BlockFadeEvent {
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
