use crate::wit::pumpkin::plugin::event::{Event, EventType, VillagerAcquireTradeEventData};

use super::super::FromIntoEvent;

/// Event triggered when a villager acquires a new trade.
pub struct VillagerAcquireTradeEvent;
impl FromIntoEvent for VillagerAcquireTradeEvent {
    const EVENT_TYPE: EventType = EventType::VillagerAcquireTradeEvent;
    type Data = VillagerAcquireTradeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VillagerAcquireTradeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VillagerAcquireTradeEvent(data)
    }
}
