use crate::wit::pumpkin::plugin::event::{
    EntityDeathEventData, Event, EventType, PlayerDeathEventData,
};

use super::super::FromIntoEvent;

/// Event triggered when an entity dies.
pub struct EntityDeathEvent;
impl FromIntoEvent for EntityDeathEvent {
    const EVENT_TYPE: EventType = EventType::EntityDeathEvent;
    type Data = EntityDeathEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntityDeathEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntityDeathEvent(data)
    }
}

/// Event triggered when a player dies.
pub struct PlayerDeathEvent;
impl FromIntoEvent for PlayerDeathEvent {
    const EVENT_TYPE: EventType = EventType::PlayerDeathEvent;
    type Data = PlayerDeathEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerDeathEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerDeathEvent(data)
    }
}
