use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1_0::{
            events::{
                ToFromV0_1_0WasmEvent, consume_player, consume_text_component, consume_world,
                from_wasm_game_mode, from_wasm_hand, from_wasm_position, to_wasm_game_mode,
                to_wasm_hand, to_wasm_position,
            },
            pumpkin::plugin::event::{
                Event, PlayerChangeWorldEventData, PlayerChangedMainHandEventData,
                PlayerChatEventData, PlayerCommandSendEventData, PlayerCustomPayloadEventData,
                PlayerExpChangeEventData, PlayerFishEventData,
                PlayerFishState as WasmPlayerFishState, PlayerGamemodeChangeEventData,
                PlayerItemHeldEventData, PlayerJoinEventData, PlayerLeaveEventData,
                PlayerLoginEventData, PlayerMoveEventData, PlayerPermissionCheckEventData,
                PlayerTeleportEventData,
            },
        },
    },
    player::{
        changed_main_hand::PlayerChangedMainHandEvent,
        exp_change::PlayerExpChangeEvent,
        fish::{PlayerFishEvent, PlayerFishState},
        item_held::PlayerItemHeldEvent,
        player_change_world::PlayerChangeWorldEvent,
        player_chat::PlayerChatEvent,
        player_command_send::PlayerCommandSendEvent,
        player_custom_payload::PlayerCustomPayloadEvent,
        player_gamemode_change::PlayerGamemodeChangeEvent,
        player_join::PlayerJoinEvent,
        player_leave::PlayerLeaveEvent,
        player_login::PlayerLoginEvent,
        player_move::PlayerMoveEvent,
        player_permission_check::PlayerPermissionCheckEvent,
        player_teleport::PlayerTeleportEvent,
    },
};
use bytes::Bytes;

const fn to_wasm_fish_state(state: PlayerFishState) -> WasmPlayerFishState {
    match state {
        PlayerFishState::Fishing => WasmPlayerFishState::Fishing,
        PlayerFishState::CaughtFish => WasmPlayerFishState::CaughtFish,
        PlayerFishState::CaughtEntity => WasmPlayerFishState::CaughtEntity,
        PlayerFishState::InGround => WasmPlayerFishState::InGround,
        PlayerFishState::FailedAttempt => WasmPlayerFishState::FailedAttempt,
        PlayerFishState::ReelIn => WasmPlayerFishState::ReelIn,
        PlayerFishState::Bite => WasmPlayerFishState::Bite,
    }
}

const fn from_wasm_fish_state(state: WasmPlayerFishState) -> PlayerFishState {
    match state {
        WasmPlayerFishState::Fishing => PlayerFishState::Fishing,
        WasmPlayerFishState::CaughtFish => PlayerFishState::CaughtFish,
        WasmPlayerFishState::CaughtEntity => PlayerFishState::CaughtEntity,
        WasmPlayerFishState::InGround => PlayerFishState::InGround,
        WasmPlayerFishState::FailedAttempt => PlayerFishState::FailedAttempt,
        WasmPlayerFishState::ReelIn => PlayerFishState::ReelIn,
        WasmPlayerFishState::Bite => PlayerFishState::Bite,
    }
}

impl ToFromV0_1_0WasmEvent for PlayerJoinEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");
        let join_message = state
            .add_text_component(self.join_message.clone())
            .expect("failed to add text-component resource");

        Event::PlayerJoinEvent(PlayerJoinEventData {
            player,
            join_message,
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerJoinEvent(data) => Self {
                player: consume_player(state, &data.player),
                join_message: consume_text_component(state, &data.join_message),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerLeaveEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");
        let leave_message = state
            .add_text_component(self.leave_message.clone())
            .expect("failed to add text-component resource");

        Event::PlayerLeaveEvent(PlayerLeaveEventData {
            player,
            leave_message,
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerLeaveEvent(data) => Self {
                player: consume_player(state, &data.player),
                leave_message: consume_text_component(state, &data.leave_message),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerLoginEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");
        let kick_message = state
            .add_text_component(self.kick_message.clone())
            .expect("failed to add text-component resource");

        Event::PlayerLoginEvent(PlayerLoginEventData {
            player,
            kick_message,
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerLoginEvent(data) => Self {
                player: consume_player(state, &data.player),
                kick_message: consume_text_component(state, &data.kick_message),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerChatEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");
        let recipients = self
            .recipients
            .iter()
            .cloned()
            .map(|recipient| {
                state
                    .add_player(recipient)
                    .expect("failed to add player resource")
            })
            .collect();

        Event::PlayerChatEvent(PlayerChatEventData {
            player,
            message: self.message.clone(),
            recipients,
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerChatEvent(data) => Self {
                player: consume_player(state, &data.player),
                message: data.message,
                recipients: data
                    .recipients
                    .into_iter()
                    .map(|recipient| consume_player(state, &recipient))
                    .collect(),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerCommandSendEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerCommandSendEvent(PlayerCommandSendEventData {
            player,
            command: self.command.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerCommandSendEvent(data) => Self {
                player: consume_player(state, &data.player),
                command: data.command,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerPermissionCheckEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerPermissionCheckEvent(PlayerPermissionCheckEventData {
            player,
            permission: self.permission.clone(),
            permission_result: self.result,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerPermissionCheckEvent(data) => Self {
                player: consume_player(state, &data.player),
                permission: data.permission,
                result: data.permission_result,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerMoveEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerMoveEvent(PlayerMoveEventData {
            player,
            from_position: to_wasm_position(self.from),
            to_position: to_wasm_position(self.to),
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerMoveEvent(data) => Self {
                player: consume_player(state, &data.player),
                from: from_wasm_position(data.from_position),
                to: from_wasm_position(data.to_position),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerTeleportEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerTeleportEvent(PlayerTeleportEventData {
            player,
            from_position: to_wasm_position(self.from),
            to_position: to_wasm_position(self.to),
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerTeleportEvent(data) => Self {
                player: consume_player(state, &data.player),
                from: from_wasm_position(data.from_position),
                to: from_wasm_position(data.to_position),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerChangeWorldEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");
        let previous_world = state
            .add_world(self.previous_world.clone())
            .expect("failed to add world resource");
        let new_world = state
            .add_world(self.new_world.clone())
            .expect("failed to add world resource");

        Event::PlayerChangeWorldEvent(PlayerChangeWorldEventData {
            player,
            previous_world,
            new_world,
            position: to_wasm_position(self.position),
            yaw: self.yaw,
            pitch: self.pitch,
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerChangeWorldEvent(data) => Self {
                player: consume_player(state, &data.player),
                previous_world: consume_world(state, &data.previous_world),
                new_world: consume_world(state, &data.new_world),
                position: from_wasm_position(data.position),
                yaw: data.yaw,
                pitch: data.pitch,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerExpChangeEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerExpChangeEvent(PlayerExpChangeEventData {
            player,
            amount: self.amount,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerExpChangeEvent(data) => Self {
                player: consume_player(state, &data.player),
                amount: data.amount,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerItemHeldEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerItemHeldEvent(PlayerItemHeldEventData {
            player,
            previous_slot: self.previous_slot,
            new_slot: self.new_slot,
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerItemHeldEvent(data) => Self {
                player: consume_player(state, &data.player),
                previous_slot: data.previous_slot,
                new_slot: data.new_slot,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerChangedMainHandEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerChangedMainHandEvent(PlayerChangedMainHandEventData {
            player,
            main_hand: to_wasm_hand(self.main_hand),
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerChangedMainHandEvent(data) => Self {
                player: consume_player(state, &data.player),
                main_hand: from_wasm_hand(data.main_hand),
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerGamemodeChangeEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerGamemodeChangeEvent(PlayerGamemodeChangeEventData {
            player,
            previous_gamemode: to_wasm_game_mode(self.previous_gamemode),
            new_gamemode: to_wasm_game_mode(self.new_gamemode),
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerGamemodeChangeEvent(data) => Self {
                player: consume_player(state, &data.player),
                previous_gamemode: from_wasm_game_mode(data.previous_gamemode),
                new_gamemode: from_wasm_game_mode(data.new_gamemode),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerCustomPayloadEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerCustomPayloadEvent(PlayerCustomPayloadEventData {
            player,
            channel: self.channel.clone(),
            data: self.data.to_vec(),
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerCustomPayloadEvent(data) => Self {
                player: consume_player(state, &data.player),
                channel: data.channel,
                data: Bytes::from(data.data),
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromV0_1_0WasmEvent for PlayerFishEvent {
    fn to_v0_1_0_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerFishEvent(PlayerFishEventData {
            player,
            caught_uuid: self.caught_uuid.map(|uuid| uuid.to_string()),
            caught_type: self.caught_type.clone(),
            hook_uuid: self.hook_uuid.to_string(),
            state: to_wasm_fish_state(self.state),
            hand: to_wasm_hand(self.hand),
            exp_to_drop: self.exp_to_drop,
            cancelled: self.cancelled,
        })
    }

    fn from_v0_1_0_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerFishEvent(data) => Self {
                player: consume_player(state, &data.player),
                caught_uuid: data
                    .caught_uuid
                    .map(|uuid| uuid::Uuid::parse_str(&uuid).expect("invalid caught UUID")),
                caught_type: data.caught_type,
                hook_uuid: uuid::Uuid::parse_str(&data.hook_uuid).expect("invalid hook UUID"),
                state: from_wasm_fish_state(data.state),
                hand: from_wasm_hand(data.hand),
                exp_to_drop: data.exp_to_drop,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
