use super::{EventHandler, FromIntoEvent, PluginEvent};
use crate::Server;
use crate::wit::pumpkin::plugin::event::{
    ConnectionPacketReceivedEventData, ConnectionPacketSentEventData, Event, EventType,
    PacketReceivedEventData, PacketSentEventData,
};

/// An event fired when a packet is received from a client
pub struct PacketReceivedEvent;

impl PluginEvent for PacketReceivedEvent {
    const EVENT_TYPE: EventType = EventType::PacketReceivedEvent;
    type Data = PacketReceivedEventData;

    fn route_packet_received<H: EventHandler<Self>>(
        handler: &H,
        server: Server,
        event: PacketReceivedEventData,
    ) -> PacketReceivedEventData {
        handler.handle(server, event)
    }
}

/// An event fired when a packet is sent to a client
pub struct PacketSentEvent;

impl PluginEvent for PacketSentEvent {
    const EVENT_TYPE: EventType = EventType::PacketSentEvent;
    type Data = PacketSentEventData;

    fn route_packet_sent<H: EventHandler<Self>>(
        handler: &H,
        server: Server,
        event: PacketSentEventData,
    ) -> PacketSentEventData {
        handler.handle(server, event)
    }
}

/// fired before play for a packet from a client below the server's
/// protocol, in the client's format
pub struct ConnectionPacketReceivedEvent;

impl FromIntoEvent for ConnectionPacketReceivedEvent {
    const EVENT_TYPE: EventType = EventType::ConnectionPacketReceivedEvent;
    type Data = ConnectionPacketReceivedEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ConnectionPacketReceivedEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ConnectionPacketReceivedEvent(data)
    }
}

/// fired before play for a packet to a client below the server's
/// protocol, in the server's format
pub struct ConnectionPacketSentEvent;

impl FromIntoEvent for ConnectionPacketSentEvent {
    const EVENT_TYPE: EventType = EventType::ConnectionPacketSentEvent;
    type Data = ConnectionPacketSentEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ConnectionPacketSentEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ConnectionPacketSentEvent(data)
    }
}
