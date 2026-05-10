use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerItemHeldEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player changes the selected hotbar slot.
///
/// The associated [`PlayerItemHeldEventData`] contains the player, the previous slot index,
/// and the new slot index. This event is cancellable.
pub struct PlayerItemHeldEvent;
impl FromIntoEvent for PlayerItemHeldEvent {
    const EVENT_TYPE: EventType = EventType::PlayerItemHeldEvent;
    type Data = PlayerItemHeldEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerItemHeldEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerItemHeldEvent(data)
    }
}
