use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerItemDamageEventData};

/// Event triggered when a player's held item receives damage.
pub struct PlayerItemDamageEvent;
impl FromIntoEvent for PlayerItemDamageEvent {
    const EVENT_TYPE: EventType = EventType::PlayerItemDamageEvent;
    type Data = PlayerItemDamageEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerItemDamageEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerItemDamageEvent(data)
    }
}
