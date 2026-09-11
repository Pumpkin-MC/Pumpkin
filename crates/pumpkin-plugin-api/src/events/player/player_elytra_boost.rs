use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerElytraBoostEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player boosts their elytra flight using a firework.
pub struct PlayerElytraBoostEvent;
impl FromIntoEvent for PlayerElytraBoostEvent {
    const EVENT_TYPE: EventType = EventType::PlayerElytraBoostEvent;
    type Data = PlayerElytraBoostEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerElytraBoostEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerElytraBoostEvent(data)
    }
}
