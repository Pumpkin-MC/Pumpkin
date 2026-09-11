use crate::wit::pumpkin::plugin::event::{Event, EventType, VillagerReplenishTradeEventData};

use super::super::FromIntoEvent;

/// Event triggered when a villager replenishes its trades.
pub struct VillagerReplenishTradeEvent;
impl FromIntoEvent for VillagerReplenishTradeEvent {
    const EVENT_TYPE: EventType = EventType::VillagerReplenishTradeEvent;
    type Data = VillagerReplenishTradeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::VillagerReplenishTradeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::VillagerReplenishTradeEvent(data)
    }
}
