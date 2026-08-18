use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, LightningStrikeEventData};

/// Event triggered when a lightning strikes in a world.
pub struct LightningStrikeEvent;
impl FromIntoEvent for LightningStrikeEvent {
    const EVENT_TYPE: EventType = EventType::LightningStrikeEvent;
    type Data = LightningStrikeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::LightningStrikeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::LightningStrikeEvent(data)
    }
}
