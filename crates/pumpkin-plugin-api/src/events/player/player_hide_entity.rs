use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerHideEntityEventData};

use super::super::FromIntoEvent;

/// An event that occurs when an entity is hidden from a player.
pub struct PlayerHideEntityEvent;
impl FromIntoEvent for PlayerHideEntityEvent {
    const EVENT_TYPE: EventType = EventType::PlayerHideEntityEvent;
    type Data = PlayerHideEntityEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerHideEntityEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerHideEntityEvent(data)
    }
}
