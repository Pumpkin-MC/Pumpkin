use crate::wit::pumpkin::plugin::event::{Event, EventType, PlayerPermissionCheckEventData};

use super::super::FromIntoEvent;

/// An event that occurs when a permission check is performed for a player.
///
/// The associated [`PlayerPermissionCheckEventData`] contains the player, the permission
/// node being checked, and the current result which can be overridden.
pub struct PlayerPermissionCheckEvent;
impl FromIntoEvent for PlayerPermissionCheckEvent {
    const EVENT_TYPE: EventType = EventType::PlayerPermissionCheckEvent;
    type Data = PlayerPermissionCheckEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerPermissionCheckEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerPermissionCheckEvent(data)
    }
}
