use crate::wit::pumpkin::plugin::event::{Event, EventType, PluginLoadEventData};

use super::super::FromIntoEvent;

/// An event that fires after a plugin has been successfully loaded.
///
/// Contains the loaded plugin's name and version. This event is fired
/// after the plugin's `on_load` hook has completed and cannot be cancelled.
pub struct PluginLoadEvent;
impl FromIntoEvent for PluginLoadEvent {
    const EVENT_TYPE: EventType = EventType::PluginLoadEvent;
    type Data = PluginLoadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PluginLoadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PluginLoadEvent(data)
    }
}
