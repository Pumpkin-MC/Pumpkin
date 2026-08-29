use pumpkin_macros::packet;

use crate::{
    codec::var_long::VarLong,
    serial::{PacketRead, PacketWrite},
};

pub const BOSS_EVENT_SHOW: u8 = 0;
pub const BOSS_EVENT_REGISTER_PLAYER: u8 = 1;
pub const BOSS_EVENT_HIDE: u8 = 2;
pub const BOSS_EVENT_UNREGISTER_PLAYER: u8 = 3;
pub const BOSS_EVENT_HEALTH_PERCENTAGE: u8 = 4;
pub const BOSS_EVENT_TITLE: u8 = 5;
pub const BOSS_EVENT_APPEARANCE_PROPERTIES: u8 = 6;
pub const BOSS_EVENT_TEXTURE: u8 = 7;
pub const BOSS_EVENT_REQUEST: u8 = 8;

pub const BOSS_EVENT_COLOUR_PINK: u8 = 0;
pub const BOSS_EVENT_COLOUR_BLUE: u8 = 1;
pub const BOSS_EVENT_COLOUR_RED: u8 = 2;
pub const BOSS_EVENT_COLOUR_GREEN: u8 = 3;
pub const BOSS_EVENT_COLOUR_YELLOW: u8 = 4;
pub const BOSS_EVENT_COLOUR_PURPLE: u8 = 5;
pub const BOSS_EVENT_COLOUR_REBECCA_PURPLE: u8 = 6;
pub const BOSS_EVENT_COLOUR_WHITE: u8 = 7;

pub const BOSS_EVENT_OVERLAY_PROGRESS: u8 = 0;
pub const BOSS_EVENT_OVERLAY_NOTCHED_6: u8 = 1;
pub const BOSS_EVENT_OVERLAY_NOTCHED_10: u8 = 2;
pub const BOSS_EVENT_OVERLAY_NOTCHED_12: u8 = 3;
pub const BOSS_EVENT_OVERLAY_NOTCHED_20: u8 = 4;

/// Sent by the server to make a specific 'boss event' occur in the world.
///
/// Packet ID: `74`
#[derive(PacketWrite, PacketRead, Clone, Debug, PartialEq)]
#[packet(74)]
pub struct CBossEvent {
    /// The unique ID of the boss entity that the boss event sent involves.
    pub boss_entity_id: VarLong,
    /// The unique ID of the player that is registered to or unregistered from the boss fight.
    pub player_entity_id: VarLong,
    /// The type of the event (one of `BOSS_EVENT_*`).
    pub event_type: u8,
    /// The title shown above the boss bar.
    pub title: String,
    /// Filtered version of `title` with profanity removed.
    pub filtered_title: String,
    /// The percentage of health shown in the boss bar (0.0 - 1.0).
    pub health_percentage: f32,
    /// The colour of the boss bar (one of `BOSS_EVENT_COLOUR_*`).
    pub color: u8,
    /// The overlay of the boss bar (one of `BOSS_EVENT_OVERLAY_*`).
    pub overlay: u8,
}

impl CBossEvent {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        boss_entity_id: VarLong,
        player_entity_id: VarLong,
        event_type: u8,
        title: String,
        filtered_title: String,
        health_percentage: f32,
        color: u8,
        overlay: u8,
    ) -> Self {
        Self {
            boss_entity_id,
            player_entity_id,
            event_type,
            title,
            filtered_title,
            health_percentage,
            color,
            overlay,
        }
    }

    #[must_use]
    pub fn show(
        boss_entity_id: VarLong,
        player_entity_id: VarLong,
        title: impl Into<String>,
        health_percentage: f32,
        color: u8,
        overlay: u8,
    ) -> Self {
        let title = title.into();
        Self {
            boss_entity_id,
            player_entity_id,
            event_type: BOSS_EVENT_SHOW,
            filtered_title: title.clone(),
            title,
            health_percentage,
            color,
            overlay,
        }
    }

    #[must_use]
    pub const fn register_player(boss_entity_id: VarLong, player_entity_id: VarLong) -> Self {
        Self {
            boss_entity_id,
            player_entity_id,
            event_type: BOSS_EVENT_REGISTER_PLAYER,
            title: String::new(),
            filtered_title: String::new(),
            health_percentage: 0.0,
            color: 0,
            overlay: 0,
        }
    }

    #[must_use]
    pub const fn hide(boss_entity_id: VarLong) -> Self {
        Self {
            boss_entity_id,
            player_entity_id: VarLong(0),
            event_type: BOSS_EVENT_HIDE,
            title: String::new(),
            filtered_title: String::new(),
            health_percentage: 0.0,
            color: 0,
            overlay: 0,
        }
    }

    #[must_use]
    pub const fn unregister_player(boss_entity_id: VarLong, player_entity_id: VarLong) -> Self {
        Self {
            boss_entity_id,
            player_entity_id,
            event_type: BOSS_EVENT_UNREGISTER_PLAYER,
            title: String::new(),
            filtered_title: String::new(),
            health_percentage: 0.0,
            color: 0,
            overlay: 0,
        }
    }

    #[must_use]
    pub const fn update_health(boss_entity_id: VarLong, health_percentage: f32) -> Self {
        Self {
            boss_entity_id,
            player_entity_id: VarLong(0),
            event_type: BOSS_EVENT_HEALTH_PERCENTAGE,
            title: String::new(),
            filtered_title: String::new(),
            health_percentage,
            color: 0,
            overlay: 0,
        }
    }

    #[must_use]
    pub fn update_title(boss_entity_id: VarLong, title: impl Into<String>) -> Self {
        let title = title.into();
        Self {
            boss_entity_id,
            player_entity_id: VarLong(0),
            event_type: BOSS_EVENT_TITLE,
            filtered_title: title.clone(),
            title,
            health_percentage: 0.0,
            color: 0,
            overlay: 0,
        }
    }

    #[must_use]
    pub const fn update_properties(boss_entity_id: VarLong, color: u8, overlay: u8) -> Self {
        Self {
            boss_entity_id,
            player_entity_id: VarLong(0),
            event_type: BOSS_EVENT_APPEARANCE_PROPERTIES,
            title: String::new(),
            filtered_title: String::new(),
            health_percentage: 0.0,
            color,
            overlay,
        }
    }

    #[must_use]
    pub const fn update_texture(boss_entity_id: VarLong, color: u8, overlay: u8) -> Self {
        Self {
            boss_entity_id,
            player_entity_id: VarLong(0),
            event_type: BOSS_EVENT_TEXTURE,
            title: String::new(),
            filtered_title: String::new(),
            health_percentage: 0.0,
            color,
            overlay,
        }
    }

    #[must_use]
    pub const fn request(boss_entity_id: VarLong, player_entity_id: VarLong) -> Self {
        Self {
            boss_entity_id,
            player_entity_id,
            event_type: BOSS_EVENT_REQUEST,
            title: String::new(),
            filtered_title: String::new(),
            health_percentage: 0.0,
            color: 0,
            overlay: 0,
        }
    }
}
