use arc_swap::ArcSwap;
use pumpkin_data::translation;
use pumpkin_protocol::{
    ConnectionState, KnownPack, Label, Link, LinkType,
    java::client::{
        config::{
            CConfigAddResourcePack, CConfigServerLinks, CFeatureFlags, CFinishConfig, CKnownPacks,
            CRegistryData, CUpdateTags,
        },
        login::{CLoginSuccess, CSetCompression},
    },
    java::server::config::SKnownPacks,
    java::server::login::{
        SEncryptionResponse, SLoginCookieResponse, SLoginPluginResponse, SLoginStart,
    },
};
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

use crate::{
    net::{
        GameProfile,
        authentication::{self, AuthError},
        is_valid_player_name,
        java::pending::PendingConnection,
        offline_uuid,
        proxy::{bungeecord, velocity},
    },
    server::Server,
};

pub mod cookie_response;
pub mod encryption_response;
pub mod known_packs;
pub mod login_acknowledged;
pub mod login_start;
pub mod plugin_response;
