use crate::wit::pumpkin::plugin::event::{BlockDispenseArmorEventData, Event, EventType};

use super::super::FromIntoEvent;

/// An event that occurs when a dispenser equips armor on an entity.
pub struct BlockDispenseArmorEvent;
impl FromIntoEvent for BlockDispenseArmorEvent {
    const EVENT_TYPE: EventType = EventType::BlockDispenseArmorEvent;
    type Data = BlockDispenseArmorEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockDispenseArmorEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockDispenseArmorEvent(data)
    }
}
