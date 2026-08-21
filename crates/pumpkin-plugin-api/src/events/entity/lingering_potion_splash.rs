use crate::wit::pumpkin::plugin::event::{Event, EventType, LingeringPotionSplashEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a lingering potion splashes.
pub struct LingeringPotionSplashEvent;
impl FromIntoEvent for LingeringPotionSplashEvent {
    const EVENT_TYPE: EventType = EventType::LingeringPotionSplashEvent;
    type Data = LingeringPotionSplashEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::LingeringPotionSplashEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::LingeringPotionSplashEvent(data)
    }
}
