//! Vanilla `net.minecraft.world.level.gameevent.vibrations.*`

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use pumpkin_data::block_properties::{
    BlockProperties, CalibratedSculkSensorLikeProperties, SculkSensorLikeProperties,
    SculkSensorPhase,
};
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::tag::Taggable;
use pumpkin_data::{BlockDirection, BlockId, BlockStateId, particle::Particle, tag};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::CParticle;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Mutex;
use std::sync::atomic::Ordering;
use uuid::Uuid;

use crate::entity::EntityBase;
use crate::world::World;

pub trait GameEventExt {
    fn default_frequency(&self) -> u32;
}

impl GameEventExt for GameEvent {
    fn default_frequency(&self) -> u32 {
        match self {
            Self::Step | Self::Swim | Self::Flap | Self::Resonate1 => 1,
            Self::ProjectileLand
            | Self::HitGround
            | Self::Splash
            | Self::Bounce
            | Self::Resonate2 => 2,
            Self::ItemInteractFinish
            | Self::ProjectileShoot
            | Self::InstrumentPlay
            | Self::Resonate3 => 3,
            Self::EntityAction | Self::ElytraGlide | Self::Unequip | Self::Resonate4 => 4,
            Self::EntityDismount | Self::Equip | Self::Resonate5 => 5,
            Self::EntityInteract | Self::Shear | Self::EntityMount | Self::Resonate6 => 6,
            Self::EntityDamage | Self::Resonate7 => 7,
            Self::Drink | Self::Eat | Self::Resonate8 => 8,
            Self::ContainerClose
            | Self::BlockClose
            | Self::BlockDeactivate
            | Self::BlockDetach
            | Self::Resonate9 => 9,
            Self::ContainerOpen
            | Self::BlockOpen
            | Self::BlockActivate
            | Self::BlockAttach
            | Self::PrimeFuse
            | Self::NoteBlockPlay
            | Self::Resonate10 => 10,
            Self::BlockChange | Self::Resonate11 => 11,
            Self::BlockDestroy | Self::FluidPickup | Self::Resonate12 => 12,
            Self::BlockPlace | Self::FluidPlace | Self::Resonate13 => 13,
            Self::EntityPlace | Self::LightningStrike | Self::Teleport | Self::Resonate14 => 14,
            Self::EntityDie | Self::Explode | Self::Resonate15 => 15,
            _ => 0,
        }
    }
}

#[must_use]
pub fn get_redstone_strength_for_distance(d: f32, listener_radius: i32) -> i32 {
    if listener_radius == 0 {
        return 0;
    }
    let power_scale = 15.0 / (listener_radius as f32);
    (15 - (power_scale * d).floor() as i32).max(1)
}

#[derive(Clone, Default)]
pub struct GameEventContext {
    source_entity: Option<Arc<dyn EntityBase>>,
    affected_state: Option<BlockStateId>,
}

impl GameEventContext {
    pub fn of_entity(entity: &Arc<dyn EntityBase>) -> Self {
        Self {
            source_entity: Some(Arc::clone(entity)),
            affected_state: None,
        }
    }
    #[must_use]
    pub const fn with_affected_state(mut self, state: BlockStateId) -> Self {
        self.affected_state = Some(state);
        self
    }
    #[must_use]
    pub fn source_entity(&self) -> Option<&Arc<dyn EntityBase>> {
        self.source_entity.as_ref()
    }
}

pub struct VibrationInfo {
    pub game_event: GameEvent,
    pub pos: Vector3<f64>,
    pub source_entity: Option<Arc<dyn EntityBase>>,
    pub source_entity_uuid: Option<Uuid>,
    pub distance: f32,
    pub tick: i64,
}

impl VibrationInfo {
    fn to_nbt(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.put_string(
            "game_event",
            format!("minecraft:{}", self.game_event.name()),
        );
        nbt.put_float("distance", self.distance);
        nbt.put_list(
            "pos",
            vec![
                NbtTag::Double(self.pos.x),
                NbtTag::Double(self.pos.y),
                NbtTag::Double(self.pos.z),
            ],
        );
        if let Some(source) = self.source_entity.as_ref() {
            nbt.put_uuid("source", source.get_entity().entity_uuid);
        } else if let Some(source) = self.source_entity_uuid {
            nbt.put_uuid("source", source);
        }
        nbt
    }

    fn from_nbt(nbt: &NbtCompound, tick: i64) -> Option<Self> {
        let pos = nbt.get_list("pos")?;
        let [x, y, z] = pos else { return None };
        Some(Self {
            game_event: GameEvent::from_name(nbt.get_string("game_event")?)?,
            pos: Vector3::new(
                x.extract_double()?,
                y.extract_double()?,
                z.extract_double()?,
            ),
            source_entity: None,
            source_entity_uuid: nbt.get_uuid("source"),
            distance: nbt.get_float("distance")?,
            tick,
        })
    }
}

pub struct VibrationSelector {
    candidate: Option<VibrationInfo>,
}

impl Default for VibrationSelector {
    fn default() -> Self {
        Self::new()
    }
}
impl VibrationSelector {
    #[must_use]
    pub const fn new() -> Self {
        Self { candidate: None }
    }

    pub fn add_candidate(&mut self, info: VibrationInfo) {
        let should_replace = match &self.candidate {
            None => true,
            Some(prev) => {
                if info.tick != prev.tick {
                    false
                } else if info.distance < prev.distance {
                    true
                } else if info.distance > prev.distance {
                    false
                } else {
                    // Same distance — higher frequency wins tiebreak.
                    info.game_event.default_frequency() > prev.game_event.default_frequency()
                }
            }
        };
        if should_replace {
            self.candidate = Some(info);
        }
    }

    pub fn choose(&mut self, time: i64) -> Option<VibrationInfo> {
        let info = self.candidate.as_ref()?;
        if info.tick < time {
            self.candidate.take()
        } else {
            None
        }
    }

    fn to_nbt(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        if let Some(candidate) = self.candidate.as_ref() {
            nbt.put_compound("event", candidate.to_nbt());
            nbt.put_long("tick", candidate.tick);
        } else {
            nbt.put_long("tick", -1);
        }
        nbt
    }

    fn from_nbt(nbt: &NbtCompound) -> Self {
        let candidate = nbt
            .get_compound("event")
            .and_then(|event| VibrationInfo::from_nbt(event, nbt.get_long("tick").unwrap_or(-1)));
        Self { candidate }
    }
}

pub struct VibrationData {
    current_vibration: Option<VibrationInfo>,
    receive_time: i32,
    selector: VibrationSelector,
    reload_particle: bool,
}

impl Default for VibrationData {
    fn default() -> Self {
        Self::new()
    }
}
impl VibrationData {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current_vibration: None,
            receive_time: 0,
            selector: VibrationSelector::new(),
            reload_particle: false,
        }
    }

    pub const fn selector_mut(&mut self) -> &mut VibrationSelector {
        &mut self.selector
    }

    #[must_use]
    pub const fn has_current_vibration(&self) -> bool {
        self.current_vibration.is_some()
    }

    pub fn try_select_and_schedule(
        &mut self,
        world_tick: i64,
        user: &dyn VibrationUser,
    ) -> Option<(Vector3<f64>, i32)> {
        if self.current_vibration.is_some() {
            return None;
        }
        let vib = self.selector.choose(world_tick)?;
        self.receive_time = user.calculate_travel_time_in_ticks(vib.distance);
        let particle = (vib.pos, self.receive_time);
        self.current_vibration = Some(vib);
        Some(particle)
    }

    pub const fn tick_receive(&mut self) -> bool {
        if self.receive_time > 0 {
            self.receive_time -= 1;
        }
        self.receive_time == 0 && self.current_vibration.is_some()
    }

    pub const fn consume_current(&mut self) -> Option<VibrationInfo> {
        self.current_vibration.take()
    }

    fn reload_particle(
        &mut self,
        destination: Vector3<f64>,
        user: &dyn VibrationUser,
    ) -> Option<(Vector3<f64>, i32)> {
        if !std::mem::take(&mut self.reload_particle) {
            return None;
        }
        let vibration = self.current_vibration.as_ref()?;
        let initial_time = user.calculate_travel_time_in_ticks(vibration.distance);
        let progress = if initial_time == 0 {
            0.0
        } else {
            1.0 - f64::from(self.receive_time) / f64::from(initial_time)
        };
        Some((
            vibration.pos.lerp(&destination, progress),
            self.receive_time,
        ))
    }

    #[must_use]
    pub fn to_nbt(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        if let Some(vibration) = self.current_vibration.as_ref() {
            nbt.put_compound("event", vibration.to_nbt());
        }
        nbt.put_compound("selector", self.selector.to_nbt());
        nbt.put_int("event_delay", self.receive_time.max(0));
        nbt
    }

    pub fn from_nbt(nbt: &NbtCompound) -> Self {
        let current_vibration = nbt
            .get_compound("event")
            .and_then(|event| VibrationInfo::from_nbt(event, -1));
        Self {
            reload_particle: current_vibration.is_some(),
            current_vibration,
            receive_time: nbt.get_int("event_delay").unwrap_or(0).max(0),
            selector: nbt
                .get_compound("selector")
                .map_or_else(VibrationSelector::new, VibrationSelector::from_nbt),
        }
    }
}

pub trait VibrationUser: Send + Sync {
    fn get_listener_radius(&self) -> i32;

    fn can_receive_vibration(
        &self,
        world: &Arc<World>,
        listener_pos: &BlockPos,
        source_pos: &BlockPos,
        event: &GameEvent,
        context: &GameEventContext,
    ) -> bool;

    fn on_receive_vibration<'a>(
        &'a self,
        world: &'a Arc<World>,
        listener_pos: &'a BlockPos,
        event: &'a GameEvent,
        context: &'a GameEventContext,
        receiving_distance: f32,
        source_entity: Option<&'a Arc<dyn EntityBase>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

    fn calculate_travel_time_in_ticks(&self, distance: f32) -> i32 {
        distance.floor() as i32
    }
}

pub struct VibrationListener {
    pub position: BlockPos,
    pub data: Mutex<VibrationData>,
}

impl VibrationListener {
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            data: Mutex::new(VibrationData::new()),
        }
    }

    #[must_use]
    pub fn from_nbt(position: BlockPos, nbt: &NbtCompound) -> Self {
        Self {
            position,
            data: Mutex::new(VibrationData::from_nbt(nbt)),
        }
    }

    pub async fn handle_game_event(
        &self,
        world: &Arc<World>,
        event: GameEvent,
        context: &GameEventContext,
        source_position: &Vector3<f64>,
        user: &dyn VibrationUser,
    ) -> bool {
        let listener_center = Vector3::new(
            f64::from(self.position.0.x) + 0.5,
            f64::from(self.position.0.y) + 0.5,
            f64::from(self.position.0.z) + 0.5,
        );
        let d_sq = source_position.squared_distance_to_vec(&listener_center);
        let r = user.get_listener_radius();
        if d_sq > f64::from(r * r) {
            return false;
        }
        if self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .has_current_vibration()
        {
            return false;
        }
        if !is_valid_vibration(event, context)
            || is_occluded(world, source_position, &listener_center).await
        {
            return false;
        }
        if !user.can_receive_vibration(
            world,
            &self.position,
            &BlockPos::floored_v(*source_position),
            &event,
            context,
        ) {
            return false;
        }

        let distance = d_sq.sqrt() as f32;
        let world_tick = world.get_world_age().await;
        let mut data = self
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if data.has_current_vibration() {
            return false;
        }
        data.selector_mut().add_candidate(VibrationInfo {
            game_event: event,
            pos: *source_position,
            source_entity: context.source_entity().cloned(),
            source_entity_uuid: None,
            distance,
            tick: world_tick,
        });
        drop(data);
        if let Some(block_entity) = world.get_block_entity(&self.position) {
            world.update_block_entity(&block_entity);
        }
        true
    }
}

fn is_valid_vibration(event: GameEvent, context: &GameEventContext) -> bool {
    if event.default_frequency() == 0 {
        return false;
    }
    if let Some(entity) = context.source_entity()
        && (entity.is_spectator()
            || entity.dampens_vibrations()
            || (entity.get_entity().is_sneaking() && is_ignored_when_sneaking(event)))
    {
        return false;
    }
    context.affected_state.is_none_or(|state| {
        !state
            .to_block()
            .has_tag(&tag::Block::MINECRAFT_DAMPENS_VIBRATIONS)
    })
}

const fn is_ignored_when_sneaking(event: GameEvent) -> bool {
    matches!(
        event,
        GameEvent::HitGround
            | GameEvent::ProjectileShoot
            | GameEvent::Step
            | GameEvent::Swim
            | GameEvent::ItemInteractStart
            | GameEvent::ItemInteractFinish
    )
}

async fn is_occluded(
    world: &Arc<World>,
    source: &Vector3<f64>,
    destination: &Vector3<f64>,
) -> bool {
    let from = BlockPos::floored_v(*source).to_centered_f64();
    let to = BlockPos::floored_v(*destination).to_centered_f64();
    for direction in BlockDirection::all() {
        let offset = direction.to_offset().to_f64() * 1.0e-5;
        if world
            .raycast(from + offset, to, async |pos, world| {
                world
                    .get_block(pos)
                    .has_tag(&tag::Block::MINECRAFT_OCCLUDES_VIBRATION_SIGNALS)
            })
            .await
            .is_none()
        {
            return false;
        }
    }
    true
}

pub async fn vibration_tick(
    world: &Arc<World>,
    listener: &VibrationListener,
    user: &dyn VibrationUser,
) {
    let world_tick = world.get_world_age().await;
    let listener_chunk = listener.position.chunk_position();
    let active_chunks = world.active_chunks.load();
    let adjacent_chunks_ticking = (-1..=1).all(|x| {
        (-1..=1).all(|z| {
            let chunk = pumpkin_util::math::vector2::Vector2::new(
                listener_chunk.x + x,
                listener_chunk.y + z,
            );
            active_chunks.contains(&chunk) && world.level.is_chunk_loaded(&chunk)
        })
    });
    let (particle, vib, should_persist) = {
        let mut data = listener
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let particle = data
            .try_select_and_schedule(world_tick, user)
            .or_else(|| data.reload_particle(listener.position.to_centered_f64(), user));
        let should_persist = data.has_current_vibration();
        let vibration = if data.tick_receive() && adjacent_chunks_ticking {
            data.consume_current()
        } else {
            None
        };
        (particle, vibration, should_persist)
    };

    if should_persist && let Some(block_entity) = world.get_block_entity(&listener.position) {
        world.update_block_entity(&block_entity);
    }

    if let Some((origin, arrival_in_ticks)) = particle {
        let data = vibration_particle_data(listener.position, arrival_in_ticks);
        let packet = CParticle::new(
            false,
            false,
            origin,
            Vector3::new(0.0, 0.0, 0.0),
            0.0,
            1,
            VarInt(Particle::Vibration as i32),
            &data,
        );
        for player in world
            .players
            .load()
            .iter()
            .filter(|player| player.position().squared_distance_to_vec(&origin) <= 1024.0)
        {
            player.client.try_enqueue_packet(&packet);
        }
    }

    let Some(vib) = vib else { return };

    let source_entity = vib.source_entity.or_else(|| {
        vib.source_entity_uuid
            .and_then(|uuid| world.get_entity_by_uuid(uuid))
    });
    user.on_receive_vibration(
        world,
        &listener.position,
        &vib.game_event,
        &GameEventContext::default(),
        vib.distance,
        source_entity.as_ref(),
    )
    .await;
}

fn vibration_particle_data(destination: BlockPos, arrival_in_ticks: i32) -> Vec<u8> {
    let arrival_in_ticks = VarInt(arrival_in_ticks);
    let mut data = Vec::with_capacity(9 + arrival_in_ticks.written_size());
    data.push(0); // minecraft:block position source
    data.extend_from_slice(&destination.as_long().to_be_bytes());
    if arrival_in_ticks.encode(&mut data).is_err() {
        return Vec::new();
    }
    data
}

fn is_phase_inactive(state_id: BlockStateId) -> bool {
    let block = state_id.to_block();
    if block.id == BlockId::SCULK_SENSOR {
        let props = SculkSensorLikeProperties::from_state_id(state_id, block);
        props.sculk_sensor_phase == SculkSensorPhase::Inactive
    } else if block.id == BlockId::CALIBRATED_SCULK_SENSOR {
        let props = CalibratedSculkSensorLikeProperties::from_state_id(state_id, block);
        props.sculk_sensor_phase == SculkSensorPhase::Inactive
    } else {
        false
    }
}

pub struct SculkSensorVibrationUser {
    pub position: BlockPos,
    pub radius: i32,
}

impl SculkSensorVibrationUser {
    #[must_use]
    pub const fn new(position: BlockPos, radius: i32) -> Self {
        Self { position, radius }
    }
}

impl VibrationUser for SculkSensorVibrationUser {
    fn get_listener_radius(&self) -> i32 {
        self.radius
    }

    fn can_receive_vibration(
        &self,
        world: &Arc<World>,
        listener_pos: &BlockPos,
        source_pos: &BlockPos,
        event: &GameEvent,
        _context: &GameEventContext,
    ) -> bool {
        if source_pos == listener_pos
            && matches!(event, GameEvent::BlockPlace | GameEvent::BlockDestroy)
        {
            return false;
        }
        let state = world.get_block_state(listener_pos);
        if !is_phase_inactive(state.id) {
            return false;
        }
        true
    }
    fn on_receive_vibration<'a>(
        &'a self,
        world: &'a Arc<World>,
        listener_pos: &'a BlockPos,
        event: &'a GameEvent,
        _context: &'a GameEventContext,
        receiving_distance: f32,
        _source_entity: Option<&'a Arc<dyn EntityBase>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        use crate::block::blocks::redstone::sculk_sensor::SculkSensorBlock;

        Box::pin(async move {
            let state = world.get_block_state(listener_pos);
            if !is_phase_inactive(state.id) {
                return;
            }

            let event_frequency = event.default_frequency() as i32;
            let power = get_redstone_strength_for_distance(receiving_distance, self.radius);
            let block = state.id.to_block();

            if let Some(be) = world.get_block_entity(listener_pos) {
                use crate::block::entities::calibrated_sculk_sensor::CalibratedSculkSensorBlockEntity;
                use crate::block::entities::sculk_sensor::SculkSensorBlockEntity;
                if let Some(sensor) = be.as_any().downcast_ref::<SculkSensorBlockEntity>() {
                    sensor
                        .last_vibration_frequency
                        .store(event_frequency, Ordering::Relaxed);
                } else if let Some(sensor) = be
                    .as_any()
                    .downcast_ref::<CalibratedSculkSensorBlockEntity>()
                {
                    sensor
                        .last_vibration_frequency
                        .store(event_frequency, Ordering::Relaxed);
                }
                world.update_block_entity(&be);
            }
            SculkSensorBlock::trigger(world, listener_pos, block, power as u8).await;
        })
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::{Block, game_event::GameEvent};
    use pumpkin_util::{math::position::BlockPos, math::vector3::Vector3};
    use uuid::Uuid;

    use super::{
        GameEventContext, VibrationData, VibrationInfo, VibrationSelector,
        is_ignored_when_sneaking, is_valid_vibration, vibration_particle_data,
    };

    #[test]
    fn vibration_particle_encodes_block_destination_and_arrival() {
        assert_eq!(
            vibration_particle_data(BlockPos(Vector3::new(0, 0, 0)), 300),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0xAC, 0x02]
        );
    }

    #[test]
    fn shared_vibration_filters_use_vanilla_tags() {
        assert!(is_valid_vibration(
            GameEvent::Step,
            &GameEventContext::default()
        ));
        assert!(!is_valid_vibration(
            GameEvent::JukeboxPlay,
            &GameEventContext::default()
        ));
        assert!(!is_valid_vibration(
            GameEvent::BlockPlace,
            &GameEventContext::default().with_affected_state(Block::WHITE_WOOL.default_state.id)
        ));
        assert!(is_ignored_when_sneaking(GameEvent::ProjectileShoot));
        assert!(!is_ignored_when_sneaking(GameEvent::ProjectileLand));
    }

    #[test]
    fn vibration_data_nbt_round_trips_in_flight_state() {
        let source = Uuid::from_u128(42);
        let data = VibrationData {
            current_vibration: Some(VibrationInfo {
                game_event: GameEvent::ProjectileShoot,
                pos: Vector3::new(1.25, 2.5, 3.75),
                source_entity: None,
                source_entity_uuid: Some(source),
                distance: 6.5,
                tick: -1,
            }),
            receive_time: 4,
            selector: VibrationSelector {
                candidate: Some(VibrationInfo {
                    game_event: GameEvent::Step,
                    pos: Vector3::new(4.0, 5.0, 6.0),
                    source_entity: None,
                    source_entity_uuid: None,
                    distance: 3.0,
                    tick: 99,
                }),
            },
            reload_particle: false,
        };

        let restored = VibrationData::from_nbt(&data.to_nbt());
        let current = restored.current_vibration.as_ref().unwrap();
        let candidate = restored.selector.candidate.as_ref().unwrap();
        assert_eq!(current.game_event, GameEvent::ProjectileShoot);
        assert_eq!(current.pos, Vector3::new(1.25, 2.5, 3.75));
        assert_eq!(current.source_entity_uuid, Some(source));
        assert_eq!(restored.receive_time, 4);
        assert!(restored.reload_particle);
        assert_eq!(candidate.game_event, GameEvent::Step);
        assert_eq!(candidate.tick, 99);
    }
}
