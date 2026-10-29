use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    events::{
        CleanupEvent, ToFromWasmEvent, consume_player, consume_text_component, consume_world,
        from_wasm_block_name, from_wasm_block_position, from_wasm_click_type,
        from_wasm_entity_interaction_action, from_wasm_entity_type, from_wasm_game_mode,
        from_wasm_hand, from_wasm_position, to_wasm_block_position, to_wasm_click_type,
        to_wasm_entity_interaction_action, to_wasm_entity_type, to_wasm_game_mode, to_wasm_hand,
        to_wasm_position,
    },
    gui::{from_wit_screen, to_wit_screen},
    pumpkin::plugin::{
        event::{
            AsyncPlayerChatEvent as AsyncPlayerChatEventData,
            AsyncPlayerPreLoginEvent as AsyncPlayerPreLoginEventData,
            BedrockFormResponseEvent as BedrockFormResponseEventData,
            InteractAction as WasmInteractAction, InventoryClickEvent as InventoryClickEventData,
            InventoryCloseEvent as InventoryCloseEventData,
            PlayerAdvancementDoneEvent as PlayerAdvancementDoneEventData,
            PlayerAnimationEvent as PlayerAnimationEventData,
            PlayerArmorStandManipulateEvent as PlayerArmorStandManipulateEventData,
            PlayerBedEnterEvent as PlayerBedEnterEventData,
            PlayerBedLeaveEvent as PlayerBedLeaveEventData,
            PlayerBucketEmptyEvent as PlayerBucketEmptyEventData,
            PlayerBucketEntityEvent as PlayerBucketEntityEventData,
            PlayerBucketFillEvent as PlayerBucketFillEventData,
            PlayerChangeWorldEvent as PlayerChangeWorldEventData,
            PlayerChangedMainHandEvent as PlayerChangedMainHandEventData,
            PlayerChangedWorldEvent as PlayerChangedWorldEventData,
            PlayerChannelEvent as PlayerChannelEventData, PlayerChatEvent as PlayerChatEventData,
            PlayerCommandPreprocessEvent as PlayerCommandPreprocessEventData,
            PlayerCommandSendEvent as PlayerCommandSendEventData,
            PlayerCustomPayloadEvent as PlayerCustomPayloadEventData,
            PlayerDropItemEvent as PlayerDropItemEventData,
            PlayerEditBookEvent as PlayerEditBookEventData,
            PlayerEggThrowEvent as PlayerEggThrowEventData,
            PlayerElytraBoostEvent as PlayerElytraBoostEventData,
            PlayerExpChangeEvent as PlayerExpChangeEventData,
            PlayerExpCooldownChangeEvent as PlayerExpCooldownChangeEventData,
            PlayerFishEvent as PlayerFishEventData, PlayerFishState as WasmPlayerFishState,
            PlayerGamemodeChangeEvent as PlayerGamemodeChangeEventData,
            PlayerHarvestBlockEvent as PlayerHarvestBlockEventData,
            PlayerHideEntityEvent as PlayerHideEntityEventData,
            PlayerInputEvent as PlayerInputEventData,
            PlayerInteractAtEntityEvent as PlayerInteractAtEntityEventData,
            PlayerInteractEntityEvent as PlayerInteractEntityEventData,
            PlayerInteractEvent as PlayerInteractEventData,
            PlayerInteractUnknownEntityEvent as PlayerInteractUnknownEntityEventData,
            PlayerItemBreakEvent as PlayerItemBreakEventData,
            PlayerItemConsumeEvent as PlayerItemConsumeEventData,
            PlayerItemDamageEvent as PlayerItemDamageEventData,
            PlayerItemHeldEvent as PlayerItemHeldEventData,
            PlayerItemMendEvent as PlayerItemMendEventData, PlayerJoinEvent as PlayerJoinEventData,
            PlayerKickEvent as PlayerKickEventData,
            PlayerLeashEntityEvent as PlayerLeashEntityEventData,
            PlayerLeaveEvent as PlayerLeaveEventData,
            PlayerLevelChangeEvent as PlayerLevelChangeEventData,
            PlayerLinksSendEvent as PlayerLinksSendEventData,
            PlayerLocaleChangeEvent as PlayerLocaleChangeEventData,
            PlayerLoginEvent as PlayerLoginEventData, PlayerMoveEvent as PlayerMoveEventData,
            PlayerNameEntityEvent as PlayerNameEntityEventData,
            PlayerOpenSignEvent as PlayerOpenSignEventData,
            PlayerPermissionCheckEvent as PlayerPermissionCheckEventData,
            PlayerPickupArrowEvent as PlayerPickupArrowEventData,
            PlayerPortalEvent as PlayerPortalEventData,
            PlayerPreLoginEvent as PlayerPreLoginEventData,
            PlayerRecipeBookClickEvent as PlayerRecipeBookClickEventData,
            PlayerRecipeBookSettingsChangeEvent as PlayerRecipeBookSettingsChangeEventData,
            PlayerRecipeDiscoverEvent as PlayerRecipeDiscoverEventData,
            PlayerRegisterChannelEvent as PlayerRegisterChannelEventData,
            PlayerResourcePackStatusEvent as PlayerResourcePackStatusEventData,
            PlayerRespawnEvent as PlayerRespawnEventData,
            PlayerRiptideEvent as PlayerRiptideEventData,
            PlayerShearEntityEvent as PlayerShearEntityEventData,
            PlayerShowEntityEvent as PlayerShowEntityEventData,
            PlayerSpawnChangeEvent as PlayerSpawnChangeEventData,
            PlayerSpawnLocationEvent as PlayerSpawnLocationEventData,
            PlayerStatisticIncrementEvent as PlayerStatisticIncrementEventData,
            PlayerSwapHandsEvent as PlayerSwapHandsEventData,
            PlayerTakeLecternBookEvent as PlayerTakeLecternBookEventData,
            PlayerTeleportEvent as PlayerTeleportEventData,
            PlayerToggleFlightEvent as PlayerToggleFlightEventData,
            PlayerToggleSneakEvent as PlayerToggleSneakEventData,
            PlayerToggleSprintEvent as PlayerToggleSprintEventData,
            PlayerUnleashEntityEvent as PlayerUnleashEntityEventData,
            PlayerUnregisterChannelEvent as PlayerUnregisterChannelEventData,
            PlayerVelocityEvent as PlayerVelocityEventData,
        },
        uuid::Uuid as WitUuid,
    },
    uuid::UuidExt,
};
use bytes::Bytes;
use pumpkin_core::plugin::player::{
    async_player_chat::AsyncPlayerChatEvent,
    async_player_pre_login::AsyncPlayerPreLoginEvent,
    bedrock_form_response::BedrockFormResponseEvent,
    changed_main_hand::PlayerChangedMainHandEvent,
    egg_throw::PlayerEggThrowEvent,
    exp_change::PlayerExpChangeEvent,
    fish::{PlayerFishEvent, PlayerFishState},
    inventory_close::InventoryCloseEvent,
    inventory_interact::InventoryClickEvent,
    item_held::PlayerItemHeldEvent,
    player_advancement_done::PlayerAdvancementDoneEvent,
    player_animation::{PlayerAnimationEvent, PlayerAnimationType},
    player_armor_stand_manipulate::PlayerArmorStandManipulateEvent,
    player_bucket_entity::PlayerBucketEntityEvent,
    player_change_world::PlayerChangeWorldEvent,
    player_changed_world::PlayerChangedWorldEvent,
    player_channel::PlayerChannelEvent,
    player_chat::PlayerChatEvent,
    player_command_preprocess::PlayerCommandPreprocessEvent,
    player_command_send::PlayerCommandSendEvent,
    player_custom_payload::PlayerCustomPayloadEvent,
    player_edit_book::PlayerEditBookEvent,
    player_elytra_boost::PlayerElytraBoostEvent,
    player_exp_cooldown_change::PlayerExpCooldownChangeEvent,
    player_gamemode_change::PlayerGamemodeChangeEvent,
    player_harvest_block::PlayerHarvestBlockEvent,
    player_hide_entity::PlayerHideEntityEvent,
    player_input::PlayerInputEvent,
    player_interact_at_entity::PlayerInteractAtEntityEvent,
    player_interact_entity_event::PlayerInteractEntityEvent,
    player_interact_event::{InteractAction, PlayerInteractEvent},
    player_interact_unknown_entity_event::PlayerInteractUnknownEntityEvent,
    player_item_break::PlayerItemBreakEvent,
    player_item_mend::PlayerItemMendEvent,
    player_join::PlayerJoinEvent,
    player_kick::PlayerKickEvent,
    player_leash_entity::PlayerLeashEntityEvent,
    player_leave::PlayerLeaveEvent,
    player_level_change::PlayerLevelChangeEvent,
    player_links_send::PlayerLinksSendEvent,
    player_locale_change::PlayerLocaleChangeEvent,
    player_login::PlayerLoginEvent,
    player_move::PlayerMoveEvent,
    player_name_entity::PlayerNameEntityEvent,
    player_open_sign::PlayerOpenSignEvent,
    player_permission_check::PlayerPermissionCheckEvent,
    player_pickup_arrow::PlayerPickupArrowEvent,
    player_portal::PlayerPortalEvent,
    player_pre_login::PlayerPreLoginEvent,
    player_recipe_book_click::PlayerRecipeBookClickEvent,
    player_recipe_book_settings_change::PlayerRecipeBookSettingsChangeEvent,
    player_recipe_discover::PlayerRecipeDiscoverEvent,
    player_register_channel::PlayerRegisterChannelEvent,
    player_resource_pack_status::PlayerResourcePackStatusEvent,
    player_respawn::PlayerRespawnEvent,
    player_riptide::PlayerRiptideEvent,
    player_shear_entity::PlayerShearEntityEvent,
    player_show_entity::PlayerShowEntityEvent,
    player_spawn_change::PlayerSpawnChangeEvent,
    player_spawn_location::PlayerSpawnLocationEvent,
    player_statistic_increment::PlayerStatisticIncrementEvent,
    player_swap_hands::PlayerSwapHandItemsEvent,
    player_take_lectern_book::PlayerTakeLecternBookEvent,
    player_teleport::PlayerTeleportEvent,
    player_toggle_flight_event::PlayerToggleFlightEvent,
    player_toggle_sneak_event::PlayerToggleSneakEvent,
    player_toggle_sprint_event::PlayerToggleSprintEvent,
    player_unleash_entity::PlayerUnleashEntityEvent,
    player_unregister_channel::PlayerUnregisterChannelEvent,
    player_velocity::PlayerVelocityEvent,
};
use pumpkin_wasm_host_common::state::PluginHostState;

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

impl ToFromWasmEvent for InventoryCloseEvent {
    type Wit = InventoryCloseEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            window_type: self.window_type.map(to_wit_screen),
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            window_type: data.window_type.map(from_wit_screen),
        }
    }
}

impl ToFromWasmEvent for InventoryClickEvent {
    type Wit = InventoryClickEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            window_type: self.window_type.map(to_wit_screen),
            click_type: to_wasm_click_type(self.click_type),
            slot: self.slot,
            raw_slot: self.raw_slot,
            clicked_item: self.clicked_item.as_ref().map(|stack| {
                state
                    .add(Arc::new(Mutex::new(stack.clone())))
                    .expect("failed to add item stack resource")
            }),
            cursor: self.cursor.as_ref().map(|stack| {
                state
                    .add(Arc::new(Mutex::new(stack.clone())))
                    .expect("failed to add item stack resource")
            }),
            hotbar_button: self.hotbar_button,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            window_type: data.window_type.map(from_wit_screen),
            click_type: from_wasm_click_type(data.click_type),
            slot: data.slot,
            raw_slot: data.raw_slot,
            clicked_item: None, // We don't change clicked_item from WASM
            cursor: None,       // We don't change cursor from WASM
            hotbar_button: data.hotbar_button,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerJoinEvent {
    type Wit = PlayerJoinEventData;
    fn to_wasm_event(&self, state: &mut PluginHostState) -> PlayerJoinEventData {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let join_message = state
            .add(self.join_message.clone())
            .expect("failed to add text-component resource");

        PlayerJoinEventData {
            player,
            join_message,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(event: PlayerJoinEventData, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &event.player),
            join_message: consume_text_component(state, &event.join_message),
            cancelled: event.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerLeaveEvent {
    type Wit = PlayerLeaveEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let leave_message = state
            .add(self.leave_message.clone())
            .expect("failed to add text-component resource");

        Self::Wit {
            player,
            leave_message,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            leave_message: consume_text_component(state, &data.leave_message),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerLoginEvent {
    type Wit = PlayerLoginEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let kick_message = state
            .add(self.kick_message.clone())
            .expect("failed to add text-component resource");

        Self::Wit {
            player,
            kick_message,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            kick_message: consume_text_component(state, &data.kick_message),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerChatEvent {
    type Wit = PlayerChatEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let recipients = self
            .recipients
            .iter()
            .cloned()
            .map(|recipient| state.add(recipient).expect("failed to add player resource"))
            .collect();

        Self::Wit {
            player,
            message: self.message.clone(),
            recipients,
            signature: self.signature.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            message: data.message,
            recipients: data
                .recipients
                .into_iter()
                .map(|recipient| consume_player(state, &recipient))
                .collect(),
            signature: data.signature,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerCommandSendEvent {
    type Wit = PlayerCommandSendEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            command: self.command.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            command: data.command,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerPermissionCheckEvent {
    type Wit = PlayerPermissionCheckEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            permission: self.permission.clone(),
            permission_result: self.result,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            permission: data.permission,
            result: data.permission_result,
        }
    }
}

impl ToFromWasmEvent for PlayerMoveEvent {
    type Wit = PlayerMoveEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            from_position: to_wasm_position(self.from),
            to_position: to_wasm_position(self.to),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            from: from_wasm_position(data.from_position),
            to: from_wasm_position(data.to_position),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerTeleportEvent {
    type Wit = PlayerTeleportEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            from_position: to_wasm_position(self.from),
            to_position: to_wasm_position(self.to),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            from: from_wasm_position(data.from_position),
            to: from_wasm_position(data.to_position),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerChangeWorldEvent {
    type Wit = PlayerChangeWorldEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let previous_world = state
            .add(self.previous_world.clone())
            .expect("failed to add world resource");
        let new_world = state
            .add(self.new_world.clone())
            .expect("failed to add world resource");

        Self::Wit {
            player,
            previous_world,
            new_world,
            position: to_wasm_position(self.position),
            yaw: self.yaw,
            pitch: self.pitch,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            previous_world: consume_world(state, &data.previous_world),
            new_world: consume_world(state, &data.new_world),
            position: from_wasm_position(data.position),
            yaw: data.yaw,
            pitch: data.pitch,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerRespawnEvent {
    type Wit = PlayerRespawnEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let previous_world = state
            .add(self.previous_world.clone())
            .expect("failed to add world resource");
        let respawned_world = state
            .add(self.respawned_world.clone())
            .expect("failed to add world resource");

        Self::Wit {
            player,
            previous_world,
            respawned_world,
            position: to_wasm_position(self.position),
            yaw: self.yaw,
            pitch: self.pitch,
            alive: self.alive,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            previous_world: consume_world(state, &data.previous_world),
            respawned_world: consume_world(state, &data.respawned_world),
            position: from_wasm_position(data.position),
            yaw: data.yaw,
            pitch: data.pitch,
            alive: data.alive,
        }
    }
}

impl ToFromWasmEvent for PlayerExpChangeEvent {
    type Wit = PlayerExpChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            amount: self.amount,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            amount: data.amount,
        }
    }
}

impl ToFromWasmEvent for PlayerItemHeldEvent {
    type Wit = PlayerItemHeldEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            previous_slot: self.previous_slot,
            new_slot: self.new_slot,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            previous_slot: data.previous_slot,
            new_slot: data.new_slot,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerChangedMainHandEvent {
    type Wit = PlayerChangedMainHandEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            main_hand: to_wasm_hand(self.main_hand),
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            main_hand: from_wasm_hand(data.main_hand),
        }
    }
}

impl ToFromWasmEvent for PlayerGamemodeChangeEvent {
    type Wit = PlayerGamemodeChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            previous_gamemode: to_wasm_game_mode(self.previous_gamemode),
            new_gamemode: to_wasm_game_mode(self.new_gamemode),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            previous_gamemode: from_wasm_game_mode(data.previous_gamemode),
            new_gamemode: from_wasm_game_mode(data.new_gamemode),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerCustomPayloadEvent {
    type Wit = PlayerCustomPayloadEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            channel: self.channel.clone(),
            data: self.data.to_vec(),
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            channel: data.channel,
            data: Bytes::from(data.data),
        }
    }
}

impl ToFromWasmEvent for PlayerFishEvent {
    type Wit = PlayerFishEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            caught_uuid: self.caught_uuid.as_ref().map(WitUuid::to_wit),
            caught_type: self.caught_type.clone(),
            hook_uuid: WitUuid::to_wit(&self.hook_uuid),
            state: to_wasm_fish_state(self.state),
            hand: to_wasm_hand(self.hand),
            exp_to_drop: self.exp_to_drop,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            caught_uuid: data.caught_uuid.map(|id| WitUuid::from_wit(&id)),
            caught_type: data.caught_type,
            hook_uuid: WitUuid::from_wit(&data.hook_uuid),
            state: from_wasm_fish_state(data.state),
            hand: from_wasm_hand(data.hand),
            exp_to_drop: data.exp_to_drop,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerEggThrowEvent {
    type Wit = PlayerEggThrowEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            egg_uuid: WitUuid::to_wit(&self.egg_uuid),
            hatching: self.hatching,
            num_hatches: self.num_hatches,
            hatching_type: to_wasm_entity_type(self.hatching_type),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            egg_uuid: WitUuid::from_wit(&data.egg_uuid),
            hatching: data.hatching,
            num_hatches: data.num_hatches,
            hatching_type: from_wasm_entity_type(&data.hatching_type),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerInteractUnknownEntityEvent {
    type Wit = PlayerInteractUnknownEntityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            entity_id: self.entity_id,
            action: to_wasm_entity_interaction_action(self.action),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            entity_id: data.entity_id,
            action: from_wasm_entity_interaction_action(data.action),
            cancelled: data.cancelled,
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

const fn from_wasm_interact_action(action: WasmInteractAction) -> InteractAction {
    match action {
        WasmInteractAction::LeftClickBlock => InteractAction::LeftClickBlock,
        WasmInteractAction::LeftClickAir => InteractAction::LeftClickAir,
        WasmInteractAction::RightClickAir => InteractAction::RightClickAir,
        WasmInteractAction::RightClickBlock => InteractAction::RightClickBlock,
    }
}

impl ToFromWasmEvent for PlayerInteractEvent {
    type Wit = PlayerInteractEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            action: to_wasm_interact_action(&self.action),
            clicked_pos: self.clicked_pos.map(to_wasm_block_position),
            block: self.block.name.to_string(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            action: from_wasm_interact_action(data.action),
            clicked_pos: data.clicked_pos.map(from_wasm_block_position),
            block: from_wasm_block_name(&data.block),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerToggleSneakEvent {
    type Wit = PlayerToggleSneakEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            is_sneaking: self.is_sneaking,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            is_sneaking: data.is_sneaking,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerToggleFlightEvent {
    type Wit = PlayerToggleFlightEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            is_flying: self.is_flying,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            is_flying: data.is_flying,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerToggleSprintEvent {
    type Wit = PlayerToggleSprintEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            is_sprinting: self.is_sprinting,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            is_sprinting: data.is_sprinting,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BedrockFormResponseEvent {
    type Wit = BedrockFormResponseEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            form_id: self.form_id,
            response_data: self.response_data.clone(),
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            form_id: data.form_id,
            response_data: data.response_data,
        }
    }
}

impl ToFromWasmEvent for PlayerInteractEntityEvent {
    type Wit = PlayerInteractEntityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            entity_id: self.target.get_entity().entity_id,
            action: to_wasm_entity_interaction_action(self.action),
            sneaking: self.sneaking,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        let player = consume_player(state, &data.player);
        let target = player
            .world()
            .get_entity_by_id(data.entity_id)
            .expect("entity not found");
        Self {
            player,
            target,
            action: from_wasm_entity_interaction_action(data.action),
            target_position: None,
            sneaking: data.sneaking,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::player::player_item_consume::PlayerItemConsumeEvent
{
    type Wit = PlayerItemConsumeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            item_name: self.item_name.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            item_name: data.item_name,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::player::player_item_damage::PlayerItemDamageEvent
{
    type Wit = PlayerItemDamageEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            item_name: self.item_name.clone(),
            damage: self.damage,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            item_name: data.item_name,
            damage: data.damage,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::player::player_drop_item::PlayerDropItemEvent
{
    type Wit = PlayerDropItemEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            item_name: self.item_name.clone(),
            count: self.count,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            item_name: data.item_name,
            count: data.count,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::player::player_bed::PlayerBedEnterEvent
{
    type Wit = PlayerBedEnterEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            bed_pos: to_wasm_block_position(self.bed_pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            bed_pos: from_wasm_block_position(data.bed_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::player::player_bed::PlayerBedLeaveEvent
{
    type Wit = PlayerBedLeaveEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            bed_pos: to_wasm_block_position(self.bed_pos),
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            bed_pos: from_wasm_block_position(data.bed_pos),
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::player::player_bucket::PlayerBucketEmptyEvent
{
    type Wit = PlayerBucketEmptyEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            bucket: self.bucket.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            block_pos: from_wasm_block_position(data.block_pos),
            bucket: data.bucket,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::player::player_bucket::PlayerBucketFillEvent
{
    type Wit = PlayerBucketFillEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");

        Self::Wit {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            bucket: self.bucket.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            block_pos: from_wasm_block_position(data.block_pos),
            bucket: data.bucket,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for AsyncPlayerChatEvent {
    type Wit = AsyncPlayerChatEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let format = state
            .add(self.format.clone())
            .expect("failed to add text-component resource");
        Self::Wit {
            player,
            message: self.message.clone(),
            format,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        let returned = Self::from_wasm_event(data, state);
        self.cancelled = returned.cancelled;
        self.message = returned.message;
        self.format = returned.format;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            message: data.message,
            format: consume_text_component(state, &data.format),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for AsyncPlayerPreLoginEvent {
    type Wit = AsyncPlayerPreLoginEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let kick_message = state
            .add(self.kick_message.clone())
            .expect("failed to add text-component resource");
        Self::Wit {
            player_name: self.player_name.clone(),
            player_uuid: self.player_uuid.to_string(),
            ip_address: self.ip_address.to_string(),
            kick_message,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        let returned = Self::from_wasm_event(data, state);
        self.cancelled = returned.cancelled;
        self.kick_message = returned.kick_message;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player_name: data.player_name,
            player_uuid: data.player_uuid.parse().unwrap_or_default(),
            ip_address: data.ip_address.parse().unwrap_or_else(|_| {
                std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 0)
            }),
            kick_message: consume_text_component(state, &data.kick_message),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerAdvancementDoneEvent {
    type Wit = PlayerAdvancementDoneEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            advancement_id: self.advancement_id.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            advancement_id: data.advancement_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerAnimationEvent {
    type Wit = PlayerAnimationEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let animation_type = match self.animation_type {
            PlayerAnimationType::ArmSwingOff => "ArmSwingOff".to_string(),
            PlayerAnimationType::LeaveBed => "LeaveBed".to_string(),
            PlayerAnimationType::ArmSwingMain => "ArmSwingMain".to_string(),
        };
        Self::Wit {
            player,
            animation_type,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            animation_type: match data.animation_type.as_str() {
                "ArmSwingOff" => PlayerAnimationType::ArmSwingOff,
                "LeaveBed" => PlayerAnimationType::LeaveBed,
                _ => PlayerAnimationType::ArmSwingMain,
            },
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerArmorStandManipulateEvent {
    type Wit = PlayerArmorStandManipulateEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            armor_stand_id: self.armor_stand_id,
            slot: self.slot,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            armor_stand_id: data.armor_stand_id,
            slot: data.slot,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerBucketEntityEvent {
    type Wit = PlayerBucketEntityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            entity_id: self.entity_id,
            bucket_item: self.bucket_item.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            entity_id: data.entity_id,
            bucket_item: data.bucket_item,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerChangedWorldEvent {
    type Wit = PlayerChangedWorldEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let from_world = state
            .add(self.from_world.clone())
            .expect("failed to add world resource");
        let to_world = state
            .add(self.to_world.clone())
            .expect("failed to add world resource");
        Self::Wit {
            player,
            from_world,
            to_world,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            from_world: consume_world(state, &data.from_world),
            to_world: consume_world(state, &data.to_world),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerChannelEvent {
    type Wit = PlayerChannelEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            channel: self.channel.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            channel: data.channel,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerCommandPreprocessEvent {
    type Wit = PlayerCommandPreprocessEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            command: self.command.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.command = data.command;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            command: data.command,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerEditBookEvent {
    type Wit = PlayerEditBookEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            slot: self.slot,
            pages: self.pages.clone(),
            title: self.title.clone(),
            signing: self.signing,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.pages = data.pages;
        self.title = data.title;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            slot: data.slot,
            pages: data.pages,
            title: data.title,
            signing: data.signing,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerElytraBoostEvent {
    type Wit = PlayerElytraBoostEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            firework_id: self.firework_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            firework_id: data.firework_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerExpCooldownChangeEvent {
    type Wit = PlayerExpCooldownChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            new_cooldown: self.new_cooldown,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.new_cooldown = data.new_cooldown;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            new_cooldown: data.new_cooldown,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerHarvestBlockEvent {
    type Wit = PlayerHarvestBlockEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let harvested_items = self
            .harvested_items
            .iter()
            .map(|i| {
                state
                    .add(Arc::new(Mutex::new(i.clone())))
                    .expect("failed to add item stack resource")
            })
            .collect();
        Self::Wit {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            harvested_items,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct PlayerHarvestBlockEvent from WASM")
    }
}

impl ToFromWasmEvent for PlayerHideEntityEvent {
    type Wit = PlayerHideEntityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            entity_id: self.entity_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            entity_id: data.entity_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerItemBreakEvent {
    type Wit = PlayerItemBreakEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            item_name: self.item_name.clone(),
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            item_name: data.item_name,
        }
    }
}

impl ToFromWasmEvent for PlayerItemMendEvent {
    type Wit = PlayerItemMendEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            item_name: self.item_name.clone(),
            repair_amount: self.repair_amount,
            exp_consumed: self.exp_consumed,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.repair_amount = data.repair_amount;
        self.exp_consumed = data.exp_consumed;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            item_name: data.item_name,
            repair_amount: data.repair_amount,
            exp_consumed: data.exp_consumed,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerKickEvent {
    type Wit = PlayerKickEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            reason: self.reason.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.reason = data.reason;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            reason: data.reason,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerLeashEntityEvent {
    type Wit = PlayerLeashEntityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            entity_id: self.entity_id,
            holder_id: self.holder_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            entity_id: data.entity_id,
            holder_id: data.holder_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerLevelChangeEvent {
    type Wit = PlayerLevelChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            old_level: self.old_level,
            new_level: self.new_level,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            old_level: data.old_level,
            new_level: data.new_level,
        }
    }
}

impl ToFromWasmEvent for PlayerLocaleChangeEvent {
    type Wit = PlayerLocaleChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            new_locale: self.new_locale.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            new_locale: data.new_locale,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerNameEntityEvent {
    type Wit = PlayerNameEntityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let name = state
            .add(self.name.clone())
            .expect("failed to add text-component resource");
        Self::Wit {
            player,
            entity_id: self.entity_id,
            name,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        let returned = Self::from_wasm_event(data, state);
        self.cancelled = returned.cancelled;
        self.name = returned.name;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            entity_id: data.entity_id,
            name: consume_text_component(state, &data.name),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerOpenSignEvent {
    type Wit = PlayerOpenSignEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            is_front: self.is_front,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            block_pos: from_wasm_block_position(data.block_pos),
            is_front: data.is_front,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerPortalEvent {
    type Wit = PlayerPortalEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            from_pos: to_wasm_block_position(self.from_pos),
            to_pos: self.to_pos.map(to_wasm_block_position),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.to_pos = data.to_pos.map(from_wasm_block_position);
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            from_pos: from_wasm_block_position(data.from_pos),
            to_pos: data.to_pos.map(from_wasm_block_position),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerPreLoginEvent {
    type Wit = PlayerPreLoginEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let kick_message = state
            .add(self.kick_message.clone())
            .expect("failed to add text-component resource");
        Self::Wit {
            player_name: self.player_name.clone(),
            player_uuid: self.player_uuid.to_string(),
            ip_address: self.ip_address.to_string(),
            kick_message,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        let returned = Self::from_wasm_event(data, state);
        self.cancelled = returned.cancelled;
        self.kick_message = returned.kick_message;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player_name: data.player_name,
            player_uuid: data.player_uuid.parse().unwrap_or_default(),
            ip_address: data.ip_address.parse().unwrap_or_else(|_| {
                std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 0)
            }),
            kick_message: consume_text_component(state, &data.kick_message),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerRiptideEvent {
    type Wit = PlayerRiptideEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            item_name: self.item_name.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            item_name: data.item_name,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerShearEntityEvent {
    type Wit = PlayerShearEntityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            entity_id: self.entity_id,
            hand: self.hand,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            entity_id: data.entity_id,
            hand: data.hand,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerShowEntityEvent {
    type Wit = PlayerShowEntityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            entity_id: self.entity_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            entity_id: data.entity_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerSpawnChangeEvent {
    type Wit = PlayerSpawnChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            new_spawn: self.new_spawn.map(to_wasm_block_position),
            forced: self.forced,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.new_spawn = data.new_spawn.map(from_wasm_block_position);
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            new_spawn: data.new_spawn.map(from_wasm_block_position),
            forced: data.forced,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerStatisticIncrementEvent {
    type Wit = PlayerStatisticIncrementEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            statistic_id: self.statistic_id.clone(),
            amount: self.amount,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.amount = data.amount;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            statistic_id: data.statistic_id,
            amount: data.amount,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerSwapHandItemsEvent {
    type Wit = PlayerSwapHandsEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerTakeLecternBookEvent {
    type Wit = PlayerTakeLecternBookEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let book = state
            .add(Arc::new(Mutex::new(self.book.clone())))
            .expect("failed to add item stack resource");
        Self::Wit {
            player,
            block_pos: to_wasm_block_position(self.block_pos),
            book,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct PlayerTakeLecternBookEvent from WASM")
    }
}

impl ToFromWasmEvent for PlayerUnleashEntityEvent {
    type Wit = PlayerUnleashEntityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            entity_id: self.entity_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            entity_id: data.entity_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerVelocityEvent {
    type Wit = PlayerVelocityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            velocity: to_wasm_position(self.velocity),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.velocity = from_wasm_position(data.velocity);
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            velocity: from_wasm_position(data.velocity),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerInputEvent {
    type Wit = PlayerInputEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            input: self.input.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.input = data.input;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            input: data.input,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerInteractAtEntityEvent {
    type Wit = PlayerInteractAtEntityEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            entity_id: self.entity_id,
            clicked_x: self.clicked_x,
            clicked_y: self.clicked_y,
            clicked_z: self.clicked_z,
            hand: self.hand,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            entity_id: data.entity_id,
            clicked_x: data.clicked_x,
            clicked_y: data.clicked_y,
            clicked_z: data.clicked_z,
            hand: data.hand,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerLinksSendEvent {
    type Wit = PlayerLinksSendEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            links: self.links.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.links = data.links;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            links: data.links,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerPickupArrowEvent {
    type Wit = PlayerPickupArrowEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            arrow_id: self.arrow_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            arrow_id: data.arrow_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerRecipeBookClickEvent {
    type Wit = PlayerRecipeBookClickEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            recipe_id: self.recipe_id.clone(),
            make_all: self.make_all,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.recipe_id = data.recipe_id;
        self.make_all = data.make_all;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            recipe_id: data.recipe_id,
            make_all: data.make_all,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerRecipeBookSettingsChangeEvent {
    type Wit = PlayerRecipeBookSettingsChangeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            book_type: self.book_type.clone(),
            is_open: self.is_open,
            is_filtering: self.is_filtering,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.book_type = data.book_type;
        self.is_open = data.is_open;
        self.is_filtering = data.is_filtering;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            book_type: data.book_type,
            is_open: data.is_open,
            is_filtering: data.is_filtering,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerRecipeDiscoverEvent {
    type Wit = PlayerRecipeDiscoverEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            recipe_id: self.recipe_id.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.recipe_id = data.recipe_id;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            recipe_id: data.recipe_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerRegisterChannelEvent {
    type Wit = PlayerRegisterChannelEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            channel: self.channel.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.channel = data.channel;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            channel: data.channel,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerResourcePackStatusEvent {
    type Wit = PlayerResourcePackStatusEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            pack_id: self.pack_id.clone(),
            status: self.status.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.status = data.status;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            pack_id: data.pack_id,
            status: data.status,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerSpawnLocationEvent {
    type Wit = PlayerSpawnLocationEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            spawn_pos: to_wasm_position(self.spawn_pos),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.spawn_pos = from_wasm_position(data.spawn_pos);
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            spawn_pos: from_wasm_position(data.spawn_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PlayerUnregisterChannelEvent {
    type Wit = PlayerUnregisterChannelEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        Self::Wit {
            player,
            channel: self.channel.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, state: &mut PluginHostState) {
        data.cleanup_event(state);
        self.cancelled = data.cancelled;
        self.channel = data.channel;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            channel: data.channel,
            cancelled: data.cancelled,
        }
    }
}
