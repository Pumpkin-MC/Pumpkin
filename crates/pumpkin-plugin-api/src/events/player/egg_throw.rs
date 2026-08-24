use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerEggThrowEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a thrown egg resolves.
///
/// The associated [`PlayerEggThrowEventData`] contains the player, the egg entity UUID,
/// whether the egg hatched, how many entities hatched, and the entity type to hatch.
/// This event is cancellable.
pub struct PlayerEggThrowEvent;
impl FromIntoEvent for PlayerEggThrowEvent {
    const EVENT_TYPE: EventType = EventType::PlayerEggThrowEvent;
    type Data = PlayerEggThrowEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerEggThrowEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerEggThrowEvent(data)
    }
}
