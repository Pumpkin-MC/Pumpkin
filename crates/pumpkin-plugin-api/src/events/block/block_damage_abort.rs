use crate::wit::pumpkin::plugin::event::{BlockDamageAbortEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a player stops damaging a block.
pub struct BlockDamageAbortEvent;
impl FromIntoEvent for BlockDamageAbortEvent {
    const EVENT_TYPE: EventType = EventType::BlockDamageAbortEvent;
    type Data = BlockDamageAbortEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockDamageAbortEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockDamageAbortEvent(data)
    }
}
