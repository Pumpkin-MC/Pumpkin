use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerLoginStartEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a direct Java connection sends Login Start, before the
/// server decides whether to authenticate it with Mojang.
///
/// Set `online_mode` in [`PlayerLoginStartEventData`] to choose, for this connection
/// only, whether the player is authenticated with Mojang and gets their online UUID.
pub struct PlayerLoginStartEvent;
impl FromIntoEvent for PlayerLoginStartEvent {
    const EVENT_TYPE: EventType = EventType::PlayerLoginStartEvent;
    type Data = PlayerLoginStartEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerLoginStartEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerLoginStartEvent(data)
    }
}
