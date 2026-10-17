use pumpkin_macros::Event;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::{entity::player::Player, world::World};

use super::PlayerEvent;

/// An event that occurs when a player respawns.
///
/// This is a notification fired once a respawn destination (world, position and
/// rotation) has been determined. It is not cancellable.
///
/// On a cross-dimension respawn the cancellable
/// [`PlayerChangeWorldEvent`](super::player_change_world::PlayerChangeWorldEvent)
/// fires first, before the world transfer; this event fires after the transfer
/// and reflects the world the player actually ended up in — including
/// `PlayerChangeWorldEvent` redirects and the no-op fallback (player stays in
/// the previous world) when that event is cancelled. See that event's docs for
/// the mutation-vs-cancellation semantics; in particular, edits made by a
/// cancelled `PlayerChangeWorldEvent` do not carry over here.
#[derive(Event, Clone)]
pub struct PlayerRespawnEvent {
    /// The player who respawned.
    pub player: Arc<Player>,

    /// The world the player respawned from (where they died).
    pub previous_world: Arc<World>,

    /// The world the player respawned into.
    pub respawned_world: Arc<World>,

    /// The position the player respawned at.
    pub position: Vector3<f64>,

    /// The yaw the player respawned with.
    pub yaw: f32,

    /// The pitch the player respawned with.
    pub pitch: f32,

    /// Whether the player kept their data (`true` when respawning while still
    /// alive, e.g. when leaving the End).
    pub alive: bool,
}

impl PlayerRespawnEvent {
    /// Creates a new instance of `PlayerRespawnEvent`.
    ///
    /// # Arguments
    /// - `player`: The player who respawned.
    /// - `previous_world`: The world the player respawned from.
    /// - `respawned_world`: The world the player respawned into.
    /// - `position`: The position the player respawned at.
    /// - `yaw`: The yaw the player respawned with.
    /// - `pitch`: The pitch the player respawned with.
    /// - `alive`: Whether the player kept their data.
    ///
    /// # Returns
    /// A new instance of `PlayerRespawnEvent`.
    pub const fn new(
        player: Arc<Player>,
        previous_world: Arc<World>,
        respawned_world: Arc<World>,
        position: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        alive: bool,
    ) -> Self {
        Self {
            player,
            previous_world,
            respawned_world,
            position,
            yaw,
            pitch,
            alive,
        }
    }
}

impl PlayerEvent for PlayerRespawnEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
