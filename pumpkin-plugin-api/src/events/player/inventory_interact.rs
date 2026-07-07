use crate::wit::pumpkin::plugin::event::{Event, EventType, InventoryClickEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a player clicks in an inventory.
///
/// The associated [`InventoryClickEventData`] contains the player, window type,
/// click type, slot information, cursor/clicked items, and hotbar button. This
/// event is cancellable.
pub struct InventoryClickEvent;

impl FromIntoEvent for InventoryClickEvent {
    const EVENT_TYPE: EventType = EventType::InventoryClickEvent;
    type Data = InventoryClickEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::InventoryClickEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::InventoryClickEvent(data)
    }
}
