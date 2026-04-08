use crate::wit::pumpkin::plugin::event::{BlockBurnEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a block is destroyed by fire.
///
/// The associated [`BlockBurnEventData`] contains the block that caught fire and
/// the igniting block that caused it to burn. This event is cancellable.
pub struct BlockBurnEvent;
impl FromIntoEvent for BlockBurnEvent {
    const EVENT_TYPE: EventType = EventType::BlockBurnEvent;
    type Data = BlockBurnEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockBurnEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockBurnEvent(data)
    }
}
