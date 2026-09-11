use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerHarvestBlockEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player harvests a block.
pub struct PlayerHarvestBlockEvent;
impl FromIntoEvent for PlayerHarvestBlockEvent {
    const EVENT_TYPE: EventType = EventType::PlayerHarvestBlockEvent;
    type Data = PlayerHarvestBlockEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerHarvestBlockEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerHarvestBlockEvent(data)
    }
}
