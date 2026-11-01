use pumpkin_data::{
    BlockDirection,
    game_event::{GameEvent, PositionSourceType},
    particle::Particle,
    tag::{self, Taggable},
};
use pumpkin_nbt::{compound::NbtCompound, tag::NbtTag};
use pumpkin_protocol::{codec::var_int::VarInt, java::client::play::CParticle, ser::WritingError};
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use uuid::Uuid;

use super::{AllayEntity, ai};
use crate::entity::EntityBase;
use crate::world::{
    World,
    game_event::{GameEventContext, GameEventListener},
};

const VIBRATION_EVENT_LISTENER_RANGE: i32 = 16;
const PARTICLE_SEND_DISTANCE: f64 = 32.0;

#[derive(Clone)]
struct VibrationInfo {
    event: GameEvent,
    distance: f32,
    pos: Vector3<f64>,
    source: Option<Uuid>,
    projectile_owner: Option<Uuid>,
}

impl VibrationInfo {
    fn read_nbt(nbt: &NbtCompound) -> Option<Self> {
        let event = GameEvent::from_name(nbt.get_string("game_event")?)?;
        let distance = nbt.get_float("distance")?;
        let pos = nbt.get_list("pos")?;
        if pos.len() != 3 || !distance.is_finite() || distance < 0.0 {
            return None;
        }
        let pos = Vector3::new(
            pos[0].extract_double()?,
            pos[1].extract_double()?,
            pos[2].extract_double()?,
        );
        if !pos.x.is_finite() || !pos.y.is_finite() || !pos.z.is_finite() {
            return None;
        }
        Some(Self {
            event,
            distance,
            pos,
            source: nbt.get_uuid("source"),
            projectile_owner: nbt.get_uuid("projectile_owner"),
        })
    }

    fn write_nbt(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.put_string("game_event", format!("minecraft:{}", self.event.name()));
        nbt.put_float("distance", self.distance);
        nbt.put_list(
            "pos",
            vec![
                NbtTag::Double(self.pos.x),
                NbtTag::Double(self.pos.y),
                NbtTag::Double(self.pos.z),
            ],
        );
        if let Some(source) = self.source {
            nbt.put_uuid("source", source);
        }
        if let Some(owner) = self.projectile_owner {
            nbt.put_uuid("projectile_owner", owner);
        }
        nbt
    }
}

#[derive(Default)]
pub(super) struct VibrationData {
    current: Option<VibrationInfo>,
    candidate: Option<(VibrationInfo, i64)>,
    delay: i32,
    reload_particle: bool,
}

impl VibrationData {
    pub(super) fn read_nbt(nbt: &NbtCompound) -> Self {
        let Some(selector) = nbt.get_compound("selector") else {
            return Self::default();
        };
        let Some(tick) = selector.get_long("tick") else {
            return Self::default();
        };
        let delay = nbt.get_int("event_delay").unwrap_or(0);
        if delay < 0 {
            return Self::default();
        }
        Self {
            current: nbt.get_compound("event").and_then(VibrationInfo::read_nbt),
            candidate: selector
                .get_compound("event")
                .and_then(VibrationInfo::read_nbt)
                .map(|event| (event, tick)),
            delay,
            reload_particle: true,
        }
    }

    pub(super) fn write_nbt(&self) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        if let Some(event) = &self.current {
            nbt.put_compound("event", event.write_nbt());
        }
        let mut selector = NbtCompound::new();
        selector.put_long(
            "tick",
            self.candidate.as_ref().map_or(-1, |(_, tick)| *tick),
        );
        if let Some((event, _)) = &self.candidate {
            selector.put_compound("event", event.write_nbt());
        }
        nbt.put_compound("selector", selector);
        nbt.put_int("event_delay", self.delay);
        nbt
    }
}

pub(super) struct JukeboxListener<'a>(pub(super) &'a AllayEntity);

impl GameEventListener for JukeboxListener<'_> {
    fn listener_position(&self) -> Vector3<f64> {
        self.0.get_eye_pos()
    }

    fn listener_radius(&self) -> i32 {
        GameEvent::JukeboxPlay.notification_radius()
    }

    fn handle_game_event(
        &self,
        _world: &World,
        event: GameEvent,
        _context: &GameEventContext<'_>,
        source_position: Vector3<f64>,
    ) -> bool {
        let playing = match event {
            GameEvent::JukeboxPlay => true,
            GameEvent::JukeboxStopPlay => false,
            _ => return false,
        };
        self.0
            .set_jukebox_playing(BlockPos::floored_v(source_position), playing);
        true
    }
}

pub(super) struct VibrationListener<'a>(pub(super) &'a AllayEntity);

impl GameEventListener for VibrationListener<'_> {
    fn listener_position(&self) -> Vector3<f64> {
        self.0.get_eye_pos()
    }

    fn listener_radius(&self) -> i32 {
        VIBRATION_EVENT_LISTENER_RANGE
    }

    fn handle_game_event(
        &self,
        world: &World,
        event: GameEvent,
        context: &GameEventContext<'_>,
        source_position: Vector3<f64>,
    ) -> bool {
        let allay = self.0;
        if allay
            .vibration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .current
            .is_some()
            || !is_valid_vibration(event, context)
        {
            return false;
        }
        let destination = allay.get_eye_pos();
        if !allay.can_receive_vibration(world, BlockPos::floored_v(source_position))
            || is_occluded(world, source_position, destination)
        {
            return false;
        }
        let time = world.get_world_age();
        let distance = (source_position - destination).length() as f32;
        let mut data = allay
            .vibration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if data
            .candidate
            .as_ref()
            .is_none_or(|(previous, tick)| *tick == time && distance < previous.distance)
        {
            data.candidate = Some((
                VibrationInfo {
                    event,
                    distance,
                    pos: source_position,
                    source: context
                        .source_entity
                        .map(|source| source.get_entity().entity_uuid),
                    projectile_owner: None,
                },
                time,
            ));
        }
        true
    }
}

fn is_valid_vibration(event: GameEvent, context: &GameEventContext<'_>) -> bool {
    if !tag::GameEvent::MINECRAFT_ALLAY_CAN_LISTEN
        .0
        .contains(&event.name())
    {
        return false;
    }
    let Some(source) = context.source_entity else {
        return true;
    };
    let ignored_while_sneaking = tag::GameEvent::MINECRAFT_IGNORE_VIBRATIONS_SNEAKING
        .0
        .contains(&event.name());
    !source.is_spectator()
        && (!source.get_entity().is_sneaking() || !ignored_while_sneaking)
        && !source.dampens_vibrations()
}

fn is_occluded(world: &World, origin: Vector3<f64>, destination: Vector3<f64>) -> bool {
    let from = BlockPos::floored_v(origin).to_centered_f64();
    let to = BlockPos::floored_v(destination).to_centered_f64();
    BlockDirection::all().iter().all(|direction| {
        let nudged_source = from + direction.to_offset().to_f64() * f64::from(1.0e-5f32);
        pumpkin_world::clip::traverse_blocks(nudged_source, to, |pos| {
            world
                .get_block(pos)
                .has_tag(&tag::Block::MINECRAFT_OCCLUDES_VIBRATION_SIGNALS)
                .then_some(())
        })
        .is_some()
    })
}

impl AllayEntity {
    fn can_receive_vibration(&self, world: &World, pos: BlockPos) -> bool {
        if self.mob_entity.is_no_ai() {
            return false;
        }
        let entity = self.get_entity();
        self.mob_entity.with_brain(self, |brain| {
            brain
                .get(crate::entity::ai::brain::memory::types::LIKED_NOTEBLOCK_POSITION)
                .is_none_or(|liked| {
                    liked.is_close_enough(
                        &world.dimension,
                        entity.block_pos.load(),
                        ai::MAX_NOTEBLOCK_DISTANCE,
                    ) && liked.pos == pos
                })
        })
    }

    pub(super) fn tick_vibration(&self) {
        let world = self.get_entity().world.load();
        let mut data = self
            .vibration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if data.current.is_none()
            && data
                .candidate
                .as_ref()
                .is_some_and(|(_, time)| *time < world.get_world_age())
            && let Some((event, _)) = data.candidate.take()
        {
            data.delay = event.distance.floor() as i32;
            self.send_vibration_particle(event.pos, data.delay);
            data.current = Some(event);
        }
        if data.reload_particle {
            if let Some(event) = &data.current {
                let initial = event.distance.floor() as i32;
                let alpha = 1.0 - f64::from(data.delay) / f64::from(initial);
                let pos = event.pos.lerp(&self.get_eye_pos(), alpha);
                if self.send_vibration_particle(pos, data.delay) {
                    data.reload_particle = false;
                }
            } else {
                data.reload_particle = false;
            }
        }
        if data.current.is_none() {
            return;
        }
        data.delay = (data.delay - 1).max(0);
        if data.delay == 0 {
            let event = data.current.take();
            drop(data);
            if let Some(event) = event.filter(|event| event.event == GameEvent::NoteBlockPlay) {
                ai::hear_noteblock(self, BlockPos::floored_v(event.pos));
            }
        }
    }

    fn send_vibration_particle(&self, pos: Vector3<f64>, delay: i32) -> bool {
        let Ok(payload) = vibration_payload(
            self.get_entity().entity_id,
            self.get_entity().get_eye_height() as f32,
            delay,
        ) else {
            return false;
        };
        let packet = CParticle::new(
            false,
            false,
            pos,
            Vector3::new(0.0, 0.0, 0.0),
            0.0,
            1,
            VarInt(i32::from(Particle::Vibration.to_id())),
            &payload,
        );
        let world = self.get_entity().world.load();
        let mut sent = false;
        for player in world.players.load().iter() {
            if player.position().squared_distance_to_vec(&pos)
                <= PARTICLE_SEND_DISTANCE * PARTICLE_SEND_DISTANCE
            {
                player.try_send_client_packet(&packet);
                sent = true;
            }
        }
        sent
    }
}

fn vibration_payload(entity_id: i32, eye_offset: f32, delay: i32) -> Result<Vec<u8>, WritingError> {
    let mut payload = Vec::new();
    VarInt(PositionSourceType::Entity.to_id()).encode(&mut payload)?;
    VarInt(entity_id).encode(&mut payload)?;
    payload.extend_from_slice(&eye_offset.to_be_bytes());
    VarInt(delay).encode(&mut payload)?;
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{
        Entity,
        ai::{brain::memory::types, test_support::world},
    };
    use pumpkin_data::entity::EntityType;

    #[test]
    fn entity_destination_particle_matches_vanilla_stream_codec() -> Result<(), WritingError> {
        assert_eq!(
            vibration_payload(300, 0.36, 12)?,
            vec![1, 172, 2, 62, 184, 81, 236, 12]
        );
        Ok(())
    }

    #[tokio::test]
    async fn wool_occludes_notes_but_a_solid_stone_block_does_not()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        world.level.loaded_chunks.insert(
            pumpkin_util::math::vector2::Vector2::new(0, 0),
            pumpkin_world::chunk::ChunkData::empty_sync(0, 0),
        );
        let allay = AllayEntity::new(Entity::new(
            world.clone(),
            Vector3::new(6.5, 64.0, 2.5),
            &EntityType::ALLAY,
        ));
        let blocker = BlockPos::new(4, 64, 2);
        let origin = BlockPos::new(2, 64, 2).to_centered_f64();
        world.set_block_state(
            &blocker,
            pumpkin_data::Block::STONE.default_state.id,
            pumpkin_world::world::BlockFlags::empty(),
        );
        VibrationListener(&allay).handle_game_event(
            &world,
            GameEvent::NoteBlockPlay,
            &GameEventContext {
                source_entity: None,
            },
            origin,
        );
        assert!(
            allay
                .vibration
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .candidate
                .take()
                .is_some()
        );
        world.set_block_state(
            &blocker,
            pumpkin_data::Block::WHITE_WOOL.default_state.id,
            pumpkin_world::world::BlockFlags::empty(),
        );
        VibrationListener(&allay).handle_game_event(
            &world,
            GameEvent::NoteBlockPlay,
            &GameEventContext {
                source_entity: None,
            },
            origin,
        );
        assert!(
            allay
                .vibration
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .candidate
                .is_none()
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn note_delivery_waits_for_travel_and_survives_reload()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let allay = AllayEntity::new(Entity::new(
            world.clone(),
            Vector3::new(3.5, 64.0, 0.5),
            &EntityType::ALLAY,
        ));
        let origin = BlockPos::new(0, 64, 0);
        VibrationListener(&allay).handle_game_event(
            &world,
            GameEvent::NoteBlockPlay,
            &GameEventContext {
                source_entity: None,
            },
            origin.to_centered_f64(),
        );
        allay.tick_vibration();
        assert!(
            allay
                .vibration
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .current
                .is_none()
        );
        world
            .level_time
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .world_age = 1;
        allay.tick_vibration();
        let nbt = allay
            .vibration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .write_nbt();
        assert_eq!(nbt.get_int("event_delay"), Some(2));
        assert!(!allay.mob_entity.with_brain(allay.as_ref(), |brain| {
            brain.has_memory_value(types::LIKED_NOTEBLOCK_POSITION.id())
        }));
        *allay
            .vibration
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = VibrationData::read_nbt(&nbt);
        allay.tick_vibration();
        assert!(!allay.mob_entity.with_brain(allay.as_ref(), |brain| {
            brain.has_memory_value(types::LIKED_NOTEBLOCK_POSITION.id())
        }));
        allay.tick_vibration();
        assert_eq!(
            allay.mob_entity.with_brain(allay.as_ref(), |brain| brain
                .get(types::LIKED_NOTEBLOCK_POSITION)
                .map(|liked| liked.pos)),
            Some(origin)
        );
        world.level.shutdown().await;
        Ok(())
    }
}
