use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerCommandSendEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player sends a command.
///
/// The associated [`PlayerCommandSendEventData`] contains the player and the command
/// string (without the leading `/`). This event is cancellable.
pub struct PlayerCommandSendEvent;
impl FromIntoEvent for PlayerCommandSendEvent {
    const EVENT_TYPE: EventType = EventType::PlayerCommandSendEvent;
    type Data = PlayerCommandSendEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerCommandSendEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerCommandSendEvent(data)
    }
}
