use std::{num::NonZeroU8, sync::Arc, sync::atomic::Ordering};

use crate::{
    entity::player::ChatMode,
    net::{
        PlayerConfig, can_not_join,
        java::{JavaClient, PacketHandlerResult},
    },
    server::Server,
};
use core::str;
use pumpkin_data::{registry::Registry, translation};
use pumpkin_protocol::{
    ConnectionState, KnownPack,
    java::{
        client::config::{CFeatureFlags, CFinishConfig, CKnownPacks, CRegistryData, CUpdateTags},
        server::config::{
            ResourcePackResponseResult, SClientInformationConfig, SConfigCookieResponse,
            SConfigResourcePack, SKeepAlive, SKnownPacks, SPluginMessage,
        },
    },
};
use pumpkin_util::{Hand, text::TextComponent, version::JavaMinecraftVersion};
use tracing::{debug, trace, warn};

const BRAND_CHANNEL_PREFIX: &str = "minecraft:brand";

pub mod client_information;
pub mod config_acknowledged;
pub(super) use config_acknowledged::build_dimension_nbt;
pub mod cookie_response;
pub mod keep_alive;
pub mod known_packs;
pub mod plugin_message;
pub mod resource_pack;
