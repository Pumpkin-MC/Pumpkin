use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerGamemodeChangeEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player's game mode changes.
///
/// The associated [`PlayerGamemodeChangeEventData`] contains the player, the previous
/// game mode, and the new game mode. This event is cancellable.
pub struct PlayerGamemodeChangeEvent;
impl FromIntoEvent for PlayerGamemodeChangeEvent {
    const EVENT_TYPE: EventType = EventType::PlayerGamemodeChangeEvent;
    type Data = PlayerGamemodeChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerGamemodeChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerGamemodeChangeEvent(data)
    }
}
