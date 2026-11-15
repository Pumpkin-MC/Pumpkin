/* This file is generated from WIT. Do not edit manually. */
#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::too_many_lines,
    clippy::match_same_arms
)]
use super::{
    ToFromWasmEvent, WasmPluginEventHandler, cleanup_entity, cleanup_item_stack, cleanup_player,
    cleanup_text_component, cleanup_world, consume_entity, consume_item_stack, consume_player,
    consume_text_component, consume_world, from_wasm_block_name, from_wasm_block_position,
    from_wasm_click_type, from_wasm_entity_interaction_action, from_wasm_entity_type,
    from_wasm_game_mode, from_wasm_hand, from_wasm_position, to_wasm_block_name,
    to_wasm_block_position, to_wasm_click_type, to_wasm_entity_interaction_action,
    to_wasm_entity_type, to_wasm_game_mode, to_wasm_hand, to_wasm_position,
};
const fn to_wit_player_fish_state(
    val: pumpkin_core::plugin::PlayerFishState,
) -> crate::pumpkin::plugin::event::PlayerFishState {
    match val {
        pumpkin_core::plugin::PlayerFishState::Fishing => {
            crate::pumpkin::plugin::event::PlayerFishState::Fishing
        }
        pumpkin_core::plugin::PlayerFishState::CaughtFish => {
            crate::pumpkin::plugin::event::PlayerFishState::CaughtFish
        }
        pumpkin_core::plugin::PlayerFishState::CaughtEntity => {
            crate::pumpkin::plugin::event::PlayerFishState::CaughtEntity
        }
        pumpkin_core::plugin::PlayerFishState::InGround => {
            crate::pumpkin::plugin::event::PlayerFishState::InGround
        }
        pumpkin_core::plugin::PlayerFishState::FailedAttempt => {
            crate::pumpkin::plugin::event::PlayerFishState::FailedAttempt
        }
        pumpkin_core::plugin::PlayerFishState::ReelIn => {
            crate::pumpkin::plugin::event::PlayerFishState::ReelIn
        }
        pumpkin_core::plugin::PlayerFishState::Bite => {
            crate::pumpkin::plugin::event::PlayerFishState::Bite
        }
    }
}
const fn from_wit_player_fish_state(
    val: crate::pumpkin::plugin::event::PlayerFishState,
) -> pumpkin_core::plugin::PlayerFishState {
    match val {
        crate::pumpkin::plugin::event::PlayerFishState::Fishing => {
            pumpkin_core::plugin::PlayerFishState::Fishing
        }
        crate::pumpkin::plugin::event::PlayerFishState::CaughtFish => {
            pumpkin_core::plugin::PlayerFishState::CaughtFish
        }
        crate::pumpkin::plugin::event::PlayerFishState::CaughtEntity => {
            pumpkin_core::plugin::PlayerFishState::CaughtEntity
        }
        crate::pumpkin::plugin::event::PlayerFishState::InGround => {
            pumpkin_core::plugin::PlayerFishState::InGround
        }
        crate::pumpkin::plugin::event::PlayerFishState::FailedAttempt => {
            pumpkin_core::plugin::PlayerFishState::FailedAttempt
        }
        crate::pumpkin::plugin::event::PlayerFishState::ReelIn => {
            pumpkin_core::plugin::PlayerFishState::ReelIn
        }
        crate::pumpkin::plugin::event::PlayerFishState::Bite => {
            pumpkin_core::plugin::PlayerFishState::Bite
        }
    }
}
const fn to_wit_interact_action(
    val: pumpkin_core::plugin::InteractAction,
) -> crate::pumpkin::plugin::event::InteractAction {
    match val {
        pumpkin_core::plugin::InteractAction::LeftClickBlock => {
            crate::pumpkin::plugin::event::InteractAction::LeftClickBlock
        }
        pumpkin_core::plugin::InteractAction::LeftClickAir => {
            crate::pumpkin::plugin::event::InteractAction::LeftClickAir
        }
        pumpkin_core::plugin::InteractAction::RightClickAir => {
            crate::pumpkin::plugin::event::InteractAction::RightClickAir
        }
        pumpkin_core::plugin::InteractAction::RightClickBlock => {
            crate::pumpkin::plugin::event::InteractAction::RightClickBlock
        }
    }
}
const fn from_wit_interact_action(
    val: crate::pumpkin::plugin::event::InteractAction,
) -> pumpkin_core::plugin::InteractAction {
    match val {
        crate::pumpkin::plugin::event::InteractAction::LeftClickBlock => {
            pumpkin_core::plugin::InteractAction::LeftClickBlock
        }
        crate::pumpkin::plugin::event::InteractAction::LeftClickAir => {
            pumpkin_core::plugin::InteractAction::LeftClickAir
        }
        crate::pumpkin::plugin::event::InteractAction::RightClickAir => {
            pumpkin_core::plugin::InteractAction::RightClickAir
        }
        crate::pumpkin::plugin::event::InteractAction::RightClickBlock => {
            pumpkin_core::plugin::InteractAction::RightClickBlock
        }
    }
}
const fn to_wit_server_load_type(
    val: pumpkin_core::plugin::ServerLoadType,
) -> crate::pumpkin::plugin::event::ServerLoadType {
    match val {
        pumpkin_core::plugin::ServerLoadType::Startup => {
            crate::pumpkin::plugin::event::ServerLoadType::Startup
        }
        pumpkin_core::plugin::ServerLoadType::Reload => {
            crate::pumpkin::plugin::event::ServerLoadType::Reload
        }
    }
}
const fn from_wit_server_load_type(
    val: crate::pumpkin::plugin::event::ServerLoadType,
) -> pumpkin_core::plugin::ServerLoadType {
    match val {
        crate::pumpkin::plugin::event::ServerLoadType::Startup => {
            pumpkin_core::plugin::ServerLoadType::Startup
        }
        crate::pumpkin::plugin::event::ServerLoadType::Reload => {
            pumpkin_core::plugin::ServerLoadType::Reload
        }
    }
}
const fn to_wit_target_reason(
    val: pumpkin_core::plugin::TargetReason,
) -> crate::pumpkin::plugin::event::TargetReason {
    match val {
        pumpkin_core::plugin::TargetReason::ClosestPlayer => {
            crate::pumpkin::plugin::event::TargetReason::ClosestPlayer
        }
        pumpkin_core::plugin::TargetReason::ClosestEntity => {
            crate::pumpkin::plugin::event::TargetReason::ClosestEntity
        }
        pumpkin_core::plugin::TargetReason::RandomTarget => {
            crate::pumpkin::plugin::event::TargetReason::RandomTarget
        }
        pumpkin_core::plugin::TargetReason::TargetAttackedEntity => {
            crate::pumpkin::plugin::event::TargetReason::TargetAttackedEntity
        }
        pumpkin_core::plugin::TargetReason::TargetAttackedNearbyEntity => {
            crate::pumpkin::plugin::event::TargetReason::TargetAttackedNearbyEntity
        }
        pumpkin_core::plugin::TargetReason::TargetAttackedOwner => {
            crate::pumpkin::plugin::event::TargetReason::TargetAttackedOwner
        }
        pumpkin_core::plugin::TargetReason::OwnerAttackedTarget => {
            crate::pumpkin::plugin::event::TargetReason::OwnerAttackedTarget
        }
        pumpkin_core::plugin::TargetReason::PigZombieTarget => {
            crate::pumpkin::plugin::event::TargetReason::PigZombieTarget
        }
        pumpkin_core::plugin::TargetReason::Angry => {
            crate::pumpkin::plugin::event::TargetReason::Angry
        }
        pumpkin_core::plugin::TargetReason::ForgotTarget => {
            crate::pumpkin::plugin::event::TargetReason::ForgotTarget
        }
        pumpkin_core::plugin::TargetReason::Fleeing => {
            crate::pumpkin::plugin::event::TargetReason::Fleeing
        }
        pumpkin_core::plugin::TargetReason::Unknown => {
            crate::pumpkin::plugin::event::TargetReason::Unknown
        }
    }
}
const fn from_wit_target_reason(
    val: crate::pumpkin::plugin::event::TargetReason,
) -> pumpkin_core::plugin::TargetReason {
    match val {
        crate::pumpkin::plugin::event::TargetReason::ClosestPlayer => {
            pumpkin_core::plugin::TargetReason::ClosestPlayer
        }
        crate::pumpkin::plugin::event::TargetReason::ClosestEntity => {
            pumpkin_core::plugin::TargetReason::ClosestEntity
        }
        crate::pumpkin::plugin::event::TargetReason::RandomTarget => {
            pumpkin_core::plugin::TargetReason::RandomTarget
        }
        crate::pumpkin::plugin::event::TargetReason::TargetAttackedEntity => {
            pumpkin_core::plugin::TargetReason::TargetAttackedEntity
        }
        crate::pumpkin::plugin::event::TargetReason::TargetAttackedNearbyEntity => {
            pumpkin_core::plugin::TargetReason::TargetAttackedNearbyEntity
        }
        crate::pumpkin::plugin::event::TargetReason::TargetAttackedOwner => {
            pumpkin_core::plugin::TargetReason::TargetAttackedOwner
        }
        crate::pumpkin::plugin::event::TargetReason::OwnerAttackedTarget => {
            pumpkin_core::plugin::TargetReason::OwnerAttackedTarget
        }
        crate::pumpkin::plugin::event::TargetReason::PigZombieTarget => {
            pumpkin_core::plugin::TargetReason::PigZombieTarget
        }
        crate::pumpkin::plugin::event::TargetReason::Angry => {
            pumpkin_core::plugin::TargetReason::Angry
        }
        crate::pumpkin::plugin::event::TargetReason::ForgotTarget => {
            pumpkin_core::plugin::TargetReason::ForgotTarget
        }
        crate::pumpkin::plugin::event::TargetReason::Fleeing => {
            pumpkin_core::plugin::TargetReason::Fleeing
        }
        crate::pumpkin::plugin::event::TargetReason::Unknown => {
            pumpkin_core::plugin::TargetReason::Unknown
        }
    }
}
const fn to_wit_creature_spawn_reason(
    val: pumpkin_core::plugin::CreatureSpawnReason,
) -> crate::pumpkin::plugin::event::CreatureSpawnReason {
    match val {
        pumpkin_core::plugin::CreatureSpawnReason::SpawnerEgg => {
            crate::pumpkin::plugin::event::CreatureSpawnReason::SpawnerEgg
        }
        pumpkin_core::plugin::CreatureSpawnReason::DispenseEgg => {
            crate::pumpkin::plugin::event::CreatureSpawnReason::DispenseEgg
        }
        pumpkin_core::plugin::CreatureSpawnReason::Bucket => {
            crate::pumpkin::plugin::event::CreatureSpawnReason::Bucket
        }
    }
}
const fn from_wit_creature_spawn_reason(
    val: crate::pumpkin::plugin::event::CreatureSpawnReason,
) -> pumpkin_core::plugin::CreatureSpawnReason {
    match val {
        crate::pumpkin::plugin::event::CreatureSpawnReason::SpawnerEgg => {
            pumpkin_core::plugin::CreatureSpawnReason::SpawnerEgg
        }
        crate::pumpkin::plugin::event::CreatureSpawnReason::DispenseEgg => {
            pumpkin_core::plugin::CreatureSpawnReason::DispenseEgg
        }
        crate::pumpkin::plugin::event::CreatureSpawnReason::Bucket => {
            pumpkin_core::plugin::CreatureSpawnReason::Bucket
        }
    }
}
const fn to_wit_player_animation_type(
    val: pumpkin_core::plugin::PlayerAnimationType,
) -> crate::pumpkin::plugin::event::PlayerAnimationType {
    match val {
        pumpkin_core::plugin::PlayerAnimationType::ArmSwingMain => {
            crate::pumpkin::plugin::event::PlayerAnimationType::ArmSwingMain
        }
        pumpkin_core::plugin::PlayerAnimationType::ArmSwingOff => {
            crate::pumpkin::plugin::event::PlayerAnimationType::ArmSwingOff
        }
        pumpkin_core::plugin::PlayerAnimationType::LeaveBed => {
            crate::pumpkin::plugin::event::PlayerAnimationType::LeaveBed
        }
    }
}
const fn from_wit_player_animation_type(
    val: crate::pumpkin::plugin::event::PlayerAnimationType,
) -> pumpkin_core::plugin::PlayerAnimationType {
    match val {
        crate::pumpkin::plugin::event::PlayerAnimationType::ArmSwingMain => {
            pumpkin_core::plugin::PlayerAnimationType::ArmSwingMain
        }
        crate::pumpkin::plugin::event::PlayerAnimationType::ArmSwingOff => {
            pumpkin_core::plugin::PlayerAnimationType::ArmSwingOff
        }
        crate::pumpkin::plugin::event::PlayerAnimationType::LeaveBed => {
            pumpkin_core::plugin::PlayerAnimationType::LeaveBed
        }
    }
}
const fn to_wit_cauldron_change_reason(
    val: pumpkin_core::plugin::CauldronChangeReason,
) -> crate::pumpkin::plugin::event::CauldronChangeReason {
    match val {
        pumpkin_core::plugin::CauldronChangeReason::BucketEmpty => {
            crate::pumpkin::plugin::event::CauldronChangeReason::BucketEmpty
        }
        pumpkin_core::plugin::CauldronChangeReason::BucketFill => {
            crate::pumpkin::plugin::event::CauldronChangeReason::BucketFill
        }
        pumpkin_core::plugin::CauldronChangeReason::BottleEmpty => {
            crate::pumpkin::plugin::event::CauldronChangeReason::BottleEmpty
        }
        pumpkin_core::plugin::CauldronChangeReason::BottleFill => {
            crate::pumpkin::plugin::event::CauldronChangeReason::BottleFill
        }
        pumpkin_core::plugin::CauldronChangeReason::NaturalFill => {
            crate::pumpkin::plugin::event::CauldronChangeReason::NaturalFill
        }
        pumpkin_core::plugin::CauldronChangeReason::Extinguish => {
            crate::pumpkin::plugin::event::CauldronChangeReason::Extinguish
        }
        pumpkin_core::plugin::CauldronChangeReason::Unknown => {
            crate::pumpkin::plugin::event::CauldronChangeReason::Unknown
        }
    }
}
const fn from_wit_cauldron_change_reason(
    val: crate::pumpkin::plugin::event::CauldronChangeReason,
) -> pumpkin_core::plugin::CauldronChangeReason {
    match val {
        crate::pumpkin::plugin::event::CauldronChangeReason::BucketEmpty => {
            pumpkin_core::plugin::CauldronChangeReason::BucketEmpty
        }
        crate::pumpkin::plugin::event::CauldronChangeReason::BucketFill => {
            pumpkin_core::plugin::CauldronChangeReason::BucketFill
        }
        crate::pumpkin::plugin::event::CauldronChangeReason::BottleEmpty => {
            pumpkin_core::plugin::CauldronChangeReason::BottleEmpty
        }
        crate::pumpkin::plugin::event::CauldronChangeReason::BottleFill => {
            pumpkin_core::plugin::CauldronChangeReason::BottleFill
        }
        crate::pumpkin::plugin::event::CauldronChangeReason::NaturalFill => {
            pumpkin_core::plugin::CauldronChangeReason::NaturalFill
        }
        crate::pumpkin::plugin::event::CauldronChangeReason::Extinguish => {
            pumpkin_core::plugin::CauldronChangeReason::Extinguish
        }
        crate::pumpkin::plugin::event::CauldronChangeReason::Unknown => {
            pumpkin_core::plugin::CauldronChangeReason::Unknown
        }
    }
}
const fn to_wit_portal_type(
    val: pumpkin_core::plugin::PortalType,
) -> crate::pumpkin::plugin::event::PortalType {
    match val {
        pumpkin_core::plugin::PortalType::Nether => {
            crate::pumpkin::plugin::event::PortalType::Nether
        }
        pumpkin_core::plugin::PortalType::End => crate::pumpkin::plugin::event::PortalType::End,
        pumpkin_core::plugin::PortalType::Custom => {
            crate::pumpkin::plugin::event::PortalType::Custom
        }
    }
}
const fn from_wit_portal_type(
    val: crate::pumpkin::plugin::event::PortalType,
) -> pumpkin_core::plugin::PortalType {
    match val {
        crate::pumpkin::plugin::event::PortalType::Nether => {
            pumpkin_core::plugin::PortalType::Nether
        }
        crate::pumpkin::plugin::event::PortalType::End => pumpkin_core::plugin::PortalType::End,
        crate::pumpkin::plugin::event::PortalType::Custom => {
            pumpkin_core::plugin::PortalType::Custom
        }
    }
}
const fn to_wit_tree_type(
    val: pumpkin_core::plugin::TreeType,
) -> crate::pumpkin::plugin::event::TreeType {
    match val {
        pumpkin_core::plugin::TreeType::Oak => crate::pumpkin::plugin::event::TreeType::Oak,
        pumpkin_core::plugin::TreeType::Spruce => crate::pumpkin::plugin::event::TreeType::Spruce,
        pumpkin_core::plugin::TreeType::Birch => crate::pumpkin::plugin::event::TreeType::Birch,
        pumpkin_core::plugin::TreeType::Jungle => crate::pumpkin::plugin::event::TreeType::Jungle,
        pumpkin_core::plugin::TreeType::Acacia => crate::pumpkin::plugin::event::TreeType::Acacia,
        pumpkin_core::plugin::TreeType::DarkOak => crate::pumpkin::plugin::event::TreeType::DarkOak,
        pumpkin_core::plugin::TreeType::Mangrove => {
            crate::pumpkin::plugin::event::TreeType::Mangrove
        }
        pumpkin_core::plugin::TreeType::Cherry => crate::pumpkin::plugin::event::TreeType::Cherry,
        pumpkin_core::plugin::TreeType::Poplar => crate::pumpkin::plugin::event::TreeType::Poplar,
        pumpkin_core::plugin::TreeType::Azalea => crate::pumpkin::plugin::event::TreeType::Azalea,
        pumpkin_core::plugin::TreeType::BrownMushroom => {
            crate::pumpkin::plugin::event::TreeType::BrownMushroom
        }
        pumpkin_core::plugin::TreeType::RedMushroom => {
            crate::pumpkin::plugin::event::TreeType::RedMushroom
        }
        pumpkin_core::plugin::TreeType::Custom => crate::pumpkin::plugin::event::TreeType::Custom,
    }
}
const fn from_wit_tree_type(
    val: crate::pumpkin::plugin::event::TreeType,
) -> pumpkin_core::plugin::TreeType {
    match val {
        crate::pumpkin::plugin::event::TreeType::Oak => pumpkin_core::plugin::TreeType::Oak,
        crate::pumpkin::plugin::event::TreeType::Spruce => pumpkin_core::plugin::TreeType::Spruce,
        crate::pumpkin::plugin::event::TreeType::Birch => pumpkin_core::plugin::TreeType::Birch,
        crate::pumpkin::plugin::event::TreeType::Jungle => pumpkin_core::plugin::TreeType::Jungle,
        crate::pumpkin::plugin::event::TreeType::Acacia => pumpkin_core::plugin::TreeType::Acacia,
        crate::pumpkin::plugin::event::TreeType::DarkOak => pumpkin_core::plugin::TreeType::DarkOak,
        crate::pumpkin::plugin::event::TreeType::Mangrove => {
            pumpkin_core::plugin::TreeType::Mangrove
        }
        crate::pumpkin::plugin::event::TreeType::Cherry => pumpkin_core::plugin::TreeType::Cherry,
        crate::pumpkin::plugin::event::TreeType::Poplar => pumpkin_core::plugin::TreeType::Poplar,
        crate::pumpkin::plugin::event::TreeType::Azalea => pumpkin_core::plugin::TreeType::Azalea,
        crate::pumpkin::plugin::event::TreeType::BrownMushroom => {
            pumpkin_core::plugin::TreeType::BrownMushroom
        }
        crate::pumpkin::plugin::event::TreeType::RedMushroom => {
            pumpkin_core::plugin::TreeType::RedMushroom
        }
        crate::pumpkin::plugin::event::TreeType::Custom => pumpkin_core::plugin::TreeType::Custom,
    }
}
const fn to_wit_event_dye_color(
    val: pumpkin_data::dye_color::DyeColor,
) -> crate::pumpkin::plugin::world::DyeColor {
    match val {
        pumpkin_data::dye_color::DyeColor::White => crate::pumpkin::plugin::world::DyeColor::White,
        pumpkin_data::dye_color::DyeColor::Orange => {
            crate::pumpkin::plugin::world::DyeColor::Orange
        }
        pumpkin_data::dye_color::DyeColor::Magenta => {
            crate::pumpkin::plugin::world::DyeColor::Magenta
        }
        pumpkin_data::dye_color::DyeColor::LightBlue => {
            crate::pumpkin::plugin::world::DyeColor::LightBlue
        }
        pumpkin_data::dye_color::DyeColor::Yellow => {
            crate::pumpkin::plugin::world::DyeColor::Yellow
        }
        pumpkin_data::dye_color::DyeColor::Lime => crate::pumpkin::plugin::world::DyeColor::Lime,
        pumpkin_data::dye_color::DyeColor::Pink => crate::pumpkin::plugin::world::DyeColor::Pink,
        pumpkin_data::dye_color::DyeColor::Gray => crate::pumpkin::plugin::world::DyeColor::Gray,
        pumpkin_data::dye_color::DyeColor::LightGray => {
            crate::pumpkin::plugin::world::DyeColor::LightGray
        }
        pumpkin_data::dye_color::DyeColor::Cyan => crate::pumpkin::plugin::world::DyeColor::Cyan,
        pumpkin_data::dye_color::DyeColor::Purple => {
            crate::pumpkin::plugin::world::DyeColor::Purple
        }
        pumpkin_data::dye_color::DyeColor::Blue => crate::pumpkin::plugin::world::DyeColor::Blue,
        pumpkin_data::dye_color::DyeColor::Brown => crate::pumpkin::plugin::world::DyeColor::Brown,
        pumpkin_data::dye_color::DyeColor::Green => crate::pumpkin::plugin::world::DyeColor::Green,
        pumpkin_data::dye_color::DyeColor::Red => crate::pumpkin::plugin::world::DyeColor::Red,
        pumpkin_data::dye_color::DyeColor::Black => crate::pumpkin::plugin::world::DyeColor::Black,
    }
}
const fn from_wit_event_dye_color(
    val: crate::pumpkin::plugin::world::DyeColor,
) -> pumpkin_data::dye_color::DyeColor {
    match val {
        crate::pumpkin::plugin::world::DyeColor::White => pumpkin_data::dye_color::DyeColor::White,
        crate::pumpkin::plugin::world::DyeColor::Orange => {
            pumpkin_data::dye_color::DyeColor::Orange
        }
        crate::pumpkin::plugin::world::DyeColor::Magenta => {
            pumpkin_data::dye_color::DyeColor::Magenta
        }
        crate::pumpkin::plugin::world::DyeColor::LightBlue => {
            pumpkin_data::dye_color::DyeColor::LightBlue
        }
        crate::pumpkin::plugin::world::DyeColor::Yellow => {
            pumpkin_data::dye_color::DyeColor::Yellow
        }
        crate::pumpkin::plugin::world::DyeColor::Lime => pumpkin_data::dye_color::DyeColor::Lime,
        crate::pumpkin::plugin::world::DyeColor::Pink => pumpkin_data::dye_color::DyeColor::Pink,
        crate::pumpkin::plugin::world::DyeColor::Gray => pumpkin_data::dye_color::DyeColor::Gray,
        crate::pumpkin::plugin::world::DyeColor::LightGray => {
            pumpkin_data::dye_color::DyeColor::LightGray
        }
        crate::pumpkin::plugin::world::DyeColor::Cyan => pumpkin_data::dye_color::DyeColor::Cyan,
        crate::pumpkin::plugin::world::DyeColor::Purple => {
            pumpkin_data::dye_color::DyeColor::Purple
        }
        crate::pumpkin::plugin::world::DyeColor::Blue => pumpkin_data::dye_color::DyeColor::Blue,
        crate::pumpkin::plugin::world::DyeColor::Brown => pumpkin_data::dye_color::DyeColor::Brown,
        crate::pumpkin::plugin::world::DyeColor::Green => pumpkin_data::dye_color::DyeColor::Green,
        crate::pumpkin::plugin::world::DyeColor::Red => pumpkin_data::dye_color::DyeColor::Red,
        crate::pumpkin::plugin::world::DyeColor::Black => pumpkin_data::dye_color::DyeColor::Black,
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerJoinEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerJoinEvent(
            crate::pumpkin::plugin::event::PlayerJoinEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                join_message: state
                    .add((&self.join_message).clone())
                    .expect("failed to add text-component resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerJoinEvent(data) => Self {
                player: consume_player(state, &data.player),
                join_message: consume_text_component(state, &data.join_message),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerLeaveEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerLeaveEvent(
            crate::pumpkin::plugin::event::PlayerLeaveEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                leave_message: state
                    .add((&self.leave_message).clone())
                    .expect("failed to add text-component resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerLeaveEvent(data) => Self {
                player: consume_player(state, &data.player),
                leave_message: consume_text_component(state, &data.leave_message),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerLoginEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerLoginEvent(
            crate::pumpkin::plugin::event::PlayerLoginEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                kick_message: state
                    .add((&self.kick_message).clone())
                    .expect("failed to add text-component resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerLoginEvent(data) => Self {
                player: consume_player(state, &data.player),
                kick_message: consume_text_component(state, &data.kick_message),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerChatEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerChatEvent(
            crate::pumpkin::plugin::event::PlayerChatEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                message: (&self.message).clone(),
                recipients: (&self.recipients)
                    .iter()
                    .map(|v| {
                        state
                            .add(std::sync::Arc::clone(v))
                            .expect("failed to add player resource")
                    })
                    .collect(),
                signature: (&self.signature).as_ref().map(|v| (v).clone()),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerChatEvent(data) => Self {
                player: consume_player(state, &data.player),
                message: data.message,
                recipients: (data.recipients)
                    .into_iter()
                    .map(|v| consume_player(state, &v))
                    .collect(),
                signature: (data.signature).map(|v| v),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerCommandSendEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerCommandSendEvent(
            crate::pumpkin::plugin::event::PlayerCommandSendEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                command: (&self.command).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerCommandSendEvent(data) => Self {
                player: consume_player(state, &data.player),
                command: data.command,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerPermissionCheckEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerPermissionCheckEvent(
            crate::pumpkin::plugin::event::PlayerPermissionCheckEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                permission: (&self.permission).clone(),
                permission_result: self.permission_result,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerPermissionCheckEvent(data) => Self {
                player: consume_player(state, &data.player),
                permission: data.permission,
                permission_result: data.permission_result,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerMoveEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerMoveEvent(
            crate::pumpkin::plugin::event::PlayerMoveEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                from_position: to_wasm_position(self.from_position),
                to_position: to_wasm_position(self.to_position),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerMoveEvent(data) => Self {
                player: consume_player(state, &data.player),
                from_position: from_wasm_position(data.from_position),
                to_position: from_wasm_position(data.to_position),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerTeleportEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerTeleportEvent(
            crate::pumpkin::plugin::event::PlayerTeleportEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                from_position: to_wasm_position(self.from_position),
                to_position: to_wasm_position(self.to_position),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerTeleportEvent(data) => Self {
                player: consume_player(state, &data.player),
                from_position: from_wasm_position(data.from_position),
                to_position: from_wasm_position(data.to_position),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerChangeWorldEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerChangeWorldEvent(
            crate::pumpkin::plugin::event::PlayerChangeWorldEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                previous_world: state
                    .add(std::sync::Arc::clone(&self.previous_world))
                    .expect("failed to add world resource"),
                new_world: state
                    .add(std::sync::Arc::clone(&self.new_world))
                    .expect("failed to add world resource"),
                position: to_wasm_position(self.position),
                yaw: self.yaw,
                pitch: self.pitch,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerChangeWorldEvent(data) => Self {
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
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerRespawnEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerRespawnEvent(
            crate::pumpkin::plugin::event::PlayerRespawnEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                previous_world: state
                    .add(std::sync::Arc::clone(&self.previous_world))
                    .expect("failed to add world resource"),
                respawned_world: state
                    .add(std::sync::Arc::clone(&self.respawned_world))
                    .expect("failed to add world resource"),
                position: to_wasm_position(self.position),
                yaw: self.yaw,
                pitch: self.pitch,
                alive: self.alive,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerRespawnEvent(data) => Self {
                player: consume_player(state, &data.player),
                previous_world: consume_world(state, &data.previous_world),
                respawned_world: consume_world(state, &data.respawned_world),
                position: from_wasm_position(data.position),
                yaw: data.yaw,
                pitch: data.pitch,
                alive: data.alive,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerExpChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerExpChangeEvent(
            crate::pumpkin::plugin::event::PlayerExpChangeEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                amount: self.amount,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerExpChangeEvent(data) => Self {
                player: consume_player(state, &data.player),
                amount: data.amount,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerItemHeldEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerItemHeldEvent(
            crate::pumpkin::plugin::event::PlayerItemHeldEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                previous_slot: self.previous_slot,
                new_slot: self.new_slot,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerItemHeldEvent(data) => Self {
                player: consume_player(state, &data.player),
                previous_slot: data.previous_slot,
                new_slot: data.new_slot,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerChangedMainHandEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerChangedMainHandEvent(
            crate::pumpkin::plugin::event::PlayerChangedMainHandEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                main_hand: to_wasm_hand(self.main_hand),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerChangedMainHandEvent(data) => Self {
                player: consume_player(state, &data.player),
                main_hand: from_wasm_hand(data.main_hand),
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerGamemodeChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerGamemodeChangeEvent(
            crate::pumpkin::plugin::event::PlayerGamemodeChangeEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                previous_gamemode: to_wasm_game_mode(self.previous_gamemode),
                new_gamemode: to_wasm_game_mode(self.new_gamemode),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerGamemodeChangeEvent(data) => Self {
                player: consume_player(state, &data.player),
                previous_gamemode: from_wasm_game_mode(data.previous_gamemode),
                new_gamemode: from_wasm_game_mode(data.new_gamemode),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerCustomPayloadEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerCustomPayloadEvent(
            crate::pumpkin::plugin::event::PlayerCustomPayloadEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                channel: (&self.channel).clone(),
                data: (&self.data).to_vec(),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerCustomPayloadEvent(data) => Self {
                player: consume_player(state, &data.player),
                channel: data.channel,
                data: bytes::Bytes::from(data.data),
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerFishEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerFishEvent(
            crate::pumpkin::plugin::event::PlayerFishEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                caught_uuid: (self.caught_uuid).map(|v| {
                    <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::to_wit(&v)
                }),
                caught_type: (&self.caught_type).clone(),
                hook_uuid: <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::to_wit(
                    &self.hook_uuid,
                ),
                state: to_wit_player_fish_state(self.state),
                hand: to_wasm_hand(self.hand),
                exp_to_drop: self.exp_to_drop,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerFishEvent(data) => Self {
                player: consume_player(state, &data.player),
                caught_uuid: (data.caught_uuid).map(|v| {
                    <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::from_wit(&v)
                }),
                caught_type: data.caught_type,
                hook_uuid: <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::from_wit(
                    &data.hook_uuid,
                ),
                state: from_wit_player_fish_state(data.state),
                hand: from_wasm_hand(data.hand),
                exp_to_drop: data.exp_to_drop,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerEggThrowEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerEggThrowEvent(
            crate::pumpkin::plugin::event::PlayerEggThrowEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                egg_uuid: <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::to_wit(
                    &self.egg_uuid,
                ),
                hatching: self.hatching,
                num_hatches: self.num_hatches,
                hatching_type: to_wasm_entity_type(self.hatching_type),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerEggThrowEvent(data) => Self {
                player: consume_player(state, &data.player),
                egg_uuid: <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::from_wit(
                    &data.egg_uuid,
                ),
                hatching: data.hatching,
                num_hatches: data.num_hatches,
                hatching_type: from_wasm_entity_type(&data.hatching_type),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerInteractUnknownEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerInteractUnknownEntityEvent(
            crate::pumpkin::plugin::event::PlayerInteractUnknownEntityEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                entity_id: self.entity_id,
                action: to_wasm_entity_interaction_action(self.action),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerInteractUnknownEntityEvent(data) => Self {
                player: consume_player(state, &data.player),
                entity_id: data.entity_id,
                action: from_wasm_entity_interaction_action(data.action),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerInteractEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerInteractEntityEvent(
            crate::pumpkin::plugin::event::PlayerInteractEntityEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                target: state
                    .add(std::sync::Arc::clone(&self.target))
                    .expect("failed to add entity resource"),
                action: to_wasm_entity_interaction_action(self.action),
                target_position: (self.target_position).map(|v| to_wasm_position(v)),
                sneaking: self.sneaking,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerInteractEntityEvent(data) => Self {
                player: consume_player(state, &data.player),
                target: consume_entity(state, &data.target),
                action: from_wasm_entity_interaction_action(data.action),
                target_position: (data.target_position).map(|v| from_wasm_position(v)),
                sneaking: data.sneaking,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerInteractEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerInteractEvent(
            crate::pumpkin::plugin::event::PlayerInteractEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                action: to_wit_interact_action(self.action),
                clicked_pos: (self.clicked_pos).map(|v| to_wasm_block_position(v)),
                block: to_wasm_block_name(self.block),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerInteractEvent(data) => Self {
                player: consume_player(state, &data.player),
                action: from_wit_interact_action(data.action),
                clicked_pos: (data.clicked_pos).map(|v| from_wasm_block_position(v)),
                block: from_wasm_block_name(&data.block),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerToggleSneakEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerToggleSneakEvent(
            crate::pumpkin::plugin::event::PlayerToggleSneakEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                is_sneaking: self.is_sneaking,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerToggleSneakEvent(data) => Self {
                player: consume_player(state, &data.player),
                is_sneaking: data.is_sneaking,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerToggleFlightEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerToggleFlightEvent(
            crate::pumpkin::plugin::event::PlayerToggleFlightEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                is_flying: self.is_flying,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerToggleFlightEvent(data) => Self {
                player: consume_player(state, &data.player),
                is_flying: data.is_flying,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerToggleSprintEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerToggleSprintEvent(
            crate::pumpkin::plugin::event::PlayerToggleSprintEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                is_sprinting: self.is_sprinting,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerToggleSprintEvent(data) => Self {
                player: consume_player(state, &data.player),
                is_sprinting: data.is_sprinting,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::InventoryClickEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::InventoryClickEvent(
            crate::pumpkin::plugin::event::InventoryClickEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                window_type: (self.window_type).map(|v| crate::gui::to_wit_screen(v)),
                click_type: to_wasm_click_type(self.click_type),
                slot: self.slot,
                raw_slot: self.raw_slot,
                clicked_item: (&self.clicked_item).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::new(tokio::sync::Mutex::new((v).clone())))
                        .expect("failed to add item-stack resource")
                }),
                cursor: (&self.cursor).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::new(tokio::sync::Mutex::new((v).clone())))
                        .expect("failed to add item-stack resource")
                }),
                hotbar_button: self.hotbar_button,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::InventoryClickEvent(data) => Self {
                player: consume_player(state, &data.player),
                window_type: (data.window_type).map(|v| crate::gui::from_wit_screen(v)),
                click_type: from_wasm_click_type(data.click_type),
                slot: data.slot,
                raw_slot: data.raw_slot,
                clicked_item: (data.clicked_item).map(|v| consume_item_stack(state, &v)),
                cursor: (data.cursor).map(|v| consume_item_stack(state, &v)),
                hotbar_button: data.hotbar_button,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::InventoryCloseEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::InventoryCloseEvent(
            crate::pumpkin::plugin::event::InventoryCloseEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                window_type: (self.window_type).map(|v| crate::gui::to_wit_screen(v)),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::InventoryCloseEvent(data) => Self {
                player: consume_player(state, &data.player),
                window_type: (data.window_type).map(|v| crate::gui::from_wit_screen(v)),
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockRedstoneEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockRedstoneEvent(
            crate::pumpkin::plugin::event::BlockRedstoneEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                state_id: (self.state_id).as_u16(),
                block_pos: to_wasm_block_position(self.block_pos),
                old_current: self.old_current,
                new_current: self.new_current,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockRedstoneEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                state_id: pumpkin_data::BlockStateId::new_or_air(data.state_id),
                block_pos: from_wasm_block_position(data.block_pos),
                old_current: data.old_current,
                new_current: data.new_current,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockBreakEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockBreakEvent(
            crate::pumpkin::plugin::event::BlockBreakEventData {
                player: (&self.player).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::clone(v))
                        .expect("failed to add player resource")
                }),
                block: to_wasm_block_name(self.block),
                block_pos: to_wasm_block_position(self.block_pos),
                exp: self.exp,
                should_drop: self.should_drop,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockBreakEvent(data) => Self {
                player: (data.player).map(|v| consume_player(state, &v)),
                block: from_wasm_block_name(&data.block),
                block_pos: from_wasm_block_position(data.block_pos),
                exp: data.exp,
                should_drop: data.should_drop,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockBurnEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockBurnEvent(
            crate::pumpkin::plugin::event::BlockBurnEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                igniting_block: to_wasm_block_name(self.igniting_block),
                block: to_wasm_block_name(self.block),
                block_pos: to_wasm_block_position(self.block_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockBurnEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                igniting_block: from_wasm_block_name(&data.igniting_block),
                block: from_wasm_block_name(&data.block),
                block_pos: from_wasm_block_position(data.block_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockCanBuildEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockCanBuildEvent(
            crate::pumpkin::plugin::event::BlockCanBuildEventData {
                block_to_build: to_wasm_block_name(self.block_to_build),
                buildable: self.buildable,
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                block: to_wasm_block_name(self.block),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockCanBuildEvent(data) => Self {
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
impl ToFromWasmEvent for pumpkin_core::plugin::BlockGrowEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockGrowEvent(
            crate::pumpkin::plugin::event::BlockGrowEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                old_block: to_wasm_block_name(self.old_block),
                old_state_id: (self.old_state_id).as_u16(),
                new_block: to_wasm_block_name(self.new_block),
                new_state_id: (self.new_state_id).as_u16(),
                block_pos: to_wasm_block_position(self.block_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockGrowEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                old_block: from_wasm_block_name(&data.old_block),
                old_state_id: pumpkin_data::BlockStateId::new_or_air(data.old_state_id),
                new_block: from_wasm_block_name(&data.new_block),
                new_state_id: pumpkin_data::BlockStateId::new_or_air(data.new_state_id),
                block_pos: from_wasm_block_position(data.block_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockPlaceEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockPlaceEvent(
            crate::pumpkin::plugin::event::BlockPlaceEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                block_placed: to_wasm_block_name(self.block_placed),
                block_placed_against: to_wasm_block_name(self.block_placed_against),
                block_pos: to_wasm_block_position(self.block_pos),
                can_build: self.can_build,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockPlaceEvent(data) => Self {
                player: consume_player(state, &data.player),
                block_placed: from_wasm_block_name(&data.block_placed),
                block_placed_against: from_wasm_block_name(&data.block_placed_against),
                block_pos: from_wasm_block_position(data.block_pos),
                can_build: data.can_build,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::DialogClickActionEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::DialogClickActionEvent(
            crate::pumpkin::plugin::event::DialogClickActionEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                id: (&self.id).clone(),
                payload: (&self.payload).as_ref().map(|v| (v).to_vec()),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::DialogClickActionEvent(data) => Self {
                player: consume_player(state, &data.player),
                id: data.id,
                payload: (data.payload).map(|v| bytes::Bytes::from(v)),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::DialogShowEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::DialogShowEvent(
            crate::pumpkin::plugin::event::DialogShowEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                dialog: crate::events::dialog::protocol_dialog_to_wasm(state, &self.dialog),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::DialogShowEvent(data) => Self {
                player: consume_player(state, &data.player),
                dialog: {
                    let dialog_ref = &data.dialog;
                    let converted =
                        crate::events::dialog::protocol_dialog_from_wasm(state, dialog_ref);
                    crate::events::dialog::cleanup_dialog(state, dialog_ref);
                    converted
                },
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::DialogClearEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::DialogClearEvent(
            crate::pumpkin::plugin::event::DialogClearEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::DialogClearEvent(data) => Self {
                player: consume_player(state, &data.player),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BedrockFormResponseEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BedrockFormResponseEvent(
            crate::pumpkin::plugin::event::BedrockFormResponseEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                form_id: self.form_id,
                response_data: (&self.response_data).as_ref().map(|v| (v).clone()),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BedrockFormResponseEvent(data) => Self {
                player: consume_player(state, &data.player),
                form_id: data.form_id,
                response_data: data.response_data,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ServerCommandEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ServerCommandEvent(
            crate::pumpkin::plugin::event::ServerCommandEventData {
                command: (&self.command).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ServerCommandEvent(data) => Self {
                command: data.command,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ServerListPingEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ServerListPingEvent(
            crate::pumpkin::plugin::event::ServerListPingEventData {
                hostname: (&self.hostname).clone(),
                address: {
                    let rec = &self.address;
                    crate::pumpkin::plugin::event::ServerListPingAddress {
                        host: (&rec.host).clone(),
                        port: *&rec.port,
                    }
                },
                motd: state
                    .add((&self.motd).clone())
                    .expect("failed to add text-component resource"),
                max_players: self.max_players,
                num_players: self.num_players,
                favicon: (&self.favicon).as_ref().map(|v| (v).clone()),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ServerListPingEvent(data) => Self {
                hostname: data.hostname,
                address: {
                    let rec = data.address;
                    pumpkin_core::plugin::ServerListPingAddress {
                        host: rec.host,
                        port: rec.port,
                    }
                },
                motd: consume_text_component(state, &data.motd),
                max_players: data.max_players,
                num_players: data.num_players,
                favicon: data.favicon,
            },
            _ => panic!("unexpected event type"),
        }
    }
    fn apply_wasm_event(
        &mut self,
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) {
        if !matches!(
            &event,
            crate::pumpkin::plugin::event::Event::ServerListPingEvent(_)
        ) {
            cleanup_event(&event, state);
            panic!("unexpected event type");
        }
        let returned = Self::from_wasm_event(event, state);
        self.motd = returned.motd;
        self.max_players = returned.max_players;
        self.num_players = returned.num_players;
        self.favicon = returned.favicon;
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ServerLoadEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ServerLoadEvent(
            crate::pumpkin::plugin::event::ServerLoadEventData {
                load_type: to_wit_server_load_type(self.load_type),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ServerLoadEvent(data) => Self {
                load_type: from_wit_server_load_type(data.load_type),
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::SpawnChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::SpawnChangeEvent(
            crate::pumpkin::plugin::event::SpawnChangeEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                previous_position: to_wasm_block_position(self.previous_position),
                previous_yaw: self.previous_yaw,
                previous_pitch: self.previous_pitch,
                new_position: to_wasm_block_position(self.new_position),
                new_yaw: self.new_yaw,
                new_pitch: self.new_pitch,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::SpawnChangeEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                previous_position: from_wasm_block_position(data.previous_position),
                previous_yaw: data.previous_yaw,
                previous_pitch: data.previous_pitch,
                new_position: from_wasm_block_position(data.new_position),
                new_yaw: data.new_yaw,
                new_pitch: data.new_pitch,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ServerBroadcastEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ServerBroadcastEvent(
            crate::pumpkin::plugin::event::ServerBroadcastEventData {
                message: state
                    .add((&self.message).clone())
                    .expect("failed to add text-component resource"),
                sender: state
                    .add((&self.sender).clone())
                    .expect("failed to add text-component resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ServerBroadcastEvent(data) => Self {
                message: consume_text_component(state, &data.message),
                sender: consume_text_component(state, &data.sender),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ServerTickStartEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ServerTickStartEvent(
            crate::pumpkin::plugin::event::ServerTickStartEventData { tick: self.tick },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ServerTickStartEvent(data) => {
                Self { tick: data.tick }
            }
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ServerTickEndEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ServerTickEndEvent(
            crate::pumpkin::plugin::event::ServerTickEndEventData {
                tick: self.tick,
                duration_nanos: self.duration_nanos,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ServerTickEndEvent(data) => Self {
                tick: data.tick,
                duration_nanos: data.duration_nanos,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ChunkLoadEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ChunkLoadEvent(
            crate::pumpkin::plugin::event::ChunkLoadEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                chunk_x: self.chunk_x,
                chunk_z: self.chunk_z,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ChunkLoadEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ChunkSaveEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ChunkSaveEvent(
            crate::pumpkin::plugin::event::ChunkSaveEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                chunk_x: self.chunk_x,
                chunk_z: self.chunk_z,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ChunkSaveEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ChunkSendEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ChunkSendEvent(
            crate::pumpkin::plugin::event::ChunkSendEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                chunk_x: self.chunk_x,
                chunk_z: self.chunk_z,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ChunkSendEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityDamageEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityDamageEvent(
            crate::pumpkin::plugin::event::EntityDamageEventData {
                entity_id: self.entity_id,
                damage: self.damage,
                damage_type: crate::living_entity::to_wit_damage_type(&self.damage_type),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityDamageEvent(data) => Self {
                entity_id: data.entity_id,
                damage: data.damage,
                damage_type: crate::living_entity::from_wit_damage_type(data.damage_type),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityDeathEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityDeathEvent(
            crate::pumpkin::plugin::event::EntityDeathEventData {
                entity_id: self.entity_id,
                dropped_exp: self.dropped_exp,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityDeathEvent(data) => Self {
                entity_id: data.entity_id,
                dropped_exp: data.dropped_exp,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerDeathEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerDeathEvent(
            crate::pumpkin::plugin::event::PlayerDeathEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                death_message: state
                    .add((&self.death_message).clone())
                    .expect("failed to add text-component resource"),
                dropped_exp: self.dropped_exp,
                keep_inventory: self.keep_inventory,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerDeathEvent(data) => Self {
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
impl ToFromWasmEvent for pumpkin_core::plugin::EntitySpawnEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntitySpawnEvent(
            crate::pumpkin::plugin::event::EntitySpawnEventData {
                entity_id: self.entity_id,
                entity_type: (&self.entity_type).clone(),
                position: to_wasm_position(self.position),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntitySpawnEvent(data) => Self {
                entity_id: data.entity_id,
                entity_type: data.entity_type,
                position: from_wasm_position(data.position),
                target_world: consume_world(state, &data.target_world),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityCombustEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityCombustEvent(
            crate::pumpkin::plugin::event::EntityCombustEventData {
                entity_id: self.entity_id,
                duration_secs: self.duration_secs,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityCombustEvent(data) => Self {
                entity_id: data.entity_id,
                duration_secs: data.duration_secs,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityRegainHealthEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityRegainHealthEvent(
            crate::pumpkin::plugin::event::EntityRegainHealthEventData {
                entity_id: self.entity_id,
                amount: self.amount,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityRegainHealthEvent(data) => Self {
                entity_id: data.entity_id,
                amount: data.amount,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityAirChangeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityAirChangeEvent(
            crate::pumpkin::plugin::event::EntityAirChangeEventData {
                entity_id: self.entity_id,
                amount: self.amount,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityAirChangeEvent(data) => Self {
                entity_id: data.entity_id,
                amount: data.amount,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityBreedEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityBreedEvent(
            crate::pumpkin::plugin::event::EntityBreedEventData {
                father_id: self.father_id,
                mother_id: self.mother_id,
                child_id: self.child_id,
                experience: self.experience,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityBreedEvent(data) => Self {
                father_id: data.father_id,
                mother_id: data.mother_id,
                child_id: data.child_id,
                experience: data.experience,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityDismountEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityDismountEvent(
            crate::pumpkin::plugin::event::EntityDismountEventData {
                entity_id: self.entity_id,
                dismounted_id: self.dismounted_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityDismountEvent(data) => Self {
                entity_id: data.entity_id,
                dismounted_id: data.dismounted_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityDyeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityDyeEvent(
            crate::pumpkin::plugin::event::EntityDyeEventData {
                entity_id: self.entity_id,
                color: to_wit_event_dye_color(self.color),
                player: (&self.player).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::clone(v))
                        .expect("failed to add player resource")
                }),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityDyeEvent(data) => Self {
                entity_id: data.entity_id,
                color: from_wit_event_dye_color(data.color),
                player: (data.player).map(|v| consume_player(state, &v)),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityEnterLoveModeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityEnterLoveModeEvent(
            crate::pumpkin::plugin::event::EntityEnterLoveModeEventData {
                entity_id: self.entity_id,
                human_entity_id: self.human_entity_id,
                ticks_in_love: self.ticks_in_love,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityEnterLoveModeEvent(data) => Self {
                entity_id: data.entity_id,
                human_entity_id: data.human_entity_id,
                ticks_in_love: data.ticks_in_love,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityExplodeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityExplodeEvent(
            crate::pumpkin::plugin::event::EntityExplodeEventData {
                entity_id: self.entity_id,
                position: to_wasm_position(self.position),
                yield_rate: self.yield_rate,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityExplodeEvent(data) => Self {
                entity_id: data.entity_id,
                position: from_wasm_position(data.position),
                yield_rate: data.yield_rate,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityMountEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityMountEvent(
            crate::pumpkin::plugin::event::EntityMountEventData {
                entity_id: self.entity_id,
                mounted_id: self.mounted_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityMountEvent(data) => Self {
                entity_id: data.entity_id,
                mounted_id: data.mounted_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityPickupItemEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityPickupItemEvent(
            crate::pumpkin::plugin::event::EntityPickupItemEventData {
                entity_id: self.entity_id,
                item_name: (&self.item_name).clone(),
                count: self.count,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityPickupItemEvent(data) => Self {
                entity_id: data.entity_id,
                item_name: data.item_name,
                count: data.count,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityPortalEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityPortalEvent(
            crate::pumpkin::plugin::event::EntityPortalEventData {
                entity_id: self.entity_id,
                portal_pos: to_wasm_block_position(self.portal_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityPortalEvent(data) => Self {
                entity_id: data.entity_id,
                portal_pos: from_wasm_block_position(data.portal_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityResurrectEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityResurrectEvent(
            crate::pumpkin::plugin::event::EntityResurrectEventData {
                entity_id: self.entity_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityResurrectEvent(data) => Self {
                entity_id: data.entity_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityShootBowEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityShootBowEvent(
            crate::pumpkin::plugin::event::EntityShootBowEventData {
                entity_id: self.entity_id,
                weapon_name: (&self.weapon_name).clone(),
                force: self.force,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityShootBowEvent(data) => Self {
                entity_id: data.entity_id,
                weapon_name: data.weapon_name,
                force: data.force,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityTameEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityTameEvent(
            crate::pumpkin::plugin::event::EntityTameEventData {
                entity_id: self.entity_id,
                owner: state
                    .add(std::sync::Arc::clone(&self.owner))
                    .expect("failed to add player resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityTameEvent(data) => Self {
                entity_id: data.entity_id,
                owner: consume_player(state, &data.owner),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityTargetEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityTargetEvent(
            crate::pumpkin::plugin::event::EntityTargetEventData {
                entity_id: self.entity_id,
                target_id: self.target_id,
                reason: to_wit_target_reason(self.reason),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityTargetEvent(data) => Self {
                entity_id: data.entity_id,
                target_id: data.target_id,
                reason: from_wit_target_reason(data.reason),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityTeleportEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityTeleportEvent(
            crate::pumpkin::plugin::event::EntityTeleportEventData {
                entity_id: self.entity_id,
                from_position: to_wasm_position(self.from_position),
                to_position: to_wasm_position(self.to_position),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityTeleportEvent(data) => Self {
                entity_id: data.entity_id,
                from_position: from_wasm_position(data.from_position),
                to_position: from_wasm_position(data.to_position),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityToggleGlideEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityToggleGlideEvent(
            crate::pumpkin::plugin::event::EntityToggleGlideEventData {
                entity_id: self.entity_id,
                is_gliding: self.is_gliding,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityToggleGlideEvent(data) => Self {
                entity_id: data.entity_id,
                is_gliding: data.is_gliding,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityTransformEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityTransformEvent(
            crate::pumpkin::plugin::event::EntityTransformEventData {
                entity_id: self.entity_id,
                new_entity_id: self.new_entity_id,
                transform_reason: (&self.transform_reason).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityTransformEvent(data) => Self {
                entity_id: data.entity_id,
                new_entity_id: data.new_entity_id,
                transform_reason: data.transform_reason,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerItemConsumeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerItemConsumeEvent(
            crate::pumpkin::plugin::event::PlayerItemConsumeEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                item_name: (&self.item_name).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerItemConsumeEvent(data) => Self {
                player: consume_player(state, &data.player),
                item_name: data.item_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerItemDamageEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerItemDamageEvent(
            crate::pumpkin::plugin::event::PlayerItemDamageEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                item_name: (&self.item_name).clone(),
                damage: self.damage,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerItemDamageEvent(data) => Self {
                player: consume_player(state, &data.player),
                item_name: data.item_name,
                damage: data.damage,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerDropItemEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerDropItemEvent(
            crate::pumpkin::plugin::event::PlayerDropItemEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                item_name: (&self.item_name).clone(),
                count: self.count,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerDropItemEvent(data) => Self {
                player: consume_player(state, &data.player),
                item_name: data.item_name,
                count: data.count,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerBedEnterEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerBedEnterEvent(
            crate::pumpkin::plugin::event::PlayerBedEnterEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                bed_pos: to_wasm_block_position(self.bed_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerBedEnterEvent(data) => Self {
                player: consume_player(state, &data.player),
                bed_pos: from_wasm_block_position(data.bed_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerBedLeaveEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerBedLeaveEvent(
            crate::pumpkin::plugin::event::PlayerBedLeaveEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                bed_pos: to_wasm_block_position(self.bed_pos),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerBedLeaveEvent(data) => Self {
                player: consume_player(state, &data.player),
                bed_pos: from_wasm_block_position(data.bed_pos),
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerBucketEmptyEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerBucketEmptyEvent(
            crate::pumpkin::plugin::event::PlayerBucketEmptyEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                block_pos: to_wasm_block_position(self.block_pos),
                bucket: (&self.bucket).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerBucketEmptyEvent(data) => Self {
                player: consume_player(state, &data.player),
                block_pos: from_wasm_block_position(data.block_pos),
                bucket: data.bucket,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerBucketFillEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerBucketFillEvent(
            crate::pumpkin::plugin::event::PlayerBucketFillEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                block_pos: to_wasm_block_position(self.block_pos),
                bucket: (&self.bucket).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerBucketFillEvent(data) => Self {
                player: consume_player(state, &data.player),
                block_pos: from_wasm_block_position(data.block_pos),
                bucket: data.bucket,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockDamageEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockDamageEvent(
            crate::pumpkin::plugin::event::BlockDamageEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                block: to_wasm_block_name(self.block),
                block_pos: to_wasm_block_position(self.block_pos),
                insta_break: self.insta_break,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockDamageEvent(data) => Self {
                player: consume_player(state, &data.player),
                block: from_wasm_block_name(&data.block),
                block_pos: from_wasm_block_position(data.block_pos),
                insta_break: data.insta_break,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockIgniteEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockIgniteEvent(
            crate::pumpkin::plugin::event::BlockIgniteEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                igniting_block: to_wasm_block_name(self.igniting_block),
                player: (&self.player).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::clone(v))
                        .expect("failed to add player resource")
                }),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockIgniteEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                igniting_block: from_wasm_block_name(&data.igniting_block),
                player: (data.player).map(|v| consume_player(state, &v)),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockFromToEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockFromToEvent(
            crate::pumpkin::plugin::event::BlockFromToEventData {
                from_pos: to_wasm_block_position(self.from_pos),
                to_pos: to_wasm_block_position(self.to_pos),
                block: to_wasm_block_name(self.block),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockFromToEvent(data) => Self {
                from_pos: from_wasm_block_position(data.from_pos),
                to_pos: from_wasm_block_position(data.to_pos),
                block: from_wasm_block_name(&data.block),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockFormEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockFormEvent(
            crate::pumpkin::plugin::event::BlockFormEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                block: to_wasm_block_name(self.block),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockFormEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                block: from_wasm_block_name(&data.block),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockFadeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockFadeEvent(
            crate::pumpkin::plugin::event::BlockFadeEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                block: to_wasm_block_name(self.block),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockFadeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                block: from_wasm_block_name(&data.block),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockDispenseEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockDispenseEvent(
            crate::pumpkin::plugin::event::BlockDispenseEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                item_name: (&self.item_name).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockDispenseEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                item_name: data.item_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockExplodeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockExplodeEvent(
            crate::pumpkin::plugin::event::BlockExplodeEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                yield_rate: self.yield_rate,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockExplodeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                yield_rate: data.yield_rate,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockPhysicsEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockPhysicsEvent(
            crate::pumpkin::plugin::event::BlockPhysicsEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                changed_pos: to_wasm_block_position(self.changed_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockPhysicsEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                changed_pos: from_wasm_block_position(data.changed_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockPistonExtendEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockPistonExtendEvent(
            crate::pumpkin::plugin::event::BlockPistonExtendEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                direction: (&self.direction).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockPistonExtendEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                direction: data.direction,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockPistonRetractEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockPistonRetractEvent(
            crate::pumpkin::plugin::event::BlockPistonRetractEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                direction: (&self.direction).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockPistonRetractEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                direction: data.direction,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::NotePlayEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::NotePlayEvent(
            crate::pumpkin::plugin::event::NotePlayEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                instrument: (&self.instrument).clone(),
                note: self.note,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::NotePlayEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                instrument: data.instrument,
                note: data.note,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::SignChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::SignChangeEvent(
            crate::pumpkin::plugin::event::SignChangeEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                block_pos: to_wasm_block_position(self.block_pos),
                lines: (&self.lines).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::SignChangeEvent(data) => Self {
                player: consume_player(state, &data.player),
                block_pos: from_wasm_block_position(data.block_pos),
                lines: data.lines,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::SpongeAbsorbEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::SpongeAbsorbEvent(
            crate::pumpkin::plugin::event::SpongeAbsorbEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::SpongeAbsorbEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::TntPrimeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::TntPrimeEvent(
            crate::pumpkin::plugin::event::TntPrimeEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                prime_reason: (&self.prime_reason).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::TntPrimeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                prime_reason: data.prime_reason,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::WeatherChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::WeatherChangeEvent(
            crate::pumpkin::plugin::event::WeatherChangeEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                to_weather_state: self.to_weather_state,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::WeatherChangeEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                to_weather_state: data.to_weather_state,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ThunderChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ThunderChangeEvent(
            crate::pumpkin::plugin::event::ThunderChangeEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                to_thunder_state: self.to_thunder_state,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ThunderChangeEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                to_thunder_state: data.to_thunder_state,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::WorldLoadEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::WorldLoadEvent(
            crate::pumpkin::plugin::event::WorldLoadEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::WorldLoadEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::WorldUnloadEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::WorldUnloadEvent(
            crate::pumpkin::plugin::event::WorldUnloadEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::WorldUnloadEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::AsyncStructureGenerateEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::AsyncStructureGenerateEvent(
            crate::pumpkin::plugin::event::AsyncStructureGenerateEventData {
                world_name: (&self.world_name).clone(),
                structure_name: (&self.structure_name).clone(),
                pos: to_wasm_block_position(self.pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::AsyncStructureGenerateEvent(data) => Self {
                world_name: data.world_name,
                structure_name: data.structure_name,
                pos: from_wasm_block_position(data.pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::AsyncStructureSpawnEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::AsyncStructureSpawnEvent(
            crate::pumpkin::plugin::event::AsyncStructureSpawnEventData {
                world_name: (&self.world_name).clone(),
                structure_name: (&self.structure_name).clone(),
                pos: to_wasm_block_position(self.pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::AsyncStructureSpawnEvent(data) => Self {
                world_name: data.world_name,
                structure_name: data.structure_name,
                pos: from_wasm_block_position(data.pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ChunkPopulateEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ChunkPopulateEvent(
            crate::pumpkin::plugin::event::ChunkPopulateEventData {
                chunk_x: self.chunk_x,
                chunk_z: self.chunk_z,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ChunkPopulateEvent(data) => Self {
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ChunkUnloadEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ChunkUnloadEvent(
            crate::pumpkin::plugin::event::ChunkUnloadEventData {
                chunk_x: self.chunk_x,
                chunk_z: self.chunk_z,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ChunkUnloadEvent(data) => Self {
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntitiesLoadEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntitiesLoadEvent(
            crate::pumpkin::plugin::event::EntitiesLoadEventData {
                chunk_x: self.chunk_x,
                chunk_z: self.chunk_z,
                entity_count: self.entity_count,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntitiesLoadEvent(data) => Self {
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                entity_count: data.entity_count,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntitiesUnloadEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntitiesUnloadEvent(
            crate::pumpkin::plugin::event::EntitiesUnloadEventData {
                chunk_x: self.chunk_x,
                chunk_z: self.chunk_z,
                entity_count: self.entity_count,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntitiesUnloadEvent(data) => Self {
                chunk_x: data.chunk_x,
                chunk_z: data.chunk_z,
                entity_count: data.entity_count,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::GenericGameEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::GenericGameEvent(
            crate::pumpkin::plugin::event::GenericGameEventData {
                event_id: (&self.event_id).clone(),
                pos: to_wasm_position(self.pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::GenericGameEvent(data) => Self {
                event_id: data.event_id,
                pos: from_wasm_position(data.pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::LootGenerateEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::LootGenerateEvent(
            crate::pumpkin::plugin::event::LootGenerateEventData {
                loot_table: (&self.loot_table).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::LootGenerateEvent(data) => Self {
                loot_table: data.loot_table,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PortalCreateEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PortalCreateEvent(
            crate::pumpkin::plugin::event::PortalCreateEventData {
                pos: to_wasm_block_position(self.pos),
                portal_type: to_wit_portal_type(self.portal_type),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PortalCreateEvent(data) => Self {
                pos: from_wasm_block_position(data.pos),
                portal_type: from_wit_portal_type(data.portal_type),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::StructureGrowEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::StructureGrowEvent(
            crate::pumpkin::plugin::event::StructureGrowEventData {
                pos: to_wasm_block_position(self.pos),
                species: to_wit_tree_type(self.species),
                bone_meal: self.bone_meal,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::StructureGrowEvent(data) => Self {
                pos: from_wasm_block_position(data.pos),
                species: from_wit_tree_type(data.species),
                bone_meal: data.bone_meal,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::TimeSkipEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::TimeSkipEvent(
            crate::pumpkin::plugin::event::TimeSkipEventData {
                skip_amount: self.skip_amount,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::TimeSkipEvent(data) => Self {
                skip_amount: data.skip_amount,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::WorldInitEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::WorldInitEvent(
            crate::pumpkin::plugin::event::WorldInitEventData {
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::WorldInitEvent(data) => Self {
                target_world: consume_world(state, &data.target_world),
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::WorldSaveEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::WorldSaveEvent(
            crate::pumpkin::plugin::event::WorldSaveEventData {
                world_name: (&self.world_name).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::WorldSaveEvent(data) => Self {
                world_name: data.world_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::InventoryOpenEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::InventoryOpenEvent(
            crate::pumpkin::plugin::event::InventoryOpenEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::InventoryOpenEvent(data) => Self {
                player: consume_player(state, &data.player),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::InventoryDragEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::InventoryDragEvent(
            crate::pumpkin::plugin::event::InventoryDragEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::InventoryDragEvent(data) => Self {
                player: consume_player(state, &data.player),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::CraftItemEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::CraftItemEvent(
            crate::pumpkin::plugin::event::CraftItemEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                recipe_id: (&self.recipe_id).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::CraftItemEvent(data) => Self {
                player: consume_player(state, &data.player),
                recipe_id: data.recipe_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::FurnaceSmeltEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::FurnaceSmeltEvent(
            crate::pumpkin::plugin::event::FurnaceSmeltEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                source_item: (&self.source_item).clone(),
                result_item: (&self.result_item).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::FurnaceSmeltEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                source_item: data.source_item,
                result_item: data.result_item,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BrewEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BrewEvent(
            crate::pumpkin::plugin::event::BrewEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                fuel_level: self.fuel_level,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BrewEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                fuel_level: data.fuel_level,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BrewingStandFuelEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BrewingStandFuelEvent(
            crate::pumpkin::plugin::event::BrewingStandFuelEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                fuel_power: self.fuel_power,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BrewingStandFuelEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                fuel_power: data.fuel_power,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::FurnaceBurnEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::FurnaceBurnEvent(
            crate::pumpkin::plugin::event::FurnaceBurnEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                fuel_item: (&self.fuel_item).clone(),
                burn_time: self.burn_time,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::FurnaceBurnEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                fuel_item: data.fuel_item,
                burn_time: data.burn_time,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::FurnaceExtractEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::FurnaceExtractEvent(
            crate::pumpkin::plugin::event::FurnaceExtractEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                block_pos: to_wasm_block_position(self.block_pos),
                item_id: (&self.item_id).clone(),
                item_amount: self.item_amount,
                exp_gained: self.exp_gained,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::FurnaceExtractEvent(data) => Self {
                player: consume_player(state, &data.player),
                block_pos: from_wasm_block_position(data.block_pos),
                item_id: data.item_id,
                item_amount: data.item_amount,
                exp_gained: data.exp_gained,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::FurnaceStartSmeltEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::FurnaceStartSmeltEvent(
            crate::pumpkin::plugin::event::FurnaceStartSmeltEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                source_item: (&self.source_item).clone(),
                cooking_time: self.cooking_time,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::FurnaceStartSmeltEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                source_item: data.source_item,
                cooking_time: data.cooking_time,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::HopperInventorySearchEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::HopperInventorySearchEvent(
            crate::pumpkin::plugin::event::HopperInventorySearchEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                search_pos: to_wasm_block_position(self.search_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::HopperInventorySearchEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                search_pos: from_wasm_block_position(data.search_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::InventoryCreativeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::InventoryCreativeEvent(
            crate::pumpkin::plugin::event::InventoryCreativeEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                slot: self.slot,
                item_id: (&self.item_id).clone(),
                item_count: self.item_count,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::InventoryCreativeEvent(data) => Self {
                player: consume_player(state, &data.player),
                slot: data.slot,
                item_id: data.item_id,
                item_count: data.item_count,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::InventoryInteractEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::InventoryInteractEvent(
            crate::pumpkin::plugin::event::InventoryInteractEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::InventoryInteractEvent(data) => Self {
                player: consume_player(state, &data.player),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::InventoryMoveItemEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::InventoryMoveItemEvent(
            crate::pumpkin::plugin::event::InventoryMoveItemEventData {
                source_pos: to_wasm_block_position(self.source_pos),
                target_pos: to_wasm_block_position(self.target_pos),
                item_id: (&self.item_id).clone(),
                item_amount: self.item_amount,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::InventoryMoveItemEvent(data) => Self {
                source_pos: from_wasm_block_position(data.source_pos),
                target_pos: from_wasm_block_position(data.target_pos),
                item_id: data.item_id,
                item_amount: data.item_amount,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::InventoryPickupItemEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::InventoryPickupItemEvent(
            crate::pumpkin::plugin::event::InventoryPickupItemEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                item_entity_id: self.item_entity_id,
                item_id: (&self.item_id).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::InventoryPickupItemEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                item_entity_id: data.item_entity_id,
                item_id: data.item_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PrepareAnvilEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PrepareAnvilEvent(
            crate::pumpkin::plugin::event::PrepareAnvilEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                rename_text: (&self.rename_text).clone(),
                repair_cost: self.repair_cost,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PrepareAnvilEvent(data) => Self {
                player: consume_player(state, &data.player),
                rename_text: data.rename_text,
                repair_cost: data.repair_cost,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PrepareGrindstoneEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PrepareGrindstoneEvent(
            crate::pumpkin::plugin::event::PrepareGrindstoneEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                result_item: (&self.result_item).as_ref().map(|v| (v).clone()),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PrepareGrindstoneEvent(data) => Self {
                player: consume_player(state, &data.player),
                result_item: data.result_item,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PrepareInventoryResultEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PrepareInventoryResultEvent(
            crate::pumpkin::plugin::event::PrepareInventoryResultEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                result_item: (&self.result_item).as_ref().map(|v| (v).clone()),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PrepareInventoryResultEvent(data) => Self {
                player: consume_player(state, &data.player),
                result_item: data.result_item,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PrepareItemCraftEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PrepareItemCraftEvent(
            crate::pumpkin::plugin::event::PrepareItemCraftEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                recipe_id: (&self.recipe_id).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PrepareItemCraftEvent(data) => Self {
                player: consume_player(state, &data.player),
                recipe_id: data.recipe_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PrepareSmithingEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PrepareSmithingEvent(
            crate::pumpkin::plugin::event::PrepareSmithingEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                result_item: (&self.result_item).as_ref().map(|v| (v).clone()),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PrepareSmithingEvent(data) => Self {
                player: consume_player(state, &data.player),
                result_item: data.result_item,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::SmithItemEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::SmithItemEvent(
            crate::pumpkin::plugin::event::SmithItemEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                recipe_id: (&self.recipe_id).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::SmithItemEvent(data) => Self {
                player: consume_player(state, &data.player),
                recipe_id: data.recipe_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::TradeSelectEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::TradeSelectEvent(
            crate::pumpkin::plugin::event::TradeSelectEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                slot_index: self.slot_index,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::TradeSelectEvent(data) => Self {
                player: consume_player(state, &data.player),
                slot_index: data.slot_index,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VehicleBlockCollisionEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VehicleBlockCollisionEvent(
            crate::pumpkin::plugin::event::VehicleBlockCollisionEventData {
                vehicle_id: self.vehicle_id,
                block_pos: to_wasm_block_position(self.block_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VehicleBlockCollisionEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                block_pos: from_wasm_block_position(data.block_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VehicleCollisionEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VehicleCollisionEvent(
            crate::pumpkin::plugin::event::VehicleCollisionEventData {
                vehicle_id: self.vehicle_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VehicleCollisionEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VehicleCreateEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VehicleCreateEvent(
            crate::pumpkin::plugin::event::VehicleCreateEventData {
                vehicle_id: self.vehicle_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VehicleCreateEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VehicleDamageEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VehicleDamageEvent(
            crate::pumpkin::plugin::event::VehicleDamageEventData {
                vehicle_id: self.vehicle_id,
                damage: self.damage,
                attacker_id: self.attacker_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VehicleDamageEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                damage: data.damage,
                attacker_id: data.attacker_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VehicleDestroyEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VehicleDestroyEvent(
            crate::pumpkin::plugin::event::VehicleDestroyEventData {
                vehicle_id: self.vehicle_id,
                attacker_id: self.attacker_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VehicleDestroyEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                attacker_id: data.attacker_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VehicleEnterEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VehicleEnterEvent(
            crate::pumpkin::plugin::event::VehicleEnterEventData {
                vehicle_id: self.vehicle_id,
                entered_id: self.entered_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VehicleEnterEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                entered_id: data.entered_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VehicleEntityCollisionEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VehicleEntityCollisionEvent(
            crate::pumpkin::plugin::event::VehicleEntityCollisionEventData {
                vehicle_id: self.vehicle_id,
                collided_entity_id: self.collided_entity_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VehicleEntityCollisionEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                collided_entity_id: data.collided_entity_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VehicleExitEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VehicleExitEvent(
            crate::pumpkin::plugin::event::VehicleExitEventData {
                vehicle_id: self.vehicle_id,
                exited_id: self.exited_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VehicleExitEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                exited_id: data.exited_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VehicleMoveEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VehicleMoveEvent(
            crate::pumpkin::plugin::event::VehicleMoveEventData {
                vehicle_id: self.vehicle_id,
                from_position: to_wasm_position(self.from_position),
                to_position: to_wasm_position(self.to_position),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VehicleMoveEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                from_position: from_wasm_position(data.from_position),
                to_position: from_wasm_position(data.to_position),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VehicleUpdateEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VehicleUpdateEvent(
            crate::pumpkin::plugin::event::VehicleUpdateEventData {
                vehicle_id: self.vehicle_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VehicleUpdateEvent(data) => Self {
                vehicle_id: data.vehicle_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PrepareItemEnchantEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PrepareItemEnchantEvent(
            crate::pumpkin::plugin::event::PrepareItemEnchantEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                item: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.item).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                offers: (&self.offers)
                    .iter()
                    .copied()
                    .map(|v| {
                        let rec = &v;
                        crate::pumpkin::plugin::event::EnchantmentOffer {
                            cost: *&rec.cost,
                            enchantment_id: *&rec.enchantment_id,
                            enchantment_level: *&rec.enchantment_level,
                        }
                    })
                    .collect(),
                bookshelf_count: self.bookshelf_count,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PrepareItemEnchantEvent(data) => Self {
                player: consume_player(state, &data.player),
                item: consume_item_stack(state, &data.item),
                offers: (data.offers)
                    .into_iter()
                    .map(|v| {
                        let rec = v;
                        pumpkin_core::plugin::EnchantmentOffer {
                            cost: rec.cost,
                            enchantment_id: rec.enchantment_id,
                            enchantment_level: rec.enchantment_level,
                        }
                    })
                    .collect(),
                bookshelf_count: data.bookshelf_count,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EnchantItemEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EnchantItemEvent(
            crate::pumpkin::plugin::event::EnchantItemEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                item: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.item).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                option: self.option,
                cost: self.cost,
                enchantments_to_add: (&self.enchantments_to_add)
                    .iter()
                    .copied()
                    .map(|v| {
                        let rec = &v;
                        crate::pumpkin::plugin::event::EnchantmentValue {
                            enchantment: crate::item_stack::to_wit_enchantment(*&rec.enchantment),
                            level: *&rec.level,
                        }
                    })
                    .collect(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EnchantItemEvent(data) => Self {
                player: consume_player(state, &data.player),
                item: consume_item_stack(state, &data.item),
                option: data.option,
                cost: data.cost,
                enchantments_to_add: (data.enchantments_to_add)
                    .into_iter()
                    .map(|v| {
                        let rec = v;
                        pumpkin_core::plugin::EnchantmentValue {
                            enchantment: crate::item_stack::from_wit_enchantment(rec.enchantment),
                            level: rec.level,
                        }
                    })
                    .collect(),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::MapInitializeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::MapInitializeEvent(
            crate::pumpkin::plugin::event::MapInitializeEventData {
                map_id: self.map_id,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::MapInitializeEvent(data) => Self {
                map_id: data.map_id,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::HangingBreakEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::HangingBreakEvent(
            crate::pumpkin::plugin::event::HangingBreakEventData {
                entity: state
                    .add(std::sync::Arc::clone(&self.entity))
                    .expect("failed to add entity resource"),
                remover: (&self.remover).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::clone(v))
                        .expect("failed to add entity resource")
                }),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::HangingBreakEvent(data) => Self {
                entity: consume_entity(state, &data.entity),
                remover: (data.remover).map(|v| consume_entity(state, &v)),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::HangingBreakByEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::HangingBreakByEntityEvent(
            crate::pumpkin::plugin::event::HangingBreakByEntityEventData {
                entity: state
                    .add(std::sync::Arc::clone(&self.entity))
                    .expect("failed to add entity resource"),
                remover: state
                    .add(std::sync::Arc::clone(&self.remover))
                    .expect("failed to add entity resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::HangingBreakByEntityEvent(data) => Self {
                entity: consume_entity(state, &data.entity),
                remover: consume_entity(state, &data.remover),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::HangingPlaceEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::HangingPlaceEvent(
            crate::pumpkin::plugin::event::HangingPlaceEventData {
                entity: state
                    .add(std::sync::Arc::clone(&self.entity))
                    .expect("failed to add entity resource"),
                player: (&self.player).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::clone(v))
                        .expect("failed to add player resource")
                }),
                block_pos: to_wasm_block_position(self.block_pos),
                block_face: crate::mob::to_wit_block_direction(self.block_face),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::HangingPlaceEvent(data) => Self {
                entity: consume_entity(state, &data.entity),
                player: (data.player).map(|v| consume_player(state, &v)),
                block_pos: from_wasm_block_position(data.block_pos),
                block_face: crate::mob::from_wit_block_direction(data.block_face),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BellResonateEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BellResonateEvent(
            crate::pumpkin::plugin::event::BellResonateEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BellResonateEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BellRingEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BellRingEvent(
            crate::pumpkin::plugin::event::BellRingEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                entity: (&self.entity).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::clone(v))
                        .expect("failed to add entity resource")
                }),
                direction: (self.direction).map(|v| crate::mob::to_wit_block_direction(v)),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BellRingEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                entity: (data.entity).map(|v| consume_entity(state, &v)),
                direction: (data.direction).map(|v| crate::mob::from_wit_block_direction(v)),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockBrushEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockBrushEvent(
            crate::pumpkin::plugin::event::BlockBrushEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                item: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.item).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockBrushEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                player: consume_player(state, &data.player),
                item: consume_item_stack(state, &data.item),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockCookEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockCookEvent(
            crate::pumpkin::plugin::event::BlockCookEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                source: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.source).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                result: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.result).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockCookEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                source: consume_item_stack(state, &data.source),
                result: consume_item_stack(state, &data.result),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockDamageAbortEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockDamageAbortEvent(
            crate::pumpkin::plugin::event::BlockDamageAbortEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                item_in_hand: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.item_in_hand).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockDamageAbortEvent(data) => Self {
                player: consume_player(state, &data.player),
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                item_in_hand: consume_item_stack(state, &data.item_in_hand),
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockDispenseArmorEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockDispenseArmorEvent(
            crate::pumpkin::plugin::event::BlockDispenseArmorEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                target: state
                    .add(std::sync::Arc::clone(&self.target))
                    .expect("failed to add entity resource"),
                item: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.item).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockDispenseArmorEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                target: consume_entity(state, &data.target),
                item: consume_item_stack(state, &data.item),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockDispenseLootEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockDispenseLootEvent(
            crate::pumpkin::plugin::event::BlockDispenseLootEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                items: (&self.items)
                    .iter()
                    .map(|v| {
                        state
                            .add(std::sync::Arc::new(tokio::sync::Mutex::new((v).clone())))
                            .expect("failed to add item-stack resource")
                    })
                    .collect(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockDispenseLootEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                items: (data.items)
                    .into_iter()
                    .map(|v| consume_item_stack(state, &v))
                    .collect(),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockDropItemEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockDropItemEvent(
            crate::pumpkin::plugin::event::BlockDropItemEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                player: (&self.player).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::clone(v))
                        .expect("failed to add player resource")
                }),
                items: (&self.items)
                    .iter()
                    .map(|v| {
                        state
                            .add(std::sync::Arc::new(tokio::sync::Mutex::new((v).clone())))
                            .expect("failed to add item-stack resource")
                    })
                    .collect(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockDropItemEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                player: (data.player).map(|v| consume_player(state, &v)),
                items: (data.items)
                    .into_iter()
                    .map(|v| consume_item_stack(state, &v))
                    .collect(),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockExpEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockExpEvent(
            crate::pumpkin::plugin::event::BlockExpEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                exp: self.exp,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockExpEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                exp: data.exp,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockFertilizeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockFertilizeEvent(
            crate::pumpkin::plugin::event::BlockFertilizeEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                player: (&self.player).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::clone(v))
                        .expect("failed to add player resource")
                }),
                changed_blocks: (&self.changed_blocks)
                    .iter()
                    .copied()
                    .map(|v| {
                        let (t0, t1) = v;
                        (to_wasm_block_position(t0), (t1).as_u16())
                    })
                    .collect(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockFertilizeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                player: (data.player).map(|v| consume_player(state, &v)),
                changed_blocks: (data.changed_blocks)
                    .into_iter()
                    .map(|v| {
                        let (t0, t1) = v;
                        (
                            from_wasm_block_position(t0),
                            pumpkin_data::BlockStateId::new_or_air(t1),
                        )
                    })
                    .collect(),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockMultiPlaceEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockMultiPlaceEvent(
            crate::pumpkin::plugin::event::BlockMultiPlaceEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                placed_blocks: (&self.placed_blocks)
                    .iter()
                    .copied()
                    .map(|v| {
                        let (t0, t1) = v;
                        (to_wasm_block_position(t0), (t1).as_u16())
                    })
                    .collect(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockMultiPlaceEvent(data) => Self {
                player: consume_player(state, &data.player),
                target_world: consume_world(state, &data.target_world),
                placed_blocks: (data.placed_blocks)
                    .into_iter()
                    .map(|v| {
                        let (t0, t1) = v;
                        (
                            from_wasm_block_position(t0),
                            pumpkin_data::BlockStateId::new_or_air(t1),
                        )
                    })
                    .collect(),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockReceiveGameEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockReceiveGameEvent(
            crate::pumpkin::plugin::event::BlockReceiveGameEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                game_event: (&self.game_event).clone(),
                source_entity: (&self.source_entity).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::clone(v))
                        .expect("failed to add entity resource")
                }),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockReceiveGameEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                game_event: data.game_event,
                source_entity: (data.source_entity).map(|v| consume_entity(state, &v)),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockShearEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockShearEntityEvent(
            crate::pumpkin::plugin::event::BlockShearEntityEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                target: state
                    .add(std::sync::Arc::clone(&self.target))
                    .expect("failed to add entity resource"),
                item: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.item).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockShearEntityEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                target: consume_entity(state, &data.target),
                item: consume_item_stack(state, &data.item),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BlockSpreadEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BlockSpreadEvent(
            crate::pumpkin::plugin::event::BlockSpreadEventData {
                source_pos: to_wasm_block_position(self.source_pos),
                target_pos: to_wasm_block_position(self.target_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                new_state_id: (self.new_state_id).as_u16(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BlockSpreadEvent(data) => Self {
                source_pos: from_wasm_block_position(data.source_pos),
                target_pos: from_wasm_block_position(data.target_pos),
                target_world: consume_world(state, &data.target_world),
                new_state_id: pumpkin_data::BlockStateId::new_or_air(data.new_state_id),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BrewingStartEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BrewingStartEvent(
            crate::pumpkin::plugin::event::BrewingStartEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                brewing_time: self.brewing_time,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BrewingStartEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                brewing_time: data.brewing_time,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::CampfireStartEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::CampfireStartEvent(
            crate::pumpkin::plugin::event::CampfireStartEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                item: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.item).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                slot: self.slot,
                cooking_time: self.cooking_time,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::CampfireStartEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                item: consume_item_stack(state, &data.item),
                slot: data.slot,
                cooking_time: data.cooking_time,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::CauldronLevelChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::CauldronLevelChangeEvent(
            crate::pumpkin::plugin::event::CauldronLevelChangeEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                old_level: self.old_level,
                new_level: self.new_level,
                reason: to_wit_cauldron_change_reason(self.reason),
                entity: (&self.entity).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::clone(v))
                        .expect("failed to add entity resource")
                }),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::CauldronLevelChangeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                old_level: data.old_level,
                new_level: data.new_level,
                reason: from_wit_cauldron_change_reason(data.reason),
                entity: (data.entity).map(|v| consume_entity(state, &v)),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::CrafterCraftEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::CrafterCraftEvent(
            crate::pumpkin::plugin::event::CrafterCraftEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                result: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.result).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::CrafterCraftEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                result: consume_item_stack(state, &data.result),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityBlockFormEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityBlockFormEvent(
            crate::pumpkin::plugin::event::EntityBlockFormEventData {
                entity: state
                    .add(std::sync::Arc::clone(&self.entity))
                    .expect("failed to add entity resource"),
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                new_state_id: (self.new_state_id).as_u16(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityBlockFormEvent(data) => Self {
                entity: consume_entity(state, &data.entity),
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                new_state_id: pumpkin_data::BlockStateId::new_or_air(data.new_state_id),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::FluidLevelChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::FluidLevelChangeEvent(
            crate::pumpkin::plugin::event::FluidLevelChangeEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                new_state_id: (self.new_state_id).as_u16(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::FluidLevelChangeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                new_state_id: pumpkin_data::BlockStateId::new_or_air(data.new_state_id),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::InventoryBlockStartEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::InventoryBlockStartEvent(
            crate::pumpkin::plugin::event::InventoryBlockStartEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::InventoryBlockStartEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::LeavesDecayEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::LeavesDecayEvent(
            crate::pumpkin::plugin::event::LeavesDecayEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::LeavesDecayEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::MoistureChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::MoistureChangeEvent(
            crate::pumpkin::plugin::event::MoistureChangeEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                new_moisture: self.new_moisture,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::MoistureChangeEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                new_moisture: data.new_moisture,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::SculkBloomEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::SculkBloomEvent(
            crate::pumpkin::plugin::event::SculkBloomEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                charge: self.charge,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::SculkBloomEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                charge: data.charge,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VaultDisplayItemEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VaultDisplayItemEvent(
            crate::pumpkin::plugin::event::VaultDisplayItemEventData {
                block_pos: to_wasm_block_position(self.block_pos),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                item: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.item).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VaultDisplayItemEvent(data) => Self {
                block_pos: from_wasm_block_position(data.block_pos),
                target_world: consume_world(state, &data.target_world),
                item: consume_item_stack(state, &data.item),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::CreatureSpawnEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::CreatureSpawnEvent(
            crate::pumpkin::plugin::event::CreatureSpawnEventData {
                entity_id: self.entity_id,
                entity_type: (&self.entity_type).clone(),
                position: to_wasm_position(self.position),
                target_world: state
                    .add(std::sync::Arc::clone(&self.target_world))
                    .expect("failed to add world resource"),
                spawn_reason: to_wit_creature_spawn_reason(self.spawn_reason),
                player: (&self.player).as_ref().map(|v| {
                    state
                        .add(std::sync::Arc::clone(v))
                        .expect("failed to add player resource")
                }),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::CreatureSpawnEvent(data) => Self {
                entity_id: data.entity_id,
                entity_type: data.entity_type,
                position: from_wasm_position(data.position),
                target_world: consume_world(state, &data.target_world),
                spawn_reason: from_wit_creature_spawn_reason(data.spawn_reason),
                player: (data.player).map(|v| consume_player(state, &v)),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EnderDragonChangePhaseEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EnderDragonChangePhaseEvent(
            crate::pumpkin::plugin::event::EnderDragonChangePhaseEventData {
                entity_id: self.entity_id,
                current_phase: (&self.current_phase).clone(),
                new_phase: (&self.new_phase).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EnderDragonChangePhaseEvent(data) => Self {
                entity_id: data.entity_id,
                current_phase: data.current_phase,
                new_phase: data.new_phase,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityBreakDoorEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityBreakDoorEvent(
            crate::pumpkin::plugin::event::EntityBreakDoorEventData {
                entity_id: self.entity_id,
                block_pos: to_wasm_block_position(self.block_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityBreakDoorEvent(data) => Self {
                entity_id: data.entity_id,
                block_pos: from_wasm_block_position(data.block_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityChangeBlockEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityChangeBlockEvent(
            crate::pumpkin::plugin::event::EntityChangeBlockEventData {
                entity_id: self.entity_id,
                block_pos: to_wasm_block_position(self.block_pos),
                new_block: (&self.new_block).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityChangeBlockEvent(data) => Self {
                entity_id: data.entity_id,
                block_pos: from_wasm_block_position(data.block_pos),
                new_block: data.new_block,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityDamageByBlockEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityDamageByBlockEvent(
            crate::pumpkin::plugin::event::EntityDamageByBlockEventData {
                entity_id: self.entity_id,
                damager_pos: (self.damager_pos).map(|v| to_wasm_block_position(v)),
                damage: self.damage,
                cause: (&self.cause).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityDamageByBlockEvent(data) => Self {
                entity_id: data.entity_id,
                damager_pos: (data.damager_pos).map(|v| from_wasm_block_position(v)),
                damage: data.damage,
                cause: data.cause,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityDamageByEntityEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityDamageByEntityEvent(
            crate::pumpkin::plugin::event::EntityDamageByEntityEventData {
                entity_id: self.entity_id,
                damager_id: self.damager_id,
                damage: self.damage,
                cause: (&self.cause).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityDamageByEntityEvent(data) => Self {
                entity_id: data.entity_id,
                damager_id: data.damager_id,
                damage: data.damage,
                cause: data.cause,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityDropItemEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityDropItemEvent(
            crate::pumpkin::plugin::event::EntityDropItemEventData {
                entity_id: self.entity_id,
                item_name: (&self.item_name).clone(),
                count: self.count,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityDropItemEvent(data) => Self {
                entity_id: data.entity_id,
                item_name: data.item_name,
                count: data.count,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityEnterBlockEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityEnterBlockEvent(
            crate::pumpkin::plugin::event::EntityEnterBlockEventData {
                entity_id: self.entity_id,
                block_pos: to_wasm_block_position(self.block_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityEnterBlockEvent(data) => Self {
                entity_id: data.entity_id,
                block_pos: from_wasm_block_position(data.block_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityExhaustionEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityExhaustionEvent(
            crate::pumpkin::plugin::event::EntityExhaustionEventData {
                entity_id: self.entity_id,
                exhaustion: self.exhaustion,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityExhaustionEvent(data) => Self {
                entity_id: data.entity_id,
                exhaustion: data.exhaustion,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityInteractEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityInteractEvent(
            crate::pumpkin::plugin::event::EntityInteractEventData {
                entity_id: self.entity_id,
                block_pos: to_wasm_block_position(self.block_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityInteractEvent(data) => Self {
                entity_id: data.entity_id,
                block_pos: from_wasm_block_position(data.block_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityKnockbackEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityKnockbackEvent(
            crate::pumpkin::plugin::event::EntityKnockbackEventData {
                entity_id: self.entity_id,
                hit_by_id: self.hit_by_id,
                knockback: to_wasm_position(self.knockback),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityKnockbackEvent(data) => Self {
                entity_id: data.entity_id,
                hit_by_id: data.hit_by_id,
                knockback: from_wasm_position(data.knockback),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityPlaceEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityPlaceEvent(
            crate::pumpkin::plugin::event::EntityPlaceEventData {
                entity_id: self.entity_id,
                block_pos: to_wasm_block_position(self.block_pos),
                block_name: (&self.block_name).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityPlaceEvent(data) => Self {
                entity_id: data.entity_id,
                block_pos: from_wasm_block_position(data.block_pos),
                block_name: data.block_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityPoseChangeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityPoseChangeEvent(
            crate::pumpkin::plugin::event::EntityPoseChangeEventData {
                entity_id: self.entity_id,
                pose: (&self.pose).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityPoseChangeEvent(data) => Self {
                entity_id: data.entity_id,
                pose: data.pose,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityPotionEffectEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityPotionEffectEvent(
            crate::pumpkin::plugin::event::EntityPotionEffectEventData {
                entity_id: self.entity_id,
                effect_name: (&self.effect_name).clone(),
                duration: self.duration,
                amplifier: self.amplifier,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityPotionEffectEvent(data) => Self {
                entity_id: data.entity_id,
                effect_name: data.effect_name,
                duration: data.duration,
                amplifier: data.amplifier,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntitySpellCastEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntitySpellCastEvent(
            crate::pumpkin::plugin::event::EntitySpellCastEventData {
                entity_id: self.entity_id,
                spell: (&self.spell).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntitySpellCastEvent(data) => Self {
                entity_id: data.entity_id,
                spell: data.spell,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityTargetLivingEntityEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityTargetLivingEntityEvent(
            crate::pumpkin::plugin::event::EntityTargetLivingEntityEventData {
                entity_id: self.entity_id,
                target_id: self.target_id,
                reason: (&self.reason).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityTargetLivingEntityEvent(data) => Self {
                entity_id: data.entity_id,
                target_id: data.target_id,
                reason: data.reason,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityToggleSitEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityToggleSitEvent(
            crate::pumpkin::plugin::event::EntityToggleSitEventData {
                entity_id: self.entity_id,
                sitting: self.sitting,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityToggleSitEvent(data) => Self {
                entity_id: data.entity_id,
                sitting: data.sitting,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityToggleSwimEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityToggleSwimEvent(
            crate::pumpkin::plugin::event::EntityToggleSwimEventData {
                entity_id: self.entity_id,
                is_swimming: self.is_swimming,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityToggleSwimEvent(data) => Self {
                entity_id: data.entity_id,
                is_swimming: data.is_swimming,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ExplosionPrimeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ExplosionPrimeEvent(
            crate::pumpkin::plugin::event::ExplosionPrimeEventData {
                entity_id: self.entity_id,
                radius: self.radius,
                fire: self.fire,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ExplosionPrimeEvent(data) => Self {
                entity_id: data.entity_id,
                radius: data.radius,
                fire: data.fire,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::FireworkExplodeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::FireworkExplodeEvent(
            crate::pumpkin::plugin::event::FireworkExplodeEventData {
                entity_id: self.entity_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::FireworkExplodeEvent(data) => Self {
                entity_id: data.entity_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::FoodLevelChangeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::FoodLevelChangeEvent(
            crate::pumpkin::plugin::event::FoodLevelChangeEventData {
                entity_id: self.entity_id,
                food_level: self.food_level,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::FoodLevelChangeEvent(data) => Self {
                entity_id: data.entity_id,
                food_level: data.food_level,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ItemDespawnEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ItemDespawnEvent(
            crate::pumpkin::plugin::event::ItemDespawnEventData {
                entity_id: self.entity_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ItemDespawnEvent(data) => Self {
                entity_id: data.entity_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ItemMergeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ItemMergeEvent(
            crate::pumpkin::plugin::event::ItemMergeEventData {
                entity_id: self.entity_id,
                target_id: self.target_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ItemMergeEvent(data) => Self {
                entity_id: data.entity_id,
                target_id: data.target_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ItemSpawnEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ItemSpawnEvent(
            crate::pumpkin::plugin::event::ItemSpawnEventData {
                entity_id: self.entity_id,
                position: to_wasm_position(self.position),
                item_name: (&self.item_name).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ItemSpawnEvent(data) => Self {
                entity_id: data.entity_id,
                position: from_wasm_position(data.position),
                item_name: data.item_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PiglinBarterEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PiglinBarterEvent(
            crate::pumpkin::plugin::event::PiglinBarterEventData {
                entity_id: self.entity_id,
                input_item: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.input_item).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                outcome: (&self.outcome)
                    .iter()
                    .map(|v| {
                        state
                            .add(std::sync::Arc::new(tokio::sync::Mutex::new((v).clone())))
                            .expect("failed to add item-stack resource")
                    })
                    .collect(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PiglinBarterEvent(data) => Self {
                entity_id: data.entity_id,
                input_item: consume_item_stack(state, &data.input_item),
                outcome: (data.outcome)
                    .into_iter()
                    .map(|v| consume_item_stack(state, &v))
                    .collect(),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ProjectileHitEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ProjectileHitEvent(
            crate::pumpkin::plugin::event::ProjectileHitEventData {
                entity_id: self.entity_id,
                hit_position: to_wasm_position(self.hit_position),
                hit_entity_id: self.hit_entity_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ProjectileHitEvent(data) => Self {
                entity_id: data.entity_id,
                hit_position: from_wasm_position(data.hit_position),
                hit_entity_id: data.hit_entity_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ProjectileLaunchEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ProjectileLaunchEvent(
            crate::pumpkin::plugin::event::ProjectileLaunchEventData {
                entity_id: self.entity_id,
                shooter_id: self.shooter_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ProjectileLaunchEvent(data) => Self {
                entity_id: data.entity_id,
                shooter_id: data.shooter_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::SheepDyeWoolEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::SheepDyeWoolEvent(
            crate::pumpkin::plugin::event::SheepDyeWoolEventData {
                entity_id: self.entity_id,
                dye_color: self.dye_color,
                player_id: self.player_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::SheepDyeWoolEvent(data) => Self {
                entity_id: data.entity_id,
                dye_color: data.dye_color,
                player_id: data.player_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::SheepRegrowWoolEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::SheepRegrowWoolEvent(
            crate::pumpkin::plugin::event::SheepRegrowWoolEventData {
                entity_id: self.entity_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::SheepRegrowWoolEvent(data) => Self {
                entity_id: data.entity_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::SlimeSplitEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::SlimeSplitEvent(
            crate::pumpkin::plugin::event::SlimeSplitEventData {
                entity_id: self.entity_id,
                count: self.count,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::SlimeSplitEvent(data) => Self {
                entity_id: data.entity_id,
                count: data.count,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::StriderTemperatureChangeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::StriderTemperatureChangeEvent(
            crate::pumpkin::plugin::event::StriderTemperatureChangeEventData {
                entity_id: self.entity_id,
                is_shivering: self.is_shivering,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::StriderTemperatureChangeEvent(data) => Self {
                entity_id: data.entity_id,
                is_shivering: data.is_shivering,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VillagerAcquireTradeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VillagerAcquireTradeEvent(
            crate::pumpkin::plugin::event::VillagerAcquireTradeEventData {
                entity_id: self.entity_id,
                recipe_index: self.recipe_index,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VillagerAcquireTradeEvent(data) => Self {
                entity_id: data.entity_id,
                recipe_index: data.recipe_index,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VillagerCareerChangeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VillagerCareerChangeEvent(
            crate::pumpkin::plugin::event::VillagerCareerChangeEventData {
                entity_id: self.entity_id,
                profession: (&self.profession).clone(),
                reason: (&self.reason).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VillagerCareerChangeEvent(data) => Self {
                entity_id: data.entity_id,
                profession: data.profession,
                reason: data.reason,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VillagerReplenishTradeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VillagerReplenishTradeEvent(
            crate::pumpkin::plugin::event::VillagerReplenishTradeEventData {
                entity_id: self.entity_id,
                restock_quantity: self.restock_quantity,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VillagerReplenishTradeEvent(data) => Self {
                entity_id: data.entity_id,
                restock_quantity: data.restock_quantity,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::WardenAngerChangeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::WardenAngerChangeEvent(
            crate::pumpkin::plugin::event::WardenAngerChangeEventData {
                entity_id: self.entity_id,
                target_id: self.target_id,
                old_anger: self.old_anger,
                new_anger: self.new_anger,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::WardenAngerChangeEvent(data) => Self {
                entity_id: data.entity_id,
                target_id: data.target_id,
                old_anger: data.old_anger,
                new_anger: data.new_anger,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::AreaEffectCloudApplyEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::AreaEffectCloudApplyEvent(
            crate::pumpkin::plugin::event::AreaEffectCloudApplyEventData {
                entity_id: self.entity_id,
                affected_entities: (&self.affected_entities).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::AreaEffectCloudApplyEvent(data) => Self {
                entity_id: data.entity_id,
                affected_entities: data.affected_entities,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ArrowBodyCountChangeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ArrowBodyCountChangeEvent(
            crate::pumpkin::plugin::event::ArrowBodyCountChangeEventData {
                entity_id: self.entity_id,
                old_amount: self.old_amount,
                new_amount: self.new_amount,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ArrowBodyCountChangeEvent(data) => Self {
                entity_id: data.entity_id,
                old_amount: data.old_amount,
                new_amount: data.new_amount,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::BatToggleSleepEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::BatToggleSleepEvent(
            crate::pumpkin::plugin::event::BatToggleSleepEventData {
                entity_id: self.entity_id,
                is_awake: self.is_awake,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::BatToggleSleepEvent(data) => Self {
                entity_id: data.entity_id,
                is_awake: data.is_awake,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::CreeperPowerEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::CreeperPowerEvent(
            crate::pumpkin::plugin::event::CreeperPowerEventData {
                entity_id: self.entity_id,
                lightning_id: self.lightning_id,
                cause: (&self.cause).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::CreeperPowerEvent(data) => Self {
                entity_id: data.entity_id,
                lightning_id: data.lightning_id,
                cause: data.cause,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityCombustByBlockEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityCombustByBlockEvent(
            crate::pumpkin::plugin::event::EntityCombustByBlockEventData {
                entity_id: self.entity_id,
                combuster: to_wasm_block_position(self.combuster),
                duration: self.duration,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityCombustByBlockEvent(data) => Self {
                entity_id: data.entity_id,
                combuster: from_wasm_block_position(data.combuster),
                duration: data.duration,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityCombustByEntityEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityCombustByEntityEvent(
            crate::pumpkin::plugin::event::EntityCombustByEntityEventData {
                entity_id: self.entity_id,
                combuster_id: self.combuster_id,
                duration: self.duration,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityCombustByEntityEvent(data) => Self {
                entity_id: data.entity_id,
                combuster_id: data.combuster_id,
                duration: data.duration,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityKnockbackByEntityEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityKnockbackByEntityEvent(
            crate::pumpkin::plugin::event::EntityKnockbackByEntityEventData {
                entity_id: self.entity_id,
                hit_by_id: self.hit_by_id,
                force: self.force,
                x: self.x,
                y: self.y,
                z: self.z,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityKnockbackByEntityEvent(data) => Self {
                entity_id: data.entity_id,
                hit_by_id: data.hit_by_id,
                force: data.force,
                x: data.x,
                y: data.y,
                z: data.z,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityPortalEnterEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityPortalEnterEvent(
            crate::pumpkin::plugin::event::EntityPortalEnterEventData {
                entity_id: self.entity_id,
                location: to_wasm_block_position(self.location),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityPortalEnterEvent(data) => Self {
                entity_id: data.entity_id,
                location: from_wasm_block_position(data.location),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityPortalExitEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityPortalExitEvent(
            crate::pumpkin::plugin::event::EntityPortalExitEventData {
                entity_id: self.entity_id,
                from_pos: to_wasm_block_position(self.from_pos),
                to_pos: (self.to_pos).map(|v| to_wasm_block_position(v)),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityPortalExitEvent(data) => Self {
                entity_id: data.entity_id,
                from_pos: from_wasm_block_position(data.from_pos),
                to_pos: (data.to_pos).map(|v| from_wasm_block_position(v)),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityRemoveEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityRemoveEvent(
            crate::pumpkin::plugin::event::EntityRemoveEventData {
                entity_id: self.entity_id,
                cause: (&self.cause).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityRemoveEvent(data) => Self {
                entity_id: data.entity_id,
                cause: data.cause,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityTargetBlockEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityTargetBlockEvent(
            crate::pumpkin::plugin::event::EntityTargetBlockEventData {
                entity_id: self.entity_id,
                block_pos: to_wasm_block_position(self.block_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityTargetBlockEvent(data) => Self {
                entity_id: data.entity_id,
                block_pos: from_wasm_block_position(data.block_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::EntityUnleashEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::EntityUnleashEvent(
            crate::pumpkin::plugin::event::EntityUnleashEventData {
                entity_id: self.entity_id,
                reason: (&self.reason).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::EntityUnleashEvent(data) => Self {
                entity_id: data.entity_id,
                reason: data.reason,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ExpBottleEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ExpBottleEvent(
            crate::pumpkin::plugin::event::ExpBottleEventData {
                entity_id: self.entity_id,
                experience: self.experience,
                location: to_wasm_block_position(self.location),
                show_effect: self.show_effect,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ExpBottleEvent(data) => Self {
                entity_id: data.entity_id,
                experience: data.experience,
                location: from_wasm_block_position(data.location),
                show_effect: data.show_effect,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::HorseJumpEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::HorseJumpEvent(
            crate::pumpkin::plugin::event::HorseJumpEventData {
                entity_id: self.entity_id,
                power: self.power,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::HorseJumpEvent(data) => Self {
                entity_id: data.entity_id,
                power: data.power,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::LingeringPotionSplashEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::LingeringPotionSplashEvent(
            crate::pumpkin::plugin::event::LingeringPotionSplashEventData {
                entity_id: self.entity_id,
                location: to_wasm_block_position(self.location),
                potion_item: (&self.potion_item).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::LingeringPotionSplashEvent(data) => Self {
                entity_id: data.entity_id,
                location: from_wasm_block_position(data.location),
                potion_item: data.potion_item,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PigZapEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PigZapEvent(
            crate::pumpkin::plugin::event::PigZapEventData {
                entity_id: self.entity_id,
                lightning_id: self.lightning_id,
                pig_zombie_id: self.pig_zombie_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PigZapEvent(data) => Self {
                entity_id: data.entity_id,
                lightning_id: data.lightning_id,
                pig_zombie_id: data.pig_zombie_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PigZombieAngerEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PigZombieAngerEvent(
            crate::pumpkin::plugin::event::PigZombieAngerEventData {
                entity_id: self.entity_id,
                target_id: self.target_id,
                new_anger: self.new_anger,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PigZombieAngerEvent(data) => Self {
                entity_id: data.entity_id,
                target_id: data.target_id,
                new_anger: data.new_anger,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PotionSplashEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PotionSplashEvent(
            crate::pumpkin::plugin::event::PotionSplashEventData {
                entity_id: self.entity_id,
                location: to_wasm_block_position(self.location),
                potion_item: (&self.potion_item).clone(),
                affected_entities: (&self.affected_entities).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PotionSplashEvent(data) => Self {
                entity_id: data.entity_id,
                location: from_wasm_block_position(data.location),
                potion_item: data.potion_item,
                affected_entities: data.affected_entities,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::SpawnerSpawnEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::SpawnerSpawnEvent(
            crate::pumpkin::plugin::event::SpawnerSpawnEventData {
                entity_id: self.entity_id,
                spawner_pos: to_wasm_block_position(self.spawner_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::SpawnerSpawnEvent(data) => Self {
                entity_id: data.entity_id,
                spawner_pos: from_wasm_block_position(data.spawner_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::TrialSpawnerSpawnEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::TrialSpawnerSpawnEvent(
            crate::pumpkin::plugin::event::TrialSpawnerSpawnEventData {
                entity_id: self.entity_id,
                spawner_pos: to_wasm_block_position(self.spawner_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::TrialSpawnerSpawnEvent(data) => Self {
                entity_id: data.entity_id,
                spawner_pos: from_wasm_block_position(data.spawner_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::VillagerReputationChangeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::VillagerReputationChangeEvent(
            crate::pumpkin::plugin::event::VillagerReputationChangeEventData {
                entity_id: self.entity_id,
                target_id: self.target_id,
                reputation_change: self.reputation_change,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::VillagerReputationChangeEvent(data) => Self {
                entity_id: data.entity_id,
                target_id: data.target_id,
                reputation_change: data.reputation_change,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::AsyncPlayerChatEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::AsyncPlayerChatEvent(
            crate::pumpkin::plugin::event::AsyncPlayerChatEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                message: (&self.message).clone(),
                format: state
                    .add((&self.format).clone())
                    .expect("failed to add text-component resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::AsyncPlayerChatEvent(data) => Self {
                player: consume_player(state, &data.player),
                message: data.message,
                format: consume_text_component(state, &data.format),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::AsyncPlayerPreLoginEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::AsyncPlayerPreLoginEvent(
            crate::pumpkin::plugin::event::AsyncPlayerPreLoginEventData {
                player_name: (&self.player_name).clone(),
                player_uuid: <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::to_wit(
                    &self.player_uuid,
                ),
                ip_address: (&self.ip_address).to_string(),
                kick_message: state
                    .add((&self.kick_message).clone())
                    .expect("failed to add text-component resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::AsyncPlayerPreLoginEvent(data) => Self {
                player_name: data.player_name,
                player_uuid: <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::from_wit(
                    &data.player_uuid,
                ),
                ip_address: (data.ip_address)
                    .parse()
                    .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 0))),
                kick_message: consume_text_component(state, &data.kick_message),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerAdvancementDoneEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerAdvancementDoneEvent(
            crate::pumpkin::plugin::event::PlayerAdvancementDoneEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                advancement_id: (&self.advancement_id).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerAdvancementDoneEvent(data) => Self {
                player: consume_player(state, &data.player),
                advancement_id: data.advancement_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerAnimationEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerAnimationEvent(
            crate::pumpkin::plugin::event::PlayerAnimationEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                animation_type: to_wit_player_animation_type(self.animation_type),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerAnimationEvent(data) => Self {
                player: consume_player(state, &data.player),
                animation_type: from_wit_player_animation_type(data.animation_type),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerArmorStandManipulateEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerArmorStandManipulateEvent(
            crate::pumpkin::plugin::event::PlayerArmorStandManipulateEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                armor_stand_id: self.armor_stand_id,
                slot: self.slot,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerArmorStandManipulateEvent(data) => Self {
                player: consume_player(state, &data.player),
                armor_stand_id: data.armor_stand_id,
                slot: data.slot,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerBucketEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerBucketEntityEvent(
            crate::pumpkin::plugin::event::PlayerBucketEntityEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                entity_id: self.entity_id,
                bucket_item: (&self.bucket_item).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerBucketEntityEvent(data) => Self {
                player: consume_player(state, &data.player),
                entity_id: data.entity_id,
                bucket_item: data.bucket_item,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerChangedWorldEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerChangedWorldEvent(
            crate::pumpkin::plugin::event::PlayerChangedWorldEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                from_world: state
                    .add(std::sync::Arc::clone(&self.from_world))
                    .expect("failed to add world resource"),
                to_world: state
                    .add(std::sync::Arc::clone(&self.to_world))
                    .expect("failed to add world resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerChangedWorldEvent(data) => Self {
                player: consume_player(state, &data.player),
                from_world: consume_world(state, &data.from_world),
                to_world: consume_world(state, &data.to_world),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerChannelEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerChannelEvent(
            crate::pumpkin::plugin::event::PlayerChannelEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                channel: (&self.channel).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerChannelEvent(data) => Self {
                player: consume_player(state, &data.player),
                channel: data.channel,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerCommandPreprocessEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerCommandPreprocessEvent(
            crate::pumpkin::plugin::event::PlayerCommandPreprocessEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                command: (&self.command).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerCommandPreprocessEvent(data) => Self {
                player: consume_player(state, &data.player),
                command: data.command,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerEditBookEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerEditBookEvent(
            crate::pumpkin::plugin::event::PlayerEditBookEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                slot: self.slot,
                pages: (&self.pages).clone(),
                title: (&self.title).as_ref().map(|v| (v).clone()),
                signing: self.signing,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerEditBookEvent(data) => Self {
                player: consume_player(state, &data.player),
                slot: data.slot,
                pages: data.pages,
                title: data.title,
                signing: data.signing,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerElytraBoostEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerElytraBoostEvent(
            crate::pumpkin::plugin::event::PlayerElytraBoostEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                firework_id: self.firework_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerElytraBoostEvent(data) => Self {
                player: consume_player(state, &data.player),
                firework_id: data.firework_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerExpCooldownChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerExpCooldownChangeEvent(
            crate::pumpkin::plugin::event::PlayerExpCooldownChangeEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                new_cooldown: self.new_cooldown,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerExpCooldownChangeEvent(data) => Self {
                player: consume_player(state, &data.player),
                new_cooldown: data.new_cooldown,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerHarvestBlockEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerHarvestBlockEvent(
            crate::pumpkin::plugin::event::PlayerHarvestBlockEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                block_pos: to_wasm_block_position(self.block_pos),
                harvested_items: (&self.harvested_items)
                    .iter()
                    .map(|v| {
                        state
                            .add(std::sync::Arc::new(tokio::sync::Mutex::new((v).clone())))
                            .expect("failed to add item-stack resource")
                    })
                    .collect(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerHarvestBlockEvent(data) => Self {
                player: consume_player(state, &data.player),
                block_pos: from_wasm_block_position(data.block_pos),
                harvested_items: (data.harvested_items)
                    .into_iter()
                    .map(|v| consume_item_stack(state, &v))
                    .collect(),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerHideEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerHideEntityEvent(
            crate::pumpkin::plugin::event::PlayerHideEntityEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                entity_id: self.entity_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerHideEntityEvent(data) => Self {
                player: consume_player(state, &data.player),
                entity_id: data.entity_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerItemBreakEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerItemBreakEvent(
            crate::pumpkin::plugin::event::PlayerItemBreakEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                item_name: (&self.item_name).clone(),
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerItemBreakEvent(data) => Self {
                player: consume_player(state, &data.player),
                item_name: data.item_name,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerItemMendEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerItemMendEvent(
            crate::pumpkin::plugin::event::PlayerItemMendEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                item_name: (&self.item_name).clone(),
                repair_amount: self.repair_amount,
                exp_consumed: self.exp_consumed,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerItemMendEvent(data) => Self {
                player: consume_player(state, &data.player),
                item_name: data.item_name,
                repair_amount: data.repair_amount,
                exp_consumed: data.exp_consumed,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerKickEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerKickEvent(
            crate::pumpkin::plugin::event::PlayerKickEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                reason: (&self.reason).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerKickEvent(data) => Self {
                player: consume_player(state, &data.player),
                reason: data.reason,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerLeashEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerLeashEntityEvent(
            crate::pumpkin::plugin::event::PlayerLeashEntityEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                entity_id: self.entity_id,
                holder_id: self.holder_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerLeashEntityEvent(data) => Self {
                player: consume_player(state, &data.player),
                entity_id: data.entity_id,
                holder_id: data.holder_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerLevelChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerLevelChangeEvent(
            crate::pumpkin::plugin::event::PlayerLevelChangeEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                old_level: self.old_level,
                new_level: self.new_level,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerLevelChangeEvent(data) => Self {
                player: consume_player(state, &data.player),
                old_level: data.old_level,
                new_level: data.new_level,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerLocaleChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerLocaleChangeEvent(
            crate::pumpkin::plugin::event::PlayerLocaleChangeEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                new_locale: (&self.new_locale).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerLocaleChangeEvent(data) => Self {
                player: consume_player(state, &data.player),
                new_locale: data.new_locale,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerNameEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerNameEntityEvent(
            crate::pumpkin::plugin::event::PlayerNameEntityEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                entity_id: self.entity_id,
                name: state
                    .add((&self.name).clone())
                    .expect("failed to add text-component resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerNameEntityEvent(data) => Self {
                player: consume_player(state, &data.player),
                entity_id: data.entity_id,
                name: consume_text_component(state, &data.name),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerOpenSignEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerOpenSignEvent(
            crate::pumpkin::plugin::event::PlayerOpenSignEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                block_pos: to_wasm_block_position(self.block_pos),
                is_front: self.is_front,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerOpenSignEvent(data) => Self {
                player: consume_player(state, &data.player),
                block_pos: from_wasm_block_position(data.block_pos),
                is_front: data.is_front,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerPortalEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerPortalEvent(
            crate::pumpkin::plugin::event::PlayerPortalEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                from_pos: to_wasm_block_position(self.from_pos),
                to_pos: (self.to_pos).map(|v| to_wasm_block_position(v)),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerPortalEvent(data) => Self {
                player: consume_player(state, &data.player),
                from_pos: from_wasm_block_position(data.from_pos),
                to_pos: (data.to_pos).map(|v| from_wasm_block_position(v)),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerPreLoginEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerPreLoginEvent(
            crate::pumpkin::plugin::event::PlayerPreLoginEventData {
                player_name: (&self.player_name).clone(),
                player_uuid: <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::to_wit(
                    &self.player_uuid,
                ),
                ip_address: (&self.ip_address).to_string(),
                kick_message: state
                    .add((&self.kick_message).clone())
                    .expect("failed to add text-component resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerPreLoginEvent(data) => Self {
                player_name: data.player_name,
                player_uuid: <crate::pumpkin::plugin::uuid::Uuid as crate::uuid::UuidExt>::from_wit(
                    &data.player_uuid,
                ),
                ip_address: (data.ip_address)
                    .parse()
                    .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 0))),
                kick_message: consume_text_component(state, &data.kick_message),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerRiptideEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerRiptideEvent(
            crate::pumpkin::plugin::event::PlayerRiptideEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                item_name: (&self.item_name).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerRiptideEvent(data) => Self {
                player: consume_player(state, &data.player),
                item_name: data.item_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerShearEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerShearEntityEvent(
            crate::pumpkin::plugin::event::PlayerShearEntityEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                entity_id: self.entity_id,
                hand: self.hand,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerShearEntityEvent(data) => Self {
                player: consume_player(state, &data.player),
                entity_id: data.entity_id,
                hand: data.hand,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerShowEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerShowEntityEvent(
            crate::pumpkin::plugin::event::PlayerShowEntityEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                entity_id: self.entity_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerShowEntityEvent(data) => Self {
                player: consume_player(state, &data.player),
                entity_id: data.entity_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerSpawnChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerSpawnChangeEvent(
            crate::pumpkin::plugin::event::PlayerSpawnChangeEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                new_spawn: (self.new_spawn).map(|v| to_wasm_block_position(v)),
                forced: self.forced,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerSpawnChangeEvent(data) => Self {
                player: consume_player(state, &data.player),
                new_spawn: (data.new_spawn).map(|v| from_wasm_block_position(v)),
                forced: data.forced,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerStatisticIncrementEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerStatisticIncrementEvent(
            crate::pumpkin::plugin::event::PlayerStatisticIncrementEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                statistic_id: (&self.statistic_id).clone(),
                amount: self.amount,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerStatisticIncrementEvent(data) => Self {
                player: consume_player(state, &data.player),
                statistic_id: data.statistic_id,
                amount: data.amount,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerSwapHandsEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerSwapHandsEvent(
            crate::pumpkin::plugin::event::PlayerSwapHandsEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerSwapHandsEvent(data) => Self {
                player: consume_player(state, &data.player),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerTakeLecternBookEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerTakeLecternBookEvent(
            crate::pumpkin::plugin::event::PlayerTakeLecternBookEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                block_pos: to_wasm_block_position(self.block_pos),
                book: state
                    .add(std::sync::Arc::new(tokio::sync::Mutex::new(
                        (&self.book).clone(),
                    )))
                    .expect("failed to add item-stack resource"),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerTakeLecternBookEvent(data) => Self {
                player: consume_player(state, &data.player),
                block_pos: from_wasm_block_position(data.block_pos),
                book: consume_item_stack(state, &data.book),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerUnleashEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerUnleashEntityEvent(
            crate::pumpkin::plugin::event::PlayerUnleashEntityEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                entity_id: self.entity_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerUnleashEntityEvent(data) => Self {
                player: consume_player(state, &data.player),
                entity_id: data.entity_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerVelocityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerVelocityEvent(
            crate::pumpkin::plugin::event::PlayerVelocityEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                velocity: to_wasm_position(self.velocity),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerVelocityEvent(data) => Self {
                player: consume_player(state, &data.player),
                velocity: from_wasm_position(data.velocity),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerInputEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerInputEvent(
            crate::pumpkin::plugin::event::PlayerInputEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                input: (&self.input).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerInputEvent(data) => Self {
                player: consume_player(state, &data.player),
                input: data.input,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerInteractAtEntityEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerInteractAtEntityEvent(
            crate::pumpkin::plugin::event::PlayerInteractAtEntityEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                entity_id: self.entity_id,
                clicked_x: self.clicked_x,
                clicked_y: self.clicked_y,
                clicked_z: self.clicked_z,
                hand: self.hand,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerInteractAtEntityEvent(data) => Self {
                player: consume_player(state, &data.player),
                entity_id: data.entity_id,
                clicked_x: data.clicked_x,
                clicked_y: data.clicked_y,
                clicked_z: data.clicked_z,
                hand: data.hand,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerLinksSendEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerLinksSendEvent(
            crate::pumpkin::plugin::event::PlayerLinksSendEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                links: (&self.links).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerLinksSendEvent(data) => Self {
                player: consume_player(state, &data.player),
                links: data.links,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerPickupArrowEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerPickupArrowEvent(
            crate::pumpkin::plugin::event::PlayerPickupArrowEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                arrow_id: self.arrow_id,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerPickupArrowEvent(data) => Self {
                player: consume_player(state, &data.player),
                arrow_id: data.arrow_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerRecipeBookClickEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerRecipeBookClickEvent(
            crate::pumpkin::plugin::event::PlayerRecipeBookClickEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                recipe_id: (&self.recipe_id).clone(),
                make_all: self.make_all,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerRecipeBookClickEvent(data) => Self {
                player: consume_player(state, &data.player),
                recipe_id: data.recipe_id,
                make_all: data.make_all,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerRecipeBookSettingsChangeEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerRecipeBookSettingsChangeEvent(
            crate::pumpkin::plugin::event::PlayerRecipeBookSettingsChangeEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                book_type: (&self.book_type).clone(),
                is_open: self.is_open,
                is_filtering: self.is_filtering,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerRecipeBookSettingsChangeEvent(data) => {
                Self {
                    player: consume_player(state, &data.player),
                    book_type: data.book_type,
                    is_open: data.is_open,
                    is_filtering: data.is_filtering,
                    cancelled: data.cancelled,
                }
            }
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerRecipeDiscoverEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerRecipeDiscoverEvent(
            crate::pumpkin::plugin::event::PlayerRecipeDiscoverEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                recipe_id: (&self.recipe_id).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerRecipeDiscoverEvent(data) => Self {
                player: consume_player(state, &data.player),
                recipe_id: data.recipe_id,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerRegisterChannelEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerRegisterChannelEvent(
            crate::pumpkin::plugin::event::PlayerRegisterChannelEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                channel: (&self.channel).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerRegisterChannelEvent(data) => Self {
                player: consume_player(state, &data.player),
                channel: data.channel,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerResourcePackStatusEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerResourcePackStatusEvent(
            crate::pumpkin::plugin::event::PlayerResourcePackStatusEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                pack_id: (&self.pack_id).clone(),
                status: (&self.status).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerResourcePackStatusEvent(data) => Self {
                player: consume_player(state, &data.player),
                pack_id: data.pack_id,
                status: data.status,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerSpawnLocationEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerSpawnLocationEvent(
            crate::pumpkin::plugin::event::PlayerSpawnLocationEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                spawn_pos: to_wasm_position(self.spawn_pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerSpawnLocationEvent(data) => Self {
                player: consume_player(state, &data.player),
                spawn_pos: from_wasm_position(data.spawn_pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PlayerUnregisterChannelEvent {
    fn to_wasm_event(
        &self,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PlayerUnregisterChannelEvent(
            crate::pumpkin::plugin::event::PlayerUnregisterChannelEventData {
                player: state
                    .add(std::sync::Arc::clone(&self.player))
                    .expect("failed to add player resource"),
                channel: (&self.channel).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PlayerUnregisterChannelEvent(data) => Self {
                player: consume_player(state, &data.player),
                channel: data.channel,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::RaidFinishEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::RaidFinishEvent(
            crate::pumpkin::plugin::event::RaidFinishEventData {
                victory: self.victory,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::RaidFinishEvent(data) => Self {
                victory: data.victory,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::RaidSpawnWaveEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::RaidSpawnWaveEvent(
            crate::pumpkin::plugin::event::RaidSpawnWaveEventData {
                wave: self.wave,
                pos: to_wasm_block_position(self.pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::RaidSpawnWaveEvent(data) => Self {
                wave: data.wave,
                pos: from_wasm_block_position(data.pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::RaidStopEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::RaidStopEvent(
            crate::pumpkin::plugin::event::RaidStopEventData {
                reason: (&self.reason).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::RaidStopEvent(data) => Self {
                reason: data.reason,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::RaidTriggerEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::RaidTriggerEvent(
            crate::pumpkin::plugin::event::RaidTriggerEventData {
                pos: to_wasm_block_position(self.pos),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::RaidTriggerEvent(data) => Self {
                pos: from_wasm_block_position(data.pos),
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::LightningStrikeEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::LightningStrikeEvent(
            crate::pumpkin::plugin::event::LightningStrikeEventData {
                position: to_wasm_position(self.position),
                is_effect: self.is_effect,
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::LightningStrikeEvent(data) => Self {
                position: from_wasm_position(data.position),
                is_effect: data.is_effect,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PluginEnableEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PluginEnableEvent(
            crate::pumpkin::plugin::event::PluginEnableEventData {
                plugin_name: (&self.plugin_name).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PluginEnableEvent(data) => Self {
                plugin_name: data.plugin_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::PluginDisableEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::PluginDisableEvent(
            crate::pumpkin::plugin::event::PluginDisableEventData {
                plugin_name: (&self.plugin_name).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::PluginDisableEvent(data) => Self {
                plugin_name: data.plugin_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ServiceRegisterEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ServiceRegisterEvent(
            crate::pumpkin::plugin::event::ServiceRegisterEventData {
                service_name: (&self.service_name).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ServiceRegisterEvent(data) => Self {
                service_name: data.service_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::ServiceUnregisterEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::ServiceUnregisterEvent(
            crate::pumpkin::plugin::event::ServiceUnregisterEventData {
                service_name: (&self.service_name).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::ServiceUnregisterEvent(data) => Self {
                service_name: data.service_name,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
impl ToFromWasmEvent for pumpkin_core::plugin::TabCompleteEvent {
    fn to_wasm_event(
        &self,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> crate::pumpkin::plugin::event::Event {
        crate::pumpkin::plugin::event::Event::TabCompleteEvent(
            crate::pumpkin::plugin::event::TabCompleteEventData {
                buffer: (&self.buffer).clone(),
                completions: (&self.completions).clone(),
                cancelled: self.cancelled,
            },
        )
    }
    fn from_wasm_event(
        event: crate::pumpkin::plugin::event::Event,
        _state: &mut pumpkin_wasm_host_common::state::PluginHostState,
    ) -> Self {
        match event {
            crate::pumpkin::plugin::event::Event::TabCompleteEvent(data) => Self {
                buffer: data.buffer,
                completions: data.completions,
                cancelled: data.cancelled,
            },
            _ => panic!("unexpected event type"),
        }
    }
}
pub fn cleanup_event(
    event: &crate::pumpkin::plugin::event::Event,
    state: &mut pumpkin_wasm_host_common::state::PluginHostState,
) {
    match event {
        crate::pumpkin::plugin::event::Event::PlayerJoinEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.join_message);
        }
        crate::pumpkin::plugin::event::Event::PlayerLeaveEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.leave_message);
        }
        crate::pumpkin::plugin::event::Event::PlayerLoginEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.kick_message);
        }
        crate::pumpkin::plugin::event::Event::PlayerChatEvent(data) => {
            cleanup_player(state, &data.player);
            for res in &data.recipients {
                cleanup_player(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::PlayerCommandSendEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerPermissionCheckEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerMoveEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerTeleportEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerChangeWorldEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_world(state, &data.previous_world);
            cleanup_world(state, &data.new_world);
        }
        crate::pumpkin::plugin::event::Event::PlayerRespawnEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_world(state, &data.previous_world);
            cleanup_world(state, &data.respawned_world);
        }
        crate::pumpkin::plugin::event::Event::PlayerExpChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerItemHeldEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerChangedMainHandEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerGamemodeChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerCustomPayloadEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerFishEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerEggThrowEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerInteractUnknownEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerInteractEntityEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_entity(state, &data.target);
        }
        crate::pumpkin::plugin::event::Event::PlayerInteractEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerToggleSneakEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerToggleFlightEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerToggleSprintEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::InventoryClickEvent(data) => {
            cleanup_player(state, &data.player);
            if let Some(res) = &data.clicked_item {
                cleanup_item_stack(state, res);
            }
            if let Some(res) = &data.cursor {
                cleanup_item_stack(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::InventoryCloseEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::BlockRedstoneEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::BlockBreakEvent(data) => {
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::BlockBurnEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::BlockCanBuildEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::BlockGrowEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::BlockPlaceEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::DialogClickActionEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::DialogShowEvent(data) => {
            cleanup_player(state, &data.player);
            crate::events::dialog::cleanup_dialog(state, &data.dialog);
        }
        crate::pumpkin::plugin::event::Event::DialogClearEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::BedrockFormResponseEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::ServerCommandEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ServerListPingEvent(data) => {
            cleanup_text_component(state, &data.motd);
        }
        crate::pumpkin::plugin::event::Event::ServerLoadEvent(_) => {}
        crate::pumpkin::plugin::event::Event::SpawnChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::ServerBroadcastEvent(data) => {
            cleanup_text_component(state, &data.message);
            cleanup_text_component(state, &data.sender);
        }
        crate::pumpkin::plugin::event::Event::ServerTickStartEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ServerTickEndEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PacketReceivedEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PacketSentEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::JavaConnectionPacketReceivedEvent(_) => {}
        crate::pumpkin::plugin::event::Event::JavaConnectionPacketSentEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BedrockConnectionPacketReceivedEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BedrockConnectionPacketSentEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ChunkLoadEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::ChunkSaveEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::ChunkSendEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::EntityDamageEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityDeathEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PlayerDeathEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.death_message);
        }
        crate::pumpkin::plugin::event::Event::EntitySpawnEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::EntityCombustEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityRegainHealthEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityAirChangeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityBreedEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityDismountEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityDyeEvent(data) => {
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::EntityEnterLoveModeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityExplodeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityMountEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityPickupItemEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityPortalEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityResurrectEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityShootBowEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityTameEvent(data) => {
            cleanup_player(state, &data.owner);
        }
        crate::pumpkin::plugin::event::Event::EntityTargetEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityTeleportEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityToggleGlideEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityTransformEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PlayerItemConsumeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerItemDamageEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerDropItemEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerBedEnterEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerBedLeaveEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerBucketEmptyEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerBucketFillEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::BlockDamageEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::BlockIgniteEvent(data) => {
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::BlockFromToEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BlockFormEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BlockFadeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BlockDispenseEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BlockExplodeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BlockPhysicsEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BlockPistonExtendEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BlockPistonRetractEvent(_) => {}
        crate::pumpkin::plugin::event::Event::NotePlayEvent(_) => {}
        crate::pumpkin::plugin::event::Event::SignChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::SpongeAbsorbEvent(_) => {}
        crate::pumpkin::plugin::event::Event::TntPrimeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::WeatherChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::ThunderChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::WorldLoadEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::WorldUnloadEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::AsyncStructureGenerateEvent(_) => {}
        crate::pumpkin::plugin::event::Event::AsyncStructureSpawnEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ChunkPopulateEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ChunkUnloadEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntitiesLoadEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntitiesUnloadEvent(_) => {}
        crate::pumpkin::plugin::event::Event::GenericGameEvent(_) => {}
        crate::pumpkin::plugin::event::Event::LootGenerateEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PortalCreateEvent(_) => {}
        crate::pumpkin::plugin::event::Event::StructureGrowEvent(_) => {}
        crate::pumpkin::plugin::event::Event::TimeSkipEvent(_) => {}
        crate::pumpkin::plugin::event::Event::WorldInitEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::WorldSaveEvent(_) => {}
        crate::pumpkin::plugin::event::Event::InventoryOpenEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::InventoryDragEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::CraftItemEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::FurnaceSmeltEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BrewEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BrewingStandFuelEvent(_) => {}
        crate::pumpkin::plugin::event::Event::FurnaceBurnEvent(_) => {}
        crate::pumpkin::plugin::event::Event::FurnaceExtractEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::FurnaceStartSmeltEvent(_) => {}
        crate::pumpkin::plugin::event::Event::HopperInventorySearchEvent(_) => {}
        crate::pumpkin::plugin::event::Event::InventoryCreativeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::InventoryInteractEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::InventoryMoveItemEvent(_) => {}
        crate::pumpkin::plugin::event::Event::InventoryPickupItemEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PrepareAnvilEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PrepareGrindstoneEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PrepareInventoryResultEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PrepareItemCraftEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PrepareSmithingEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::SmithItemEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::TradeSelectEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::VehicleBlockCollisionEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VehicleCollisionEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VehicleCreateEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VehicleDamageEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VehicleDestroyEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VehicleEnterEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VehicleEntityCollisionEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VehicleExitEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VehicleMoveEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VehicleUpdateEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PrepareItemEnchantEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_item_stack(state, &data.item);
        }
        crate::pumpkin::plugin::event::Event::EnchantItemEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_item_stack(state, &data.item);
        }
        crate::pumpkin::plugin::event::Event::MapInitializeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::HangingBreakEvent(data) => {
            cleanup_entity(state, &data.entity);
            if let Some(res) = &data.remover {
                cleanup_entity(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::HangingBreakByEntityEvent(data) => {
            cleanup_entity(state, &data.entity);
            cleanup_entity(state, &data.remover);
        }
        crate::pumpkin::plugin::event::Event::HangingPlaceEvent(data) => {
            cleanup_entity(state, &data.entity);
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::BellResonateEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::BellRingEvent(data) => {
            cleanup_world(state, &data.target_world);
            if let Some(res) = &data.entity {
                cleanup_entity(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::BlockBrushEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_player(state, &data.player);
            cleanup_item_stack(state, &data.item);
        }
        crate::pumpkin::plugin::event::Event::BlockCookEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.source);
            cleanup_item_stack(state, &data.result);
        }
        crate::pumpkin::plugin::event::Event::BlockDamageAbortEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.item_in_hand);
        }
        crate::pumpkin::plugin::event::Event::BlockDispenseArmorEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_entity(state, &data.target);
            cleanup_item_stack(state, &data.item);
        }
        crate::pumpkin::plugin::event::Event::BlockDispenseLootEvent(data) => {
            cleanup_world(state, &data.target_world);
            for res in &data.items {
                cleanup_item_stack(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::BlockDropItemEvent(data) => {
            cleanup_world(state, &data.target_world);
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
            for res in &data.items {
                cleanup_item_stack(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::BlockExpEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::BlockFertilizeEvent(data) => {
            cleanup_world(state, &data.target_world);
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::BlockMultiPlaceEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::BlockReceiveGameEvent(data) => {
            cleanup_world(state, &data.target_world);
            if let Some(res) = &data.source_entity {
                cleanup_entity(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::BlockShearEntityEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_entity(state, &data.target);
            cleanup_item_stack(state, &data.item);
        }
        crate::pumpkin::plugin::event::Event::BlockSpreadEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::BrewingStartEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::CampfireStartEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.item);
        }
        crate::pumpkin::plugin::event::Event::CauldronLevelChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
            if let Some(res) = &data.entity {
                cleanup_entity(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::CrafterCraftEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.result);
        }
        crate::pumpkin::plugin::event::Event::EntityBlockFormEvent(data) => {
            cleanup_entity(state, &data.entity);
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::FluidLevelChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::InventoryBlockStartEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::LeavesDecayEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::MoistureChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::SculkBloomEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        crate::pumpkin::plugin::event::Event::VaultDisplayItemEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.item);
        }
        crate::pumpkin::plugin::event::Event::CreatureSpawnEvent(data) => {
            cleanup_world(state, &data.target_world);
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::EnderDragonChangePhaseEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityBreakDoorEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityChangeBlockEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityDamageByBlockEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityDamageByEntityEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityDropItemEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityEnterBlockEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityExhaustionEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityInteractEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityKnockbackEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityPlaceEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityPoseChangeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityPotionEffectEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntitySpellCastEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityTargetLivingEntityEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityToggleSitEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityToggleSwimEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ExplosionPrimeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::FireworkExplodeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::FoodLevelChangeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ItemDespawnEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ItemMergeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ItemSpawnEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PiglinBarterEvent(data) => {
            cleanup_item_stack(state, &data.input_item);
            for res in &data.outcome {
                cleanup_item_stack(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::ProjectileHitEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ProjectileLaunchEvent(_) => {}
        crate::pumpkin::plugin::event::Event::SheepDyeWoolEvent(_) => {}
        crate::pumpkin::plugin::event::Event::SheepRegrowWoolEvent(_) => {}
        crate::pumpkin::plugin::event::Event::SlimeSplitEvent(_) => {}
        crate::pumpkin::plugin::event::Event::StriderTemperatureChangeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VillagerAcquireTradeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VillagerCareerChangeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VillagerReplenishTradeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::WardenAngerChangeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::AreaEffectCloudApplyEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ArrowBodyCountChangeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::BatToggleSleepEvent(_) => {}
        crate::pumpkin::plugin::event::Event::CreeperPowerEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityCombustByBlockEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityCombustByEntityEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityKnockbackByEntityEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityPortalEnterEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityPortalExitEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityRemoveEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityTargetBlockEvent(_) => {}
        crate::pumpkin::plugin::event::Event::EntityUnleashEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ExpBottleEvent(_) => {}
        crate::pumpkin::plugin::event::Event::HorseJumpEvent(_) => {}
        crate::pumpkin::plugin::event::Event::LingeringPotionSplashEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PigZapEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PigZombieAngerEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PotionSplashEvent(_) => {}
        crate::pumpkin::plugin::event::Event::SpawnerSpawnEvent(_) => {}
        crate::pumpkin::plugin::event::Event::TrialSpawnerSpawnEvent(_) => {}
        crate::pumpkin::plugin::event::Event::VillagerReputationChangeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::AsyncPlayerChatEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.format);
        }
        crate::pumpkin::plugin::event::Event::AsyncPlayerPreLoginEvent(data) => {
            cleanup_text_component(state, &data.kick_message);
        }
        crate::pumpkin::plugin::event::Event::PlayerAdvancementDoneEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerAnimationEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerArmorStandManipulateEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerBucketEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerChangedWorldEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_world(state, &data.from_world);
            cleanup_world(state, &data.to_world);
        }
        crate::pumpkin::plugin::event::Event::PlayerChannelEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerCommandPreprocessEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerEditBookEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerElytraBoostEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerExpCooldownChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerHarvestBlockEvent(data) => {
            cleanup_player(state, &data.player);
            for res in &data.harvested_items {
                cleanup_item_stack(state, res);
            }
        }
        crate::pumpkin::plugin::event::Event::PlayerHideEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerItemBreakEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerItemMendEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerKickEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerLeashEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerLevelChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerLocaleChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerNameEntityEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.name);
        }
        crate::pumpkin::plugin::event::Event::PlayerOpenSignEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerPortalEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerPreLoginEvent(data) => {
            cleanup_text_component(state, &data.kick_message);
        }
        crate::pumpkin::plugin::event::Event::PlayerRiptideEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerShearEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerShowEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerSpawnChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerStatisticIncrementEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerSwapHandsEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerTakeLecternBookEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_item_stack(state, &data.book);
        }
        crate::pumpkin::plugin::event::Event::PlayerUnleashEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerVelocityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerInputEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerInteractAtEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerLinksSendEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerPickupArrowEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerRecipeBookClickEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerRecipeBookSettingsChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerRecipeDiscoverEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerRegisterChannelEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerResourcePackStatusEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerSpawnLocationEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::PlayerUnregisterChannelEvent(data) => {
            cleanup_player(state, &data.player);
        }
        crate::pumpkin::plugin::event::Event::RaidFinishEvent(_) => {}
        crate::pumpkin::plugin::event::Event::RaidSpawnWaveEvent(_) => {}
        crate::pumpkin::plugin::event::Event::RaidStopEvent(_) => {}
        crate::pumpkin::plugin::event::Event::RaidTriggerEvent(_) => {}
        crate::pumpkin::plugin::event::Event::LightningStrikeEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PluginEnableEvent(_) => {}
        crate::pumpkin::plugin::event::Event::PluginDisableEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ServiceRegisterEvent(_) => {}
        crate::pumpkin::plugin::event::Event::ServiceUnregisterEvent(_) => {}
        crate::pumpkin::plugin::event::Event::TabCompleteEvent(_) => {}
    }
}
pub fn register_generated_event(
    ctx: &pumpkin_core::plugin::Context,
    handler: &std::sync::Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: crate::pumpkin::plugin::event::EventType,
) {
    match event_type {
        crate::pumpkin::plugin::event::EventType::PlayerJoinEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerJoinEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerLeaveEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerLeaveEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerLoginEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerLoginEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerChatEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerChatEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerCommandSendEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerCommandSendEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerPermissionCheckEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerPermissionCheckEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerMoveEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerMoveEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerTeleportEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerTeleportEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerChangeWorldEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerChangeWorldEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerRespawnEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerRespawnEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerExpChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerExpChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerItemHeldEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerItemHeldEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerChangedMainHandEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerChangedMainHandEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerGamemodeChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerGamemodeChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerCustomPayloadEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerCustomPayloadEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerFishEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerFishEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerEggThrowEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerEggThrowEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerInteractUnknownEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerInteractUnknownEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerInteractEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerInteractEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerInteractEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerInteractEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerToggleSneakEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerToggleSneakEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerToggleFlightEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerToggleFlightEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerToggleSprintEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerToggleSprintEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::InventoryClickEvent => {
            ctx.register_event::<pumpkin_core::plugin::InventoryClickEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::InventoryCloseEvent => {
            ctx.register_event::<pumpkin_core::plugin::InventoryCloseEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockRedstoneEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockRedstoneEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockBreakEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockBreakEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockBurnEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockBurnEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockCanBuildEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockCanBuildEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockGrowEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockGrowEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockPlaceEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockPlaceEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::DialogClickActionEvent => {
            ctx.register_event::<pumpkin_core::plugin::DialogClickActionEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::DialogShowEvent => {
            ctx.register_event::<pumpkin_core::plugin::DialogShowEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::DialogClearEvent => {
            ctx.register_event::<pumpkin_core::plugin::DialogClearEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BedrockFormResponseEvent => {
            ctx.register_event::<pumpkin_core::plugin::BedrockFormResponseEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ServerCommandEvent => {
            ctx.register_event::<pumpkin_core::plugin::ServerCommandEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ServerListPingEvent => {
            ctx.register_event::<pumpkin_core::plugin::ServerListPingEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ServerLoadEvent => {
            ctx.register_event::<pumpkin_core::plugin::ServerLoadEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::SpawnChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::SpawnChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ServerBroadcastEvent => {
            ctx.register_event::<pumpkin_core::plugin::ServerBroadcastEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ServerTickStartEvent => {
            ctx.register_event::<pumpkin_core::plugin::ServerTickStartEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ServerTickEndEvent => {
            ctx.register_event::<pumpkin_core::plugin::ServerTickEndEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PacketReceivedEvent => {
            ctx.register_event::<pumpkin_core::plugin::PacketReceivedEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PacketSentEvent => {
            ctx.register_event::<pumpkin_core::plugin::PacketSentEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::JavaConnectionPacketReceivedEvent => {
            ctx.register_event::<pumpkin_core::plugin::JavaConnectionPacketReceivedEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::JavaConnectionPacketSentEvent => {
            ctx.register_event::<pumpkin_core::plugin::JavaConnectionPacketSentEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BedrockConnectionPacketReceivedEvent => {
            ctx.register_event::<pumpkin_core::plugin::BedrockConnectionPacketReceivedEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BedrockConnectionPacketSentEvent => {
            ctx.register_event::<pumpkin_core::plugin::BedrockConnectionPacketSentEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ChunkLoadEvent => {
            ctx.register_event::<pumpkin_core::plugin::ChunkLoadEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ChunkSaveEvent => {
            ctx.register_event::<pumpkin_core::plugin::ChunkSaveEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ChunkSendEvent => {
            ctx.register_event::<pumpkin_core::plugin::ChunkSendEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityDamageEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityDamageEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityDeathEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityDeathEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerDeathEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerDeathEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntitySpawnEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntitySpawnEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityCombustEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityCombustEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityRegainHealthEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityRegainHealthEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityAirChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityAirChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityBreedEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityBreedEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityDismountEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityDismountEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityDyeEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityDyeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityEnterLoveModeEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityEnterLoveModeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityExplodeEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityExplodeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityMountEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityMountEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityPickupItemEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityPickupItemEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityPortalEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityPortalEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityResurrectEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityResurrectEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityShootBowEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityShootBowEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityTameEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityTameEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityTargetEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityTargetEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityTeleportEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityTeleportEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityToggleGlideEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityToggleGlideEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityTransformEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityTransformEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerItemConsumeEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerItemConsumeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerItemDamageEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerItemDamageEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerDropItemEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerDropItemEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerBedEnterEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerBedEnterEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerBedLeaveEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerBedLeaveEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerBucketEmptyEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerBucketEmptyEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerBucketFillEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerBucketFillEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockDamageEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockDamageEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockIgniteEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockIgniteEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockFromToEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockFromToEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockFormEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockFormEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockFadeEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockFadeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockDispenseEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockDispenseEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockExplodeEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockExplodeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockPhysicsEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockPhysicsEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockPistonExtendEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockPistonExtendEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockPistonRetractEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockPistonRetractEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::NotePlayEvent => {
            ctx.register_event::<pumpkin_core::plugin::NotePlayEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::SignChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::SignChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::SpongeAbsorbEvent => {
            ctx.register_event::<pumpkin_core::plugin::SpongeAbsorbEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::TntPrimeEvent => {
            ctx.register_event::<pumpkin_core::plugin::TntPrimeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::WeatherChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::WeatherChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ThunderChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::ThunderChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::WorldLoadEvent => {
            ctx.register_event::<pumpkin_core::plugin::WorldLoadEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::WorldUnloadEvent => {
            ctx.register_event::<pumpkin_core::plugin::WorldUnloadEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::AsyncStructureGenerateEvent => {
            ctx.register_event::<pumpkin_core::plugin::AsyncStructureGenerateEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::AsyncStructureSpawnEvent => {
            ctx.register_event::<pumpkin_core::plugin::AsyncStructureSpawnEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ChunkPopulateEvent => {
            ctx.register_event::<pumpkin_core::plugin::ChunkPopulateEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ChunkUnloadEvent => {
            ctx.register_event::<pumpkin_core::plugin::ChunkUnloadEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntitiesLoadEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntitiesLoadEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntitiesUnloadEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntitiesUnloadEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::GenericGameEvent => {
            ctx.register_event::<pumpkin_core::plugin::GenericGameEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::LootGenerateEvent => {
            ctx.register_event::<pumpkin_core::plugin::LootGenerateEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PortalCreateEvent => {
            ctx.register_event::<pumpkin_core::plugin::PortalCreateEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::StructureGrowEvent => {
            ctx.register_event::<pumpkin_core::plugin::StructureGrowEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::TimeSkipEvent => {
            ctx.register_event::<pumpkin_core::plugin::TimeSkipEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::WorldInitEvent => {
            ctx.register_event::<pumpkin_core::plugin::WorldInitEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::WorldSaveEvent => {
            ctx.register_event::<pumpkin_core::plugin::WorldSaveEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::InventoryOpenEvent => {
            ctx.register_event::<pumpkin_core::plugin::InventoryOpenEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::InventoryDragEvent => {
            ctx.register_event::<pumpkin_core::plugin::InventoryDragEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::CraftItemEvent => {
            ctx.register_event::<pumpkin_core::plugin::CraftItemEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::FurnaceSmeltEvent => {
            ctx.register_event::<pumpkin_core::plugin::FurnaceSmeltEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BrewEvent => {
            ctx.register_event::<pumpkin_core::plugin::BrewEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BrewingStandFuelEvent => {
            ctx.register_event::<pumpkin_core::plugin::BrewingStandFuelEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::FurnaceBurnEvent => {
            ctx.register_event::<pumpkin_core::plugin::FurnaceBurnEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::FurnaceExtractEvent => {
            ctx.register_event::<pumpkin_core::plugin::FurnaceExtractEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::FurnaceStartSmeltEvent => {
            ctx.register_event::<pumpkin_core::plugin::FurnaceStartSmeltEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::HopperInventorySearchEvent => {
            ctx.register_event::<pumpkin_core::plugin::HopperInventorySearchEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::InventoryCreativeEvent => {
            ctx.register_event::<pumpkin_core::plugin::InventoryCreativeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::InventoryInteractEvent => {
            ctx.register_event::<pumpkin_core::plugin::InventoryInteractEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::InventoryMoveItemEvent => {
            ctx.register_event::<pumpkin_core::plugin::InventoryMoveItemEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::InventoryPickupItemEvent => {
            ctx.register_event::<pumpkin_core::plugin::InventoryPickupItemEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PrepareAnvilEvent => {
            ctx.register_event::<pumpkin_core::plugin::PrepareAnvilEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PrepareGrindstoneEvent => {
            ctx.register_event::<pumpkin_core::plugin::PrepareGrindstoneEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PrepareInventoryResultEvent => {
            ctx.register_event::<pumpkin_core::plugin::PrepareInventoryResultEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PrepareItemCraftEvent => {
            ctx.register_event::<pumpkin_core::plugin::PrepareItemCraftEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PrepareSmithingEvent => {
            ctx.register_event::<pumpkin_core::plugin::PrepareSmithingEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::SmithItemEvent => {
            ctx.register_event::<pumpkin_core::plugin::SmithItemEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::TradeSelectEvent => {
            ctx.register_event::<pumpkin_core::plugin::TradeSelectEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VehicleBlockCollisionEvent => {
            ctx.register_event::<pumpkin_core::plugin::VehicleBlockCollisionEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VehicleCollisionEvent => {
            ctx.register_event::<pumpkin_core::plugin::VehicleCollisionEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VehicleCreateEvent => {
            ctx.register_event::<pumpkin_core::plugin::VehicleCreateEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VehicleDamageEvent => {
            ctx.register_event::<pumpkin_core::plugin::VehicleDamageEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VehicleDestroyEvent => {
            ctx.register_event::<pumpkin_core::plugin::VehicleDestroyEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VehicleEnterEvent => {
            ctx.register_event::<pumpkin_core::plugin::VehicleEnterEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VehicleEntityCollisionEvent => {
            ctx.register_event::<pumpkin_core::plugin::VehicleEntityCollisionEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VehicleExitEvent => {
            ctx.register_event::<pumpkin_core::plugin::VehicleExitEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VehicleMoveEvent => {
            ctx.register_event::<pumpkin_core::plugin::VehicleMoveEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VehicleUpdateEvent => {
            ctx.register_event::<pumpkin_core::plugin::VehicleUpdateEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PrepareItemEnchantEvent => {
            ctx.register_event::<pumpkin_core::plugin::PrepareItemEnchantEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EnchantItemEvent => {
            ctx.register_event::<pumpkin_core::plugin::EnchantItemEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::MapInitializeEvent => {
            ctx.register_event::<pumpkin_core::plugin::MapInitializeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::HangingBreakEvent => {
            ctx.register_event::<pumpkin_core::plugin::HangingBreakEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::HangingBreakByEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::HangingBreakByEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::HangingPlaceEvent => {
            ctx.register_event::<pumpkin_core::plugin::HangingPlaceEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BellResonateEvent => {
            ctx.register_event::<pumpkin_core::plugin::BellResonateEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BellRingEvent => {
            ctx.register_event::<pumpkin_core::plugin::BellRingEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockBrushEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockBrushEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockCookEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockCookEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockDamageAbortEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockDamageAbortEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockDispenseArmorEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockDispenseArmorEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockDispenseLootEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockDispenseLootEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockDropItemEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockDropItemEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockExpEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockExpEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockFertilizeEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockFertilizeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockMultiPlaceEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockMultiPlaceEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockReceiveGameEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockReceiveGameEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockShearEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockShearEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BlockSpreadEvent => {
            ctx.register_event::<pumpkin_core::plugin::BlockSpreadEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BrewingStartEvent => {
            ctx.register_event::<pumpkin_core::plugin::BrewingStartEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::CampfireStartEvent => {
            ctx.register_event::<pumpkin_core::plugin::CampfireStartEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::CauldronLevelChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::CauldronLevelChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::CrafterCraftEvent => {
            ctx.register_event::<pumpkin_core::plugin::CrafterCraftEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityBlockFormEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityBlockFormEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::FluidLevelChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::FluidLevelChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::InventoryBlockStartEvent => {
            ctx.register_event::<pumpkin_core::plugin::InventoryBlockStartEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::LeavesDecayEvent => {
            ctx.register_event::<pumpkin_core::plugin::LeavesDecayEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::MoistureChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::MoistureChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::SculkBloomEvent => {
            ctx.register_event::<pumpkin_core::plugin::SculkBloomEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VaultDisplayItemEvent => {
            ctx.register_event::<pumpkin_core::plugin::VaultDisplayItemEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::CreatureSpawnEvent => {
            ctx.register_event::<pumpkin_core::plugin::CreatureSpawnEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EnderDragonChangePhaseEvent => {
            ctx.register_event::<pumpkin_core::plugin::EnderDragonChangePhaseEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityBreakDoorEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityBreakDoorEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityChangeBlockEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityChangeBlockEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityDamageByBlockEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityDamageByBlockEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityDamageByEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityDamageByEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityDropItemEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityDropItemEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityEnterBlockEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityEnterBlockEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityExhaustionEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityExhaustionEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityInteractEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityInteractEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityKnockbackEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityKnockbackEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityPlaceEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityPlaceEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityPoseChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityPoseChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityPotionEffectEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityPotionEffectEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntitySpellCastEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntitySpellCastEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityTargetLivingEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityTargetLivingEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityToggleSitEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityToggleSitEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityToggleSwimEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityToggleSwimEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ExplosionPrimeEvent => {
            ctx.register_event::<pumpkin_core::plugin::ExplosionPrimeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::FireworkExplodeEvent => {
            ctx.register_event::<pumpkin_core::plugin::FireworkExplodeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::FoodLevelChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::FoodLevelChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ItemDespawnEvent => {
            ctx.register_event::<pumpkin_core::plugin::ItemDespawnEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ItemMergeEvent => {
            ctx.register_event::<pumpkin_core::plugin::ItemMergeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ItemSpawnEvent => {
            ctx.register_event::<pumpkin_core::plugin::ItemSpawnEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PiglinBarterEvent => {
            ctx.register_event::<pumpkin_core::plugin::PiglinBarterEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ProjectileHitEvent => {
            ctx.register_event::<pumpkin_core::plugin::ProjectileHitEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ProjectileLaunchEvent => {
            ctx.register_event::<pumpkin_core::plugin::ProjectileLaunchEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::SheepDyeWoolEvent => {
            ctx.register_event::<pumpkin_core::plugin::SheepDyeWoolEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::SheepRegrowWoolEvent => {
            ctx.register_event::<pumpkin_core::plugin::SheepRegrowWoolEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::SlimeSplitEvent => {
            ctx.register_event::<pumpkin_core::plugin::SlimeSplitEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::StriderTemperatureChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::StriderTemperatureChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VillagerAcquireTradeEvent => {
            ctx.register_event::<pumpkin_core::plugin::VillagerAcquireTradeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VillagerCareerChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::VillagerCareerChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VillagerReplenishTradeEvent => {
            ctx.register_event::<pumpkin_core::plugin::VillagerReplenishTradeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::WardenAngerChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::WardenAngerChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::AreaEffectCloudApplyEvent => {
            ctx.register_event::<pumpkin_core::plugin::AreaEffectCloudApplyEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ArrowBodyCountChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::ArrowBodyCountChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::BatToggleSleepEvent => {
            ctx.register_event::<pumpkin_core::plugin::BatToggleSleepEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::CreeperPowerEvent => {
            ctx.register_event::<pumpkin_core::plugin::CreeperPowerEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityCombustByBlockEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityCombustByBlockEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityCombustByEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityCombustByEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityKnockbackByEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityKnockbackByEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityPortalEnterEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityPortalEnterEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityPortalExitEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityPortalExitEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityRemoveEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityRemoveEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityTargetBlockEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityTargetBlockEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::EntityUnleashEvent => {
            ctx.register_event::<pumpkin_core::plugin::EntityUnleashEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ExpBottleEvent => {
            ctx.register_event::<pumpkin_core::plugin::ExpBottleEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::HorseJumpEvent => {
            ctx.register_event::<pumpkin_core::plugin::HorseJumpEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::LingeringPotionSplashEvent => {
            ctx.register_event::<pumpkin_core::plugin::LingeringPotionSplashEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PigZapEvent => {
            ctx.register_event::<pumpkin_core::plugin::PigZapEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PigZombieAngerEvent => {
            ctx.register_event::<pumpkin_core::plugin::PigZombieAngerEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PotionSplashEvent => {
            ctx.register_event::<pumpkin_core::plugin::PotionSplashEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::SpawnerSpawnEvent => {
            ctx.register_event::<pumpkin_core::plugin::SpawnerSpawnEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::TrialSpawnerSpawnEvent => {
            ctx.register_event::<pumpkin_core::plugin::TrialSpawnerSpawnEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::VillagerReputationChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::VillagerReputationChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::AsyncPlayerChatEvent => {
            ctx.register_event::<pumpkin_core::plugin::AsyncPlayerChatEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::AsyncPlayerPreLoginEvent => {
            ctx.register_event::<pumpkin_core::plugin::AsyncPlayerPreLoginEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerAdvancementDoneEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerAdvancementDoneEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerAnimationEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerAnimationEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerArmorStandManipulateEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerArmorStandManipulateEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerBucketEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerBucketEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerChangedWorldEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerChangedWorldEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerChannelEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerChannelEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerCommandPreprocessEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerCommandPreprocessEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerEditBookEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerEditBookEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerElytraBoostEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerElytraBoostEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerExpCooldownChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerExpCooldownChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerHarvestBlockEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerHarvestBlockEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerHideEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerHideEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerItemBreakEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerItemBreakEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerItemMendEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerItemMendEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerKickEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerKickEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerLeashEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerLeashEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerLevelChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerLevelChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerLocaleChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerLocaleChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerNameEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerNameEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerOpenSignEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerOpenSignEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerPortalEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerPortalEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerPreLoginEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerPreLoginEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerRiptideEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerRiptideEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerShearEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerShearEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerShowEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerShowEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerSpawnChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerSpawnChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerStatisticIncrementEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerStatisticIncrementEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerSwapHandsEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerSwapHandsEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerTakeLecternBookEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerTakeLecternBookEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerUnleashEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerUnleashEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerVelocityEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerVelocityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerInputEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerInputEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerInteractAtEntityEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerInteractAtEntityEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerLinksSendEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerLinksSendEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerPickupArrowEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerPickupArrowEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerRecipeBookClickEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerRecipeBookClickEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerRecipeBookSettingsChangeEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerRecipeBookSettingsChangeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerRecipeDiscoverEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerRecipeDiscoverEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerRegisterChannelEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerRegisterChannelEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerResourcePackStatusEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerResourcePackStatusEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerSpawnLocationEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerSpawnLocationEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PlayerUnregisterChannelEvent => {
            ctx.register_event::<pumpkin_core::plugin::PlayerUnregisterChannelEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::RaidFinishEvent => {
            ctx.register_event::<pumpkin_core::plugin::RaidFinishEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::RaidSpawnWaveEvent => {
            ctx.register_event::<pumpkin_core::plugin::RaidSpawnWaveEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::RaidStopEvent => {
            ctx.register_event::<pumpkin_core::plugin::RaidStopEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::RaidTriggerEvent => {
            ctx.register_event::<pumpkin_core::plugin::RaidTriggerEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::LightningStrikeEvent => {
            ctx.register_event::<pumpkin_core::plugin::LightningStrikeEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PluginEnableEvent => {
            ctx.register_event::<pumpkin_core::plugin::PluginEnableEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::PluginDisableEvent => {
            ctx.register_event::<pumpkin_core::plugin::PluginDisableEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ServiceRegisterEvent => {
            ctx.register_event::<pumpkin_core::plugin::ServiceRegisterEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::ServiceUnregisterEvent => {
            ctx.register_event::<pumpkin_core::plugin::ServiceUnregisterEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
        crate::pumpkin::plugin::event::EventType::TabCompleteEvent => {
            ctx.register_event::<pumpkin_core::plugin::TabCompleteEvent, _>(
                std::sync::Arc::clone(handler),
                priority,
                blocking,
            );
        }
    }
}
