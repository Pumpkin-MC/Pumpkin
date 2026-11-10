use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// Why a player is being teleported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TeleportCause {
    /// A thrown ender pearl landed.
    EnderPearl,
    /// A command such as `/tp`.
    Command,
    /// A plugin asked for it.
    Plugin,
    NetherPortal,
    EndPortal,
    EndGateway,
    /// Spectating an entity or player.
    Spectate,
    /// A consumed item, such as a chorus fruit.
    ConsumableEffect,
    /// Not set anywhere yet.
    Dismount,
    /// Not set anywhere yet.
    ExitBed,
    /// The caller gave no cause.
    Unknown,
}

/// An event that occurs when a player teleports.
///
/// If the event is cancelled, the teleportation will not happen.
///
/// This event contains information about the player, the position from which the player teleported, and the position to which the player teleported.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerTeleportEvent {
    /// The player who teleported.
    pub player: Arc<Player>,

    /// The position from which the player teleported.
    pub from: Vector3<f64>,

    /// The position to which the player teleported.
    pub to: Vector3<f64>,

    /// The yaw the player ends up with. `None` keeps the player's own view.
    pub yaw: Option<f32>,

    /// The pitch the player ends up with. `None` keeps the player's own view.
    pub pitch: Option<f32>,

    /// Why the player is teleported.
    pub cause: TeleportCause,
}

impl PlayerTeleportEvent {
    /// Creates a new instance of `PlayerTeleportEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who teleported.
    /// - `from`: The position from which the player teleported.
    /// - `to`: The position to which the player teleported.
    /// - `yaw`: The new yaw, or `None` to keep the view.
    /// - `pitch`: The new pitch, or `None` to keep the view.
    /// - `cause`: Why the player is teleported.
    ///
    /// # Returns
    /// A new instance of `PlayerTeleportEvent`.
    pub const fn new(
        player: Arc<Player>,
        from: Vector3<f64>,
        to: Vector3<f64>,
        yaw: Option<f32>,
        pitch: Option<f32>,
        cause: TeleportCause,
    ) -> Self {
        Self {
            player,
            from,
            to,
            yaw,
            pitch,
            cause,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerTeleportEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
