use crate::plugin::{
    entity::{
        entity_combust::EntityCombustEvent,
        entity_damage::EntityDamageEvent,
        entity_death::{EntityDeathEvent, PlayerDeathEvent},
        entity_regain_health::EntityRegainHealthEvent,
        entity_spawn::EntitySpawnEvent,
    },
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{
                ToFromWasmEvent, consume_player, consume_text_component, consume_world,
                to_wasm_position,
            },
            pumpkin::plugin::event::{
                EntityCombustEventData, EntityDamageEventData, EntityDeathEventData,
                EntityRegainHealthEventData, EntitySpawnEventData, Event, PlayerDeathEventData,
            },
        },
    },
};

impl ToFromWasmEvent for EntityDamageEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityDamageEvent(EntityDamageEventData {
            entity_id: self.entity_id,
            damage: self.damage,
            cause: self.cause.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityDamageEvent(data) => Self {
                entity_id: data.entity_id,
                damage: data.damage,
                cause: data.cause,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for EntityDeathEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityDeathEvent(EntityDeathEventData {
            entity_id: self.entity_id,
            dropped_exp: self.dropped_exp,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityDeathEvent(data) => Self {
                entity_id: data.entity_id,
                dropped_exp: data.dropped_exp,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for PlayerDeathEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");
        let death_message = state
            .add_text_component(self.death_message.clone())
            .expect("failed to add text component resource");

        Event::PlayerDeathEvent(PlayerDeathEventData {
            player,
            death_message,
            dropped_exp: self.dropped_exp,
            keep_inventory: self.keep_inventory,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerDeathEvent(data) => Self {
                player: consume_player(state, &data.player),
                death_message: consume_text_component(state, &data.death_message),
                dropped_exp: data.dropped_exp,
                keep_inventory: data.keep_inventory,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for EntitySpawnEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let target_world = state
            .add_world(self.world.clone())
            .expect("failed to add world resource");

        Event::EntitySpawnEvent(EntitySpawnEventData {
            entity_id: self.entity_id,
            entity_type: self.entity_type.clone(),
            position: to_wasm_position(self.position),
            target_world,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::EntitySpawnEvent(data) => {
                let world = consume_world(state, &data.target_world);
                Self {
                    entity_id: data.entity_id,
                    entity_type: data.entity_type,
                    position: pumpkin_util::math::vector3::Vector3::new(
                        data.position.0,
                        data.position.1,
                        data.position.2,
                    ),
                    world,
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for EntityCombustEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityCombustEvent(EntityCombustEventData {
            entity_id: self.entity_id,
            duration_secs: self.duration_secs,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityCombustEvent(data) => Self {
                entity_id: data.entity_id,
                duration_secs: data.duration_secs,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for EntityRegainHealthEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::EntityRegainHealthEvent(EntityRegainHealthEventData {
            entity_id: self.entity_id,
            amount: self.amount,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::EntityRegainHealthEvent(data) => Self {
                entity_id: data.entity_id,
                amount: data.amount,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
