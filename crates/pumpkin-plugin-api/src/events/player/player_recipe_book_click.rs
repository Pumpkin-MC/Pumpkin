use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerRecipeBookClickEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player clicks a recipe in the recipe book.
pub struct PlayerRecipeBookClickEvent;
impl FromIntoEvent for PlayerRecipeBookClickEvent {
    const EVENT_TYPE: EventType = EventType::PlayerRecipeBookClickEvent;
    type Data = PlayerRecipeBookClickEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerRecipeBookClickEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerRecipeBookClickEvent(data)
    }
}
