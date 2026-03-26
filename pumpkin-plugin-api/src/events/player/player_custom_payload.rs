use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerCustomPayloadEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player sends a custom plugin channel payload.
///
/// The associated [`PlayerCustomPayloadEventData`] contains the player, the channel
/// identifier, and the raw payload bytes.
pub struct PlayerCustomPayloadEvent;
impl FromIntoEvent for PlayerCustomPayloadEvent {
    const EVENT_TYPE: EventType = EventType::PlayerCustomPayloadEvent;
    type Data = PlayerCustomPayloadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerCustomPayloadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerCustomPayloadEvent(data)
    }
}
