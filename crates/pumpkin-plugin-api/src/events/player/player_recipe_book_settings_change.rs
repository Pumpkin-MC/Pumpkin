use crate::wit::pumpkin::plugin::event::{
    Event, EventType, PlayerRecipeBookSettingsChangeEventData,
};

use super::super::FromIntoEvent;

/// An event that occurs when a player changes recipe book settings.
pub struct PlayerRecipeBookSettingsChangeEvent;
impl FromIntoEvent for PlayerRecipeBookSettingsChangeEvent {
    const EVENT_TYPE: EventType = EventType::PlayerRecipeBookSettingsChangeEvent;
    type Data = PlayerRecipeBookSettingsChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerRecipeBookSettingsChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerRecipeBookSettingsChangeEvent(data)
    }
}
