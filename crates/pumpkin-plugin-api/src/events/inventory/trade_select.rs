use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{Event, EventType, TradeSelectEventData};

/// Event triggered when a player selects a merchant trade option.
pub struct TradeSelectEvent;
impl FromIntoEvent for TradeSelectEvent {
    const EVENT_TYPE: EventType = EventType::TradeSelectEvent;
    type Data = TradeSelectEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::TradeSelectEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::TradeSelectEvent(data)
    }
}
