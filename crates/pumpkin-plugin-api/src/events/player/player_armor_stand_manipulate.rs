use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerArmorStandManipulateEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player manipulates an armor stand.
pub struct PlayerArmorStandManipulateEvent;
impl FromIntoEvent for PlayerArmorStandManipulateEvent {
    const EVENT_TYPE: EventType = EventType::PlayerArmorStandManipulateEvent;
    type Data = PlayerArmorStandManipulateEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerArmorStandManipulateEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerArmorStandManipulateEvent(data)
    }
}
