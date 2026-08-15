use crate::wit::pumpkin::plugin::event::{Event, EventType, RaidSpawnWaveEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a wave of a raid spawns.
pub struct RaidSpawnWaveEvent;
impl FromIntoEvent for RaidSpawnWaveEvent {
    const EVENT_TYPE: EventType = EventType::RaidSpawnWaveEvent;
    type Data = RaidSpawnWaveEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::RaidSpawnWaveEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::RaidSpawnWaveEvent(data)
    }
}
