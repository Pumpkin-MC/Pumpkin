// Phase 1 game event / vibration engine. Cites vanilla
// net.minecraft.world.level.gameevent.{GameEvent, GameEventListener, GameEventDispatcher,
// PositionSource, BlockPositionSource, EntityPositionSource} and
// .vibrations.{VibrationSystem, VibrationSelector, VibrationInfo}.
//
// Deliberate scope reductions vs. vanilla:
// - Vanilla shards listeners per chunk-section (EuclideanGameEventListenerRegistry) for
//   performance at scale. This is a single flat per-world Vec<Arc<dyn GameEventListener>>,
//   registered/unregistered explicitly.
// - Vanilla's VibrationSystem.Ticker simulates a per-listener travel time before a
//   selected vibration is actually delivered, and streams a travelling particle to
//   clients. This resolves synchronously in the same tick the event is emitted; the
//   VibrationSelector in vibration.rs is ported and unit-tested but not yet wired into
//   this path (needs a per-tick driver, deferred to a future phase).
// - Vanilla's occlusion check (VibrationSystem.Listener.isOccluded) nudges the ray in
//   all 6 directions before a ClipBlockStateContext raycast. This does a straight-line
//   block-center sampling approximation against the same
//   minecraft:occludes_vibration_signals tag instead of a full raycast.

pub mod vibration;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use pumpkin_data::game_event::GameEvent;
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::EntityBase;
use crate::world::World;

pub type GameEventFuture<'a> = Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

// GameEvent.Context (GameEvent.java)
pub struct GameEventContext {
    pub source_entity: Option<Arc<dyn EntityBase>>,
}

impl GameEventContext {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            source_entity: None,
        }
    }

    pub fn of_entity(entity: Arc<dyn EntityBase>) -> Self {
        Self {
            source_entity: Some(entity),
        }
    }
}

// PositionSource / BlockPositionSource / EntityPositionSource
#[derive(Clone, Copy)]
pub enum PositionSource {
    Block(BlockPos),
    Entity(uuid::Uuid),
}

impl PositionSource {
    pub fn get_position(&self, world: &World) -> Option<Vector3<f64>> {
        match self {
            Self::Block(pos) => Some(Vector3::new(
                f64::from(pos.0.x) + 0.5,
                f64::from(pos.0.y) + 0.5,
                f64::from(pos.0.z) + 0.5,
            )),
            Self::Entity(uuid) => world
                .get_entity_by_uuid(*uuid)
                .map(|entity| entity.get_entity().pos.load()),
        }
    }
}

// GameEventListener.java
pub trait GameEventListener: Send + Sync {
    fn listener_source(&self) -> PositionSource;
    fn listener_radius(&self) -> i32;

    fn handle_game_event<'a>(
        &'a self,
        world: &'a Arc<World>,
        event: &'a GameEvent,
        context: &'a GameEventContext,
        source_position: Vector3<f64>,
    ) -> GameEventFuture<'a>;
}

// GameEvent.java bootstrap: DEFAULT_NOTIFICATION_RADIUS = 16; jukebox_play and
// jukebox_stop_play register at 10; shriek registers at 32. All others use the default.
#[must_use]
pub const fn notification_radius(event: &GameEvent) -> i32 {
    match event {
        GameEvent::JukeboxPlay | GameEvent::JukeboxStopPlay => 10,
        GameEvent::Shriek => 32,
        _ => 16,
    }
}

// VibrationSystem.VIBRATION_FREQUENCY_FOR_EVENT
#[must_use]
pub const fn vibration_frequency(event: &GameEvent) -> i32 {
    match event {
        GameEvent::Step | GameEvent::Swim | GameEvent::Flap | GameEvent::Resonate1 => 1,
        GameEvent::ProjectileLand
        | GameEvent::HitGround
        | GameEvent::Splash
        | GameEvent::Bounce
        | GameEvent::Resonate2 => 2,
        GameEvent::ItemInteractFinish
        | GameEvent::ProjectileShoot
        | GameEvent::InstrumentPlay
        | GameEvent::Resonate3 => 3,
        GameEvent::EntityAction
        | GameEvent::ElytraGlide
        | GameEvent::Unequip
        | GameEvent::Resonate4 => 4,
        GameEvent::EntityDismount | GameEvent::Equip | GameEvent::Resonate5 => 5,
        GameEvent::EntityInteract
        | GameEvent::Shear
        | GameEvent::EntityMount
        | GameEvent::Resonate6 => 6,
        GameEvent::EntityDamage | GameEvent::Resonate7 => 7,
        GameEvent::Drink | GameEvent::Eat | GameEvent::Resonate8 => 8,
        GameEvent::ContainerClose
        | GameEvent::BlockClose
        | GameEvent::BlockDeactivate
        | GameEvent::BlockDetach
        | GameEvent::Resonate9 => 9,
        GameEvent::ContainerOpen
        | GameEvent::BlockOpen
        | GameEvent::BlockActivate
        | GameEvent::BlockAttach
        | GameEvent::PrimeFuse
        | GameEvent::NoteBlockPlay
        | GameEvent::Resonate10 => 10,
        GameEvent::BlockChange | GameEvent::Resonate11 => 11,
        GameEvent::BlockDestroy | GameEvent::FluidPickup | GameEvent::Resonate12 => 12,
        GameEvent::BlockPlace | GameEvent::FluidPlace | GameEvent::Resonate13 => 13,
        GameEvent::EntityPlace
        | GameEvent::LightningStrike
        | GameEvent::Teleport
        | GameEvent::Resonate14 => 14,
        GameEvent::EntityDie | GameEvent::Explode | GameEvent::Resonate15 => 15,
        _ => 0,
    }
}

// VibrationSystem.getRedstoneStrengthForDistance
#[must_use]
pub fn redstone_strength_for_distance(distance: f32, listener_radius: i32) -> u8 {
    let power_scale = 15.0 / f64::from(listener_radius);
    let strength = 15 - (power_scale * f64::from(distance)).floor() as i32;
    strength.max(1) as u8
}

// VibrationSystem.Listener.isOccluded, approximated: samples block centers along the
// straight line from `from` to `to` (exclusive of the endpoints) instead of vanilla's
// 6-direction nudged ClipBlockStateContext raycast.
fn is_occluded(world: &World, from: Vector3<f64>, to: Vector3<f64>) -> bool {
    let delta = to - from;
    let distance = delta.length();
    if distance < 1.0e-6 {
        return false;
    }
    let steps = distance.ceil() as i32;
    for i in 1..steps {
        let t = f64::from(i) / f64::from(steps);
        let sample = Vector3::new(
            from.x + delta.x * t,
            from.y + delta.y * t,
            from.z + delta.z * t,
        );
        let pos = BlockPos::new(
            sample.x.floor() as i32,
            sample.y.floor() as i32,
            sample.z.floor() as i32,
        );
        if world
            .get_block(&pos)
            .has_tag(&tag::Block::MINECRAFT_OCCLUDES_VIBRATION_SIGNALS)
        {
            return true;
        }
    }
    false
}

// GameEventDispatcher.post: notifies every registered listener within the event's
// notification radius, closest-first (GameEvent.ListenerInfo::compareTo), gated by the
// occlusion check above. Vanilla's chunk-section iteration is replaced by scanning the
// flat registry (see module doc comment).
pub async fn emit_game_event(
    world: &Arc<World>,
    event: GameEvent,
    position: Vector3<f64>,
    context: GameEventContext,
) {
    let radius = f64::from(notification_radius(&event));
    let radius_sq = radius * radius;

    let listeners = world.game_event_listeners.lock().await.clone();
    let mut in_range: Vec<(f64, Arc<dyn GameEventListener>, Vector3<f64>)> = listeners
        .into_iter()
        .filter_map(|listener| {
            let listener_pos = listener.listener_source().get_position(world)?;
            let dist_sq = (listener_pos - position).length_squared();
            let listener_radius = f64::from(listener.listener_radius());
            (dist_sq <= radius_sq.min(listener_radius * listener_radius)).then_some((
                dist_sq,
                listener,
                listener_pos,
            ))
        })
        .collect();

    in_range.sort_by(|a, b| a.0.total_cmp(&b.0));

    for (_, listener, listener_pos) in in_range {
        if is_occluded(world, position, listener_pos) {
            continue;
        }
        listener
            .handle_game_event(world, &event, &context, position)
            .await;
    }
}
