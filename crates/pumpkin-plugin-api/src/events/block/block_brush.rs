use crate::wit::pumpkin::plugin::event::{BlockBrushEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a player brushes a block.
pub struct BlockBrushEvent;
impl FromIntoEvent for BlockBrushEvent {
    const EVENT_TYPE: EventType = EventType::BlockBrushEvent;
    type Data = BlockBrushEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockBrushEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockBrushEvent(data)
    }
}
