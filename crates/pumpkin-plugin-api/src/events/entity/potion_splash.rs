use crate::wit::pumpkin::plugin::event::{Event, EventType, PotionSplashEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a potion splashes.
pub struct PotionSplashEvent;
impl FromIntoEvent for PotionSplashEvent {
    const EVENT_TYPE: EventType = EventType::PotionSplashEvent;
    type Data = PotionSplashEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PotionSplashEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PotionSplashEvent(data)
    }
}
