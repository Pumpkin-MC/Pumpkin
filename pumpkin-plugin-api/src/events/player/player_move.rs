use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerMoveEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player moves.
///
/// The associated [`PlayerMoveEventData`] contains the player, the position moved from,
/// and the position moved to. This event is cancellable.
pub struct PlayerMoveEvent;
impl FromIntoEvent for PlayerMoveEvent {
    const EVENT_TYPE: EventType = EventType::PlayerMoveEvent;
    type Data = PlayerMoveEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerMoveEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerMoveEvent(data)
    }
}
