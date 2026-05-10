use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerChangeWorldEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player changes worlds.
///
/// The associated [`PlayerChangeWorldEventData`] contains the player, the previous world,
/// the new world, and the destination position, yaw, and pitch. This event is cancellable.
pub struct PlayerChangeWorldEvent;
impl FromIntoEvent for PlayerChangeWorldEvent {
    const EVENT_TYPE: EventType = EventType::PlayerChangeWorldEvent;
    type Data = PlayerChangeWorldEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerChangeWorldEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerChangeWorldEvent(data)
    }
}
