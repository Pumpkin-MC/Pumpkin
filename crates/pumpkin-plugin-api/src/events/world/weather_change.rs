use super::super::FromIntoEvent;
use crate::wit::pumpkin::plugin::event::{
    Event, EventType, ThunderChangeEventData, WeatherChangeEventData,
};

/// Event triggered when world weather changes.
pub struct WeatherChangeEvent;
impl FromIntoEvent for WeatherChangeEvent {
    const EVENT_TYPE: EventType = EventType::WeatherChangeEvent;
    type Data = WeatherChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::WeatherChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::WeatherChangeEvent(data)
    }
}

/// Event triggered when thunderstorm state changes.
pub struct ThunderChangeEvent;
impl FromIntoEvent for ThunderChangeEvent {
    const EVENT_TYPE: EventType = EventType::ThunderChangeEvent;
    type Data = ThunderChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ThunderChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ThunderChangeEvent(data)
    }
}
