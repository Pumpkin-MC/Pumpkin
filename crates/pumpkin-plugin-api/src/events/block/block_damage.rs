use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{BlockDamageEventData, Event, EventType};

/// Event triggered when a block is damaged by a player.
pub struct BlockDamageEvent;
impl FromIntoEvent for BlockDamageEvent {
    const EVENT_TYPE: EventType = EventType::BlockDamageEvent;
    type Data = BlockDamageEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockDamageEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockDamageEvent(data)
    }
}
