use std::{
    collections::BTreeMap,
    marker::PhantomData,
    pin::Pin,
    sync::{
        Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

pub use crate::wit::pumpkin::plugin::event::{Event, EventPriority};
use crate::{
    Context, Result, Server,
    wit::pumpkin::plugin::event::{
        BlockBreakEventData,
        BlockBurnEventData,
        BlockCanBuildEventData,
        BlockGrowEventData,
        BlockPlaceEventData,
        BlockRedstoneEventData,
        EventType,
        PlayerChangeWorldEventData,
        PlayerChangedMainHandEventData,
        PlayerChatEventData,
        PlayerCommandSendEventData,
        PlayerCustomPayloadEventData,
        PlayerExpChangeEventData,
        PlayerFishEventData,
        PlayerGamemodeChangeEventData,
        PlayerItemHeldEventData,
        PlayerJoinEventData,
        PlayerLeaveEventData,
        PlayerLoginEventData,
        PlayerMoveEventData,
        PlayerPermissionCheckEventData,
        PlayerTeleportEventData,
        ServerBroadcastEventData,
        ServerCommandEventData,
        SpawnChangeEventData,
    },
};

pub(crate) static NEXT_HANDLER_ID: AtomicU32 = AtomicU32::new(0);
pub(crate) static EVENT_HANDLERS: Mutex<BTreeMap<u32, Box<dyn ErasedEventHandler>>> =
    Mutex::new(BTreeMap::new());

pub trait FromIntoEvent: Sized {
    const EVENT_TYPE: EventType;
    type Data;

    fn data_from_event(event: Event) -> Self::Data;
    fn data_into_event(data: Self::Data) -> Event;
}

pub type EventData<E> = <E as FromIntoEvent>::Data;
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait EventHandler<E: FromIntoEvent> {
    fn handle(&self, server: Server, event: E::Data) -> E::Data;
}

pub(crate) trait ErasedEventHandler: Send + Sync {
    fn handle_erased(&self, server: Server, event: Event) -> Event;
}

struct HandlerWrapper<E: FromIntoEvent, H> {
    handler: H,
    _phantom: PhantomData<E>,
}

impl<E: FromIntoEvent + Send + Sync, H: EventHandler<E> + Send + Sync> ErasedEventHandler
    for HandlerWrapper<E, H>
{
    fn handle_erased(&self, server: Server, event: Event) -> Event {
        let data = E::data_from_event(event);
        let result = self.handler.handle(server, data);
        E::data_into_event(result)
    }
}

impl Context {
    /// Registers an event handler with the plugin.
    ///
    /// The handler must implement the [`EventHandler`] trait.
    /// If the event is blocking, returning an event from the handler will modify the event.
    pub fn register_event_handler<
        E: FromIntoEvent + Send + Sync + 'static,
        H: EventHandler<E> + Send + Sync + 'static,
    >(
        &self,
        handler: H,
        event_priority: EventPriority,
        blocking: bool,
    ) -> Result<u32> {
        let id = NEXT_HANDLER_ID.fetch_add(1, Ordering::Relaxed);
        let wrapped = HandlerWrapper {
            handler,
            _phantom: PhantomData::<E>,
        };
        EVENT_HANDLERS
            .lock()
            .map_err(|e| e.to_string())?
            .insert(id, Box::new(wrapped));

        self.register_event(id, E::EVENT_TYPE, event_priority, blocking);
        Ok(id)
    }
}

// =============================================================================
// Event implementations
// New events should be added here at the bottom of the file.
// =============================================================================

// --- Player events ---

pub struct PlayerJoinEvent;
impl FromIntoEvent for PlayerJoinEvent {
    const EVENT_TYPE: EventType = EventType::PlayerJoinEvent;
    type Data = PlayerJoinEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerJoinEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerJoinEvent(data)
    }
}

pub struct PlayerLeaveEvent;
impl FromIntoEvent for PlayerLeaveEvent {
    const EVENT_TYPE: EventType = EventType::PlayerLeaveEvent;
    type Data = PlayerLeaveEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerLeaveEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerLeaveEvent(data)
    }
}

pub struct PlayerLoginEvent;
impl FromIntoEvent for PlayerLoginEvent {
    const EVENT_TYPE: EventType = EventType::PlayerLoginEvent;
    type Data = PlayerLoginEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerLoginEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerLoginEvent(data)
    }
}

pub struct PlayerChatEvent;
impl FromIntoEvent for PlayerChatEvent {
    const EVENT_TYPE: EventType = EventType::PlayerChatEvent;
    type Data = PlayerChatEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerChatEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerChatEvent(data)
    }
}

pub struct PlayerCommandSendEvent;
impl FromIntoEvent for PlayerCommandSendEvent {
    const EVENT_TYPE: EventType = EventType::PlayerCommandSendEvent;
    type Data = PlayerCommandSendEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerCommandSendEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerCommandSendEvent(data)
    }
}

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

pub struct PlayerMoveEvent;
impl FromIntoEvent for PlayerMoveEvent {
    const EVENT_TYPE: EventType = EventType::PlayerMoveEvent;
    type Data = PlayerMoveEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerMoveEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerMoveEvent(data)
    }
}

pub struct PlayerTeleportEvent;
impl FromIntoEvent for PlayerTeleportEvent {
    const EVENT_TYPE: EventType = EventType::PlayerTeleportEvent;
    type Data = PlayerTeleportEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerTeleportEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerTeleportEvent(data)
    }
}

pub struct PlayerChangeWorldEvent;
impl FromIntoEvent for PlayerChangeWorldEvent {
    const EVENT_TYPE: EventType = EventType::PlayerChangeWorldEvent;
    type Data = PlayerChangeWorldEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerChangeWorldEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerChangeWorldEvent(data)
    }
}

pub struct PlayerExpChangeEvent;
impl FromIntoEvent for PlayerExpChangeEvent {
    const EVENT_TYPE: EventType = EventType::PlayerExpChangeEvent;
    type Data = PlayerExpChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerExpChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerExpChangeEvent(data)
    }
}

pub struct PlayerItemHeldEvent;
impl FromIntoEvent for PlayerItemHeldEvent {
    const EVENT_TYPE: EventType = EventType::PlayerItemHeldEvent;
    type Data = PlayerItemHeldEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerItemHeldEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerItemHeldEvent(data)
    }
}

pub struct PlayerChangedMainHandEvent;
impl FromIntoEvent for PlayerChangedMainHandEvent {
    const EVENT_TYPE: EventType = EventType::PlayerChangedMainHandEvent;
    type Data = PlayerChangedMainHandEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerChangedMainHandEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerChangedMainHandEvent(data)
    }
}

pub struct PlayerGamemodeChangeEvent;
impl FromIntoEvent for PlayerGamemodeChangeEvent {
    const EVENT_TYPE: EventType = EventType::PlayerGamemodeChangeEvent;
    type Data = PlayerGamemodeChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerGamemodeChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerGamemodeChangeEvent(data)
    }
}

pub struct PlayerCustomPayloadEvent;
impl FromIntoEvent for PlayerCustomPayloadEvent {
    const EVENT_TYPE: EventType = EventType::PlayerCustomPayloadEvent;
    type Data = PlayerCustomPayloadEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerCustomPayloadEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerCustomPayloadEvent(data)
    }
}

pub struct PlayerFishEvent;
impl FromIntoEvent for PlayerFishEvent {
    const EVENT_TYPE: EventType = EventType::PlayerFishEvent;
    type Data = PlayerFishEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::PlayerFishEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::PlayerFishEvent(data)
    }
}

// --- Block events ---

pub struct BlockRedstoneEvent;
impl FromIntoEvent for BlockRedstoneEvent {
    const EVENT_TYPE: EventType = EventType::BlockRedstoneEvent;
    type Data = BlockRedstoneEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockRedstoneEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockRedstoneEvent(data)
    }
}

pub struct BlockBreakEvent;
impl FromIntoEvent for BlockBreakEvent {
    const EVENT_TYPE: EventType = EventType::BlockBreakEvent;
    type Data = BlockBreakEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockBreakEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockBreakEvent(data)
    }
}

pub struct BlockBurnEvent;
impl FromIntoEvent for BlockBurnEvent {
    const EVENT_TYPE: EventType = EventType::BlockBurnEvent;
    type Data = BlockBurnEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockBurnEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockBurnEvent(data)
    }
}

pub struct BlockCanBuildEvent;
impl FromIntoEvent for BlockCanBuildEvent {
    const EVENT_TYPE: EventType = EventType::BlockCanBuildEvent;
    type Data = BlockCanBuildEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockCanBuildEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockCanBuildEvent(data)
    }
}

pub struct BlockGrowEvent;
impl FromIntoEvent for BlockGrowEvent {
    const EVENT_TYPE: EventType = EventType::BlockGrowEvent;
    type Data = BlockGrowEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockGrowEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockGrowEvent(data)
    }
}

pub struct BlockPlaceEvent;
impl FromIntoEvent for BlockPlaceEvent {
    const EVENT_TYPE: EventType = EventType::BlockPlaceEvent;
    type Data = BlockPlaceEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::BlockPlaceEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::BlockPlaceEvent(data)
    }
}

// --- Server events ---

pub struct ServerCommandEvent;
impl FromIntoEvent for ServerCommandEvent {
    const EVENT_TYPE: EventType = EventType::ServerCommandEvent;
    type Data = ServerCommandEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ServerCommandEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ServerCommandEvent(data)
    }
}

pub struct SpawnChangeEvent;
impl FromIntoEvent for SpawnChangeEvent {
    const EVENT_TYPE: EventType = EventType::SpawnChangeEvent;
    type Data = SpawnChangeEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::SpawnChangeEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::SpawnChangeEvent(data)
    }
}

pub struct ServerBroadcastEvent;
impl FromIntoEvent for ServerBroadcastEvent {
    const EVENT_TYPE: EventType = EventType::ServerBroadcastEvent;
    type Data = ServerBroadcastEventData;

    fn data_from_event(event: Event) -> Self::Data {
        match event {
            Event::ServerBroadcastEvent(data) => data,
            _ => panic!("unexpected event"),
        }
    }

    fn data_into_event(data: Self::Data) -> Event {
        Event::ServerBroadcastEvent(data)
    }
}
