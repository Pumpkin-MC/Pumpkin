use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{ToFromWasmEvent, consume_text_component},
            pumpkin::plugin::event::{Event, ServerBroadcastEventData, ServerCommandEventData},
        },
    },
    server::{server_broadcast::ServerBroadcastEvent, server_command::ServerCommandEvent},
};

impl ToFromWasmEvent for ServerCommandEvent {
    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Event {
        Event::ServerCommandEvent(ServerCommandEventData {
            command: self.command.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::ServerCommandEvent(data) => Self {
                command: data.command,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for ServerBroadcastEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let message = state
            .add_text_component(self.message.clone())
            .expect("failed to add text-component resource");
        let sender = state
            .add_text_component(self.sender.clone())
            .expect("failed to add text-component resource");

        Event::ServerBroadcastEvent(ServerBroadcastEventData {
            message,
            sender,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::ServerBroadcastEvent(data) => Self {
                message: consume_text_component(state, &data.message),
                sender: consume_text_component(state, &data.sender),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
