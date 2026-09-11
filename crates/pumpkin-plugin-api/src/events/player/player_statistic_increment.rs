use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerStatisticIncrementEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player's statistic is incremented.
pub struct PlayerStatisticIncrementEvent;
impl FromIntoEvent for PlayerStatisticIncrementEvent {
    const EVENT_TYPE: EventType = EventType::PlayerStatisticIncrementEvent;
    type Data = PlayerStatisticIncrementEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerStatisticIncrementEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerStatisticIncrementEvent(data)
    }
}
