use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerChatEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player sends a chat message.
///
/// The associated [`PlayerChatEventData`] contains the player, the message, and
/// the list of recipients. The message can be modified. This event is cancellable.
pub struct PlayerChatEvent;
impl FromIntoEvent for PlayerChatEvent {
    const EVENT_TYPE: EventType = EventType::PlayerChatEvent;
    type Data = PlayerChatEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerChatEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerChatEvent(data)
    }
}
