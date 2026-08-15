use crate::wit::pumpkin::plugin::event::{EntitySpellCastEventData, Event, EventType};

use super::super::FromIntoEvent;

/// Event triggered when a spellcasting entity casts a spell.
pub struct EntitySpellCastEvent;
impl FromIntoEvent for EntitySpellCastEvent {
    const EVENT_TYPE: EventType = EventType::EntitySpellCastEvent;
    type Data = EntitySpellCastEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::EntitySpellCastEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::EntitySpellCastEvent(data)
    }
}
