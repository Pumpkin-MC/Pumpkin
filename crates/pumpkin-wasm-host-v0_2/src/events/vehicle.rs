use crate::{
    events::{CleanupEvent, ToFromWasmEvent, from_wasm_block_position, to_wasm_block_position},
    pumpkin::plugin::event::{
        VehicleBlockCollisionEvent as VehicleBlockCollisionEventData,
        VehicleCollisionEvent as VehicleCollisionEventData,
        VehicleCreateEvent as VehicleCreateEventData, VehicleDamageEvent as VehicleDamageEventData,
        VehicleDestroyEvent as VehicleDestroyEventData, VehicleEnterEvent as VehicleEnterEventData,
        VehicleEntityCollisionEvent as VehicleEntityCollisionEventData,
        VehicleExitEvent as VehicleExitEventData, VehicleMoveEvent as VehicleMoveEventData,
        VehicleUpdateEvent as VehicleUpdateEventData,
    },
};
use pumpkin_core::plugin::vehicle::{
    vehicle_block_collision::VehicleBlockCollisionEvent, vehicle_collision::VehicleCollisionEvent,
    vehicle_create::VehicleCreateEvent, vehicle_damage::VehicleDamageEvent,
    vehicle_destroy::VehicleDestroyEvent, vehicle_enter::VehicleEnterEvent,
    vehicle_entity_collision::VehicleEntityCollisionEvent, vehicle_exit::VehicleExitEvent,
    vehicle_move::VehicleMoveEvent, vehicle_update::VehicleUpdateEvent,
};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_wasm_host_common::state::PluginHostState;

impl ToFromWasmEvent for VehicleBlockCollisionEvent {
    type Wit = VehicleBlockCollisionEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            vehicle_id: self.vehicle_id,
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            vehicle_id: data.vehicle_id,
            block_pos: from_wasm_block_position(data.block_pos),
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.block_pos = from_wasm_block_position(data.block_pos);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for VehicleCollisionEvent {
    type Wit = VehicleCollisionEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            vehicle_id: self.vehicle_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            vehicle_id: data.vehicle_id,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for VehicleCreateEvent {
    type Wit = VehicleCreateEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            vehicle_id: self.vehicle_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            vehicle_id: data.vehicle_id,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for VehicleDamageEvent {
    type Wit = VehicleDamageEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            vehicle_id: self.vehicle_id,
            damage: self.damage,
            attacker_id: self.attacker_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            vehicle_id: data.vehicle_id,
            damage: data.damage,
            attacker_id: data.attacker_id,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.damage = data.damage;
        self.attacker_id = data.attacker_id;
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for VehicleDestroyEvent {
    type Wit = VehicleDestroyEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            vehicle_id: self.vehicle_id,
            attacker_id: self.attacker_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            vehicle_id: data.vehicle_id,
            attacker_id: data.attacker_id,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.attacker_id = data.attacker_id;
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for VehicleEnterEvent {
    type Wit = VehicleEnterEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            vehicle_id: self.vehicle_id,
            entered_id: self.entered_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            vehicle_id: data.vehicle_id,
            entered_id: data.entered_id,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for VehicleEntityCollisionEvent {
    type Wit = VehicleEntityCollisionEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            vehicle_id: self.vehicle_id,
            collided_entity_id: self.collided_entity_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            vehicle_id: data.vehicle_id,
            collided_entity_id: data.collided_entity_id,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for VehicleExitEvent {
    type Wit = VehicleExitEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            vehicle_id: self.vehicle_id,
            exited_id: self.exited_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            vehicle_id: data.vehicle_id,
            exited_id: data.exited_id,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for VehicleMoveEvent {
    type Wit = VehicleMoveEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            vehicle_id: self.vehicle_id,
            from_position: (self.from.x, self.from.y, self.from.z),
            to_position: (self.to.x, self.to.y, self.to.z),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            vehicle_id: data.vehicle_id,
            from: Vector3::new(
                data.from_position.0,
                data.from_position.1,
                data.from_position.2,
            ),
            to: Vector3::new(data.to_position.0, data.to_position.1, data.to_position.2),
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.from = Vector3::new(
            data.from_position.0,
            data.from_position.1,
            data.from_position.2,
        );
        self.to = Vector3::new(data.to_position.0, data.to_position.1, data.to_position.2);
        self.cancelled = data.cancelled;
    }
}

impl ToFromWasmEvent for VehicleUpdateEvent {
    type Wit = VehicleUpdateEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            vehicle_id: self.vehicle_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            vehicle_id: data.vehicle_id,
            cancelled: data.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }
}
