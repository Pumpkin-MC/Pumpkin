use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::sync::Mutex;
use wasmtime::component::Resource;

use crate::plugin::api::gui::PluginScreenHandler;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::forms::Form;
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::java_dialogs::{
    Action, AfterAction, Dialog, DialogBody, DialogInput, LinkLabel, LinkType,
};
use crate::{
    entity::{EntityBase, player::TitleMode},
    net::DisconnectReason,
    plugin::loader::wasm::wasm_host::{
        DowncastResourceExt,
        state::{
            GuiResource, PlayerResource, PluginHostState, TextComponentResource, WorldResource,
        },
        wit::v0_1::{
            events::{
                from_wasm_game_mode, from_wasm_position, to_wasm_game_mode, to_wasm_position,
            },
            pumpkin::{
                self,
                plugin::player::{Player, PlayerSkin, SkinParts},
                plugin::uuid::Uuid,
                plugin::world::World,
            },
            uuid::UuidExt,
        },
    },
};
use pumpkin_inventory::player::player_inventory::PlayerInventory;
use pumpkin_protocol::Property;
use pumpkin_protocol::bedrock::client::modal_form_request::CModalFormRequest;
use pumpkin_protocol::java::client::dialog::{
    ActionButton as ProtocolActionButton, Dialog as ProtocolDialog, DialogAction,
    DialogBody as ProtocolDialogBody, DialogInput as ProtocolDialogInput, DialogLink, DialogNBT,
};
use pumpkin_util::permission::PermissionLvl;
use pumpkin_util::translation::Locale;
use std::str::FromStr;

use pumpkin_protocol::bedrock::client::set_actor_data::{
    CSetActorData, EntityMetadata, MetadataValue, PropertySyncData, entity_data_key,
};
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_util::version::{BedrockMinecraftVersion, JavaMinecraftVersion};

use pumpkin_protocol::ser::ReadingError;
use pumpkin_protocol::ser::deserializer::Deserializer;
use serde::de::SeqAccess;

pub(crate) struct WitSeqAccess<'a> {
    pub(crate) deserializer: &'a mut Deserializer<&'a [u8]>,
}

impl<'de> SeqAccess<'de> for WitSeqAccess<'_> {
    type Error = ReadingError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.deserializer)?;
        Ok(Some(val))
    }
}

const fn to_wasm_java_version(
    version: JavaMinecraftVersion,
) -> pumpkin::plugin::player::JavaMinecraftVersion {
    match version {
        JavaMinecraftVersion::V_1_7_2 => pumpkin::plugin::player::JavaMinecraftVersion::V172,
        JavaMinecraftVersion::V_1_7_6 => pumpkin::plugin::player::JavaMinecraftVersion::V176,
        JavaMinecraftVersion::V_1_8 => pumpkin::plugin::player::JavaMinecraftVersion::V18,
        JavaMinecraftVersion::V_1_9 => pumpkin::plugin::player::JavaMinecraftVersion::V19,
        JavaMinecraftVersion::V_1_9_1 => pumpkin::plugin::player::JavaMinecraftVersion::V191,
        JavaMinecraftVersion::V_1_9_2 => pumpkin::plugin::player::JavaMinecraftVersion::V192,
        JavaMinecraftVersion::V_1_9_3 => pumpkin::plugin::player::JavaMinecraftVersion::V193,
        JavaMinecraftVersion::V_1_10 => pumpkin::plugin::player::JavaMinecraftVersion::V110,
        JavaMinecraftVersion::V_1_11 => pumpkin::plugin::player::JavaMinecraftVersion::V111,
        JavaMinecraftVersion::V_1_11_1 => pumpkin::plugin::player::JavaMinecraftVersion::V1111,
        JavaMinecraftVersion::V_1_12 => pumpkin::plugin::player::JavaMinecraftVersion::V112,
        JavaMinecraftVersion::V_1_12_1 => pumpkin::plugin::player::JavaMinecraftVersion::V1121,
        JavaMinecraftVersion::V_1_12_2 => pumpkin::plugin::player::JavaMinecraftVersion::V1122,
        JavaMinecraftVersion::V_1_13 => pumpkin::plugin::player::JavaMinecraftVersion::V113,
        JavaMinecraftVersion::V_1_13_1 => pumpkin::plugin::player::JavaMinecraftVersion::V1131,
        JavaMinecraftVersion::V_1_13_2 => pumpkin::plugin::player::JavaMinecraftVersion::V1132,
        JavaMinecraftVersion::V_1_14 => pumpkin::plugin::player::JavaMinecraftVersion::V114,
        JavaMinecraftVersion::V_1_14_1 => pumpkin::plugin::player::JavaMinecraftVersion::V1141,
        JavaMinecraftVersion::V_1_14_2 => pumpkin::plugin::player::JavaMinecraftVersion::V1142,
        JavaMinecraftVersion::V_1_14_3 => pumpkin::plugin::player::JavaMinecraftVersion::V1143,
        JavaMinecraftVersion::V_1_14_4 => pumpkin::plugin::player::JavaMinecraftVersion::V1144,
        JavaMinecraftVersion::V_1_15 => pumpkin::plugin::player::JavaMinecraftVersion::V115,
        JavaMinecraftVersion::V_1_15_1 => pumpkin::plugin::player::JavaMinecraftVersion::V1151,
        JavaMinecraftVersion::V_1_15_2 => pumpkin::plugin::player::JavaMinecraftVersion::V1152,
        JavaMinecraftVersion::V_1_16 => pumpkin::plugin::player::JavaMinecraftVersion::V116,
        JavaMinecraftVersion::V_1_16_1 => pumpkin::plugin::player::JavaMinecraftVersion::V1161,
        JavaMinecraftVersion::V_1_16_2 => pumpkin::plugin::player::JavaMinecraftVersion::V1162,
        JavaMinecraftVersion::V_1_16_3 => pumpkin::plugin::player::JavaMinecraftVersion::V1163,
        JavaMinecraftVersion::V_1_16_4 => pumpkin::plugin::player::JavaMinecraftVersion::V1164,
        JavaMinecraftVersion::V_1_17 => pumpkin::plugin::player::JavaMinecraftVersion::V117,
        JavaMinecraftVersion::V_1_17_1 => pumpkin::plugin::player::JavaMinecraftVersion::V1171,
        JavaMinecraftVersion::V_1_18 => pumpkin::plugin::player::JavaMinecraftVersion::V118,
        JavaMinecraftVersion::V_1_18_2 => pumpkin::plugin::player::JavaMinecraftVersion::V1182,
        JavaMinecraftVersion::V_1_19 => pumpkin::plugin::player::JavaMinecraftVersion::V119,
        JavaMinecraftVersion::V_1_19_1 => pumpkin::plugin::player::JavaMinecraftVersion::V1191,
        JavaMinecraftVersion::V_1_19_3 => pumpkin::plugin::player::JavaMinecraftVersion::V1193,
        JavaMinecraftVersion::V_1_19_4 => pumpkin::plugin::player::JavaMinecraftVersion::V1194,
        JavaMinecraftVersion::V_1_20 => pumpkin::plugin::player::JavaMinecraftVersion::V120,
        JavaMinecraftVersion::V_1_20_2 => pumpkin::plugin::player::JavaMinecraftVersion::V1202,
        JavaMinecraftVersion::V_1_20_3 => pumpkin::plugin::player::JavaMinecraftVersion::V1203,
        JavaMinecraftVersion::V_1_20_5 => pumpkin::plugin::player::JavaMinecraftVersion::V1205,
        JavaMinecraftVersion::V_1_21 => pumpkin::plugin::player::JavaMinecraftVersion::V121,
        JavaMinecraftVersion::V_1_21_2 => pumpkin::plugin::player::JavaMinecraftVersion::V1212,
        JavaMinecraftVersion::V_1_21_4 => pumpkin::plugin::player::JavaMinecraftVersion::V1214,
        JavaMinecraftVersion::V_1_21_5 => pumpkin::plugin::player::JavaMinecraftVersion::V1215,
        JavaMinecraftVersion::V_1_21_6 => pumpkin::plugin::player::JavaMinecraftVersion::V1216,
        JavaMinecraftVersion::V_1_21_7 => pumpkin::plugin::player::JavaMinecraftVersion::V1217,
        JavaMinecraftVersion::V_1_21_9 => pumpkin::plugin::player::JavaMinecraftVersion::V1219,
        JavaMinecraftVersion::V_1_21_11 => pumpkin::plugin::player::JavaMinecraftVersion::V12111,
        JavaMinecraftVersion::V_26_1 => pumpkin::plugin::player::JavaMinecraftVersion::V261,
        JavaMinecraftVersion::Unknown => pumpkin::plugin::player::JavaMinecraftVersion::Unknown,
    }
}

const fn to_wasm_bedrock_version(
    version: BedrockMinecraftVersion,
) -> pumpkin::plugin::player::BedrockMinecraftVersion {
    match version {
        BedrockMinecraftVersion::V_1_21 => pumpkin::plugin::player::BedrockMinecraftVersion::V121,
        BedrockMinecraftVersion::V_1_26_20 => {
            pumpkin::plugin::player::BedrockMinecraftVersion::V12620
        }
        BedrockMinecraftVersion::Unknown => {
            pumpkin::plugin::player::BedrockMinecraftVersion::Unknown
        }
    }
}

const fn to_wasm_chat_mode(
    mode: &crate::entity::player::ChatMode,
) -> pumpkin::plugin::player::ChatMode {
    match mode {
        crate::entity::player::ChatMode::Enabled => pumpkin::plugin::player::ChatMode::Enabled,
        crate::entity::player::ChatMode::CommandsOnly => {
            pumpkin::plugin::player::ChatMode::CommandsOnly
        }
        crate::entity::player::ChatMode::Hidden => pumpkin::plugin::player::ChatMode::Hidden,
    }
}

const fn to_wasm_bedrock_device_os(os: i32) -> pumpkin::plugin::player::BedrockDeviceOs {
    match os {
        1 => pumpkin::plugin::player::BedrockDeviceOs::Android,
        2 => pumpkin::plugin::player::BedrockDeviceOs::Ios,
        3 => pumpkin::plugin::player::BedrockDeviceOs::Osx,
        4 => pumpkin::plugin::player::BedrockDeviceOs::Amazon,
        5 => pumpkin::plugin::player::BedrockDeviceOs::GearVr,
        6 => pumpkin::plugin::player::BedrockDeviceOs::HoloLens,
        7 => pumpkin::plugin::player::BedrockDeviceOs::Windows10,
        8 => pumpkin::plugin::player::BedrockDeviceOs::Win32,
        9 => pumpkin::plugin::player::BedrockDeviceOs::Dedicated,
        10 => pumpkin::plugin::player::BedrockDeviceOs::TvOs,
        11 => pumpkin::plugin::player::BedrockDeviceOs::Playstation,
        12 => pumpkin::plugin::player::BedrockDeviceOs::Nintendo,
        13 => pumpkin::plugin::player::BedrockDeviceOs::Xbox,
        14 => pumpkin::plugin::player::BedrockDeviceOs::WindowsPhone,
        15 => pumpkin::plugin::player::BedrockDeviceOs::Linux,
        _ => pumpkin::plugin::player::BedrockDeviceOs::Unknown,
    }
}

const fn to_wasm_bedrock_input_mode(mode: i32) -> pumpkin::plugin::player::BedrockInputMode {
    match mode {
        1 => pumpkin::plugin::player::BedrockInputMode::Mouse,
        2 => pumpkin::plugin::player::BedrockInputMode::Touch,
        3 => pumpkin::plugin::player::BedrockInputMode::GamePad,
        4 => pumpkin::plugin::player::BedrockInputMode::MotionController,
        _ => pumpkin::plugin::player::BedrockInputMode::Unknown,
    }
}

const fn to_wasm_bedrock_ui_profile(profile: i32) -> pumpkin::plugin::player::BedrockUiProfile {
    match profile {
        0 => pumpkin::plugin::player::BedrockUiProfile::Classic,
        1 => pumpkin::plugin::player::BedrockUiProfile::Pocket,
        _ => pumpkin::plugin::player::BedrockUiProfile::Unknown,
    }
}

const fn to_wasm_bedrock_graphics_mode(mode: i32) -> pumpkin::plugin::player::BedrockGraphicsMode {
    match mode {
        0 => pumpkin::plugin::player::BedrockGraphicsMode::Simple,
        1 => pumpkin::plugin::player::BedrockGraphicsMode::Fancy,
        2 => pumpkin::plugin::player::BedrockGraphicsMode::RayTraced,
        _ => pumpkin::plugin::player::BedrockGraphicsMode::Unknown,
    }
}

const fn from_wasm_bedrock_ability(
    ability: pumpkin::plugin::player::BedrockAbility,
) -> pumpkin_protocol::bedrock::client::update_abilities::Ability {
    match ability {
        pumpkin::plugin::player::BedrockAbility::Build => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Build
        }
        pumpkin::plugin::player::BedrockAbility::Mine => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Mine
        }
        pumpkin::plugin::player::BedrockAbility::DoorsAndSwitches => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::DoorsAndSwitches
        }
        pumpkin::plugin::player::BedrockAbility::OpenContainers => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::OpenContainers
        }
        pumpkin::plugin::player::BedrockAbility::AttackPlayers => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::AttackPlayers
        }
        pumpkin::plugin::player::BedrockAbility::AttackMobs => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::AttackMobs
        }
        pumpkin::plugin::player::BedrockAbility::OperatorCommands => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::OperatorCommands
        }
        pumpkin::plugin::player::BedrockAbility::Teleport => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Teleport
        }
        pumpkin::plugin::player::BedrockAbility::Invulnerable => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Invulnerable
        }
        pumpkin::plugin::player::BedrockAbility::Flying => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Flying
        }
        pumpkin::plugin::player::BedrockAbility::MayFly => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::MayFly
        }
        pumpkin::plugin::player::BedrockAbility::Instabuild => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Instabuild
        }
        pumpkin::plugin::player::BedrockAbility::Lightning => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Lightning
        }
        pumpkin::plugin::player::BedrockAbility::FlySpeed => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::FlySpeed
        }
        pumpkin::plugin::player::BedrockAbility::WalkSpeed => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::WalkSpeed
        }
        pumpkin::plugin::player::BedrockAbility::Muted => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Muted
        }
        pumpkin::plugin::player::BedrockAbility::WorldBuilder => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::WorldBuilder
        }
        pumpkin::plugin::player::BedrockAbility::NoClip => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::NoClip
        }
        pumpkin::plugin::player::BedrockAbility::PrivilegedBuilder => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::PrivilegedBuilder
        }
        pumpkin::plugin::player::BedrockAbility::VerticalFlySpeed => {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::VerticalFlySpeed
        }
    }
}

#[allow(clippy::too_many_lines)]
const fn from_wasm_bedrock_status_flag(flag: pumpkin::plugin::player::BedrockStatusFlag) -> u32 {
    use pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag;
    match flag {
        pumpkin::plugin::player::BedrockStatusFlag::OnFire => entity_data_flag::ON_FIRE,
        pumpkin::plugin::player::BedrockStatusFlag::Sneaking => entity_data_flag::SNEAKING,
        pumpkin::plugin::player::BedrockStatusFlag::Riding => entity_data_flag::RIDING,
        pumpkin::plugin::player::BedrockStatusFlag::Sprinting => entity_data_flag::SPRINTING,
        pumpkin::plugin::player::BedrockStatusFlag::UsingItem => entity_data_flag::USING_ITEM,
        pumpkin::plugin::player::BedrockStatusFlag::Invisible => entity_data_flag::INVISIBLE,
        pumpkin::plugin::player::BedrockStatusFlag::Tempted => entity_data_flag::TEMPTED,
        pumpkin::plugin::player::BedrockStatusFlag::InLove => entity_data_flag::IN_LOVE,
        pumpkin::plugin::player::BedrockStatusFlag::Saddled => entity_data_flag::SADDLED,
        pumpkin::plugin::player::BedrockStatusFlag::Powered => entity_data_flag::POWERED,
        pumpkin::plugin::player::BedrockStatusFlag::Ignited => entity_data_flag::IGNITED,
        pumpkin::plugin::player::BedrockStatusFlag::Baby => entity_data_flag::BABY,
        pumpkin::plugin::player::BedrockStatusFlag::Converting => entity_data_flag::CONVERTING,
        pumpkin::plugin::player::BedrockStatusFlag::Critical => entity_data_flag::CRITICAL,
        pumpkin::plugin::player::BedrockStatusFlag::ShowName => entity_data_flag::SHOW_NAME,
        pumpkin::plugin::player::BedrockStatusFlag::AlwaysShowName => {
            entity_data_flag::ALWAYS_SHOW_NAME
        }
        pumpkin::plugin::player::BedrockStatusFlag::NoAi => entity_data_flag::NO_AI,
        pumpkin::plugin::player::BedrockStatusFlag::Silent => entity_data_flag::SILENT,
        pumpkin::plugin::player::BedrockStatusFlag::WallClimbing => entity_data_flag::WALL_CLIMBING,
        pumpkin::plugin::player::BedrockStatusFlag::Climb => entity_data_flag::CLIMB,
        pumpkin::plugin::player::BedrockStatusFlag::Swim => entity_data_flag::SWIM,
        pumpkin::plugin::player::BedrockStatusFlag::Fly => entity_data_flag::FLY,
        pumpkin::plugin::player::BedrockStatusFlag::Walk => entity_data_flag::WALK,
        pumpkin::plugin::player::BedrockStatusFlag::Resting => entity_data_flag::RESTING,
        pumpkin::plugin::player::BedrockStatusFlag::Sitting => entity_data_flag::SITTING,
        pumpkin::plugin::player::BedrockStatusFlag::Angry => entity_data_flag::ANGRY,
        pumpkin::plugin::player::BedrockStatusFlag::Interested => entity_data_flag::INTERESTED,
        pumpkin::plugin::player::BedrockStatusFlag::Charged => entity_data_flag::CHARGED,
        pumpkin::plugin::player::BedrockStatusFlag::Tamed => entity_data_flag::TAMED,
        pumpkin::plugin::player::BedrockStatusFlag::Orphaned => entity_data_flag::ORPHANED,
        pumpkin::plugin::player::BedrockStatusFlag::Leashed => entity_data_flag::LEASHED,
        pumpkin::plugin::player::BedrockStatusFlag::Sheared => entity_data_flag::SHEARED,
        pumpkin::plugin::player::BedrockStatusFlag::Gliding => entity_data_flag::GLIDING,
        pumpkin::plugin::player::BedrockStatusFlag::Elder => entity_data_flag::ELDER,
        pumpkin::plugin::player::BedrockStatusFlag::Moving => entity_data_flag::MOVING,
        pumpkin::plugin::player::BedrockStatusFlag::Breathing => entity_data_flag::BREATHING,
        pumpkin::plugin::player::BedrockStatusFlag::Chested => entity_data_flag::CHESTED,
        pumpkin::plugin::player::BedrockStatusFlag::Stackable => entity_data_flag::STACKABLE,
        pumpkin::plugin::player::BedrockStatusFlag::ShowBottom => entity_data_flag::SHOW_BOTTOM,
        pumpkin::plugin::player::BedrockStatusFlag::Standing => entity_data_flag::STANDING,
        pumpkin::plugin::player::BedrockStatusFlag::Shaking => entity_data_flag::SHAKING,
        pumpkin::plugin::player::BedrockStatusFlag::Idling => entity_data_flag::IDLING,
        pumpkin::plugin::player::BedrockStatusFlag::Casting => entity_data_flag::CASTING,
        pumpkin::plugin::player::BedrockStatusFlag::Charging => entity_data_flag::CHARGING,
        pumpkin::plugin::player::BedrockStatusFlag::KeyboardControlled => {
            entity_data_flag::KEYBOARD_CONTROLLED
        }
        pumpkin::plugin::player::BedrockStatusFlag::PowerJump => entity_data_flag::POWER_JUMP,
        pumpkin::plugin::player::BedrockStatusFlag::Dash => entity_data_flag::DASH,
        pumpkin::plugin::player::BedrockStatusFlag::Lingering => entity_data_flag::LINGERING,
        pumpkin::plugin::player::BedrockStatusFlag::HasCollision => entity_data_flag::HAS_COLLISION,
        pumpkin::plugin::player::BedrockStatusFlag::HasGravity => entity_data_flag::HAS_GRAVITY,
        pumpkin::plugin::player::BedrockStatusFlag::FireImmune => entity_data_flag::FIRE_IMMUNE,
        pumpkin::plugin::player::BedrockStatusFlag::Dancing => entity_data_flag::DANCING,
        pumpkin::plugin::player::BedrockStatusFlag::Enchanted => entity_data_flag::ENCHANTED,
        pumpkin::plugin::player::BedrockStatusFlag::ReturnTrident => {
            entity_data_flag::RETURN_TRIDENT
        }
        pumpkin::plugin::player::BedrockStatusFlag::ContainerPrivate => {
            entity_data_flag::CONTAINER_PRIVATE
        }
        pumpkin::plugin::player::BedrockStatusFlag::Transforming => entity_data_flag::TRANSFORMING,
        pumpkin::plugin::player::BedrockStatusFlag::DamageNearbyMobs => {
            entity_data_flag::DAMAGE_NEARBY_MOBS
        }
        pumpkin::plugin::player::BedrockStatusFlag::Swimming => entity_data_flag::SWIMMING,
        pumpkin::plugin::player::BedrockStatusFlag::Bribed => entity_data_flag::BRIBED,
        pumpkin::plugin::player::BedrockStatusFlag::Pregnant => entity_data_flag::PREGNANT,
        pumpkin::plugin::player::BedrockStatusFlag::LayingEgg => entity_data_flag::LAYING_EGG,
        pumpkin::plugin::player::BedrockStatusFlag::PassengerCanPick => {
            entity_data_flag::PASSENGER_CAN_PICK
        }
        pumpkin::plugin::player::BedrockStatusFlag::TransitionSitting => {
            entity_data_flag::TRANSITION_SITTING
        }
        pumpkin::plugin::player::BedrockStatusFlag::Eating => entity_data_flag::EATING,
        pumpkin::plugin::player::BedrockStatusFlag::LayingDown => entity_data_flag::LAYING_DOWN,
        pumpkin::plugin::player::BedrockStatusFlag::Sneezing => entity_data_flag::SNEEZING,
        pumpkin::plugin::player::BedrockStatusFlag::Trusting => entity_data_flag::TRUSTING,
        pumpkin::plugin::player::BedrockStatusFlag::Rolling => entity_data_flag::ROLLING,
        pumpkin::plugin::player::BedrockStatusFlag::Scared => entity_data_flag::SCARED,
        pumpkin::plugin::player::BedrockStatusFlag::InScaffolding => {
            entity_data_flag::IN_SCAFFOLDING
        }
        pumpkin::plugin::player::BedrockStatusFlag::OverScaffolding => {
            entity_data_flag::OVER_SCAFFOLDING
        }
        pumpkin::plugin::player::BedrockStatusFlag::DescendThroughBlock => {
            entity_data_flag::DESCEND_THROUGH_BLOCK
        }
        pumpkin::plugin::player::BedrockStatusFlag::Blocking => entity_data_flag::BLOCKING,
        pumpkin::plugin::player::BedrockStatusFlag::TransitionBlocking => {
            entity_data_flag::TRANSITION_BLOCKING
        }
        pumpkin::plugin::player::BedrockStatusFlag::BlockedUsingShield => {
            entity_data_flag::BLOCKED_USING_SHIELD
        }
        pumpkin::plugin::player::BedrockStatusFlag::BlockedUsingDamagedShield => {
            entity_data_flag::BLOCKED_USING_DAMAGED_SHIELD
        }
        pumpkin::plugin::player::BedrockStatusFlag::Sleeping => entity_data_flag::SLEEPING,
        pumpkin::plugin::player::BedrockStatusFlag::WantsToWake => entity_data_flag::WANTS_TO_WAKE,
        pumpkin::plugin::player::BedrockStatusFlag::TradeInterest => {
            entity_data_flag::TRADE_INTEREST
        }
        pumpkin::plugin::player::BedrockStatusFlag::DoorBreaker => entity_data_flag::DOOR_BREAKER,
        pumpkin::plugin::player::BedrockStatusFlag::BreakingObstruction => {
            entity_data_flag::BREAKING_OBSTRUCTION
        }
        pumpkin::plugin::player::BedrockStatusFlag::DoorOpener => entity_data_flag::DOOR_OPENER,
        pumpkin::plugin::player::BedrockStatusFlag::Captain => entity_data_flag::CAPTAIN,
        pumpkin::plugin::player::BedrockStatusFlag::Stunned => entity_data_flag::STUNNED,
        pumpkin::plugin::player::BedrockStatusFlag::Roaring => entity_data_flag::ROARING,
        pumpkin::plugin::player::BedrockStatusFlag::DelayedAttack => {
            entity_data_flag::DELAYED_ATTACK
        }
        pumpkin::plugin::player::BedrockStatusFlag::AvoidingMobs => entity_data_flag::AVOIDING_MOBS,
        pumpkin::plugin::player::BedrockStatusFlag::AvoidingBlock => {
            entity_data_flag::AVOIDING_BLOCK
        }
        pumpkin::plugin::player::BedrockStatusFlag::FacingTargetToRangeAttack => {
            entity_data_flag::FACING_TARGET_TO_RANGE_ATTACK
        }
        pumpkin::plugin::player::BedrockStatusFlag::HiddenWhenInvisible => {
            entity_data_flag::HIDDEN_WHEN_INVISIBLE
        }
        pumpkin::plugin::player::BedrockStatusFlag::InUi => entity_data_flag::IN_UI,
        pumpkin::plugin::player::BedrockStatusFlag::Stalking => entity_data_flag::STALKING,
        pumpkin::plugin::player::BedrockStatusFlag::Emoting => entity_data_flag::EMOTING,
        pumpkin::plugin::player::BedrockStatusFlag::Celebrating => entity_data_flag::CELEBRATING,
        pumpkin::plugin::player::BedrockStatusFlag::Admiring => entity_data_flag::ADMIRING,
        pumpkin::plugin::player::BedrockStatusFlag::CelebratingSpecial => {
            entity_data_flag::CELEBRATING_SPECIAL
        }
        pumpkin::plugin::player::BedrockStatusFlag::OutOfControl => {
            entity_data_flag::OUT_OF_CONTROL
        }
        pumpkin::plugin::player::BedrockStatusFlag::RamAttack => entity_data_flag::RAM_ATTACK,
        pumpkin::plugin::player::BedrockStatusFlag::PlayingDead => entity_data_flag::PLAYING_DEAD,
        pumpkin::plugin::player::BedrockStatusFlag::InAscendingBlock => {
            entity_data_flag::IN_ASCENDING_BLOCK
        }
        pumpkin::plugin::player::BedrockStatusFlag::OverDescendingBlock => {
            entity_data_flag::OVER_DESCENDING_BLOCK
        }
        pumpkin::plugin::player::BedrockStatusFlag::Croaking => entity_data_flag::CROAKING,
        pumpkin::plugin::player::BedrockStatusFlag::DigestMob => entity_data_flag::DIGEST_MOB,
        pumpkin::plugin::player::BedrockStatusFlag::JumpGoal => entity_data_flag::JUMP_GOAL,
        pumpkin::plugin::player::BedrockStatusFlag::Emerging => entity_data_flag::EMERGING,
        pumpkin::plugin::player::BedrockStatusFlag::Sniffing => entity_data_flag::SNIFFING,
        pumpkin::plugin::player::BedrockStatusFlag::Digging => entity_data_flag::DIGGING,
        pumpkin::plugin::player::BedrockStatusFlag::SonicBoom => entity_data_flag::SONIC_BOOM,
        pumpkin::plugin::player::BedrockStatusFlag::HasDashTimeout => {
            entity_data_flag::HAS_DASH_TIMEOUT
        }
        pumpkin::plugin::player::BedrockStatusFlag::PushTowardsClosestSpace => {
            entity_data_flag::PUSH_TOWARDS_CLOSEST_SPACE
        }
        pumpkin::plugin::player::BedrockStatusFlag::Scenting => entity_data_flag::SCENTING,
        pumpkin::plugin::player::BedrockStatusFlag::Rising => entity_data_flag::RISING,
        pumpkin::plugin::player::BedrockStatusFlag::FeelingHappy => entity_data_flag::FEELING_HAPPY,
        pumpkin::plugin::player::BedrockStatusFlag::Searching => entity_data_flag::SEARCHING,
        pumpkin::plugin::player::BedrockStatusFlag::Crawling => entity_data_flag::CRAWLING,
        pumpkin::plugin::player::BedrockStatusFlag::BodyRotationBlocked => {
            entity_data_flag::BODY_ROTATION_BLOCKED
        }
        pumpkin::plugin::player::BedrockStatusFlag::RenderWhenInvisible => {
            entity_data_flag::RENDER_WHEN_INVISIBLE
        }
        pumpkin::plugin::player::BedrockStatusFlag::BodyRotationAxisAligned => {
            entity_data_flag::BODY_ROTATION_AXIS_ALIGNED
        }
        pumpkin::plugin::player::BedrockStatusFlag::Collidable => entity_data_flag::COLLIDABLE,
        pumpkin::plugin::player::BedrockStatusFlag::WasdAirControlled => {
            entity_data_flag::WASD_AIR_CONTROLLED
        }
        pumpkin::plugin::player::BedrockStatusFlag::DoesServerAuthOnlyDismount => {
            entity_data_flag::DOES_SERVER_AUTH_ONLY_DISMOUNT
        }
        pumpkin::plugin::player::BedrockStatusFlag::BodyRotationAlwaysFollowsHead => {
            entity_data_flag::BODY_ROTATION_ALWAYS_FOLLOWS_HEAD
        }
        pumpkin::plugin::player::BedrockStatusFlag::CanUseVerticalMovementAction => {
            entity_data_flag::CAN_USE_VERTICAL_MOVEMENT_ACTION
        }
        pumpkin::plugin::player::BedrockStatusFlag::RotationLockedToVehicle => {
            entity_data_flag::ROTATION_LOCKED_TO_VEHICLE
        }
    }
}

pub fn player_from_resource(
    state: &PluginHostState,
    player: &Resource<Player>,
) -> wasmtime::Result<std::sync::Arc<crate::entity::player::Player>> {
    state
        .resource_table
        .get::<PlayerResource>(&Resource::new_own(player.rep()))
        .map_err(|_| wasmtime::Error::msg("invalid player resource handle"))
        .map(|resource| resource.provider.clone())
}

pub(crate) fn text_component_from_resource(
    state: &PluginHostState,
    text: &Resource<pumpkin::plugin::text::TextComponent>,
) -> pumpkin_util::text::TextComponent {
    state
        .resource_table
        .get::<TextComponentResource>(&Resource::new_own(text.rep()))
        .expect("invalid text-component resource handle")
        .provider
        .clone()
}

fn world_from_resource(
    state: &PluginHostState,
    world: &Resource<pumpkin::plugin::world::World>,
) -> std::sync::Arc<crate::world::World> {
    state
        .resource_table
        .get::<WorldResource>(&Resource::new_own(world.rep()))
        .expect("invalid world resource handle")
        .provider
        .clone()
}

const fn to_wit_permission_level(
    level: PermissionLvl,
) -> pumpkin::plugin::permission::PermissionLevel {
    match level {
        PermissionLvl::Zero => pumpkin::plugin::permission::PermissionLevel::Zero,
        PermissionLvl::One => pumpkin::plugin::permission::PermissionLevel::One,
        PermissionLvl::Two => pumpkin::plugin::permission::PermissionLevel::Two,
        PermissionLvl::Three => pumpkin::plugin::permission::PermissionLevel::Three,
        PermissionLvl::Four => pumpkin::plugin::permission::PermissionLevel::Four,
    }
}

const fn from_wit_permission_level(
    level: pumpkin::plugin::permission::PermissionLevel,
) -> PermissionLvl {
    match level {
        pumpkin::plugin::permission::PermissionLevel::Zero => PermissionLvl::Zero,
        pumpkin::plugin::permission::PermissionLevel::One => PermissionLvl::One,
        pumpkin::plugin::permission::PermissionLevel::Two => PermissionLvl::Two,
        pumpkin::plugin::permission::PermissionLevel::Three => PermissionLvl::Three,
        pumpkin::plugin::permission::PermissionLevel::Four => PermissionLvl::Four,
    }
}

impl DowncastResourceExt<PlayerResource> for Resource<Player> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a PlayerResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .map_err(|_| wasmtime::Error::msg("invalid player resource handle"))
            .unwrap()
            .downcast_ref::<PlayerResource>()
            .ok_or("resource type mismatch")
            .map_err(wasmtime::Error::msg)
            .unwrap()
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut PlayerResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .map_err(|_| wasmtime::Error::msg("invalid player resource handle"))
            .unwrap()
            .downcast_mut::<PlayerResource>()
            .ok_or("resource type mismatch")
            .map_err(wasmtime::Error::msg)
            .unwrap()
    }

    fn consume(self, state: &mut PluginHostState) -> PlayerResource {
        state
            .resource_table
            .delete::<PlayerResource>(Resource::new_own(self.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid player resource handle"))
            .unwrap()
    }
}

impl pumpkin::plugin::player::Host for PluginHostState {
    async fn get_world_players(
        &mut self,
        world_ref: Resource<pumpkin::plugin::world::World>,
    ) -> wasmtime::Result<Vec<Resource<pumpkin::plugin::player::Player>>> {
        let world = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::WorldResource>(
                &Resource::new_own(world_ref.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid world resource handle"))?
            .provider
            .clone();

        let mut players = Vec::new();
        for player in world.players.load().iter() {
            players.push(self.add_player(player.clone())?);
        }

        Ok(players)
    }
}
use crate::plugin::loader::wasm::wasm_host::wit::v0_1::events::from_wasm_hand;
use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
use pumpkin_protocol::java::client::play::CSetContainerSlot;
use pumpkin_world::inventory::Inventory;

use crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::item_stack::ItemStack as WitHostItemStack;

impl pumpkin::plugin::player::HostPlayer for PluginHostState {
    async fn set_item_in_hand(
        &mut self,
        player: Resource<Player>,
        hand: pumpkin::plugin::common::Hand,
        stack: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let stack = if let Some(stack_res) = stack {
            self.get_item_stack(&stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };

        let hand = from_wasm_hand(hand);
        let slot = match hand {
            pumpkin_util::Hand::Right => player.inventory().get_selected_slot() as usize,
            pumpkin_util::Hand::Left => PlayerInventory::OFF_HAND_SLOT,
        };

        player.inventory().set_stack(slot, stack.clone()).await;

        // Sync to client
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, slot as i16, &stack_serializer);
        player.client.enqueue_packet(&packet).await;

        Ok(())
    }

    async fn set_inventory_item(
        &mut self,
        player: Resource<Player>,
        slot: u8,
        stack: Option<Resource<WitHostItemStack>>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let stack = if let Some(stack_res) = stack {
            self.get_item_stack(&stack_res)?.lock().await.clone()
        } else {
            pumpkin_data::item_stack::ItemStack::EMPTY.clone()
        };

        player
            .inventory()
            .set_stack(slot as usize, stack.clone())
            .await;

        // Sync to client
        let stack_serializer = ItemStackSerializer::from(stack);
        let packet = CSetContainerSlot::new(0, 0, slot as i16, &stack_serializer);
        player.client.enqueue_packet(&packet).await;

        Ok(())
    }

    async fn get_inventory_item(
        &mut self,
        player: Resource<Player>,
        slot: u8,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = player_from_resource(self, &player)?;
        let stack = player.inventory().get_stack(slot as usize).await;
        if stack.lock().await.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add_item_stack(stack)?))
        }
    }

    async fn get_item_in_hand(
        &mut self,
        player: Resource<Player>,
        hand: pumpkin::plugin::common::Hand,
    ) -> wasmtime::Result<Option<Resource<WitHostItemStack>>> {
        let player = player_from_resource(self, &player)?;
        let hand = from_wasm_hand(hand);
        let stack = player.inventory().get_stack_in_hand(hand).await;
        if stack.lock().await.is_empty() {
            Ok(None)
        } else {
            Ok(Some(self.add_item_stack(stack)?))
        }
    }

    async fn as_entity(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Resource<pumpkin::plugin::world::Entity>> {
        let player = player_from_resource(self, &player)?;
        self.add_entity(player as Arc<dyn EntityBase>)
            .map_err(|_| wasmtime::Error::msg("failed to add entity resource"))
    }

    async fn get_id(&mut self, player: Resource<Player>) -> wasmtime::Result<Uuid> {
        let player = player_from_resource(self, &player)?;
        Ok(Uuid::to_wit(&player.gameprofile.id))
    }

    async fn get_name(&mut self, player: Resource<Player>) -> wasmtime::Result<String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.gameprofile.name.clone())
    }

    async fn get_position(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<pumpkin::plugin::common::Position> {
        let player = player_from_resource(self, &player)?;
        Ok(to_wasm_position(player.position()))
    }

    async fn get_yaw(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.get_entity().yaw.load())
    }

    async fn get_pitch(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.get_entity().pitch.load())
    }

    async fn get_world(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<wasmtime::component::Resource<pumpkin::plugin::world::World>> {
        let player = player_from_resource(self, &player)?;
        let world = player.world();
        self.add_world(world)
            .map_err(|_| wasmtime::Error::msg("failed to add world resource"))
    }

    async fn get_gamemode(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<pumpkin::plugin::common::GameMode> {
        let player = player_from_resource(self, &player)?;
        Ok(to_wasm_game_mode(player.gamemode.load()))
    }

    async fn set_gamemode(
        &mut self,
        player: Resource<Player>,
        mode: pumpkin::plugin::common::GameMode,
    ) -> wasmtime::Result<bool> {
        let player = player_from_resource(self, &player)?;
        Ok(player.set_gamemode(from_wasm_game_mode(mode)).await)
    }

    async fn get_locale(&mut self, player: Resource<Player>) -> wasmtime::Result<String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.config.load().locale.clone())
    }

    async fn get_ping(&mut self, player: Resource<Player>) -> wasmtime::Result<u32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.ping.load(Ordering::Relaxed))
    }

    async fn get_permission_level(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<pumpkin::plugin::permission::PermissionLevel> {
        let player = player_from_resource(self, &player)?;
        Ok(to_wit_permission_level(player.permission_lvl.load()))
    }

    async fn set_permission_level(
        &mut self,
        player: Resource<Player>,
        level: pumpkin::plugin::permission::PermissionLevel,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");
        let level = from_wit_permission_level(level);
        let command_dispatcher = server.command_dispatcher.read().await;
        player
            .set_permission_lvl(server, level, &command_dispatcher)
            .await;
        Ok(())
    }

    async fn set_permission(
        &mut self,
        player: Resource<Player>,
        node: String,
        value: bool,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");

        let mut perm_manager = server.permission_manager.write().await;
        let attachment = perm_manager.get_attachment(player.gameprofile.id);
        drop(perm_manager);

        attachment.write().await.set_permission(&node, value);

        Ok(())
    }

    async fn unset_permission(
        &mut self,
        player: Resource<Player>,
        node: String,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");

        let mut perm_manager = server.permission_manager.write().await;
        let attachment = perm_manager.get_attachment(player.gameprofile.id);
        drop(perm_manager);

        attachment.write().await.unset_permission(&node);

        Ok(())
    }

    async fn has_permission_set(
        &mut self,
        player: Resource<Player>,
        node: String,
    ) -> wasmtime::Result<Option<bool>> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");

        let mut perm_manager = server.permission_manager.write().await;
        let attachment = perm_manager.get_attachment(player.gameprofile.id);
        drop(perm_manager);

        Ok(attachment.read().await.has_permission_set(&node))
    }

    async fn has_permission(
        &mut self,
        player: Resource<Player>,
        node: String,
    ) -> wasmtime::Result<bool> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");
        Ok(player.has_permission(server, &node).await)
    }

    async fn get_display_name(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Resource<pumpkin::plugin::text::TextComponent>> {
        let player = player_from_resource(self, &player)?;
        let display_name = player.get_display_name().await;
        self.add_text_component(display_name)
            .map_err(|_| wasmtime::Error::msg("failed to add text-component resource"))
    }

    async fn set_display_name(
        &mut self,
        player: Resource<Player>,
        display_name: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let display_name = text_component_from_resource(self, &display_name);
        let player = player_from_resource(self, &player)?;
        player.set_display_name(Some(display_name)).await;
        Ok(())
    }

    async fn get_tab_list_name(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Option<Resource<pumpkin::plugin::text::TextComponent>>> {
        let player = player_from_resource(self, &player)?;
        let tab_list_name = player.get_tab_list_name().await;
        tab_list_name.map_or_else(
            || Ok(None),
            |name| {
                self.add_text_component(name)
                    .map(Some)
                    .map_err(|_| wasmtime::Error::msg("failed to add text-component resource"))
            },
        )
    }

    async fn set_tab_list_name(
        &mut self,
        player: Resource<Player>,
        name: Option<wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>>,
    ) -> wasmtime::Result<()> {
        let name = name.map(|n| text_component_from_resource(self, &n));
        let player = player_from_resource(self, &player)?;
        player.set_tab_list_name(name).await;
        Ok(())
    }

    async fn send_system_message(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
        overlay: bool,
    ) -> wasmtime::Result<()> {
        let component = text_component_from_resource(self, &text);
        let player = player_from_resource(self, &player)?;
        player.send_system_message_raw(&component, overlay).await;
        Ok(())
    }

    async fn set_tab_list_header_footer(
        &mut self,
        player: Resource<Player>,
        header: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
        footer: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let header = text_component_from_resource(self, &header);
        let footer = text_component_from_resource(self, &footer);
        let player = player_from_resource(self, &player)?;
        player.set_tab_list_header_footer(header, footer).await;
        Ok(())
    }

    async fn set_tab_list_order(
        &mut self,
        player: Resource<Player>,
        order: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_tab_list_order(order);
        Ok(())
    }

    async fn set_tab_list_latency(
        &mut self,
        player: Resource<Player>,
        latency: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_tab_list_latency(latency);
        Ok(())
    }

    async fn set_tab_list_listed(
        &mut self,
        player: Resource<Player>,
        listed: bool,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_tab_list_listed(listed);
        Ok(())
    }

    async fn show_title(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = text_component_from_resource(self, &text);
        let player = player_from_resource(self, &player)?;
        player.show_title(&component, &TitleMode::Title).await;
        Ok(())
    }

    async fn show_subtitle(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = text_component_from_resource(self, &text);
        let player = player_from_resource(self, &player)?;
        player.show_title(&component, &TitleMode::SubTitle).await;
        Ok(())
    }

    async fn show_actionbar(
        &mut self,
        player: Resource<Player>,
        text: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = text_component_from_resource(self, &text);
        let player = player_from_resource(self, &player)?;
        player.show_title(&component, &TitleMode::ActionBar).await;
        Ok(())
    }

    async fn send_title_animation(
        &mut self,
        player: Resource<Player>,
        fade_in: i32,
        stay: i32,
        fade_out: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.send_title_animation(fade_in, stay, fade_out).await;
        Ok(())
    }

    async fn teleport(
        &mut self,
        player: Resource<Player>,
        position: pumpkin::plugin::common::Position,
        yaw: Option<f32>,
        pitch: Option<f32>,
        world: Resource<World>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let world = world_from_resource(self, &world);
        player
            .teleport(from_wasm_position(position), yaw, pitch, world)
            .await;
        Ok(())
    }

    async fn teleport_world(
        &mut self,
        player: Resource<Player>,
        world: wasmtime::component::Resource<pumpkin::plugin::world::World>,
        position: pumpkin::plugin::common::Position,
        yaw: Option<f32>,
        pitch: Option<f32>,
    ) -> wasmtime::Result<()> {
        let world = world_from_resource(self, &world);
        let player = player_from_resource(self, &player)?;
        player
            .teleport_world(world, from_wasm_position(position), yaw, pitch)
            .await;
        Ok(())
    }

    async fn kick(
        &mut self,
        player: Resource<Player>,
        message: wasmtime::component::Resource<pumpkin::plugin::text::TextComponent>,
    ) -> wasmtime::Result<()> {
        let component = text_component_from_resource(self, &message);
        let player = player_from_resource(self, &player)?;
        player.kick(DisconnectReason::Kicked, component).await;
        Ok(())
    }

    async fn respawn(&mut self, player: Resource<Player>) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.respawn().await;
        Ok(())
    }

    async fn open_gui(
        &mut self,
        player: Resource<Player>,
        gui: Resource<pumpkin::plugin::gui::Gui>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let gui_res = self
            .resource_table
            .get::<GuiResource>(&Resource::new_own(gui.rep()))
            .map_err(|_| wasmtime::Error::msg("invalid gui resource handle"))?;
        let gui = gui_res.provider.lock().await;

        player.increment_screen_handler_sync_id();
        let sync_id = player.screen_handler_sync_id.load(Ordering::Relaxed);
        let screen_handler = Arc::new(Mutex::new(PluginScreenHandler::new(
            sync_id,
            gui.window_type,
            &gui.inventory,
            gui.allow_grab_items,
            gui.allow_put_items,
        )));

        player
            .open_handled_screen_direct(screen_handler, gui.title.clone())
            .await;
        Ok(())
    }

    async fn ban(
        &mut self,
        player: Resource<Player>,
        reason: Option<Resource<pumpkin::plugin::text::TextComponent>>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");
        let reason = reason.map(|r| text_component_from_resource(self, &r));
        player.ban(server, reason).await;
        Ok(())
    }

    async fn ban_ip(
        &mut self,
        player: Resource<Player>,
        reason: Option<Resource<pumpkin::plugin::text::TextComponent>>,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let server = self.server.as_ref().expect("server not available");
        let reason = reason.map(|r| text_component_from_resource(self, &r));
        player.ban_ip(server, reason).await;
        Ok(())
    }

    async fn transfer(
        &mut self,
        player: Resource<Player>,
        host: String,
        port: u16,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        if let crate::net::ClientPlatform::Java(client) = &player.client {
            client
                .send_packet_now(&pumpkin_protocol::java::client::play::CTransfer::new(
                    &host,
                    pumpkin_protocol::codec::var_int::VarInt(i32::from(port)),
                ))
                .await;
        }
        Ok(())
    }

    async fn get_selected_slot(&mut self, player: Resource<Player>) -> wasmtime::Result<u8> {
        let player = player_from_resource(self, &player)?;
        Ok(player.inventory.get_selected_slot())
    }

    async fn get_health(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.living_entity.health.load())
    }

    async fn set_health(&mut self, player: Resource<Player>, health: f32) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_health(health).await;
        Ok(())
    }

    async fn get_max_health(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.living_entity.get_max_health())
    }

    async fn set_max_health(
        &mut self,
        player: Resource<Player>,
        max_health: f32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_max_health(max_health).await;
        Ok(())
    }

    async fn get_food_level(&mut self, player: Resource<Player>) -> wasmtime::Result<u8> {
        let player = player_from_resource(self, &player)?;
        Ok(player.hunger_manager.level.load())
    }

    async fn set_food_level(
        &mut self,
        player: Resource<Player>,
        level: u8,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_food_level(level).await;
        Ok(())
    }

    async fn get_saturation(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.hunger_manager.saturation.load())
    }

    async fn set_saturation(
        &mut self,
        player: Resource<Player>,
        saturation: f32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_saturation(saturation).await;
        Ok(())
    }

    async fn get_exhaustion(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.get_exhaustion())
    }

    async fn set_exhaustion(
        &mut self,
        player: Resource<Player>,
        exhaustion: f32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_exhaustion(exhaustion).await;
        Ok(())
    }

    async fn get_absorption(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.get_absorption())
    }

    async fn set_absorption(
        &mut self,
        player: Resource<Player>,
        absorption: f32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_absorption(absorption).await;
        Ok(())
    }

    async fn get_experience_level(&mut self, player: Resource<Player>) -> wasmtime::Result<i32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.experience_level.load(Ordering::Relaxed))
    }

    async fn get_experience_progress(&mut self, player: Resource<Player>) -> wasmtime::Result<f32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.experience_progress.load())
    }

    async fn get_experience_points(&mut self, player: Resource<Player>) -> wasmtime::Result<i32> {
        let player = player_from_resource(self, &player)?;
        Ok(player.experience_points.load(Ordering::Relaxed))
    }

    async fn set_experience_level(
        &mut self,
        player: Resource<Player>,
        level: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.set_experience_level(level, true).await;
        Ok(())
    }

    async fn set_experience_progress(
        &mut self,
        player: Resource<Player>,
        progress: f32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player
            .set_experience(
                player.experience_level.load(Ordering::Relaxed),
                progress,
                player.experience_points.load(Ordering::Relaxed),
            )
            .await;
        Ok(())
    }

    async fn set_experience_points(
        &mut self,
        player: Resource<Player>,
        points: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player
            .set_experience(
                player.experience_level.load(Ordering::Relaxed),
                player.experience_progress.load(),
                points,
            )
            .await;
        Ok(())
    }

    async fn add_experience_levels(
        &mut self,
        player: Resource<Player>,
        levels: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.add_experience_levels(levels).await;
        Ok(())
    }

    async fn add_experience_points(
        &mut self,
        player: Resource<Player>,
        points: i32,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        player.add_experience_points(points).await;
        Ok(())
    }

    async fn is_flying(&mut self, player: Resource<Player>) -> wasmtime::Result<bool> {
        let player = player_from_resource(self, &player)?;
        Ok(player.is_flying().await)
    }

    async fn set_flying(&mut self, player: Resource<Player>, flying: bool) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        {
            let mut abilities = player.abilities.lock().await;
            abilities.flying = flying;
        };
        player.send_abilities_update().await;
        Ok(())
    }

    async fn get_abilities(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<pumpkin::plugin::player::PlayerAbilities> {
        let player = player_from_resource(self, &player)?;
        let abilities = player.abilities.lock().await;
        Ok(pumpkin::plugin::player::PlayerAbilities {
            invulnerable: abilities.invulnerable,
            flying: abilities.flying,
            allow_flying: abilities.allow_flying,
            creative: abilities.creative,
            allow_modify_world: abilities.allow_modify_world,
            fly_speed: abilities.fly_speed,
            walk_speed: abilities.walk_speed,
        })
    }

    async fn set_abilities(
        &mut self,
        player: Resource<Player>,
        abilities: pumpkin::plugin::player::PlayerAbilities,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        {
            let mut a = player.abilities.lock().await;
            a.invulnerable = abilities.invulnerable;
            a.flying = abilities.flying;
            a.allow_flying = abilities.allow_flying;
            a.creative = abilities.creative;
            a.allow_modify_world = abilities.allow_modify_world;
            a.fly_speed = abilities.fly_speed;
            a.walk_speed = abilities.walk_speed;
        };
        player.send_abilities_update().await;
        Ok(())
    }

    async fn get_ip(&mut self, player: Resource<Player>) -> wasmtime::Result<String> {
        let player = player_from_resource(self, &player)?;
        Ok(player.get_ip().await)
    }

    async fn get_skin(&mut self, player: Resource<Player>) -> wasmtime::Result<Option<PlayerSkin>> {
        let player = player_from_resource(self, &player)?;
        Ok(player
            .gameprofile
            .properties
            .load()
            .iter()
            .find(|p| p.name.as_ref() == "textures")
            .map(|p| PlayerSkin {
                value: p.value.to_string(),
                signature: p.signature.as_ref().map(std::string::ToString::to_string),
            }))
    }

    async fn set_skin(
        &mut self,
        player: Resource<Player>,
        skin: PlayerSkin,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let mut properties = (**player.gameprofile.properties.load()).clone();

        properties.retain(|p| p.name.as_ref() != "textures");
        properties.push(Property {
            name: "textures".into(),
            value: skin.value.into(),
            signature: skin.signature.map(std::convert::Into::into),
        });

        player.gameprofile.properties.store(Arc::new(properties));

        Ok(())
    }

    async fn get_skin_parts(&mut self, player: Resource<Player>) -> wasmtime::Result<SkinParts> {
        let player = player_from_resource(self, &player)?;
        let mask = player.config.load().skin_parts;
        let mut parts = SkinParts::empty();
        if mask & 0x01 != 0 {
            parts |= SkinParts::CAPE;
        }
        if mask & 0x02 != 0 {
            parts |= SkinParts::JACKET;
        }
        if mask & 0x04 != 0 {
            parts |= SkinParts::LEFT_SLEEVE;
        }
        if mask & 0x08 != 0 {
            parts |= SkinParts::RIGHT_SLEEVE;
        }
        if mask & 0x10 != 0 {
            parts |= SkinParts::LEFT_PANTS_LEG;
        }
        if mask & 0x20 != 0 {
            parts |= SkinParts::RIGHT_PANTS_LEG;
        }
        if mask & 0x40 != 0 {
            parts |= SkinParts::HAT;
        }
        Ok(parts)
    }

    async fn set_skin_parts(
        &mut self,
        player: Resource<Player>,
        parts: SkinParts,
    ) -> wasmtime::Result<()> {
        let player = player_from_resource(self, &player)?;
        let mut mask = 0u8;
        if parts.contains(SkinParts::CAPE) {
            mask |= 0x01;
        }
        if parts.contains(SkinParts::JACKET) {
            mask |= 0x02;
        }
        if parts.contains(SkinParts::LEFT_SLEEVE) {
            mask |= 0x04;
        }
        if parts.contains(SkinParts::RIGHT_SLEEVE) {
            mask |= 0x08;
        }
        if parts.contains(SkinParts::LEFT_PANTS_LEG) {
            mask |= 0x10;
        }
        if parts.contains(SkinParts::RIGHT_PANTS_LEG) {
            mask |= 0x20;
        }
        if parts.contains(SkinParts::HAT) {
            mask |= 0x40;
        }

        {
            let mut config = (**player.config.load()).clone();
            config.skin_parts = mask;
            player.config.store(Arc::new(config));
        };
        player.send_client_information();
        Ok(())
    }

    async fn as_java(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Option<Resource<pumpkin::plugin::player::JavaPlayer>>> {
        let player = player_from_resource(self, &player)?;
        if let crate::net::ClientPlatform::Java(_) = player.client {
            Ok(Some(self.add_java_player(player)?))
        } else {
            Ok(None)
        }
    }

    async fn as_bedrock(
        &mut self,
        player: Resource<Player>,
    ) -> wasmtime::Result<Option<Resource<pumpkin::plugin::player::BedrockPlayer>>> {
        let player = player_from_resource(self, &player)?;
        if let crate::net::ClientPlatform::Bedrock(_) = player.client {
            Ok(Some(self.add_bedrock_player(player)?))
        } else {
            Ok(None)
        }
    }

    async fn drop(&mut self, rep: Resource<Player>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<PlayerResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}

impl pumpkin::plugin::player::HostJavaPlayer for PluginHostState {
    async fn get_version(
        &mut self,
        player: Resource<pumpkin::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<pumpkin::plugin::player::JavaMinecraftVersion> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::JavaPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid java-player resource handle"))?
            .provider
            .clone();

        let client = player
            .client
            .java()
            .ok_or_else(|| wasmtime::Error::msg("Not a java player"))?;
        Ok(to_wasm_java_version(client.version.load()))
    }

    async fn get_brand(
        &mut self,
        player: Resource<pumpkin::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<String> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::JavaPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid java-player resource handle"))?
            .provider
            .clone();

        let client = player
            .client
            .java()
            .ok_or_else(|| wasmtime::Error::msg("Not a java player"))?;
        Ok(client.brand.lock().await.clone().unwrap_or_default())
    }

    async fn get_server_address(
        &mut self,
        player: Resource<pumpkin::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<String> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::JavaPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid java-player resource handle"))?
            .provider
            .clone();

        let client = player
            .client
            .java()
            .ok_or_else(|| wasmtime::Error::msg("Not a java player"))?;
        Ok(client.server_address.lock().await.to_string())
    }

    async fn get_settings(
        &mut self,
        player: Resource<pumpkin::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<pumpkin::plugin::player::JavaPlayerSettings> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::JavaPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid java-player resource handle"))?
            .provider
            .clone();

        let config = player.config.load();
        let mask = config.skin_parts;
        let mut parts = SkinParts::empty();
        if mask & 0x01 != 0 {
            parts |= SkinParts::CAPE;
        }
        if mask & 0x02 != 0 {
            parts |= SkinParts::JACKET;
        }
        if mask & 0x04 != 0 {
            parts |= SkinParts::LEFT_SLEEVE;
        }
        if mask & 0x08 != 0 {
            parts |= SkinParts::RIGHT_SLEEVE;
        }
        if mask & 0x10 != 0 {
            parts |= SkinParts::LEFT_PANTS_LEG;
        }
        if mask & 0x20 != 0 {
            parts |= SkinParts::RIGHT_PANTS_LEG;
        }
        if mask & 0x40 != 0 {
            parts |= SkinParts::HAT;
        }

        Ok(pumpkin::plugin::player::JavaPlayerSettings {
            locale: config.locale.clone(),
            view_distance: config.view_distance.get(),
            chat_mode: to_wasm_chat_mode(&config.chat_mode),
            chat_colors: config.chat_colors,
            skin_parts: parts,
            main_hand: match config.main_hand {
                pumpkin_util::Hand::Left => pumpkin::plugin::common::Hand::Left,
                pumpkin_util::Hand::Right => pumpkin::plugin::common::Hand::Right,
            },
            text_filtering: config.text_filtering,
            server_listing: config.server_listing,
        })
    }

    async fn send_packet(
        &mut self,
        player: Resource<pumpkin::plugin::player::JavaPlayer>,
        packet: pumpkin::plugin::java_packets::ClientboundPacket,
    ) -> wasmtime::Result<()> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::JavaPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid java-player resource handle"))?
            .provider
            .clone();

        let client = player
            .client
            .java()
            .ok_or_else(|| wasmtime::Error::msg("Not a java player"))?;
        if let Some(bytes) = crate::plugin::loader::wasm::wasm_host::wit::v0_1::generated_packets::serialize_java_packet(
            &packet, client.version.load(),
        ) {
            client.send_packet_now_data(bytes).await;
        }
        Ok(())
    }

    async fn send_custom_payload(
        &mut self,
        player: Resource<pumpkin::plugin::player::JavaPlayer>,
        channel: String,
        data: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::JavaPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid java-player resource handle"))?
            .provider
            .clone();

        if let crate::net::ClientPlatform::Java(_) = player.client {
            player
                .client
                .send_packet_now(&pumpkin_protocol::java::client::play::CCustomPayload::new(
                    &channel, &data,
                ))
                .await;
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    async fn show_dialog(
        &mut self,
        player: Resource<pumpkin::plugin::player::JavaPlayer>,
        dialog: Dialog,
    ) -> wasmtime::Result<()> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::JavaPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid java-player resource handle"))?
            .provider
            .clone();

        let title = text_component_from_resource(self, &dialog.title);

        let body: Vec<_> = dialog
            .body
            .iter()
            .map(|b| match b {
                DialogBody::PlainMessage(c) => ProtocolDialogBody::PlainMessage {
                    contents: text_component_from_resource(self, c),
                },
                DialogBody::Item(_i) => {
                    // TODO: Map ItemStack correctly
                    ProtocolDialogBody::Item { item: 0 }
                }
            })
            .collect();

        let inputs: Vec<_> = dialog
            .inputs
            .iter()
            .map(|i| match i {
                DialogInput::Bool(b) => ProtocolDialogInput::Boolean {
                    label: text_component_from_resource(self, &b.label),
                    default_value: b.default_value,
                },
                DialogInput::Text(t) => ProtocolDialogInput::Text {
                    label: text_component_from_resource(self, &t.label),
                    placeholder: text_component_from_resource(self, &t.placeholder),
                    default_value: t.default_value.clone(),
                },
                DialogInput::NumberRange(n) => ProtocolDialogInput::NumberRange {
                    label: text_component_from_resource(self, &n.label),
                    min: n.min_value,
                    max: n.max_value,
                    initial: n.initial_value,
                    step: n.step,
                    label_format: n.label_format.clone(),
                },
                DialogInput::SingleOption(s) => ProtocolDialogInput::SingleOption {
                    label: text_component_from_resource(self, &s.label),
                    options: s
                        .options
                        .iter()
                        .map(|o| text_component_from_resource(self, o))
                        .collect(),
                    initial_index: s.initial_index,
                },
            })
            .collect();

        let buttons: Vec<_> = dialog
            .buttons
            .iter()
            .map(|b| ProtocolActionButton {
                text: text_component_from_resource(self, &b.text),
                tooltip: b
                    .tooltip
                    .as_ref()
                    .map(|t| text_component_from_resource(self, t)),
                width: b.width,
                action: match &b.action {
                    Action::OpenUrl(u) => DialogAction::OpenUrl { url: u.clone() },
                    Action::CustomClick(c) => DialogAction::Custom {
                        id: c.id.clone(),
                        payload: c.payload.clone(),
                    },
                },
            })
            .collect();

        let links: Vec<_> = dialog
            .links
            .iter()
            .map(|l| {
                let label = match &l.label {
                    LinkLabel::BuiltIn(t) => {
                        let link_type = match t {
                            LinkType::BugReport => pumpkin_protocol::LinkType::BugReport,
                            LinkType::CommunityGuidelines => {
                                pumpkin_protocol::LinkType::CommunityGuidelines
                            }
                            LinkType::Support => pumpkin_protocol::LinkType::Support,
                            LinkType::Status => pumpkin_protocol::LinkType::Status,
                            LinkType::Feedback => pumpkin_protocol::LinkType::Feedback,
                            LinkType::Community => pumpkin_protocol::LinkType::Community,
                            LinkType::Website => pumpkin_protocol::LinkType::Website,
                            LinkType::Forums => pumpkin_protocol::LinkType::Forums,
                            LinkType::News => pumpkin_protocol::LinkType::News,
                            LinkType::Announcements => pumpkin_protocol::LinkType::Announcements,
                        };
                        pumpkin_protocol::Label::BuiltIn(link_type)
                    }
                    LinkLabel::Custom(c) => pumpkin_protocol::Label::TextComponent(Box::new(
                        text_component_from_resource(self, c),
                    )),
                };
                DialogLink {
                    label,
                    url: l.url.clone(),
                }
            })
            .collect();

        let protocol_dialog = ProtocolDialog {
            r#type: match dialog.type_ {
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::java_dialogs::DialogType::Notice => "minecraft:notice".to_string(),
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::java_dialogs::DialogType::Confirmation => "minecraft:confirmation".to_string(),
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::java_dialogs::DialogType::MultiAction => "minecraft:multi_action".to_string(),
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::java_dialogs::DialogType::DialogList => "minecraft:dialog_list".to_string(),
                crate::plugin::loader::wasm::wasm_host::wit::v0_1::pumpkin::plugin::java_dialogs::DialogType::ServerLinks => "minecraft:server_links".to_string(),
            },
            title,
            body,
            inputs,
            buttons,
            links,
            exit_action: None, // TODO
            after_action: dialog.after_action.map(|a| match a {
                AfterAction::Peek => "peek".to_string(),
                AfterAction::Pop => "pop".to_string(),
            }),
            can_close_with_escape: dialog.can_close_with_escape,
            external_title: dialog.external_title.as_ref().map(|t| text_component_from_resource(self, t)),
        };

        if let crate::net::ClientPlatform::Java(client) = &player.client {
            match client.connection_state.load() {
                pumpkin_protocol::ConnectionState::Config => {
                    client
                        .send_packet_now(
                            &pumpkin_protocol::java::client::config::CConfigShowDialog::new(
                                pumpkin_protocol::IdOr::Value(DialogNBT(&protocol_dialog)),
                            ),
                        )
                        .await;
                }
                pumpkin_protocol::ConnectionState::Play => {
                    client
                        .send_packet_now(
                            &pumpkin_protocol::java::client::play::CPlayShowDialog::new(
                                pumpkin_protocol::IdOr::Value(DialogNBT(&protocol_dialog)),
                            ),
                        )
                        .await;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn clear_dialog(
        &mut self,
        player: Resource<pumpkin::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<()> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::JavaPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid java-player resource handle"))?
            .provider
            .clone();

        if let crate::net::ClientPlatform::Java(client) = &player.client {
            match client.connection_state.load() {
                pumpkin_protocol::ConnectionState::Config => {
                    client
                        .send_packet_now(
                            &pumpkin_protocol::java::client::config::CConfigClearDialog::new(),
                        )
                        .await;
                }
                pumpkin_protocol::ConnectionState::Play => {
                    client
                        .send_packet_now(
                            &pumpkin_protocol::java::client::play::CPlayClearDialog::new(),
                        )
                        .await;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn drop(
        &mut self,
        rep: Resource<pumpkin::plugin::player::JavaPlayer>,
    ) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<crate::plugin::loader::wasm::wasm_host::state::JavaPlayerResource>(
            Resource::new_own(rep.rep()),
        );
        Ok(())
    }
}

impl pumpkin::plugin::player::HostBedrockPlayer for PluginHostState {
    async fn get_version(
        &mut self,
        player: Resource<pumpkin::plugin::player::BedrockPlayer>,
    ) -> wasmtime::Result<pumpkin::plugin::player::BedrockMinecraftVersion> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::BedrockPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid bedrock-player resource handle"))?
            .provider
            .clone();

        if let crate::net::ClientPlatform::Bedrock(client) = &player.client {
            Ok(to_wasm_bedrock_version(client.version.load()))
        } else {
            Ok(pumpkin::plugin::player::BedrockMinecraftVersion::Unknown)
        }
    }

    async fn get_ability(
        &mut self,
        player: Resource<pumpkin::plugin::player::BedrockPlayer>,
        ability: pumpkin::plugin::player::BedrockAbility,
    ) -> wasmtime::Result<bool> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::BedrockPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid bedrock-player resource handle"))?
            .provider
            .clone();

        let ability = from_wasm_bedrock_ability(ability);
        let abilities = player.abilities.lock().await;

        match ability {
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Build
            | pumpkin_protocol::bedrock::client::update_abilities::Ability::Mine => {
                Ok(abilities.allow_modify_world)
            }
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Invulnerable => {
                Ok(abilities.invulnerable)
            }
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Flying => {
                Ok(abilities.flying)
            }
            pumpkin_protocol::bedrock::client::update_abilities::Ability::MayFly => {
                Ok(abilities.allow_flying)
            }
            pumpkin_protocol::bedrock::client::update_abilities::Ability::Instabuild => {
                Ok(abilities.creative)
            }
            _ => Ok(false), // Most Bedrock-specific abilities aren't tracked in generic Abilities struct yet
        }
    }

    async fn set_ability(
        &mut self,
        player: Resource<pumpkin::plugin::player::BedrockPlayer>,
        ability: pumpkin::plugin::player::BedrockAbility,
        value: bool,
    ) -> wasmtime::Result<()> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::BedrockPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid bedrock-player resource handle"))?
            .provider
            .clone();

        let ability = from_wasm_bedrock_ability(ability);
        {
            let mut abilities = player.abilities.lock().await;
            match ability {
                pumpkin_protocol::bedrock::client::update_abilities::Ability::Build
                | pumpkin_protocol::bedrock::client::update_abilities::Ability::Mine => {
                    abilities.allow_modify_world = value;
                }
                pumpkin_protocol::bedrock::client::update_abilities::Ability::Invulnerable => {
                    abilities.invulnerable = value;
                }
                pumpkin_protocol::bedrock::client::update_abilities::Ability::Flying => {
                    abilities.flying = value;
                }
                pumpkin_protocol::bedrock::client::update_abilities::Ability::MayFly => {
                    abilities.allow_flying = value;
                }
                pumpkin_protocol::bedrock::client::update_abilities::Ability::Instabuild => {
                    abilities.creative = value;
                }
                _ => {} // Not supported yet
            }
        };
        player.send_abilities_update().await;
        Ok(())
    }

    async fn get_status_flag(
        &mut self,
        player: Resource<pumpkin::plugin::player::BedrockPlayer>,
        flag: pumpkin::plugin::player::BedrockStatusFlag,
    ) -> wasmtime::Result<bool> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::BedrockPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid bedrock-player resource handle"))?
            .provider
            .clone();

        let flag_index = from_wasm_bedrock_status_flag(flag);
        if flag_index < 64 {
            let flags = player
                .living_entity
                .entity
                .bedrock_flags
                .load(Ordering::Relaxed);
            Ok((flags & (1 << flag_index)) != 0)
        } else {
            let flags = player
                .living_entity
                .entity
                .bedrock_flags_two
                .load(Ordering::Relaxed);
            Ok((flags & (1 << (flag_index - 64))) != 0)
        }
    }

    async fn set_status_flag(
        &mut self,
        player: Resource<pumpkin::plugin::player::BedrockPlayer>,
        flag: pumpkin::plugin::player::BedrockStatusFlag,
        value: bool,
    ) -> wasmtime::Result<()> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::BedrockPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid bedrock-player resource handle"))?
            .provider
            .clone();

        let flag_index = from_wasm_bedrock_status_flag(flag);
        if flag_index < 64 {
            let mut flags = player
                .living_entity
                .entity
                .bedrock_flags
                .load(Ordering::Relaxed);
            if value {
                flags |= 1 << flag_index;
            } else {
                flags &= !(1 << flag_index);
            }
            player
                .living_entity
                .entity
                .bedrock_flags
                .store(flags, Ordering::Relaxed);
        } else {
            let mut flags = player
                .living_entity
                .entity
                .bedrock_flags_two
                .load(Ordering::Relaxed);
            if value {
                flags |= 1 << (flag_index - 64);
            } else {
                flags &= !(1 << (flag_index - 64));
            }
            player
                .living_entity
                .entity
                .bedrock_flags_two
                .store(flags, Ordering::Relaxed);
        }

        let mut metadata = EntityMetadata(std::collections::HashMap::new());
        metadata.set(
            entity_data_key::FLAGS,
            MetadataValue::Long(
                player
                    .living_entity
                    .entity
                    .bedrock_flags
                    .load(Ordering::Relaxed),
            ),
        );
        metadata.set(
            entity_data_key::FLAGS_TWO,
            MetadataValue::Long(
                player
                    .living_entity
                    .entity
                    .bedrock_flags_two
                    .load(Ordering::Relaxed),
            ),
        );

        let packet = CSetActorData {
            actor_runtime_id: VarULong(player.get_entity().entity_id as u64),
            metadata,
            synced_properties: PropertySyncData {
                int_properties: std::collections::HashMap::new(),
                float_properties: std::collections::HashMap::new(),
            },
            tick: VarULong(0),
        };

        if let crate::net::ClientPlatform::Bedrock(client) = &player.client {
            client.send_game_packet(&packet).await;
        }

        Ok(())
    }

    async fn get_settings(
        &mut self,
        player: Resource<pumpkin::plugin::player::BedrockPlayer>,
    ) -> wasmtime::Result<pumpkin::plugin::player::BedrockPlayerSettings> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::BedrockPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid bedrock-player resource handle"))?
            .provider
            .clone();

        if let crate::net::ClientPlatform::Bedrock(client) = &player.client {
            let data = client.client_data.load();
            (**data).as_ref().map_or_else(
                || Err(wasmtime::Error::msg("client data not available")),
                |data| {
                    Ok(pumpkin::plugin::player::BedrockPlayerSettings {
                        game_version: data.game_version.clone(),
                        device_os: to_wasm_bedrock_device_os(data.device_os),
                        device_id: data.device_id.clone(),
                        device_model: data.device_model.clone(),
                        language_code: data.language_code.clone(),
                        current_input_mode: to_wasm_bedrock_input_mode(data.current_input_mode),
                        default_input_mode: to_wasm_bedrock_input_mode(data.default_input_mode),
                        ui_profile: to_wasm_bedrock_ui_profile(data.ui_profile),
                        gui_scale: data.gui_scale,
                        is_editor_mode: data.is_editor_mode,
                        max_view_distance: data.max_view_distance,
                        memory_tier: data.memory_tier,
                        graphics_mode: to_wasm_bedrock_graphics_mode(data.graphics_mode),
                        playfab_id: data.play_fab_id.clone(),
                        client_random_id: data.client_random_id,
                        platform_offline_id: data.platform_offline_id.clone(),
                        platform_online_id: data.platform_online_id.clone(),
                        skin_id: data.skin_id.clone(),
                        arm_size: data.arm_size.clone(),
                        is_persona_skin: data.persona_skin,
                        is_premium_skin: data.premium_skin,
                        is_trusted_skin: data.trusted_skin,
                    })
                },
            )
        } else {
            Err(wasmtime::Error::msg("not a bedrock player"))
        }
    }

    async fn send_packet(
        &mut self,
        player: Resource<pumpkin::plugin::player::BedrockPlayer>,
        packet: pumpkin::plugin::bedrock_packets::ClientboundPacket,
    ) -> wasmtime::Result<()> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::BedrockPlayerResource>(
                &Resource::new_own(player.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid bedrock-player resource handle"))?
            .provider
            .clone();

        if let Some(bytes) = crate::plugin::loader::wasm::wasm_host::wit::v0_1::generated_packets::serialize_bedrock_packet(
            &packet,
        ) {
            player.client.send_packet_now_data(bytes).await;
        }
        Ok(())
    }

    async fn open_form(
        &mut self,
        player_res: Resource<pumpkin::plugin::player::BedrockPlayer>,
        form: Form,
    ) -> wasmtime::Result<u32> {
        let player = self
            .resource_table
            .get::<crate::plugin::loader::wasm::wasm_host::state::BedrockPlayerResource>(
                &Resource::new_own(player_res.rep()),
            )
            .map_err(|_| wasmtime::Error::msg("invalid bedrock-player resource handle"))?
            .provider
            .clone();

        if let crate::net::ClientPlatform::Bedrock(client) = &player.client {
            let form_id = client.next_form_id.fetch_add(1, Ordering::Relaxed);

            let locale_str = player.config.load().locale.clone();
            let locale = Locale::from_str(&locale_str).unwrap_or(Locale::EnUs);

            let form_json = match form {
                Form::Simple(simple) => self.serialize_simple_form(simple, locale),
                Form::Modal(modal) => self.serialize_modal_form(&modal, locale),
                Form::Custom(custom) => self.serialize_custom_form(custom, locale),
            };

            client
                .send_game_packet(&CModalFormRequest {
                    form_id: pumpkin_protocol::codec::var_int::VarInt(form_id as i32),
                    form_data: form_json.to_string(),
                })
                .await;

            Ok(form_id)
        } else {
            Ok(0)
        }
    }

    async fn drop(
        &mut self,
        rep: Resource<pumpkin::plugin::player::BedrockPlayer>,
    ) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<crate::plugin::loader::wasm::wasm_host::state::BedrockPlayerResource>(
            Resource::new_own(rep.rep()),
        );
        Ok(())
    }
}
