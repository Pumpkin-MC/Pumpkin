use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{
    BlockPistonExtendEventData, BlockPistonRetractEventData, Event, EventType,
};

/// An event that occurs when a piston extends.
pub struct BlockPistonExtendEvent;
impl FromIntoEvent for BlockPistonExtendEvent {
    const EVENT_TYPE: EventType = EventType::BlockPistonExtendEvent;
    type Data = BlockPistonExtendEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockPistonExtendEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockPistonExtendEvent(data)
    }
}

/// An event that occurs when a piston retracts.
pub struct BlockPistonRetractEvent;
impl FromIntoEvent for BlockPistonRetractEvent {
    const EVENT_TYPE: EventType = EventType::BlockPistonRetractEvent;
    type Data = BlockPistonRetractEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockPistonRetractEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockPistonRetractEvent(data)
    }
}
