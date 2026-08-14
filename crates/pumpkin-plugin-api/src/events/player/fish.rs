use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerFishEventData};

use super::super::FromIntoEvent;

/// An event that occurs during a fishing action.
///
/// The associated [`PlayerFishEventData`] contains the player, the fishing state,
/// the hand used, the hook entity, optionally the caught entity, and the experience
/// to drop. This event is cancellable.
pub struct PlayerFishEvent;
impl FromIntoEvent for PlayerFishEvent {
    const EVENT_TYPE: EventType = EventType::PlayerFishEvent;
    type Data = PlayerFishEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerFishEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerFishEvent(data)
    }
}
