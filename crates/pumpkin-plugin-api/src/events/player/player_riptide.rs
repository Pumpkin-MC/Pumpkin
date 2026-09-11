use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerRiptideEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player activates the riptide enchantment.
pub struct PlayerRiptideEvent;
impl FromIntoEvent for PlayerRiptideEvent {
    const EVENT_TYPE: EventType = EventType::PlayerRiptideEvent;
    type Data = PlayerRiptideEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerRiptideEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerRiptideEvent(data)
    }
}
