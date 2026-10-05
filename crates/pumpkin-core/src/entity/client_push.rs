//! Entity pushes a player's own client predicts.
//!
//! Server side both editions are the same: vanilla `pushEntities` and `Entity.push`, sent to
//! watchers only (`needsSync`), never to the pushed player.
//!
//! - Java: `ClientLevel.getPushableEntities` lets every living entity push the local player at
//!   the end of its tick, so the client moves itself.
//! - Bedrock: the player definition has no `minecraft:pushable_by_entity`, so the client never
//!   does. The server runs that Java client push per `PlayerAuthInput` tick and sends it as a
//!   `CorrectPlayerMovePrediction` for that tick, which the client rewinds to and replays.
//!   Pushes the client has not acked yet (`NetworkStackLatency`, answered in order) are missing
//!   from its reports, so they are carried with the client's friction until the ack.

use std::collections::VecDeque;
use std::sync::atomic::Ordering;

use pumpkin_protocol::bedrock::client::{CCorrectPlayerMovePrediction, CNetworkStackLatency};
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use super::{EntityBase, player::Player, velocity};
use crate::net::bedrock::BedrockClient;

/// `RewindType.Player`
const REWIND_PLAYER: u8 = 0;

/// Velocity the Java client adds to the local player at the end of its tick.
#[must_use]
pub fn java_client_push(player: &Player) -> Vector3<f64> {
    let mut push = Vector3::default();
    let entity = player.get_entity();
    if entity.no_physics.load(Ordering::Relaxed)
        || entity.has_vehicle()
        || entity.has_passengers()
        || !player.is_pushable()
    {
        return push;
    }

    let pos = entity.pos.load();
    let world = entity.world.load();
    for pusher in world.get_all_at_box(&entity.bounding_box.load()) {
        let pusher_entity = pusher.get_entity();
        if pusher_entity.entity_id == entity.entity_id
            || pusher.get_living_entity().is_none()
            || pusher_entity.no_physics.load(Ordering::Relaxed)
            || entity.is_passenger_of_same_vehicle(pusher_entity)
            || !velocity::pushable_by(pusher.as_ref(), player)
        {
            continue;
        }

        // Vanilla `Entity.push(Entity)`, the local player's side.
        let pusher_pos = pusher_entity.pos.load();
        let mut dx = pusher_pos.x - pos.x;
        let mut dz = pusher_pos.z - pos.z;
        let mut d = dx.abs().max(dz.abs());
        if d < f64::from(0.01f32) {
            continue;
        }
        d = d.sqrt();
        dx /= d;
        dz /= d;
        let scale = (1.0 / d).min(1.0) * f64::from(0.05f32);
        push.x -= dx * scale;
        push.z -= dz * scale;
    }
    push
}

/// Bedrock pushes sent but not yet in the client's reports.
#[derive(Default)]
pub struct BedrockPushSync {
    /// Input tick of each sent push and its velocity carried to the last processed tick.
    unseen: Vec<(u64, Vector3<f64>)>,
    /// Input ticks of sent corrections, acked in order.
    awaiting_ack: VecDeque<u64>,
}

impl BedrockPushSync {
    fn lock(bedrock: &BedrockClient) -> std::sync::MutexGuard<'_, Self> {
        bedrock
            .push_sync
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// One `PlayerAuthInput` tick: the Java client push for it, applied at that tick.
pub fn on_bedrock_input(
    player: &Player,
    bedrock: &BedrockClient,
    tick: u64,
    position: Vector3<f32>,
    delta: Vector3<f32>,
    on_ground: bool,
) {
    let mut sync = BedrockPushSync::lock(bedrock);
    let friction = horizontal_friction(player, on_ground);
    for (_, carried) in &mut sync.unseen {
        carried.x *= friction;
        carried.z *= friction;
    }

    let push = java_client_push(player);
    // Ticks without a push need nothing: an in-flight correction replays them.
    if push == Vector3::default() {
        return;
    }
    let missing = sync
        .unseen
        .iter()
        .fold(Vector3::default(), |sum, (_, carried)| sum + *carried);
    sync.unseen.push((tick, push));
    sync.awaiting_ack.push_back(tick);
    drop(sync);

    let entity = player.get_entity();
    bedrock.try_enqueue_client_packet(&CCorrectPlayerMovePrediction {
        prediction_type: REWIND_PLAYER,
        pos: position,
        pos_delta: (delta.to_f64() + missing + push).to_f32_lossy(),
        rotation: Vector2::new(entity.pitch.load(), entity.yaw.load()),
        vehicle_angular_velocity: None,
        on_ground,
        tick: VarULong(tick),
    });
    bedrock.try_enqueue_client_packet(&CNetworkStackLatency {
        timestamp: tick,
        needs_response: true,
    });
}

/// `NetworkStackLatency` reply: reports from now on include the acked correction.
pub fn on_bedrock_ack(bedrock: &BedrockClient) {
    let mut sync = BedrockPushSync::lock(bedrock);
    if let Some(acked) = sync.awaiting_ack.pop_front() {
        sync.unseen.retain(|(tick, _)| *tick > acked);
    }
}

/// Horizontal velocity factor of one client tick: vanilla `travelInAir`, `travelInFluid`.
fn horizontal_friction(player: &Player, on_ground: bool) -> f64 {
    let entity = player.get_entity();
    if entity.touching_lava.load(Ordering::Relaxed) {
        0.5
    } else if entity.touching_water.load(Ordering::Relaxed) {
        if entity.is_sprinting() {
            f64::from(0.9f32)
        } else {
            f64::from(0.8f32)
        }
    } else if on_ground {
        f64::from(entity.get_block_with_y_offset(0.500_001).1.slipperiness * 0.91)
    } else {
        f64::from(0.91f32)
    }
}
