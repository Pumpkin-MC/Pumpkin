use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{ToFromWasmEvent, from_wasm_block_position, to_wasm_block_position},
            pumpkin::plugin::event::{
                Event, VehicleBlockCollisionEventData, VehicleCollisionEventData,
                VehicleCreateEventData, VehicleDamageEventData, VehicleDestroyEventData,
                VehicleEnterEventData, VehicleEntityCollisionEventData, VehicleExitEventData,
                VehicleMoveEventData, VehicleUpdateEventData,
            },
        },
    },
    vehicle::{
        vehicle_block_collision::VehicleBlockCollisionEvent,
        vehicle_collision::VehicleCollisionEvent, vehicle_create::VehicleCreateEvent,
        vehicle_damage::VehicleDamageEvent, vehicle_destroy::VehicleDestroyEvent,
        vehicle_enter::VehicleEnterEvent, vehicle_entity_collision::VehicleEntityCollisionEvent,
        vehicle_exit::VehicleExitEvent, vehicle_move::VehicleMoveEvent,
        vehicle_update::VehicleUpdateEvent,
    },
};
use pumpkin_util::math::vector3::Vector3;

impl ToFromWasmEvent for VehicleBlockCollisionEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::VehicleBlockCollisionEvent(VehicleBlockCollisionEventData {
            vehicle_id: self.vehicle_id,
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::VehicleBlockCollisionEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                block_pos: from_wasm_block_position(data.block_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::VehicleBlockCollisionEvent(data) = event {
            self.block_pos = from_wasm_block_position(data.block_pos);
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for VehicleCollisionEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::VehicleCollisionEvent(VehicleCollisionEventData {
            vehicle_id: self.vehicle_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::VehicleCollisionEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::VehicleCollisionEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for VehicleCreateEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::VehicleCreateEvent(VehicleCreateEventData {
            vehicle_id: self.vehicle_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::VehicleCreateEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::VehicleCreateEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for VehicleDamageEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::VehicleDamageEvent(VehicleDamageEventData {
            vehicle_id: self.vehicle_id,
            damage: self.damage,
            attacker_id: self.attacker_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::VehicleDamageEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                damage: data.damage,
                attacker_id: data.attacker_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::VehicleDamageEvent(data) = event {
            self.damage = data.damage;
            self.attacker_id = data.attacker_id;
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for VehicleDestroyEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::VehicleDestroyEvent(VehicleDestroyEventData {
            vehicle_id: self.vehicle_id,
            attacker_id: self.attacker_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::VehicleDestroyEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                attacker_id: data.attacker_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::VehicleDestroyEvent(data) = event {
            self.attacker_id = data.attacker_id;
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for VehicleEnterEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::VehicleEnterEvent(VehicleEnterEventData {
            vehicle_id: self.vehicle_id,
            entered_id: self.entered_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::VehicleEnterEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                entered_id: data.entered_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::VehicleEnterEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for VehicleEntityCollisionEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::VehicleEntityCollisionEvent(VehicleEntityCollisionEventData {
            vehicle_id: self.vehicle_id,
            collided_entity_id: self.collided_entity_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::VehicleEntityCollisionEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                collided_entity_id: data.collided_entity_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::VehicleEntityCollisionEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for VehicleExitEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::VehicleExitEvent(VehicleExitEventData {
            vehicle_id: self.vehicle_id,
            exited_id: self.exited_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::VehicleExitEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                exited_id: data.exited_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::VehicleExitEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for VehicleMoveEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::VehicleMoveEvent(VehicleMoveEventData {
            vehicle_id: self.vehicle_id,
            from_position: (self.from.x, self.from.y, self.from.z),
            to_position: (self.to.x, self.to.y, self.to.z),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::VehicleMoveEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                from: Vector3::new(
                    data.from_position.0,
                    data.from_position.1,
                    data.from_position.2,
                ),
                to: Vector3::new(data.to_position.0, data.to_position.1, data.to_position.2),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::VehicleMoveEvent(data) = event {
            self.from = Vector3::new(
                data.from_position.0,
                data.from_position.1,
                data.from_position.2,
            );
            self.to = Vector3::new(data.to_position.0, data.to_position.1, data.to_position.2);
            self.cancelled = data.cancelled;
        }
    }
}

impl ToFromWasmEvent for VehicleUpdateEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::VehicleUpdateEvent(VehicleUpdateEventData {
            vehicle_id: self.vehicle_id,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::VehicleUpdateEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }

    fn apply_wasm_event(&mut self, event: Event, _state: &mut PluginHostState) {
        if let Event::VehicleUpdateEvent(data) = event {
            self.cancelled = data.cancelled;
        }
    }
}
