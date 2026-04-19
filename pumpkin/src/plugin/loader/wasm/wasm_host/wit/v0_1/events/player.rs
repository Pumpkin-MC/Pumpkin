use crate::plugin::{
    loader::wasm::wasm_host::{
        state::PluginHostState,
        wit::v0_1::{
            events::{
                ToFromWasmEvent, consume_player, consume_text_component, consume_world,
                from_wasm_entity_interaction_action, from_wasm_entity_type, from_wasm_game_mode,
                from_wasm_hand, from_wasm_position, to_wasm_block_position,
                to_wasm_entity_interaction_action, to_wasm_entity_type, to_wasm_game_mode,
                to_wasm_hand, to_wasm_position,
            },
            pumpkin::plugin::event::{
                Event, InteractAction as WasmInteractAction, PlayerChangeWorldEventData,
                PlayerChangedMainHandEventData, PlayerChatEventData, PlayerCommandSendEventData,
                PlayerCustomPayloadEventData, PlayerEggThrowEventData, PlayerExpChangeEventData,
                PlayerFishEventData, PlayerFishState as WasmPlayerFishState,
                PlayerGamemodeChangeEventData, PlayerInteractEventData,
                PlayerInteractUnknownEntityEventData, PlayerItemHeldEventData, PlayerJoinEventData,
                PlayerLeaveEventData, PlayerLoginEventData, PlayerMoveEventData,
                PlayerPermissionCheckEventData, PlayerTeleportEventData,
                PlayerToggleFlightEventData, PlayerToggleSneakEventData,
                PlayerToggleSprintEventData,
            },
        },
    },
    player::{
        changed_main_hand::PlayerChangedMainHandEvent,
        egg_throw::PlayerEggThrowEvent,
        exp_change::PlayerExpChangeEvent,
        fish::{PlayerFishEvent, PlayerFishState},
        item_held::PlayerItemHeldEvent,
        player_change_world::PlayerChangeWorldEvent,
        player_chat::PlayerChatEvent,
        player_command_send::PlayerCommandSendEvent,
        player_custom_payload::PlayerCustomPayloadEvent,
        player_gamemode_change::PlayerGamemodeChangeEvent,
        player_interact_event::{InteractAction, PlayerInteractEvent},
        player_interact_unknown_entity_event::PlayerInteractUnknownEntityEvent,
        player_join::PlayerJoinEvent,
        player_leave::PlayerLeaveEvent,
        player_login::PlayerLoginEvent,
        player_move::PlayerMoveEvent,
        player_permission_check::PlayerPermissionCheckEvent,
        player_teleport::PlayerTeleportEvent,
        player_toggle_flight_event::PlayerToggleFlightEvent,
        player_toggle_sneak_event::PlayerToggleSneakEvent,
        player_toggle_sprint_event::PlayerToggleSprintEvent,
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

impl ToFromWasmEvent for PlayerJoinEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
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

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerLeaveEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
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

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerLoginEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
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

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerChatEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
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

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerCommandSendEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerCommandSendEvent(PlayerCommandSendEventData {
            player,
            command: self.command.clone(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerPermissionCheckEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerPermissionCheckEvent(PlayerPermissionCheckEventData {
            player,
            permission: self.permission.clone(),
            permission_result: self.result,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerMoveEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
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

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerTeleportEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
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

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerChangeWorldEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
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

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerExpChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerExpChangeEvent(PlayerExpChangeEventData {
            player,
            amount: self.amount,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerExpChangeEvent(data) => Self {
                player: consume_player(state, &data.player),
                amount: data.amount,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for PlayerItemHeldEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
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

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerChangedMainHandEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerChangedMainHandEvent(PlayerChangedMainHandEventData {
            player,
            main_hand: to_wasm_hand(self.main_hand),
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerChangedMainHandEvent(data) => Self {
                player: consume_player(state, &data.player),
                main_hand: from_wasm_hand(data.main_hand),
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for PlayerGamemodeChangeEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
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

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerCustomPayloadEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerCustomPayloadEvent(PlayerCustomPayloadEventData {
            player,
            channel: self.channel.clone(),
            data: self.data.to_vec(),
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerFishEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
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

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
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

impl ToFromWasmEvent for PlayerEggThrowEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerEggThrowEvent(PlayerEggThrowEventData {
            player,
            egg_uuid: self.egg_uuid.to_string(),
            hatching: self.hatching,
            num_hatches: self.num_hatches,
            hatching_type: to_wasm_entity_type(self.hatching_type),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerEggThrowEvent(data) => Self {
                player: consume_player(state, &data.player),
                egg_uuid: uuid::Uuid::parse_str(&data.egg_uuid).expect("invalid egg UUID"),
                hatching: data.hatching,
                num_hatches: data.num_hatches,
                hatching_type: from_wasm_entity_type(&data.hatching_type),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for PlayerInteractUnknownEntityEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerInteractUnknownEntityEvent(PlayerInteractUnknownEntityEventData {
            player,
            entity_id: self.entity_id,
            action: to_wasm_entity_interaction_action(&self.action),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerInteractUnknownEntityEvent(data) => Self {
                player: consume_player(state, &data.player),
                entity_id: data.entity_id,
                action: from_wasm_entity_interaction_action(data.action),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

const fn to_wasm_interact_action(action: &InteractAction) -> WasmInteractAction {
    match action {
        InteractAction::LeftClickBlock => WasmInteractAction::LeftClickBlock,
        InteractAction::LeftClickAir => WasmInteractAction::LeftClickAir,
        InteractAction::RightClickAir => WasmInteractAction::RightClickAir,
        InteractAction::RightClickBlock => WasmInteractAction::RightClickBlock,
    }
}

impl ToFromWasmEvent for PlayerInteractEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerInteractEvent(PlayerInteractEventData {
            player,
            action: to_wasm_interact_action(&self.action),
            clicked_pos: self.clicked_pos.map(to_wasm_block_position),
            block: self.block.name.to_string(),
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, _state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerInteractEvent(_data) => {
                panic!("from_wasm_event for PlayerInteractEvent is not supported")
            }
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for PlayerToggleSneakEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerToggleSneakEvent(PlayerToggleSneakEventData {
            player,
            is_sneaking: self.is_sneaking,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerToggleSneakEvent(data) => Self {
                player: consume_player(state, &data.player),
                is_sneaking: data.is_sneaking,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for PlayerToggleFlightEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerToggleFlightEvent(PlayerToggleFlightEventData {
            player,
            is_flying: self.is_flying,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerToggleFlightEvent(data) => Self {
                player: consume_player(state, &data.player),
                is_flying: data.is_flying,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}

impl ToFromWasmEvent for PlayerToggleSprintEvent {
    fn to_wasm_event(&self, state: &mut PluginHostState) -> Event {
        let player = state
            .add_player(self.player.clone())
            .expect("failed to add player resource");

        Event::PlayerToggleSprintEvent(PlayerToggleSprintEventData {
            player,
            is_sprinting: self.is_sprinting,
            cancelled: self.cancelled,
        })
    }

    fn from_wasm_event(event: Event, state: &mut PluginHostState) -> Self {
        match event {
            Event::PlayerToggleSprintEvent(data) => Self {
                player: consume_player(state, &data.player),
                is_sprinting: data.is_sprinting,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
