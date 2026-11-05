use super::ParrotEntity;
use crate::{
    entity::EntityBase,
    world::{
        World,
        game_event::{GameEventContext, GameEventListener},
    },
};
use pumpkin_data::game_event::GameEvent;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use std::sync::atomic::Ordering::Relaxed;

pub(super) struct JukeboxListener<'a>(pub(super) &'a ParrotEntity);

impl GameEventListener for JukeboxListener<'_> {
    fn listener_position(&self) -> Vector3<f64> {
        self.0.get_entity().pos.load()
    }
    fn listener_radius(&self) -> i32 {
        GameEvent::JukeboxPlay.notification_radius()
    }
    fn handle_game_event(
        &self,
        _world: &World,
        event: GameEvent,
        _context: &GameEventContext<'_>,
        source: Vector3<f64>,
    ) -> bool {
        let playing = match event {
            GameEvent::JukeboxPlay => true,
            GameEvent::JukeboxStopPlay => false,
            _ => return false,
        };
        self.0.jukebox.store(Some(BlockPos::floored_v(source)));
        self.0.party.store(playing, Relaxed);
        true
    }
}
