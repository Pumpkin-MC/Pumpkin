use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerRecipeDiscoverEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player discovers a recipe.
pub struct PlayerRecipeDiscoverEvent;
impl FromIntoEvent for PlayerRecipeDiscoverEvent {
    const EVENT_TYPE: EventType = EventType::PlayerRecipeDiscoverEvent;
    type Data = PlayerRecipeDiscoverEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerRecipeDiscoverEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerRecipeDiscoverEvent(data)
    }
}
