use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{
    Event, EventType, WorldLoadEventData, WorldUnloadEventData,
};

/// Event triggered when a world is loaded.
pub struct WorldLoadEvent;
impl FromIntoEvent for WorldLoadEvent {
    const EVENT_TYPE: EventType = EventType::WorldLoadEvent;
    type Data = WorldLoadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::WorldLoadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::WorldLoadEvent(data)
    }
}

/// Event triggered when a world is unloaded.
pub struct WorldUnloadEvent;
impl FromIntoEvent for WorldUnloadEvent {
    const EVENT_TYPE: EventType = EventType::WorldUnloadEvent;
    type Data = WorldUnloadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::WorldUnloadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::WorldUnloadEvent(data)
    }
}
