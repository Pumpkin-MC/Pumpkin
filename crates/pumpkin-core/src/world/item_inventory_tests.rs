use std::sync::{Arc, Mutex, PoisonError, atomic::AtomicBool, atomic::Ordering};

use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::{entity::EntityType, game_event::GameEvent, packet::CURRENT_MC_VERSION};
use pumpkin_protocol::{packet::MultiVersionJavaPacket, ser::NetworkReadExt};
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
use pumpkin_world::chunk::ChunkData;

use crate::{
    data::VanillaData,
    entity::{Entity, EntityBase, passive::pig::PigEntity},
    plugin::{
        BoxFuture, EventPriority, EventRwHandler,
        api::events::world::generic_game::GenericGameEvent, server::packet::PacketSentEvent,
    },
    server::Server,
};

use super::World;

pub(super) type TestResult = Result<(), Box<dyn std::error::Error>>;
pub(super) const CENTER: Vector3<f64> = Vector3::new(8.5, 64.0, 8.5);

pub(super) async fn inventory_world(
    folder: &tempfile::TempDir,
) -> Result<(Arc<Server>, Arc<World>), Box<dyn std::error::Error>> {
    let basic = BasicConfiguration {
        default_level_name: folder.path().join("world").to_string_lossy().into_owned(),
        allow_nether: false,
        allow_end: false,
        allow_chat_reports: false,
        ..Default::default()
    };
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.bedrock.online_mode = false;
    let server = Server::new(
        basic,
        advanced,
        TelemetryConfig::default(),
        VanillaData::default(),
        vec![],
    )
    .await?;
    let world = server.worlds.load().first().cloned().ok_or("No world")?;
    Ok((server, world))
}

pub(super) fn java_player(
    server: &Arc<Server>,
    world: &Arc<World>,
    name: &str,
) -> Result<(Arc<crate::entity::player::Player>, tokio::io::DuplexStream), Box<dyn std::error::Error>>
{
    use arc_swap::ArcSwap;
    use pumpkin_protocol::ConnectionState;
    use pumpkin_util::GameMode;

    use crate::entity::player::Player;
    use crate::net::{
        ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig,
        java::{JavaClient, pending::PendingConnection},
    };

    let (transport, peer) = tokio::io::duplex(8192);
    let (read, write) = tokio::io::split(transport);
    let pending = PendingConnection::from_transport(
        Box::new(read),
        Box::new(write),
        "127.0.0.1:25565".parse()?,
        0,
        PacketRateLimiter::new(false, 0.0, 0.0),
        Arc::downgrade(server),
    );
    let profile = GameProfile {
        id: uuid::Uuid::new_v4(),
        name: name.into(),
        properties: ArcSwap::from_pointee(vec![]),
        profile_actions: None,
    };
    let mut client = JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
    client.connection_state.store(ConnectionState::Play);
    client.start_outgoing_packet_task();
    let player = Arc::new(Player::new(
        Arc::new(ClientPlatform::Java(client)),
        profile,
        PlayerConfig::default(),
        world,
        GameMode::Survival,
    ));
    player.set_client_loaded(true);
    world.add_player(&player)?;
    Ok((player, peer))
}

type BedrockQueue = tokio::sync::mpsc::UnboundedReceiver<crate::net::bedrock::OutgoingPacket>;

pub(super) async fn bedrock_player(
    server: &Arc<Server>,
    world: &Arc<World>,
    name: &str,
) -> Result<(Arc<crate::entity::player::Player>, BedrockQueue), Box<dyn std::error::Error>> {
    use crate::entity::player::Player;
    use crate::net::{ClientPlatform, GameProfile, PlayerConfig, bedrock::BedrockClient};
    use arc_swap::ArcSwap;
    use pumpkin_util::GameMode;

    let (client, packets) = BedrockClient::test_outgoing_queue(server).await?;
    let player = Arc::new(Player::new(
        Arc::new(ClientPlatform::Bedrock(client.clone())),
        GameProfile {
            id: uuid::Uuid::new_v4(),
            name: name.into(),
            properties: ArcSwap::from_pointee(vec![]),
            profile_actions: None,
        },
        PlayerConfig::default(),
        world,
        GameMode::Survival,
    ));
    player.get_entity().set_pos(CENTER);
    player
        .watched_section
        .store(pumpkin_world::cylindrical_chunk_iterator::Cylindrical::new(
            Vector2::new(0, 0),
            std::num::NonZeroU8::new(4).ok_or("Zero view distance")?,
        ));
    client.set_player(player.clone());
    player.set_client_loaded(true);
    world.add_player(&player)?;
    Ok((player, packets))
}

#[derive(Default)]
struct BedrockUseMetadata {
    flags: Option<i64>,
    flags_two: Option<i64>,
}

fn read_bedrock_use_metadata(
    packets: &mut BedrockQueue,
    actor_id: i32,
) -> Result<BedrockUseMetadata, Box<dyn std::error::Error>> {
    use pumpkin_protocol::{
        bedrock::client::set_actor_data::{CSetActorData, entity_data_key},
        codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
        packet::Packet,
        serial::PacketRead,
    };

    let mut metadata = BedrockUseMetadata::default();
    while let Ok(packet) = packets.try_recv() {
        let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data) else {
            continue;
        };
        let mut data = payload.as_ref();
        if i32::from(id) != CSetActorData::PACKET_ID
            || VarULong::read(&mut data)?.0 != actor_id as u64
        {
            continue;
        }
        let count = VarUInt::read(&mut data)?.0;
        for _ in 0..count {
            let key = VarUInt::read(&mut data)?.0;
            let kind = VarUInt::read(&mut data)?.0;
            assert_eq!(u32::from(u8::read(&mut data)?), kind);
            match (key, kind) {
                (entity_data_key::FLAGS, 7) => metadata.flags = Some(VarLong::read(&mut data)?.0),
                (entity_data_key::FLAGS_TWO, 7) => {
                    metadata.flags_two = Some(VarLong::read(&mut data)?.0);
                }
                (_, 0) => {
                    u8::read(&mut data)?;
                }
                (_, 2) => {
                    VarInt::read(&mut data)?;
                }
                (_, 3) => {
                    f32::read(&mut data)?;
                }
                (_, 4) => {
                    String::read(&mut data)?;
                }
                (_, 7) => {
                    VarLong::read(&mut data)?;
                }
                _ => {
                    return Err(
                        format!("Unexpected Bedrock metadata type {kind} for key {key}").into(),
                    );
                }
            }
        }
    }
    Ok(metadata)
}

#[derive(Clone, Copy)]
enum ShieldBreakReplacement {
    Sword,
    SwordDuringClear,
    SwordDuringAttribute,
    Bow,
    RestartOnClear,
    RestartAtFinish,
    Shield(pumpkin_util::Hand),
}

struct ReplaceOnShieldBreak {
    pig: Arc<PigEntity>,
    replacement: ShieldBreakReplacement,
    called: AtomicBool,
    equipment_packets: std::sync::atomic::AtomicU8,
}

impl ReplaceOnShieldBreak {
    fn assert_replacement(&self) {
        use crate::entity::mob::Mob;
        use pumpkin_data::{
            attributes::Attributes, data_component_impl::EquipmentSlot, item::Item,
        };

        let living = &self.pig.get_mob_entity().living_entity;
        assert!(self.called.load(Ordering::Relaxed));
        match self.replacement {
            ShieldBreakReplacement::Sword
            | ShieldBreakReplacement::SwordDuringClear
            | ShieldBreakReplacement::SwordDuringAttribute => {
                let active_slot = living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .get(&EquipmentSlot::MAIN_HAND);
                let attack_damage = living.get_attribute_value(&Attributes::ATTACK_DAMAGE);
                let base_damage = living.get_attribute_base(&Attributes::ATTACK_DAMAGE);
                if active_slot.item == &Item::STICK {
                    assert_eq!(
                        attack_damage.to_bits(),
                        base_damage.to_bits(),
                        "A stick cannot retain sword damage"
                    );
                }
                assert_eq!(active_slot.item, &Item::DIAMOND_SWORD);
                assert!(attack_damage > base_damage);
                assert!(!living.is_blocking());
                let expected_packets =
                    if matches!(self.replacement, ShieldBreakReplacement::SwordDuringClear) {
                        2
                    } else {
                        1
                    };
                assert_eq!(
                    self.equipment_packets.load(Ordering::Relaxed),
                    expected_packets
                );
            }
            ShieldBreakReplacement::Shield(hand) => {
                let equipment = living
                    .entity_equipment
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                let slot = shield_test_slot(hand);
                assert_eq!(equipment.get(&slot).item, &Item::SHIELD);
                if hand == pumpkin_util::Hand::Left {
                    assert!(equipment.get(&EquipmentSlot::MAIN_HAND).is_empty());
                }
                drop(equipment);
                assert!(living.is_blocking());
                assert_eq!(
                    *living
                        .active_hand
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner),
                    Some(hand)
                );
            }
            ShieldBreakReplacement::Bow => {
                assert!(!living.is_blocking());
                assert_eq!(
                    *living
                        .active_hand
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner),
                    Some(pumpkin_util::Hand::Right)
                );
            }
            ShieldBreakReplacement::RestartOnClear | ShieldBreakReplacement::RestartAtFinish => {
                assert!(living.is_blocking());
            }
        }
    }
}

fn shield_test_slot(hand: pumpkin_util::Hand) -> pumpkin_data::data_component_impl::EquipmentSlot {
    use pumpkin_data::data_component_impl::EquipmentSlot;
    match hand {
        pumpkin_util::Hand::Left => EquipmentSlot::OFF_HAND,
        pumpkin_util::Hand::Right => EquipmentSlot::MAIN_HAND,
    }
}

impl ReplaceOnShieldBreak {
    fn append_attribute_extra(event: &mut PacketSentEvent, actor_id: i32) {
        use pumpkin_protocol::java::client::play::CUpdateAttributes;
        let properties = [
            (pumpkin_data::attributes::Attributes::ATTACK_SPEED, 77.0),
            (pumpkin_data::attributes::Attributes::MOVEMENT_SPEED, 0.75),
        ]
        .into_iter()
        .map(|(attribute, value)| {
            pumpkin_protocol::java::client::play::Property::new(
                i32::from(attribute.id).into(),
                value,
                Vec::new(),
            )
        })
        .collect();
        let extra = CUpdateAttributes::new(actor_id.into(), properties);
        let mut payload = Vec::new();
        assert!(
            pumpkin_protocol::ClientPacket::write_packet_data(
                &extra,
                &mut payload,
                &CURRENT_MC_VERSION
            )
            .is_ok()
        );
        event.extra_packets.push((event.packet_id, payload.into()));
    }

    fn replace_shield_from_break_status(&self) {
        use crate::entity::mob::Mob;
        use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};

        let mob = self.pig.get_mob_entity();
        match self.replacement {
            ShieldBreakReplacement::Sword => {
                mob.set_item_slot(
                    &EquipmentSlot::MAIN_HAND,
                    ItemStack::new(1, &Item::DIAMOND_SWORD),
                );
            }
            ShieldBreakReplacement::Shield(hand) => {
                let shield = ItemStack::new(1, &Item::SHIELD);
                mob.set_item_slot(&shield_test_slot(hand), shield.clone());
                let use_time = shield.get_max_use_time() - 20;
                mob.living_entity.set_active_hand(hand, shield, use_time);
            }
            ShieldBreakReplacement::Bow => {
                let bow = ItemStack::new(1, &Item::BOW);
                mob.set_item_slot(&EquipmentSlot::MAIN_HAND, bow.clone());
                let duration = bow.get_max_use_time();
                mob.living_entity
                    .set_active_hand(pumpkin_util::Hand::Right, bow, duration);
            }
            ShieldBreakReplacement::SwordDuringClear
            | ShieldBreakReplacement::SwordDuringAttribute
            | ShieldBreakReplacement::RestartOnClear
            | ShieldBreakReplacement::RestartAtFinish => {}
        }
    }
}

impl EventRwHandler<PacketSentEvent> for ReplaceOnShieldBreak {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use crate::entity::mob::Mob;
        use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};
        use pumpkin_protocol::java::client::play::{
            CEntityStatus, CSetEquipment, CUpdateAttributes,
        };

        use pumpkin_protocol::ServerPacket;
        Box::pin(async move {
            if event.player.client.java().is_none() {
                return;
            }
            let rider = self.pig.get_entity();
            let mob = self.pig.get_mob_entity();
            if matches!(
                self.replacement,
                ShieldBreakReplacement::SwordDuringAttribute
            ) && event.packet_id == CUpdateAttributes::to_id(CURRENT_MC_VERSION)
                && self.called.load(Ordering::Relaxed)
                && CUpdateAttributes::read(&mut event.payload.as_ref(), &CURRENT_MC_VERSION)
                    .is_ok_and(|packet| {
                        packet.entity_id.0 == rider.entity_id
                            && packet.properties.iter().any(|property| {
                                property.id.0
                                    == i32::from(
                                        pumpkin_data::attributes::Attributes::ATTACK_SPEED.id,
                                    )
                            })
                    })
                && self.equipment_packets.fetch_add(1, Ordering::Relaxed) == 0
            {
                mob.set_item_slot(
                    &EquipmentSlot::MAIN_HAND,
                    ItemStack::new(1, &Item::DIAMOND_SWORD),
                );
                Self::append_attribute_extra(event, rider.entity_id);
            }
            if event.packet_id == CSetEquipment::to_id(CURRENT_MC_VERSION)
                && self.called.load(Ordering::Relaxed)
                && event
                    .payload
                    .as_ref()
                    .get_var_int()
                    .is_ok_and(|id| id.0 == rider.entity_id)
            {
                if matches!(
                    self.replacement,
                    ShieldBreakReplacement::SwordDuringAttribute
                ) {
                    return;
                }
                let previous = self.equipment_packets.fetch_add(1, Ordering::Relaxed);
                if matches!(self.replacement, ShieldBreakReplacement::SwordDuringClear)
                    && previous == 0
                {
                    mob.set_item_slot(
                        &EquipmentSlot::MAIN_HAND,
                        ItemStack::new(1, &Item::DIAMOND_SWORD),
                    );
                } else if matches!(self.replacement, ShieldBreakReplacement::Sword) && previous == 1
                {
                    // A replay gives observers another chance to change the slot.
                    mob.set_item_slot(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, &Item::STICK));
                }
                return;
            }
            if event.packet_id == CEntityStatus::to_id(CURRENT_MC_VERSION)
                && event
                    .payload
                    .get(..4)
                    .is_some_and(|id| id == rider.entity_id.to_be_bytes())
                && event.payload.get(4)
                    == Some(
                        &(crate::entity::equipment_break_status(&EquipmentSlot::MAIN_HAND) as u8),
                    )
                && !self.called.swap(true, Ordering::Relaxed)
            {
                assert!(mob.living_entity.entity_equipment.try_lock().is_ok());
                self.replace_shield_from_break_status();
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shield_break_observer_can_replace_the_active_item() -> TestResult {
    use pumpkin_util::Hand;
    for replacement in [
        ShieldBreakReplacement::Sword,
        ShieldBreakReplacement::SwordDuringClear,
        ShieldBreakReplacement::SwordDuringAttribute,
        ShieldBreakReplacement::Bow,
        ShieldBreakReplacement::RestartOnClear,
        ShieldBreakReplacement::RestartAtFinish,
        ShieldBreakReplacement::Shield(Hand::Left),
        ShieldBreakReplacement::Shield(Hand::Right),
    ] {
        assert_shield_break_replacement(replacement).await?;
    }
    Ok(())
}

async fn assert_shield_break_replacement(replacement: ShieldBreakReplacement) -> TestResult {
    use crate::entity::mob::Mob;
    use pumpkin_data::damage::DamageType;
    use pumpkin_protocol::java::client::play::CKeepAlive;
    use pumpkin_util::Hand;

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (watcher, peer) = java_equipment_watcher(&server, &world)?;
    watcher
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    let client = watcher.client.java().ok_or("Java watcher")?;
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(pig.clone());
    let living = &pig.get_mob_entity().living_entity;
    let shield = test_breaking_shield(&pig, replacement)?;
    let elapsed = shield.get_max_use_time() - 20;
    let flags_check = if matches!(
        replacement,
        ShieldBreakReplacement::Bow
            | ShieldBreakReplacement::RestartOnClear
            | ShieldBreakReplacement::RestartAtFinish
    ) {
        Some(watch_shield_use_flags(&server, &world, &pig, replacement).await?)
    } else {
        None
    };
    living.set_active_hand(Hand::Right, shield, elapsed);
    assert!(living.is_blocking());
    let observer = Arc::new(ReplaceOnShieldBreak {
        pig: pig.clone(),
        replacement,
        called: AtomicBool::new(false),
        equipment_packets: std::sync::atomic::AtomicU8::new(0),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    living.damage_with_context(
        pig.as_ref(),
        10.0,
        DamageType::MOB_ATTACK,
        Some(CENTER.add_raw(0.0, 0.0, 1.0)),
        None,
        None,
    );
    if matches!(replacement, ShieldBreakReplacement::SwordDuringAttribute) {
        use pumpkin_data::attributes::Attributes;
        assert!(observer.called.load(Ordering::Relaxed));
        let speed = living.get_attribute_value(&Attributes::ATTACK_SPEED);
        assert!(speed < living.get_attribute_base(&Attributes::ATTACK_SPEED));
        assert!(client.try_send_packet(&CKeepAlive::new(73)));
        let (attack, movement) = read_attribute_updates(peer, pig.get_entity().entity_id).await?;
        assert_eq!(attack.map(f64::to_bits), Some(speed.to_bits()));
        assert_eq!(movement, Some(0.75));
        world.level.shutdown().await;
        return Ok(());
    }
    observer.assert_replacement();
    if let Some((restart, mut packets)) = flags_check {
        restart.assert_outcome(&mut packets, elapsed)?;
    }
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    assert_shield_replacement_packets(peer, pig.get_entity().entity_id, replacement).await?;
    world.level.shutdown().await;
    Ok(())
}

fn test_breaking_shield(
    pig: &PigEntity,
    replacement: ShieldBreakReplacement,
) -> Result<pumpkin_data::item_stack::ItemStack, Box<dyn std::error::Error>> {
    use crate::entity::mob::Mob;
    use pumpkin_data::{
        data_component_impl::{AttributeModifiersImpl, EquipmentSlot},
        item::Item,
        item_stack::ItemStack,
    };

    let mob = pig.get_mob_entity();
    let mut shield = ItemStack::new(1, &Item::SHIELD);
    shield.set_damage(shield.get_max_damage().ok_or("no durability")? - 1);
    if matches!(replacement, ShieldBreakReplacement::SwordDuringAttribute) {
        let sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
        shield.set_data_component(
            sword
                .get_data_component::<AttributeModifiersImpl>()
                .ok_or("sword modifiers")?
                .clone(),
        );
        mob.set_item_slot(&EquipmentSlot::MAIN_HAND, shield.clone());
    } else {
        mob.living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .put(&EquipmentSlot::MAIN_HAND, shield.clone());
    }
    Ok(shield)
}

async fn assert_shield_replacement_packets(
    peer: tokio::io::DuplexStream,
    actor_id: i32,
    replacement: ShieldBreakReplacement,
) -> TestResult {
    use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item};
    use pumpkin_util::Hand;

    let (equipment, _) = read_equipment_updates(peer, actor_id).await?;
    let main = EquipmentSlot::MAIN_HAND.discriminant();
    let off = EquipmentSlot::OFF_HAND.discriminant();
    let expected = match replacement {
        ShieldBreakReplacement::Sword
        | ShieldBreakReplacement::SwordDuringClear
        | ShieldBreakReplacement::SwordDuringAttribute => {
            vec![(main, Some(&Item::DIAMOND_SWORD))]
        }
        ShieldBreakReplacement::Bow => vec![(main, Some(&Item::BOW))],
        ShieldBreakReplacement::RestartOnClear | ShieldBreakReplacement::RestartAtFinish => {
            vec![(main, None), (off, Some(&Item::SHIELD))]
        }
        ShieldBreakReplacement::Shield(Hand::Left) => {
            vec![(off, Some(&Item::SHIELD)), (main, None)]
        }
        ShieldBreakReplacement::Shield(Hand::Right) => vec![(main, Some(&Item::SHIELD))],
    };
    let items = equipment
        .iter()
        .map(|(slot, stack)| (*slot, (!stack.is_empty()).then_some(stack.item)))
        .collect::<Vec<_>>();
    assert_eq!(items, expected);
    Ok(())
}

fn java_equipment_watcher(
    server: &Arc<Server>,
    world: &Arc<World>,
) -> Result<(Arc<crate::entity::player::Player>, tokio::io::DuplexStream), Box<dyn std::error::Error>>
{
    use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;
    let chunk = Vector2::new(0, 0);
    world
        .level
        .loaded_chunks
        .insert(chunk, ChunkData::empty_sync(0, 0));
    let (watcher, peer) = java_player(server, world, "Notch")?;
    let client = watcher.client.java().ok_or("Java watcher")?;
    client.player.store(Arc::new(Some(watcher.clone())));
    watcher.get_entity().set_pos(CENTER.add_raw(3.0, 0.0, 0.0));
    watcher.watched_section.store(Cylindrical::new(
        chunk,
        std::num::NonZeroU8::new(4).ok_or("distance")?,
    ));
    Ok((watcher, peer))
}

async fn read_equipment_updates(
    peer: tokio::io::DuplexStream,
    actor_id: i32,
) -> Result<(Vec<(i8, pumpkin_data::item_stack::ItemStack)>, usize), Box<dyn std::error::Error>> {
    use pumpkin_protocol::{
        ServerPacket,
        java::{
            client::play::{CKeepAlive, CSetEquipment, CSpawnEntity},
            packet_decoder::TCPNetworkDecoder,
        },
    };

    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut updates = Vec::new();
        let mut spawns = 0;
        loop {
            let packet = decoder.get_raw_packet().await?;
            let mut payload = packet.payload.as_ref();
            if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                assert_eq!(payload.get_i64_be()?, 73);
                return Ok((updates, spawns));
            }
            if packet.id == CSpawnEntity::to_id(CURRENT_MC_VERSION)
                && payload.get_var_int()?.0 == actor_id
            {
                spawns += 1;
                updates.clear();
            }
            if packet.id == CSetEquipment::to_id(CURRENT_MC_VERSION) {
                let packet = CSetEquipment::read(&mut payload, &CURRENT_MC_VERSION)?;
                if packet.entity_id.0 == actor_id {
                    updates.extend(
                        packet
                            .equipment
                            .into_iter()
                            .map(|(slot, stack)| (slot, stack.0.into_owned())),
                    );
                }
            }
        }
    })
    .await?
}

async fn read_attribute_updates(
    peer: tokio::io::DuplexStream,
    actor_id: i32,
) -> Result<(Option<f64>, Option<f64>), Box<dyn std::error::Error>> {
    use pumpkin_data::attributes::Attributes;
    use pumpkin_protocol::{
        ServerPacket,
        java::{
            client::play::{CKeepAlive, CUpdateAttributes},
            packet_decoder::TCPNetworkDecoder,
        },
    };
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut speed = None;
        let mut movement = None;
        loop {
            let packet = decoder.get_raw_packet().await?;
            if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                return Ok((speed, movement));
            }
            if packet.id == CUpdateAttributes::to_id(CURRENT_MC_VERSION) {
                let packet =
                    CUpdateAttributes::read(&mut packet.payload.as_ref(), &CURRENT_MC_VERSION)?;
                if packet.entity_id.0 == actor_id {
                    for property in packet.properties {
                        if property.id.0 == i32::from(Attributes::MOVEMENT_SPEED.id) {
                            movement = Some(property.value);
                        }
                        if property.id.0 == i32::from(Attributes::ATTACK_SPEED.id) {
                            speed = Some(
                                property.value
                                    + property
                                        .modifiers
                                        .iter()
                                        .map(|modifier| modifier.amount)
                                        .sum::<f64>(),
                            );
                        }
                    }
                }
            }
        }
    })
    .await?
}

struct ObserveShieldUse {
    pig: Arc<PigEntity>,
    replacement: ShieldBreakReplacement,
    start: Mutex<Option<(Option<pumpkin_util::Hand>, i32, bool)>>,
    restarted: AtomicBool,
    finish: Mutex<Option<(Option<pumpkin_util::Hand>, i32, bool)>>,
}

impl ObserveShieldUse {
    const SUCCESSOR_DURATION: i32 = 71_927;
    const SUCCESSOR_DAMAGE: i32 = 7;
    fn assert_outcome(&self, packets: &mut BedrockQueue, elapsed: i32) -> TestResult {
        use pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag::{
            BLOCKING, USING_ITEM,
        };
        use pumpkin_util::Hand;
        let restart = !matches!(self.replacement, ShieldBreakReplacement::Bow);
        let metadata = read_bedrock_use_metadata(packets, self.pig.get_entity().entity_id)?;
        assert_ne!(
            metadata.flags.ok_or("Missing use flags")? & (1i64 << USING_ITEM),
            0
        );
        assert_eq!(
            metadata.flags_two.ok_or("Missing blocking flags")? & (1i64 << (BLOCKING - 64)) != 0,
            restart,
        );
        assert_eq!(self.restarted.load(Ordering::Relaxed), restart);
        let finish = *self.finish.lock().unwrap_or_else(PoisonError::into_inner);
        assert_eq!(
            finish,
            restart.then_some((Some(Hand::Right), elapsed, false))
        );
        assert_eq!(
            *self.start.lock().unwrap_or_else(PoisonError::into_inner),
            Some((Some(Hand::Right), elapsed, true)),
        );
        if restart {
            use crate::entity::mob::Mob;
            let living = &self.pig.get_mob_entity().living_entity;
            assert_eq!(
                *living
                    .active_hand
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner),
                Some(Hand::Left)
            );
            assert_ne!(elapsed, Self::SUCCESSOR_DURATION);
            assert_eq!(
                living.item_use_time.load(Ordering::Relaxed),
                Self::SUCCESSOR_DURATION
            );
            let off = mob_offhand(living);
            assert_eq!(off.get_damage(), Self::SUCCESSOR_DAMAGE);
            let active = living
                .item_in_use
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            assert!(active.as_ref().is_some_and(|stack| stack.are_equal(&off)));
        }
        Ok(())
    }

    fn restart_shield_use(&self) {
        use crate::entity::mob::Mob;
        use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};
        let mob = self.pig.get_mob_entity();
        let mut shield = ItemStack::new(1, &Item::SHIELD);
        shield.set_damage(Self::SUCCESSOR_DAMAGE);
        mob.set_item_slot(&EquipmentSlot::OFF_HAND, shield.clone());
        mob.living_entity.set_active_hand(
            pumpkin_util::Hand::Left,
            shield,
            Self::SUCCESSOR_DURATION,
        );
    }
}

fn mob_offhand(
    living: &crate::entity::living::LivingEntity,
) -> pumpkin_data::item_stack::ItemStack {
    living
        .entity_equipment
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&pumpkin_data::data_component_impl::EquipmentSlot::OFF_HAND)
}

impl EventRwHandler<GenericGameEvent> for ObserveShieldUse {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut GenericGameEvent,
    ) -> BoxFuture<'a, ()> {
        use crate::entity::mob::Mob;
        use pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag::USING_ITEM;
        Box::pin(async move {
            let start = event.event_key == GameEvent::ItemInteractStart.name();
            if start || event.event_key == GameEvent::ItemInteractFinish.name() {
                let living = &self.pig.get_mob_entity().living_entity;
                let hand = *living
                    .active_hand
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                let using =
                    living.entity.bedrock_flags.load(Ordering::Relaxed) & (1i64 << USING_ITEM) != 0;
                let state = (hand, living.item_use_time.load(Ordering::Relaxed), using);
                if start {
                    self.start
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .get_or_insert(state);
                } else {
                    *self.finish.lock().unwrap_or_else(PoisonError::into_inner) = Some(state);
                    if matches!(self.replacement, ShieldBreakReplacement::RestartAtFinish)
                        && !self.restarted.swap(true, Ordering::Relaxed)
                    {
                        self.restart_shield_use();
                    }
                }
            }
        })
    }
}

impl EventRwHandler<PacketSentEvent> for ObserveShieldUse {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use crate::entity::mob::Mob;
        use pumpkin_protocol::{
            bedrock::client::set_actor_data::{CSetActorData, entity_data_key},
            codec::var_ulong::VarULong,
            packet::Packet,
        };
        Box::pin(async move {
            let mob = self.pig.get_mob_entity();
            let living = &mob.living_entity;
            if matches!(self.replacement, ShieldBreakReplacement::RestartOnClear)
                && event.player.client.bedrock().is_some()
                && event.packet_id == CSetActorData::PACKET_ID
                && !living.is_auto_spin_attack()
                && living.item_use_time.load(Ordering::Relaxed) == 0
                && CSetActorData::updates_key(
                    &event.payload,
                    VarULong(self.pig.get_entity().entity_id as u64),
                    pumpkin_world::CURRENT_BEDROCK_MC_PROTOCOL,
                    entity_data_key::FLAGS,
                )
                .unwrap_or(false)
                && !self.restarted.swap(true, Ordering::Relaxed)
            {
                self.restart_shield_use();
            }
        })
    }
}

async fn watch_shield_use_flags(
    server: &Arc<Server>,
    world: &Arc<World>,
    pig: &Arc<PigEntity>,
    replacement: ShieldBreakReplacement,
) -> Result<(Arc<ObserveShieldUse>, BedrockQueue), Box<dyn std::error::Error>> {
    let (viewer, mut packets) = bedrock_player(server, world, "jeb_").await?;
    viewer
        .chunk_sender
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .mark_sent_out_of_band(Vector2::new(0, 0));
    let tracked = world
        .entity_tracker
        .get_tracked_entity(pig.get_entity().entity_id)
        .ok_or("tracker")?;
    tracked.update_player(&viewer, world);
    assert!(tracked.is_seen_by(&viewer.gameprofile.id));
    while packets.try_recv().is_ok() {}
    let observer = Arc::new(ObserveShieldUse {
        pig: pig.clone(),
        replacement,
        start: Mutex::new(None),
        restarted: AtomicBool::new(false),
        finish: Mutex::new(None),
    });
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(observer.clone(), EventPriority::Normal, None);
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    Ok((observer, packets))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_riptide_stop_keeps_item_use_started_by_its_observer() -> TestResult {
    use crate::entity::mob::Mob;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag::{
        BLOCKING, DAMAGE_NEARBY_MOBS, USING_ITEM,
    };
    let directory = tempfile::tempdir()?;
    let (server, world) = inventory_world(&directory).await?;
    world
        .level
        .loaded_chunks
        .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(pig.clone());
    let living = &pig.get_mob_entity().living_entity;
    let (observer, mut packets) = watch_shield_use_flags(
        &server,
        &world,
        &pig,
        ShieldBreakReplacement::RestartOnClear,
    )
    .await?;

    living.start_auto_spin_attack(20, 1.0, ItemStack::new(1, &Item::TRIDENT), None);
    assert!(living.is_auto_spin_attack());
    let start = read_bedrock_use_metadata(&mut packets, pig.get_entity().entity_id)?;
    assert_ne!(
        start.flags.ok_or("Missing Riptide start flags")? & (1 << DAMAGE_NEARBY_MOBS),
        0
    );
    assert!(!observer.restarted.load(Ordering::Relaxed));

    living.stop_auto_spin_attack();

    assert!(observer.restarted.load(Ordering::Relaxed));
    assert!(!living.is_auto_spin_attack());
    assert!(living.is_blocking());
    let metadata = read_bedrock_use_metadata(&mut packets, pig.get_entity().entity_id)?;
    assert_eq!(
        metadata.flags.ok_or("Missing use flags")?
            & ((1 << USING_ITEM) | (1 << DAMAGE_NEARBY_MOBS)),
        1 << USING_ITEM,
    );
    assert_ne!(
        metadata.flags_two.ok_or("Missing blocking flags")? & (1 << (BLOCKING - 64)),
        0
    );
    world.level.shutdown().await;
    Ok(())
}

struct ReplacePlayerItemUse {
    player: Arc<crate::entity::player::Player>,
    trigger: GameEvent,
    calls: std::sync::atomic::AtomicU32,
    replace: bool,
}

impl EventRwHandler<GenericGameEvent> for ReplacePlayerItemUse {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut GenericGameEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.event_key == self.trigger.name()
                && self.calls.fetch_add(1, Ordering::Relaxed) == 0
                && self.replace
            {
                self.start_successor();
            }
        })
    }
}

impl ReplacePlayerItemUse {
    fn start_successor(&self) {
        use pumpkin_data::{item::Item, item_stack::ItemStack};
        use pumpkin_util::Hand;

        let mut shield = ItemStack::new(1, &Item::SHIELD);
        shield.set_damage(7);
        self.player
            .inventory()
            .set_stack_in_hand(Hand::Left, shield.clone());
        self.player
            .living_entity
            .set_active_hand(Hand::Left, shield, 61_200);
    }

    fn assert_successor(&self) {
        use pumpkin_util::Hand;
        let living = &self.player.living_entity;
        let equipped = self.player.inventory().get_stack_in_hand(Hand::Left);
        assert_eq!(equipped.get_damage(), 7);
        assert_eq!(living.item_use_time.load(Ordering::Relaxed), 61_200);
        assert_eq!(
            *living
                .active_hand
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
            Some(Hand::Left)
        );
        assert!(
            living
                .item_in_use
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .as_ref()
                .is_some_and(|stack| stack.are_equal(&equipped))
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_shield_input_starts_only_once_and_preserves_a_listener_successor() -> TestResult {
    use pumpkin_data::{data_component_impl::BlocksAttacksImpl, item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::server::play::SUseItem;
    for (item, replace) in [
        (&Item::SHIELD, false),
        (&Item::SHIELD, true),
        (&Item::STICK, false),
        (&Item::DIAMOND_SWORD, false),
        (&Item::FLINT_AND_STEEL, false),
        (&Item::POWDER_SNOW_BUCKET, false),
        (&Item::IRON_SPEAR, false),
    ] {
        let folder = tempfile::tempdir()?;
        let (server, world) = inventory_world(&folder).await?;
        let (player, _peer) = java_equipment_watcher(&server, &world)?;
        let mut stack = ItemStack::new(1, item);
        if item != &Item::SHIELD {
            stack.set_data_component(BlocksAttacksImpl::DEFAULT);
        }
        player.inventory().set_held_item(stack);
        let observer = Arc::new(ReplacePlayerItemUse {
            player: player.clone(),
            trigger: GameEvent::ItemInteractStart,
            calls: std::sync::atomic::AtomicU32::new(0),
            replace,
        });
        server
            .plugin_manager
            .register_blocking::<GenericGameEvent, _>(
                observer.clone(),
                EventPriority::Normal,
                None,
            );
        let client = player.client.java().ok_or("Java client")?;
        for sequence in 1..=2 {
            client.handle_use_item(
                &player,
                &SUseItem {
                    hand: 0.into(),
                    sequence: sequence.into(),
                    yaw: 0.0,
                    pitch: 0.0,
                },
                &server,
            );
        }
        let expected_vibrations = if replace {
            2
        } else {
            u32::from(item != &Item::IRON_SPEAR)
        };
        assert_eq!(
            observer.calls.load(Ordering::Relaxed),
            expected_vibrations,
            "{}",
            item.registry_key
        );
        if replace {
            observer.assert_successor();
        } else {
            assert!(player.living_entity.item_use_time.load(Ordering::Relaxed) > 0);
            assert!(
                player
                    .living_entity
                    .item_in_use
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .as_ref()
                    .is_some_and(|active| active.item == item)
            );
            let blocking =
                pumpkin_protocol::bedrock::client::set_actor_data::entity_data_flag::BLOCKING;
            assert_ne!(
                player
                    .get_entity()
                    .bedrock_flags_two
                    .load(Ordering::Relaxed)
                    & (1 << (blocking - 64)),
                0
            );
        }
        world.level.shutdown().await;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn release_packets_keep_the_use_started_by_the_crossbow_finish_listener() -> TestResult {
    use pumpkin_data::{
        data_component_impl::ChargedProjectilesImpl, item::Item, item_stack::ItemStack,
    };
    use pumpkin_protocol::bedrock::server::inventory_transaction::{
        HandSlot, ReleaseItemTransactionData, SInventoryTransaction, TransactionData,
    };
    use pumpkin_util::Hand;

    for (bedrock, replaced_slot) in [(false, false), (true, false), (false, true), (true, true)] {
        let folder = tempfile::tempdir()?;
        let (server, world) = inventory_world(&folder).await?;
        let (player, _java_peer, _bedrock_queue) = if bedrock {
            let (player, queue) = bedrock_player(&server, &world, "jeb_").await?;
            (player, None, Some(queue))
        } else {
            let (player, peer) = java_equipment_watcher(&server, &world)?;
            (player, Some(peer), None)
        };
        let crossbow = ItemStack::new(1, &Item::CROSSBOW);
        player.inventory().set_held_item(crossbow.clone());
        player.living_entity.set_active_hand(
            Hand::Right,
            crossbow,
            if replaced_slot { 71_950 } else { 72_000 },
        );
        if replaced_slot {
            player
                .inventory()
                .set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::STICK));
            player
                .inventory()
                .set_stack_in_hand(Hand::Left, ItemStack::new(3, &Item::ARROW));
        }
        let observer = Arc::new(ReplacePlayerItemUse {
            player: player.clone(),
            trigger: GameEvent::ItemInteractFinish,
            calls: std::sync::atomic::AtomicU32::new(0),
            replace: !replaced_slot,
        });
        server
            .plugin_manager
            .register_blocking::<GenericGameEvent, _>(
                observer.clone(),
                EventPriority::Normal,
                None,
            );
        if player.client.java().is_some() {
            send_java_item_release(&player, &server)?;
        } else {
            let client = player.client.bedrock().ok_or("Bedrock client")?;
            client.handle_inventory_action(
                &player,
                SInventoryTransaction {
                    legacy_request_id: 0.into(),
                    legacy_set_item_slots: Vec::new(),
                    has_value: false,
                    actions: Vec::new(),
                    transaction_type: 4u32.into(),
                    transaction_data: TransactionData::ReleaseItem(ReleaseItemTransactionData {
                        action_type: 0.into(),
                        hot_bar_slot: 0.into(),
                        item_in_hand:
                            pumpkin_protocol::bedrock::network_item::NetworkItemDescriptor::default(
                            ),
                        head_position: Vector3::default(),
                        hand: HandSlot::Mainhand,
                    }),
                },
            );
        }
        assert_eq!(observer.calls.load(Ordering::Relaxed), 1);
        if replaced_slot {
            let held = player.inventory().get_stack_in_hand(Hand::Right);
            assert_eq!(held.item, &Item::STICK);
            assert!(
                held.get_data_component::<ChargedProjectilesImpl>()
                    .is_none()
            );
            assert_eq!(
                player.inventory().get_stack_in_hand(Hand::Left).item_count,
                3
            );
        } else {
            observer.assert_successor();
        }
        world.level.shutdown().await;
    }
    Ok(())
}

fn send_java_item_release(
    player: &Arc<crate::entity::player::Player>,
    server: &Arc<Server>,
) -> TestResult {
    use pumpkin_protocol::java::server::play::{SPlayerAction, Status};
    use pumpkin_util::math::position::BlockPos;

    player
        .client
        .java()
        .ok_or("Java client")?
        .handle_player_action(
            player,
            &SPlayerAction {
                status: (Status::ReleaseItemInUse as i32).into(),
                position: BlockPos::new(0, 0, 0),
                face: 0,
                sequence: 0.into(),
            },
            server,
        );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_bow_release_uses_the_current_same_item_components() -> TestResult {
    use crate::plugin::api::events::entity::projectile_launch::ProjectileLaunchEvent;
    use pumpkin_data::{data_component_impl::EnchantmentsImpl, item::Item, item_stack::ItemStack};
    use pumpkin_util::Hand;

    struct ReplaceUseOnShot(Arc<ReplacePlayerItemUse>, AtomicBool);
    impl EventRwHandler<ProjectileLaunchEvent> for ReplaceUseOnShot {
        fn handle<'a>(
            &'a self,
            server: &'a Arc<Server>,
            event: &'a mut ProjectileLaunchEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.shooter_id == Some(self.0.player.entity_id())
                    && !self.1.swap(true, Ordering::Relaxed)
                {
                    assert!(send_java_item_release(&self.0.player, server).is_ok());
                    self.0.start_successor();
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    let bow = ItemStack::new(1, &Item::BOW);
    let mut infinity = bow.clone();
    infinity.set_data_component(EnchantmentsImpl {
        enchantment: vec![(&pumpkin_data::Enchantment::INFINITY, 1)].into(),
    });
    player.inventory().set_held_item(infinity.clone());
    player
        .living_entity
        .set_active_hand(Hand::Right, infinity, 71_980);
    player
        .inventory()
        .set_slot(1, ItemStack::new(2, &Item::ARROW));
    player.inventory().set_held_item(bow);
    let observer = Arc::new(ReplacePlayerItemUse {
        player: player.clone(),
        trigger: GameEvent::ItemInteractFinish,
        calls: std::sync::atomic::AtomicU32::new(0),
        replace: true,
    });
    server
        .plugin_manager
        .register_blocking::<ProjectileLaunchEvent, _>(
            Arc::new(ReplaceUseOnShot(observer.clone(), AtomicBool::new(false))),
            EventPriority::Normal,
            None,
        );
    send_java_item_release(&player, &server)?;
    assert_eq!(player.inventory().get_slot(1).item_count, 1);
    assert_eq!(player.inventory().held_item().get_damage(), 1);
    observer.assert_successor();
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_trident_release_keeps_the_damage_listeners_replacement() -> TestResult {
    use crate::plugin::api::events::player::player_item_damage::PlayerItemDamageEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_util::Hand;

    struct ReplaceTrident(AtomicBool);
    impl EventRwHandler<PlayerItemDamageEvent> for ReplaceTrident {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PlayerItemDamageEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.0.store(true, Ordering::Relaxed);
                event
                    .player
                    .inventory()
                    .set_held_item(ItemStack::new(1, &Item::DIAMOND_SWORD));
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    let trident = ItemStack::new(1, &Item::TRIDENT);
    player.inventory().set_held_item(trident.clone());
    player
        .living_entity
        .set_active_hand(Hand::Right, trident, 71_980);
    let listener = Arc::new(ReplaceTrident(AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PlayerItemDamageEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            None,
        );
    send_java_item_release(&player, &server)?;
    assert!(listener.0.load(Ordering::Relaxed));
    assert_eq!(player.inventory().held_item().item, &Item::DIAMOND_SWORD);
    assert!(
        !world
            .entities
            .load()
            .iter()
            .any(|entity| entity.get_entity().entity_type == &EntityType::TRIDENT)
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_trident_release_keeps_a_cancelled_riptide_listeners_successor() -> TestResult {
    use crate::plugin::api::events::player::player_riptide::PlayerRiptideEvent;
    use pumpkin_data::{
        data_component_impl::EnchantmentsImpl, enchantment::Enchantment, item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_util::Hand;

    struct CancelRiptide(ReplacePlayerItemUse);

    impl EventRwHandler<PlayerRiptideEvent> for CancelRiptide {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PlayerRiptideEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.0.calls.fetch_add(1, Ordering::Relaxed);
                self.0.start_successor();
                event.cancelled = true;
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    player
        .get_entity()
        .touching_water
        .store(true, Ordering::Relaxed);
    let mut trident = ItemStack::new(1, &Item::TRIDENT);
    trident.set_data_component(EnchantmentsImpl {
        enchantment: vec![(&Enchantment::RIPTIDE, 1)].into(),
    });
    player.inventory().set_held_item(trident.clone());
    player
        .living_entity
        .set_active_hand(Hand::Right, trident, 71_980);
    let observer = Arc::new(CancelRiptide(ReplacePlayerItemUse {
        player: player.clone(),
        trigger: GameEvent::ItemInteractFinish,
        calls: std::sync::atomic::AtomicU32::new(0),
        replace: true,
    }));
    server
        .plugin_manager
        .register_blocking::<PlayerRiptideEvent, _>(observer.clone(), EventPriority::Normal, None);
    send_java_item_release(&player, &server)?;
    assert_eq!(observer.0.calls.load(Ordering::Relaxed), 1);
    observer.0.assert_successor();
    assert_eq!(
        player
            .inventory()
            .get_stack_in_hand(Hand::Right)
            .get_damage(),
        0
    );
    world.level.shutdown().await;
    Ok(())
}

struct ReplaceBedrockAttribute {
    pig: Arc<PigEntity>,
    called: AtomicBool,
}

impl EventRwHandler<PacketSentEvent> for ReplaceBedrockAttribute {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use crate::entity::{attributes::send_attribute_updates_for_living, mob::Mob};
        use pumpkin_data::attributes::Attributes;
        use pumpkin_protocol::{
            bedrock::client::update_attributes::CUpdateAttributes,
            packet::Packet,
            serial::{PacketRead, PacketWrite},
        };
        Box::pin(async move {
            if event.player.client.bedrock().is_none()
                || event.packet_id != CUpdateAttributes::PACKET_ID
            {
                return;
            }
            let Ok(mut original) = CUpdateAttributes::read(&mut event.payload.as_ref()) else {
                return;
            };
            if original.target_runtime_id.0 != self.pig.get_entity().entity_id as u64
                || self.called.swap(true, Ordering::Relaxed)
            {
                return;
            }
            let living = &self.pig.get_mob_entity().living_entity;
            living.set_attribute_base(&Attributes::ATTACK_DAMAGE, 13.0);
            send_attribute_updates_for_living(living, vec![Attributes::ATTACK_DAMAGE]);
            // Include the superseded attack value alongside an unrelated override.
            if let Some(attack) = original.attribute_list.first() {
                original.attribute_list.push(
                    pumpkin_protocol::bedrock::client::update_attributes::AttributeData {
                        min_value: attack.min_value,
                        max_value: 20.0,
                        current_value: 19.0,
                        default_min_value: attack.default_min_value,
                        default_max_value: 20.0,
                        default_value: 20.0,
                        name: "minecraft:health".into(),
                        modifiers: Vec::new(),
                    },
                );
            }
            let mut extra = Vec::new();
            assert!(original.write(&mut extra).is_ok());
            event.extra_packets.push((event.packet_id, extra.into()));
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_shield_attribute_guard_keeps_an_unrelated_observer_override() -> TestResult {
    use crate::entity::{attributes::send_guarded_attribute_updates, mob::Mob};
    use pumpkin_data::attributes::Attributes;
    use pumpkin_protocol::{
        bedrock::client::update_attributes::CUpdateAttributes, packet::Packet, serial::PacketRead,
    };
    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (_viewer, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    let pig = PigEntity::new(Entity::new(world.clone(), CENTER, &EntityType::PIG));
    world.spawn_entity(pig.clone());
    let observer = Arc::new(ReplaceBedrockAttribute {
        pig: pig.clone(),
        called: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);

    let living = &pig.get_mob_entity().living_entity;
    living.set_attribute_base(&Attributes::ATTACK_DAMAGE, 3.0);
    send_guarded_attribute_updates(living, &[Attributes::ATTACK_DAMAGE]);

    let mut attack = None;
    let mut health = None;
    while let Ok(packet) = packets.try_recv() {
        let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data) else {
            continue;
        };
        if i32::from(id) == CUpdateAttributes::PACKET_ID {
            let update = CUpdateAttributes::read(&mut payload.as_ref())?;
            if update.target_runtime_id.0 == pig.get_entity().entity_id as u64 {
                for attribute in update.attribute_list {
                    match attribute.name.as_str() {
                        "minecraft:attack_damage" => attack = Some(attribute.current_value),
                        "minecraft:health" => health = Some(attribute.current_value),
                        _ => {}
                    }
                }
            }
        }
    }
    assert!(observer.called.load(Ordering::Relaxed));
    assert_eq!(attack, Some(13.0));
    assert_eq!(health, Some(19.0));
    world.level.shutdown().await;
    Ok(())
}

async fn inventory_equipment_seen_by(
    watcher: &crate::entity::player::Player,
    peer: tokio::io::DuplexStream,
    actor_id: i32,
) -> Result<Vec<(i8, u16)>, Box<dyn std::error::Error>> {
    use pumpkin_protocol::java::client::play::CKeepAlive;
    assert!(
        watcher
            .client
            .java()
            .ok_or("Java watcher")?
            .try_send_packet(&CKeepAlive::new(73))
    );
    let (updates, _) = read_equipment_updates(peer, actor_id).await?;
    Ok(updates
        .iter()
        .map(|(slot, stack)| (*slot, stack.item.id))
        .collect())
}

struct EquipmentEvents(std::sync::atomic::AtomicUsize);
impl EventRwHandler<GenericGameEvent> for EquipmentEvents {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut GenericGameEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.event_key == GameEvent::Equip.name()
                || event.event_key == GameEvent::Unequip.name()
            {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_player_inventory_publishes_committed_armor_and_creative_hand() -> TestResult {
    use pumpkin_data::{
        attributes::Attributes, data_component_impl::EquipmentSlot, item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_inventory::screen_handler::ScreenHandler;
    use pumpkin_protocol::{
        codec::item_stack_seralizer::ItemStackSerializer,
        java::server::play::{SSetCreativeSlot, SlotActionType},
    };
    use pumpkin_util::GameMode;
    use std::sync::atomic::AtomicUsize;

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _player_peer) = java_player(&server, &world, "jeb_")?;
    let (watcher, peer) = java_equipment_watcher(&server, &world)?;
    let entity: Arc<dyn EntityBase> = player.clone();
    player.get_entity().set_pos(CENTER);
    world.entity_tracker.add_entity(&entity, &world);
    world
        .entity_tracker
        .get_tracked_entity(player.entity_id())
        .ok_or("Player tracker")?
        .add_pairing(&watcher);

    let chestplate = ItemStack::new(1, &Item::DIAMOND_CHESTPLATE);
    player
        .get_entity()
        .first_tick
        .store(false, Ordering::Relaxed);
    let equipment_events = Arc::new(EquipmentEvents(AtomicUsize::new(0)));
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(
            equipment_events.clone(),
            EventPriority::Normal,
            None,
        );
    let armor = || player.living_entity.get_attribute_value(&Attributes::ARMOR);
    assert_eq!(armor(), 0.0);
    *player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour()
        .cursor_stack
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = chestplate;
    for (mode, expected_armor) in [
        (SlotActionType::Pickup, 8.0),
        (SlotActionType::QuickMove, 0.0),
    ] {
        player.on_slot_click(player_inventory_click(6, mode), &server);
        assert_eq!(armor(), expected_armor);
    }
    assert_eq!(equipment_events.0.load(Ordering::Relaxed), 2);

    assert_eq!(player.inventory().get_selected_slot(), 0);
    player
        .inventory()
        .set_held_item(ItemStack::new(1, &Item::DIRT));
    assert_eq!(player.inventory().held_item().item, &Item::DIRT);
    player.on_slot_click(player_inventory_click(36, SlotActionType::Pickup), &server);
    assert!(player.inventory().held_item().is_empty());
    assert_eq!(equipment_events.0.load(Ordering::Relaxed), 2);
    player.gamemode.store(GameMode::Creative);
    player
        .client
        .java()
        .ok_or("Java player")?
        .handle_set_creative_slot(
            &player,
            SSetCreativeSlot::new(
                36,
                ItemStackSerializer::from(ItemStack::new(1, &Item::DIAMOND_SWORD)),
            ),
        )?;
    assert_eq!(
        player
            .living_entity
            .get_attribute_value(&Attributes::ATTACK_DAMAGE),
        7.0
    );
    let equipment = inventory_equipment_seen_by(&watcher, peer, player.entity_id()).await?;
    assert_eq!(
        equipment,
        [
            (
                EquipmentSlot::CHEST.discriminant(),
                Item::DIAMOND_CHESTPLATE.id
            ),
            (EquipmentSlot::CHEST.discriminant(), Item::AIR.id),
            (EquipmentSlot::MAIN_HAND.discriminant(), Item::AIR.id),
            (
                EquipmentSlot::MAIN_HAND.discriminant(),
                Item::DIAMOND_SWORD.id
            ),
        ]
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[expect(
    clippy::too_many_lines,
    reason = "The same packet sequence verifies the failed response and committed equipment."
)]
async fn bedrock_player_inventory_publishes_committed_armor() -> TestResult {
    use pumpkin_data::{
        attributes::Attributes, data_component_impl::EquipmentSlot, item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_protocol::bedrock::{
        client::item_stack_response::CItemStackResponse,
        network_item::{ContainerName, FullContainerName},
        server::{
            inventory_transaction::{
                InventoryAction, NormalTransactionData, SInventoryTransaction, TransactionData,
                WINDOW_ID_ARMOUR,
            },
            item_stack_request::{
                ItemStackRequest, ItemStackRequestAction, ItemStackRequestSlotInfo,
                SItemStackRequest,
            },
        },
    };
    use pumpkin_protocol::{Packet, codec::var_uint::VarUInt, serial::PacketRead};

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut player_packets) = bedrock_player(&server, &world, "jeb_").await?;
    let (watcher, peer) = java_equipment_watcher(&server, &world)?;
    let entity: Arc<dyn EntityBase> = player.clone();
    world.entity_tracker.add_entity(&entity, &world);
    world
        .entity_tracker
        .get_tracked_entity(player.entity_id())
        .ok_or("Player tracker")?
        .add_pairing(&watcher);

    let chestplate = ItemStack::new(1, &Item::DIAMOND_CHESTPLATE);
    let client = player.client.bedrock().ok_or("Bedrock player")?;
    let armor = || player.living_entity.get_attribute_value(&Attributes::ARMOR);
    assert_eq!(armor(), 0.0);
    client.handle_inventory_action(
        &player,
        SInventoryTransaction {
            legacy_request_id: 0.into(),
            legacy_set_item_slots: Vec::new(),
            has_value: true,
            actions: vec![InventoryAction {
                source_type: 0,
                window_id: Some(WINDOW_ID_ARMOUR),
                source_flags: None,
                inventory_slot: 1,
                old_item: ItemStack::EMPTY.into(),
                new_item: (&chestplate).into(),
            }],
            transaction_type: pumpkin_protocol::codec::var_uint::VarUInt(0),
            transaction_data: TransactionData::Normal(NormalTransactionData),
        },
    );
    assert_eq!(armor(), 8.0);
    let chest_index = player
        .inventory()
        .equipment_slot_index(&EquipmentSlot::CHEST)
        .ok_or("Chest slot")?;
    let chest = player.inventory().get_slot(chest_index);
    assert_eq!(chest.item, &Item::DIAMOND_CHESTPLATE);
    let destroy = || ItemStackRequestAction::Destroy {
        count: 1,
        source: ItemStackRequestSlotInfo {
            container_name: FullContainerName {
                container_name: ContainerName::Armor,
                dynamic_id: None,
            },
            slot_id: 1,
            stack_id: chest.uid.get(),
        },
    };
    while player_packets.try_recv().is_ok() {}
    client.handle_item_stack_request(
        &player,
        SItemStackRequest {
            requests: vec![ItemStackRequest {
                request_id: (-1).into(),
                // The first action removes the armor and the repeated action fails.
                actions: vec![destroy(), destroy()],
                filter_strings: Vec::new(),
                filter_cause: 0,
            }],
        },
    );
    assert_eq!(armor(), 0.0);
    let mut acknowledged_failure = false;
    while let Ok(packet) = player_packets.try_recv() {
        if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
            && i32::from(id) == CItemStackResponse::PACKET_ID
        {
            let mut payload = payload.as_ref();
            assert_eq!(VarUInt::read(&mut payload)?.0, 1);
            assert_eq!(u8::read(&mut payload)?, 1);
            acknowledged_failure = true;
        }
    }
    assert!(acknowledged_failure);
    let equipment = inventory_equipment_seen_by(&watcher, peer, player.entity_id()).await?;
    assert_eq!(
        equipment,
        [
            (
                EquipmentSlot::CHEST.discriminant(),
                Item::DIAMOND_CHESTPLATE.id
            ),
            (EquipmentSlot::CHEST.discriminant(), Item::AIR.id),
        ]
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_bow_finish_uses_the_current_stack_vibration_setting() -> TestResult {
    use pumpkin_data::{data_component_impl::UseEffectsImpl, item::Item, item_stack::ItemStack};
    use pumpkin_util::Hand;

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    let observer = Arc::new(ReplacePlayerItemUse {
        player: player.clone(),
        trigger: GameEvent::ItemInteractFinish,
        calls: std::sync::atomic::AtomicU32::new(0),
        replace: false,
    });
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(observer.clone(), EventPriority::Normal, None);

    let bow = ItemStack::new(1, &Item::BOW);
    let mut quiet = bow.clone();
    quiet.set_data_component(UseEffectsImpl {
        interact_vibrations: false,
        ..UseEffectsImpl::DEFAULT
    });
    for (old, current, expected) in [(&bow, &quiet, 0), (&quiet, &bow, 1)] {
        player.inventory().set_held_item(old.clone());
        player
            .living_entity
            .set_active_hand(Hand::Right, old.clone(), 71_980);
        player.inventory().set_held_item(current.clone());
        send_java_item_release(&player, &server)?;
        assert_eq!(observer.calls.load(Ordering::Relaxed), expected);
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_rejected_equipment_swap_does_not_start_blocking() -> TestResult {
    use pumpkin_data::{
        data_component_impl::{BlocksAttacksImpl, EnchantmentsImpl, EquipmentSlot, EquippableImpl},
        item::Item,
        item_stack::ItemStack,
    };
    use pumpkin_protocol::java::server::play::SUseItem;

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    let mut helmet = ItemStack::new(1, &Item::IRON_HELMET);
    helmet.set_data_component(EnchantmentsImpl {
        enchantment: vec![(&pumpkin_data::Enchantment::BINDING_CURSE, 1)].into(),
    });
    assert!(helmet.has_prevent_armor_change());
    let slot = player
        .inventory()
        .equipment_slot_index(&EquipmentSlot::HEAD)
        .ok_or("Head slot")?;
    player.inventory().set_slot(slot, helmet.clone());
    let mut held = ItemStack::new(1, &Item::FLINT_AND_STEEL);
    held.set_data_component(BlocksAttacksImpl::DEFAULT);
    held.set_data_component(
        helmet
            .get_data_component::<EquippableImpl>()
            .ok_or("Helmet use component")?
            .clone(),
    );
    player.inventory().set_held_item(held.clone());

    player.client.java().ok_or("Java player")?.handle_use_item(
        &player,
        &SUseItem {
            hand: 0.into(),
            sequence: 1.into(),
            yaw: 0.0,
            pitch: 0.0,
        },
        &server,
    );
    assert!(player.inventory().get_slot(slot).are_equal(&helmet));
    assert!(player.inventory().held_item().are_equal(&held));
    assert!(
        player
            .living_entity
            .item_in_use
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_none()
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hand_swapping_and_equipment_validation_share_the_inventory_lock_order() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_util::Hand;
    use std::sync::{TryLockError, mpsc};
    use std::time::{Duration, Instant};

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    let inventory = player.inventory();
    inventory.set_held_item(ItemStack::new(1, &Item::STICK));
    inventory.set_stack_in_hand(Hand::Left, ItemStack::new(1, &Item::SHIELD));

    let equipment = player
        .living_entity
        .entity_equipment
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    let (finished, completion) = mpsc::channel();
    let worker = player.clone();
    std::thread::spawn(move || {
        worker.swap_item();
        let _ = finished.send(());
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    let acquired_main_first = loop {
        if matches!(
            inventory.main_inventory.try_read(),
            Err(TryLockError::WouldBlock)
        ) {
            break true;
        }
        if Instant::now() >= deadline {
            break false;
        }
        std::thread::yield_now();
    };
    drop(equipment);
    completion.recv_timeout(Duration::from_secs(5))?;
    assert!(
        acquired_main_first,
        "swap must take the main inventory before equipment"
    );
    assert_eq!(inventory.held_item().item, &Item::SHIELD);
    assert_eq!(inventory.get_stack_in_hand(Hand::Left).item, &Item::STICK);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_creative_equipment_observer_can_close_the_same_screen() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_protocol::codec::item_stack_seralizer::ItemStackSerializer;
    use pumpkin_protocol::java::client::play::CSetEquipment;
    use pumpkin_protocol::java::server::play::SSetCreativeSlot;

    struct CloseOnEquipment(Arc<crate::entity::player::Player>, AtomicBool);
    impl EventRwHandler<PacketSentEvent> for CloseOnEquipment {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CSetEquipment::to_id(CURRENT_MC_VERSION)
                    && event
                        .payload
                        .as_ref()
                        .get_var_int()
                        .is_ok_and(|id| id.0 == self.0.entity_id())
                    && !self.1.swap(true, Ordering::Relaxed)
                {
                    assert!(
                        self.0.player_screen_handler.try_lock().is_ok(),
                        "equipment observers must run outside the screen handler"
                    );
                    self.0.close_handled_screen();
                }
            })
        }
    }
    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_player(&server, &world, "jeb_")?;
    let (watcher, _wire) = java_equipment_watcher(&server, &world)?;
    let entity: Arc<dyn EntityBase> = player.clone();
    world.entity_tracker.add_entity(&entity, &world);
    world
        .entity_tracker
        .get_tracked_entity(player.entity_id())
        .ok_or("Player tracker")?
        .add_pairing(&watcher);
    let close = Arc::new(CloseOnEquipment(player.clone(), AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(close.clone(), EventPriority::Normal, None);
    player.gamemode.store(pumpkin_util::GameMode::Creative);
    player
        .client
        .java()
        .ok_or("Java player")?
        .handle_set_creative_slot(
            &player,
            SSetCreativeSlot::new(
                6,
                ItemStackSerializer::from(ItemStack::new(1, &Item::DIAMOND_CHESTPLATE)),
            ),
        )?;
    assert!(close.1.load(Ordering::Relaxed));
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_successful_armor_swap_starts_the_original_stack_cooldown() -> TestResult {
    use pumpkin_data::{data_component_impl::UseCooldownImpl, item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::server::play::SUseItem;

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    let mut helmet = ItemStack::new(1, &Item::IRON_HELMET);
    helmet.set_data_component(UseCooldownImpl {
        seconds: 5.0,
        cooldown_group: Some("minecraft:test_helmets".to_owned()),
    });
    let mut second = ItemStack::new(1, &Item::GOLDEN_HELMET);
    second.set_data_component(
        helmet
            .get_use_cooldown()
            .ok_or("Expected cooldown")?
            .clone(),
    );
    let head = player
        .inventory()
        .equipment_slot_index(&pumpkin_data::data_component_impl::EquipmentSlot::HEAD)
        .ok_or("Head slot")?;
    let client = player.client.java().ok_or("Java player")?;
    for (stack, sequence) in [(helmet, 1), (second, 2)] {
        player.inventory().set_held_item(stack);
        client.handle_use_item(
            &player,
            &SUseItem {
                hand: 0.into(),
                sequence: sequence.into(),
                yaw: 0.0,
                pitch: 0.0,
            },
            &server,
        );
        assert!(player.is_on_cooldown("minecraft:test_helmets"));
        assert_eq!(player.inventory().get_slot(head).item, &Item::IRON_HELMET);
    }
    assert_eq!(player.inventory().held_item().item, &Item::GOLDEN_HELMET);
    world.level.shutdown().await;
    Ok(())
}

use crate::plugin::api::events::player::{
    player_advancement_done::PlayerAdvancementDoneEvent, player_item_break::PlayerItemBreakEvent,
    player_item_damage::PlayerItemDamageEvent,
};
use std::sync::atomic::AtomicU32;

struct ObserveDurability {
    replace: AtomicBool,
    breaks: AtomicU32,
    awards: AtomicU32,
}

impl EventRwHandler<PlayerItemDamageEvent> for ObserveDurability {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerItemDamageEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if self.replace.load(Ordering::Relaxed) {
                event
                    .player
                    .inventory()
                    .set_held_item(pumpkin_data::item_stack::ItemStack::new(
                        1,
                        &pumpkin_data::item::Item::STICK,
                    ));
            }
        })
    }
}
impl EventRwHandler<PlayerItemBreakEvent> for ObserveDurability {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        _event: &'a mut PlayerItemBreakEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            self.breaks.fetch_add(1, Ordering::Relaxed);
        })
    }
}
impl EventRwHandler<PlayerAdvancementDoneEvent> for ObserveDurability {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerAdvancementDoneEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.advancement_id.ends_with("nether/ride_strider") {
                self.awards.fetch_add(1, Ordering::Relaxed);
            }
        })
    }
}

fn listen_for_durability_criterion(
    player: &Arc<crate::entity::player::Player>,
    advancement: &'static pumpkin_data::Advancement,
) {
    let mut progress = player
        .advancements
        .lock()
        .unwrap_or_else(PoisonError::into_inner);
    progress.set_player(player);
    assert!(
        !progress
            .progress
            .get_mut_or_start_progress(advancement)
            .is_done()
    );
    progress.register_listeners(advancement);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn durability_break_does_not_animate_a_newly_selected_item() -> TestResult {
    use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::client::play::CEntityStatus;

    #[derive(Default)]
    struct SwitchOnDamage {
        breaks: AtomicU32,
        animations: AtomicU32,
    }
    impl EventRwHandler<PlayerItemDamageEvent> for SwitchOnDamage {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PlayerItemDamageEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                event.player.inventory().set_selected_slot(1);
            })
        }
    }
    impl EventRwHandler<PlayerItemBreakEvent> for SwitchOnDamage {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            _: &'a mut PlayerItemBreakEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.breaks.fetch_add(1, Ordering::Relaxed);
            })
        }
    }
    impl EventRwHandler<PacketSentEvent> for SwitchOnDamage {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CEntityStatus::to_id(CURRENT_MC_VERSION)
                    && event.payload.get(4)
                        == Some(
                            &(crate::entity::equipment_break_status(&EquipmentSlot::MAIN_HAND)
                                as u8),
                        )
                {
                    self.animations.fetch_add(1, Ordering::Relaxed);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    let actor: Arc<dyn EntityBase> = player.clone();
    world.entity_tracker.add_entity(&actor, &world);
    let observer = Arc::new(SwitchOnDamage::default());
    server
        .plugin_manager
        .register_blocking::<PlayerItemDamageEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            None,
        );
    server
        .plugin_manager
        .register_blocking::<PlayerItemBreakEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            None,
        );
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    world.send_entity_status(
        player.get_entity(),
        crate::entity::equipment_break_status(&EquipmentSlot::MAIN_HAND),
        pumpkin_protocol::bedrock::server::actor_event::ActorEventID::None,
    );
    assert_eq!(observer.animations.swap(0, Ordering::Relaxed), 1);
    let mut rod = ItemStack::new(1, &Item::WARPED_FUNGUS_ON_A_STICK);
    rod.set_damage(rod.get_max_damage().ok_or("Rod durability")? - 1);
    player.inventory().set_selected_slot(0);
    player.inventory().set_slot(0, rod.clone());
    player
        .inventory()
        .set_slot(1, ItemStack::new(1, &Item::STICK));
    assert!(player.damage_item_in_slot_matching(&EquipmentSlot::MAIN_HAND, 0, &rod, 1));
    assert!(player.inventory().get_slot(0).is_empty());
    assert_eq!(player.inventory().held_item().item, &Item::STICK);
    assert_eq!(observer.breaks.load(Ordering::Relaxed), 1);
    assert_eq!(observer.animations.load(Ordering::Relaxed), 0);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rejected_durability_does_not_emit_break_or_award_ride_criterion() -> TestResult {
    use pumpkin_data::{
        Advancement, data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack,
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    let strider: Arc<dyn EntityBase> =
        Arc::new(Entity::new(world.clone(), CENTER, &EntityType::STRIDER));
    strider
        .get_entity()
        .add_passenger(strider.clone(), player.clone());
    let advancement = Advancement::NETHER_RIDE_STRIDER;
    listen_for_durability_criterion(&player, advancement);
    let observer = Arc::new(ObserveDurability {
        replace: AtomicBool::new(true),
        breaks: AtomicU32::new(0),
        awards: AtomicU32::new(0),
    });
    server
        .plugin_manager
        .register_blocking::<PlayerItemDamageEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            None,
        );
    server
        .plugin_manager
        .register_blocking::<PlayerItemBreakEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            None,
        );
    server
        .plugin_manager
        .register_blocking::<PlayerAdvancementDoneEvent, _>(
            observer.clone(),
            EventPriority::Normal,
            None,
        );

    let mut rod = ItemStack::new(1, &Item::WARPED_FUNGUS_ON_A_STICK);
    rod.set_damage(rod.get_max_damage().ok_or("Rod durability")? - 1);
    player.inventory().set_held_item(rod.clone());
    assert!(!player.damage_item_in_slot_matching(&EquipmentSlot::MAIN_HAND, 0, &rod, 1));
    assert_eq!(player.inventory().held_item().item, &Item::STICK);
    assert_eq!(
        observer.breaks.load(Ordering::Relaxed),
        0,
        "rejected damage emitted a break"
    );
    assert_eq!(
        observer.awards.load(Ordering::Relaxed),
        0,
        "rejected damage awarded the criterion"
    );

    observer.replace.store(false, Ordering::Relaxed);
    player.inventory().set_held_item(rod.clone());
    assert!(player.damage_item_in_slot_matching(&EquipmentSlot::MAIN_HAND, 0, &rod, 1));
    assert_eq!(observer.breaks.load(Ordering::Relaxed), 1);
    assert_eq!(observer.awards.load(Ordering::Relaxed), 1);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_container_resync_observer_can_close_the_screen() -> TestResult {
    use crate::plugin::player::inventory_interact::InventoryClickEvent;
    use pumpkin_inventory::screen_handler::ScreenHandler;
    use pumpkin_protocol::{
        codec::item_stack_seralizer::OptionalItemStackHash,
        java::{
            client::play::CSetContainerContent,
            server::play::{SClickSlot, SlotActionType},
        },
    };

    struct CloseOnResync(Arc<crate::entity::player::Player>, AtomicBool);
    struct CancelClick(AtomicBool);

    impl EventRwHandler<InventoryClickEvent> for CancelClick {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut InventoryClickEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                event.cancelled = self.0.load(Ordering::Relaxed);
            })
        }
    }
    impl EventRwHandler<PacketSentEvent> for CloseOnResync {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CSetContainerContent::to_id(CURRENT_MC_VERSION)
                    && !self.1.swap(true, Ordering::Relaxed)
                {
                    assert!(self.0.player_screen_handler.try_lock().is_ok());
                    self.0.close_handled_screen();
                }
            })
        }
    }
    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _wire) = java_equipment_watcher(&server, &world)?;
    let sync = player.screen_handler_sync_handler.clone();
    sync.store_player(player.clone());
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .update_sync_handler(sync);
    let observer = Arc::new(CloseOnResync(player.clone(), AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    let cancel = Arc::new(CancelClick(AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<InventoryClickEvent, _>(cancel.clone(), EventPriority::Normal, None);
    let click = || SClickSlot {
        sync_id: 0.into(),
        revision: 999.into(),
        slot: 9,
        button: SClickSlot::BUTTON_LEFT,
        mode: SlotActionType::Pickup,
        length_of_array: 0.into(),
        array_of_changed_slots: Vec::new(),
        carried_item: OptionalItemStackHash(None),
    };
    for (gamemode, cancelled) in [
        (pumpkin_util::GameMode::Survival, false),
        (pumpkin_util::GameMode::Survival, true),
        (pumpkin_util::GameMode::Spectator, false),
    ] {
        observer.1.store(false, Ordering::Relaxed);
        cancel.0.store(cancelled, Ordering::Relaxed);
        player.gamemode.store(gamemode);
        player.on_slot_click(click(), &server);
        assert!(
            observer.1.load(Ordering::Relaxed),
            "{gamemode:?}, cancelled={cancelled}"
        );
    }
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_container_resync_keeps_the_newest_wire_revision() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::screen_handler::ScreenHandler;
    use pumpkin_protocol::java::{
        client::play::{CKeepAlive, CSetContainerContent},
        packet_decoder::TCPNetworkDecoder,
    };

    struct ChangeOnContent(Arc<crate::entity::player::Player>, AtomicBool);
    impl EventRwHandler<PacketSentEvent> for ChangeOnContent {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CSetContainerContent::to_id(CURRENT_MC_VERSION)
                    && !self.1.swap(true, Ordering::Relaxed)
                {
                    assert!(self.0.player_screen_handler.try_lock().is_ok());
                    self.0
                        .inventory()
                        .set_held_item(ItemStack::new(1, &Item::DIAMOND));
                    self.0.resync_active_inventory_screen();
                }
            })
        }
    }
    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, peer) = java_equipment_watcher(&server, &world)?;
    let sync = player.screen_handler_sync_handler.clone();
    sync.store_player(player.clone());
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour_mut()
        .sync_handler = Some(sync);
    let listener = Arc::new(ChangeOnContent(player.clone(), AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(listener.clone(), EventPriority::Normal, None);
    player.resync_active_inventory_screen();
    assert!(listener.1.load(Ordering::Relaxed));
    assert_eq!(player.inventory().held_item().item, &Item::DIAMOND);
    assert!(
        player
            .client
            .java()
            .ok_or("Java player")?
            .try_send_packet(&CKeepAlive::new(73))
    );
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    let revisions = tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut revisions = Vec::new();
        loop {
            let packet = decoder.get_raw_packet().await?;
            let mut payload = packet.payload.as_ref();
            if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                return Ok::<_, Box<dyn std::error::Error>>(revisions);
            }
            if packet.id == CSetContainerContent::to_id(CURRENT_MC_VERSION) {
                assert_eq!(payload.get_container_id(&CURRENT_MC_VERSION)?.0, 0);
                revisions.push(payload.get_var_int()?.0);
            }
        }
    })
    .await??;
    let revision = player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour()
        .revision
        .load(Ordering::Relaxed) as i32;
    assert_eq!(
        revisions,
        [revision],
        "the nested screen update owns the wire"
    );
    world.level.shutdown().await;
    Ok(())
}

fn player_inventory_click(
    slot: i16,
    mode: pumpkin_protocol::java::server::play::SlotActionType,
) -> pumpkin_protocol::java::server::play::SClickSlot {
    pumpkin_protocol::java::server::play::SClickSlot {
        sync_id: 0.into(),
        revision: 0.into(),
        slot,
        button: 0,
        mode,
        length_of_array: 0.into(),
        array_of_changed_slots: Vec::new(),
        carried_item: pumpkin_protocol::codec::item_stack_seralizer::OptionalItemStackHash(None),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_nested_menu_open_owns_the_handler_and_the_initial_packets() -> TestResult {
    use pumpkin_inventory::{
        generic_container_screen_handler::create_generic_9x3, inventory::SimpleInventory,
        screen_handler::SharedScreenHandler,
    };
    use pumpkin_protocol::java::{
        client::play::{CKeepAlive, COpenScreen, CSetContainerContent},
        packet_decoder::TCPNetworkDecoder,
    };
    use pumpkin_util::text::TextComponent;

    struct OpenAnother {
        player: Arc<crate::entity::player::Player>,
        menu: SharedScreenHandler,
        packet_id: i32,
        ran: AtomicBool,
    }

    impl EventRwHandler<PacketSentEvent> for OpenAnother {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == self.packet_id && !self.ran.swap(true, Ordering::Relaxed) {
                    self.player.open_handled_screen_direct(
                        self.menu.clone(),
                        &TextComponent::text("Second"),
                    );
                }
            })
        }
    }

    let open_id = COpenScreen::to_id(CURRENT_MC_VERSION);
    let content_id = CSetContainerContent::to_id(CURRENT_MC_VERSION);
    for (trigger, expected_opens) in [(open_id, &[2][..]), (content_id, &[1, 2][..])] {
        let folder = tempfile::tempdir()?;
        let (server, world) = inventory_world(&folder).await?;
        let (player, peer) = java_equipment_watcher(&server, &world)?;
        player
            .screen_handler_sync_handler
            .store_player(player.clone());
        let create_menu = |id| -> SharedScreenHandler {
            Arc::new(Mutex::new(create_generic_9x3(
                id,
                player.inventory(),
                Arc::new(SimpleInventory::new(27)),
                player.as_ref(),
            )))
        };
        let second = create_menu(2);
        let observer = Arc::new(OpenAnother {
            player: player.clone(),
            menu: second.clone(),
            packet_id: trigger,
            ran: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        player.open_handled_screen_direct(create_menu(1), &TextComponent::text("First"));
        assert!(observer.ran.load(Ordering::Relaxed));
        assert!(player.is_active_screen(&second));
        assert!(
            player
                .client
                .java()
                .ok_or("Java player")?
                .try_send_packet(&CKeepAlive::new(73))
        );

        let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
        let (opens, contents) = tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let mut opens = Vec::new();
            let mut contents = Vec::new();
            loop {
                let packet = decoder.get_raw_packet().await?;
                let mut payload = packet.payload.as_ref();
                if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                    return Ok::<_, Box<dyn std::error::Error>>((opens, contents));
                }
                if packet.id == open_id {
                    opens.push(payload.get_var_int()?.0);
                } else if packet.id == content_id {
                    let screen = payload.get_container_id(&CURRENT_MC_VERSION)?.0;
                    assert_eq!(opens.last().copied(), Some(screen));
                    contents.push(screen);
                }
            }
        })
        .await??;
        assert_eq!(opens.as_slice(), expected_opens);
        assert_eq!(contents, [2]);
        world.level.shutdown().await;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_initial_personal_menu_delivers_the_crafting_input_and_cursor() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::screen_handler::ScreenHandler;
    use pumpkin_protocol::{
        Packet,
        bedrock::{
            client::{inventory_content::CInventoryContent, inventory_slot::CInventorySlot},
            network_item::{ContainerName, FullContainerName, NetworkItemStackDescriptor},
        },
        codec::var_uint::VarUInt,
        serial::PacketRead,
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    player
        .screen_handler_sync_handler
        .store_player(player.clone());
    let ingredient = ItemStack::new(1, &Item::OAK_PLANKS);
    let cursor = ItemStack::new(1, &Item::DIAMOND);
    let cursor_stack = {
        let handler = player
            .player_screen_handler
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        handler.get_slot(1).set_stack(ingredient.clone());
        handler.get_behaviour().cursor_stack.clone()
    };
    *cursor_stack.lock().unwrap_or_else(PoisonError::into_inner) = cursor.clone();

    player.on_screen_handler_opened(&player.player_screen_handler);

    let mut crafting_input = false;
    let mut carried_item = false;
    while let Ok(packet) = packets.try_recv() {
        let Some((id, data)) = crate::net::bedrock::split_raw_packet(&packet.data) else {
            continue;
        };
        let mut data = data.as_ref();
        if i32::from(id) == CInventorySlot::PACKET_ID
            && VarUInt::read(&mut data)?.0 == 124
            && VarUInt::read(&mut data)?.0 == 28
        {
            let name =
                Option::<FullContainerName>::read(&mut data)?.ok_or("crafting input container")?;
            assert_eq!(name.container_name, ContainerName::CraftingInput);
            Option::<NetworkItemStackDescriptor>::read(&mut data)?;
            let stack = NetworkItemStackDescriptor::read(&mut data)?;
            assert_eq!(stack.net_id, Some(ingredient.uid));
            crafting_input = true;
        } else if i32::from(id) == CInventoryContent::PACKET_ID && VarUInt::read(&mut data)?.0 == 59
        {
            assert_eq!(VarUInt::read(&mut data)?.0, 1);
            let stack = NetworkItemStackDescriptor::read(&mut data)?;
            assert_eq!(stack.net_id, Some(cursor.uid));
            carried_item = true;
        }
    }
    assert!(crafting_input);
    assert!(carried_item);
    world.level.shutdown().await;
    Ok(())
}

#[derive(Default)]
struct SeeCraft(AtomicBool);
impl EventRwHandler<crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent> for SeeCraft {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.statistic_id.starts_with("Crafted:") {
                assert!(event.player.player_screen_handler.try_lock().is_ok());
                self.0.store(true, Ordering::Relaxed);
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_container_drop_observer_sees_the_committed_slot() -> TestResult {
    use crate::plugin::api::events::entity::entity_spawn::EntitySpawnEvent;
    use crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::server::play::SlotActionType;

    struct SeeDrop(Arc<crate::entity::player::Player>, AtomicBool, usize);
    impl EventRwHandler<EntitySpawnEvent> for SeeDrop {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntitySpawnEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.entity_type == EntityType::ITEM.id.to_string() {
                    let screen = self.0.player_screen_handler.try_lock();
                    assert!(screen.as_ref().is_ok_and(|handler| {
                        handler.get_slot(self.2).get_cloned_stack().is_empty()
                    }));
                    self.1.store(true, Ordering::Relaxed);
                }
            })
        }
    }

    for craft_overflow in [false, true] {
        let folder = tempfile::tempdir()?;
        let (server, world) = inventory_world(&folder).await?;
        let (player, _peer) = java_equipment_watcher(&server, &world)?;
        world.add_player(&player)?;
        let craft = Arc::new(SeeCraft::default());
        server
            .plugin_manager
            .register_blocking::<PlayerStatisticIncrementEvent, _>(
                craft.clone(),
                EventPriority::Normal,
                None,
            );
        let (click_slot, checked_slot, action) = {
            let screen = player
                .player_screen_handler
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            if craft_overflow {
                screen
                    .get_slot(1)
                    .set_stack(ItemStack::new(1, &Item::OAK_LOG));
                for slot in 9..45 {
                    screen
                        .get_slot(slot)
                        .set_stack(ItemStack::new(64, &Item::STONE));
                }
                screen
                    .get_slot(44)
                    .set_stack(ItemStack::new(63, &Item::OAK_PLANKS));
                let result = screen.get_slot(0);
                result.set_stack(ItemStack::EMPTY.clone());
                assert_eq!(result.get_cloned_stack().item, &Item::OAK_PLANKS);
                assert_eq!(result.get_cloned_stack().item_count, 4);
                (0, 1, SlotActionType::QuickMove)
            } else {
                screen
                    .get_slot(6)
                    .set_stack(ItemStack::new(1, &Item::DIAMOND_CHESTPLATE));
                (6, 6, SlotActionType::Throw)
            }
        };
        let observer = Arc::new(SeeDrop(
            player.clone(),
            AtomicBool::new(false),
            checked_slot,
        ));
        server
            .plugin_manager
            .register_blocking::<EntitySpawnEvent, _>(
                observer.clone(),
                EventPriority::Normal,
                None,
            );
        player.on_slot_click(player_inventory_click(click_slot, action), &server);
        assert!(observer.1.load(Ordering::Relaxed));
        assert_eq!(craft.0.load(Ordering::Relaxed), craft_overflow);
        world.level.shutdown().await;
    }
    Ok(())
}

struct ReplaceAcknowledgedArmor(Arc<crate::entity::player::Player>, AtomicBool);
impl EventRwHandler<PacketSentEvent> for ReplaceAcknowledgedArmor {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_data::{item::Item, item_stack::ItemStack};
        use pumpkin_protocol::{Packet, bedrock::client::item_stack_response::CItemStackResponse};

        Box::pin(async move {
            if event.packet_id == CItemStackResponse::PACKET_ID
                && !self.1.swap(true, Ordering::Relaxed)
            {
                assert!(
                    self.0
                        .player_screen_handler
                        .try_lock()
                        .map(|screen| {
                            screen
                                .get_slot(6)
                                .set_stack(ItemStack::new(1, &Item::IRON_CHESTPLATE));
                        })
                        .is_ok()
                );
            }
        })
    }
}
fn drop_and_armor_requests(
    stick: &pumpkin_data::item_stack::ItemStack,
    chestplate: &pumpkin_data::item_stack::ItemStack,
) -> pumpkin_protocol::bedrock::server::item_stack_request::SItemStackRequest {
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_protocol::bedrock::{
        network_item::{ContainerName, FullContainerName},
        server::item_stack_request::{
            ItemStackRequest, ItemStackRequestAction, ItemStackRequestSlotInfo, SItemStackRequest,
        },
    };

    let request = |id: i32, name: ContainerName, slot_id, stack: &ItemStack, drop_item| {
        let source = ItemStackRequestSlotInfo {
            container_name: FullContainerName {
                container_name: name,
                dynamic_id: None,
            },
            slot_id,
            stack_id: stack.uid.get(),
        };
        let action = if drop_item {
            ItemStackRequestAction::Drop {
                count: 1,
                source,
                randomly: false,
            }
        } else {
            ItemStackRequestAction::Destroy { count: 1, source }
        };
        ItemStackRequest {
            request_id: id.into(),
            actions: vec![action],
            filter_strings: Vec::new(),
            filter_cause: 0,
        }
    };
    SItemStackRequest {
        requests: vec![
            request(-3, ContainerName::Inventory, 0, stick, true),
            request(-1, ContainerName::Armor, 1, chestplate, false),
        ],
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_stack_requests_reject_a_retired_screen_owner() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::{
        generic_container_screen_handler::create_generic_9x3, inventory::SimpleInventory,
        screen_handler::ScreenHandler,
    };
    use pumpkin_protocol::{
        Packet,
        bedrock::client::{
            inventory_content::CInventoryContent, item_stack_response::CItemStackResponse,
        },
        codec::{var_int::VarInt, var_uint::VarUInt},
        serial::PacketRead,
    };
    use pumpkin_util::text::TextComponent;

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    let sync = player.screen_handler_sync_handler.clone();
    sync.store_player(player.clone());
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour_mut()
        .sync_handler = Some(sync);
    let stick = ItemStack::new(1, &Item::STICK);
    player.inventory().set_held_item(stick.clone());
    let batch = drop_and_armor_requests(&stick, &ItemStack::new(1, &Item::IRON_CHESTPLATE));

    let chest = create_generic_9x3(
        1,
        player.inventory(),
        Arc::new(SimpleInventory::new(27)),
        player.as_ref(),
    );
    player.open_handled_screen_direct(Arc::new(Mutex::new(chest)), &TextComponent::text("Chest"));
    let owner = player.screen_owner();
    assert!(player.is_active_screen(&owner.handler));
    player.close_handled_screen();
    assert!(!player.is_active_screen(&owner.handler));
    while packets.try_recv().is_ok() {}

    player
        .client
        .bedrock()
        .ok_or("Bedrock player")?
        .handle_item_stack_request_for_owner(&player, batch, &owner);
    assert_eq!(player.inventory().held_item().uid, stick.uid);

    let mut acknowledged = false;
    let mut resynced = false;
    while let Ok(packet) = packets.try_recv() {
        let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data) else {
            continue;
        };
        let mut payload = payload.as_ref();
        if i32::from(id) == CItemStackResponse::PACKET_ID {
            assert!(!acknowledged);
            assert_eq!(VarUInt::read(&mut payload)?.0, 2);
            for id in [-3, -1] {
                assert_eq!(u8::read(&mut payload)?, 1);
                assert_eq!(VarInt::read(&mut payload)?.0, id);
                assert!(!bool::read(&mut payload)?);
            }
            assert!(payload.is_empty());
            acknowledged = true;
        } else if i32::from(id) == CInventoryContent::PACKET_ID
            && VarUInt::read(&mut payload)?.0 == 0
        {
            assert!(acknowledged);
            resynced = true;
        }
    }
    assert!(acknowledged && resynced);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_player_inventory_rejects_a_superseded_acknowledgement() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::screen_handler::ScreenHandler;
    use pumpkin_protocol::{
        Packet,
        bedrock::client::item_stack_response::CItemStackResponse,
        codec::{var_int::VarInt, var_uint::VarUInt},
        serial::PacketRead,
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    let chestplate = ItemStack::new(1, &Item::DIAMOND_CHESTPLATE);
    let stick = ItemStack::new(1, &Item::STICK);
    player.inventory().set_held_item(stick.clone());
    let sync = player.screen_handler_sync_handler.clone();
    sync.store_player(player.clone());
    {
        let mut screen = player
            .player_screen_handler
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        screen.get_slot(6).set_stack(chestplate.clone());
        screen.get_behaviour_mut().sync_handler = Some(sync);
    };
    let observer = Arc::new(ReplaceAcknowledgedArmor(
        player.clone(),
        AtomicBool::new(false),
    ));
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    player
        .client
        .bedrock()
        .ok_or("Bedrock player")?
        .handle_item_stack_request(&player, drop_and_armor_requests(&stick, &chestplate));
    assert!(observer.1.load(Ordering::Relaxed));
    let mut statuses = Vec::new();
    while let Ok(packet) = packets.try_recv() {
        if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
            && i32::from(id) == CItemStackResponse::PACKET_ID
        {
            let mut data = payload.as_ref();
            assert_eq!(VarUInt::read(&mut data)?.0, 2);
            for _ in 0..2 {
                let status = u8::read(&mut data)?;
                statuses.push((VarInt::read(&mut data)?.0, status));
                assert!(!bool::read(&mut data)?);
            }
        }
    }
    assert_eq!(statuses, [(-3, 0), (-1, 1)]);
    assert!(player.inventory().held_item().is_empty());
    assert!(
        world
            .entities
            .load()
            .iter()
            .any(|entity| entity.get_entity().entity_type == &EntityType::ITEM)
    );
    world.level.shutdown().await;
    Ok(())
}

struct SupersedeInventoryPacket {
    player: Arc<crate::entity::player::Player>,
    changed: AtomicBool,
    cursor: bool,
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_player_inventory_ack_retires_a_pending_slot() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::screen_handler::ScreenHandler;
    use pumpkin_protocol::{
        Packet,
        bedrock::{
            client::{
                inventory_content::CInventoryContent, inventory_slot::CInventorySlot,
                item_stack_response::CItemStackResponse,
            },
            network_item::{ContainerName, FullContainerName, NetworkItemStackDescriptor},
        },
        codec::var_uint::VarUInt,
        serial::PacketRead,
    };

    struct DropOnSlot(Arc<crate::entity::player::Player>, AtomicBool);
    impl EventRwHandler<PacketSentEvent> for DropOnSlot {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CInventorySlot::PACKET_ID
                    && !self.1.swap(true, Ordering::Relaxed)
                {
                    let stick = self.0.inventory().held_item();
                    assert_eq!(stick.item, &Item::STICK);
                    let mut request = drop_and_armor_requests(&stick, &stick);
                    request.requests.truncate(1);
                    if let Some(client) = self.0.client.bedrock() {
                        client.handle_item_stack_request(&self.0, request);
                    }
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    let sync = player.screen_handler_sync_handler.clone();
    sync.store_player(player.clone());
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour_mut()
        .sync_handler = Some(sync);
    player.resync_active_inventory_screen();
    while packets.try_recv().is_ok() {}
    let observer = Arc::new(DropOnSlot(player.clone(), AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    let stick = ItemStack::new(1, &Item::STICK);
    let stale = player
        .client
        .bedrock()
        .ok_or("Bedrock player")?
        .serialize_packet(&CInventorySlot {
            container_id: 0u32.into(),
            slot: 0u32.into(),
            full_container_name: Some(FullContainerName {
                container_name: ContainerName::Inventory,
                dynamic_id: None,
            }),
            storage_item: None,
            item: (&stick).into(),
        })?;

    player.inventory().set_held_item(stick);
    player.sync_inventory_to_client();
    player.sync_inventory_to_client();
    assert!(observer.1.load(Ordering::Relaxed));
    assert!(player.inventory().held_item().is_empty());
    let mut acknowledged = false;
    let mut recovered = false;
    while let Ok(packet) = packets.try_recv() {
        assert_ne!(
            packet.data, stale,
            "acknowledged drop must retire the pending stick"
        );
        if let Some((id, data)) = crate::net::bedrock::split_raw_packet(&packet.data) {
            let mut data = data.as_ref();
            if i32::from(id) == CItemStackResponse::PACKET_ID {
                acknowledged = true;
            } else if i32::from(id) == CInventoryContent::PACKET_ID
                && VarUInt::read(&mut data)?.0 == 0
            {
                assert!(acknowledged);
                assert_eq!(VarUInt::read(&mut data)?.0, 36);
                assert_eq!(NetworkItemStackDescriptor::read(&mut data)?.id, 0);
                recovered = true;
            }
        }
    }
    assert!(recovered);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_container_resync_covers_appended_inventory_and_owns_cursor() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::{
        generic_container_screen_handler::create_generic_9x3,
        inventory::SimpleInventory,
        screen_handler::{ScreenHandler, SharedScreenHandler},
    };
    use pumpkin_protocol::{
        Packet,
        bedrock::{
            client::inventory_content::CInventoryContent, network_item::NetworkItemStackDescriptor,
        },
        codec::var_uint::VarUInt,
        serial::PacketRead,
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    let sync = player.screen_handler_sync_handler.clone();
    sync.store_player(player.clone());
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour_mut()
        .sync_handler = Some(sync.clone());
    let mut chest = create_generic_9x3(
        1,
        player.inventory(),
        Arc::new(SimpleInventory::new(27)),
        player.as_ref(),
    );
    chest.get_behaviour_mut().sync_handler = Some(sync);
    let cursor = ItemStack::new(1, &Item::DIAMOND);
    *chest
        .get_behaviour()
        .cursor_stack
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = cursor.clone();
    let chest: SharedScreenHandler = Arc::new(Mutex::new(chest));
    *player
        .current_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = chest;

    let stick = ItemStack::new(1, &Item::STICK);
    player.inventory().set_held_item(stick.clone());
    player.resync_player_inventory_screen();
    while let Ok(packet) = packets.try_recv() {
        if let Some((id, data)) = crate::net::bedrock::split_raw_packet(&packet.data)
            && i32::from(id) == CInventoryContent::PACKET_ID
        {
            assert_ne!(VarUInt::read(&mut data.as_ref())?.0, 59);
        }
    }

    player.resync_active_inventory_screen();
    let mut ids = Vec::new();
    while let Ok(packet) = packets.try_recv() {
        if let Some((id, data)) = crate::net::bedrock::split_raw_packet(&packet.data)
            && i32::from(id) == CInventoryContent::PACKET_ID
        {
            let mut data = data.as_ref();
            let window = VarUInt::read(&mut data)?.0;
            let count = VarUInt::read(&mut data)?.0;
            match window {
                0 => {
                    assert_eq!(count, 36);
                    let held = NetworkItemStackDescriptor::read(&mut data)?;
                    assert_eq!(
                        held.net_id.map(std::num::NonZero::get),
                        Some(stick.uid.get())
                    );
                }
                1 => assert_eq!(count, 27),
                59 => {
                    assert_eq!(count, 1);
                    let carried = NetworkItemStackDescriptor::read(&mut data)?;
                    assert_eq!(
                        carried.net_id.map(std::num::NonZero::get),
                        Some(cursor.uid.get())
                    );
                }
                _ => continue,
            }
            ids.push(window);
        }
    }
    assert_eq!(ids, [1, 0, 59]);
    world.level.shutdown().await;
    Ok(())
}

impl EventRwHandler<PacketSentEvent> for SupersedeInventoryPacket {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_data::{item::Item, item_stack::ItemStack};
        use pumpkin_inventory::screen_handler::ScreenHandler;
        use pumpkin_protocol::java::client::play::{CSetContainerSlot, CSetCursorItem};

        Box::pin(async move {
            let id = if self.cursor {
                CSetCursorItem::to_id(CURRENT_MC_VERSION)
            } else {
                CSetContainerSlot::to_id(CURRENT_MC_VERSION)
            };
            if event.packet_id == id && !self.changed.swap(true, Ordering::Relaxed) {
                let stack = ItemStack::new(1, &Item::DIAMOND);
                if self.cursor {
                    let handler = self
                        .player
                        .player_screen_handler
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner);
                    *handler
                        .get_behaviour()
                        .cursor_stack
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner) = stack;
                } else {
                    self.player
                        .inventory()
                        .set_stack_in_hand(pumpkin_util::Hand::Left, stack);
                }
                self.player.sync_inventory_to_client();
            }
        })
    }
}

#[derive(Default)]
struct ObservedInventory {
    content: Vec<u16>,
    cursors: Vec<u16>,
    slots: Vec<u16>,
    slot_destinations: Vec<(Option<i32>, i32)>,
}

async fn read_inventory_screen(
    peer: tokio::io::DuplexStream,
) -> Result<ObservedInventory, Box<dyn std::error::Error>> {
    use pumpkin_protocol::{
        codec::item_stack_seralizer::ItemStackSerializer,
        java::{
            client::play::{
                CKeepAlive, CSetContainerContent, CSetContainerSlot, CSetCursorItem,
                CSetPlayerInventory,
            },
            packet_decoder::TCPNetworkDecoder,
        },
    };
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        let mut observed = ObservedInventory::default();
        loop {
            let packet = decoder.get_raw_packet().await?;
            let mut payload = packet.payload.as_ref();
            if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                return Ok(observed);
            }
            if packet.id == CSetContainerContent::to_id(CURRENT_MC_VERSION) {
                let _window = payload.get_container_id(&CURRENT_MC_VERSION)?;
                let _revision = payload.get_var_int()?;
                let count = payload.get_var_int()?.0;
                observed.content = (0..count)
                    .map(|_| {
                        ItemStackSerializer::read_with_version(&mut payload, &CURRENT_MC_VERSION)
                            .map(|stack| stack.0.item.id)
                    })
                    .collect::<Result<_, _>>()?;
            } else if packet.id == CSetCursorItem::to_id(CURRENT_MC_VERSION) {
                let stack =
                    ItemStackSerializer::read_with_version(&mut payload, &CURRENT_MC_VERSION)?;
                observed.cursors.push(stack.0.item.id);
            } else if packet.id == CSetPlayerInventory::to_id(CURRENT_MC_VERSION)
                || packet.id == CSetContainerSlot::to_id(CURRENT_MC_VERSION)
            {
                let destination = if packet.id == CSetPlayerInventory::to_id(CURRENT_MC_VERSION) {
                    (None, payload.get_var_int()?.0)
                } else {
                    let window = payload.get_container_id(&CURRENT_MC_VERSION)?.0;
                    payload.get_var_int()?;
                    (Some(window), i32::from(payload.get_i16_be()?))
                };
                observed.slot_destinations.push(destination);
                let stack =
                    ItemStackSerializer::read_with_version(&mut payload, &CURRENT_MC_VERSION)?;
                observed.slots.push(stack.0.item.id);
            }
        }
    })
    .await?
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_container_recovers_a_superseded_slot_batch_and_cursor() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::screen_handler::ScreenHandler;
    use pumpkin_protocol::java::client::play::CKeepAlive;

    for cursor in [false, true] {
        let folder = tempfile::tempdir()?;
        let (server, world) = inventory_world(&folder).await?;
        let (player, peer) = java_equipment_watcher(&server, &world)?;
        let sync = player.screen_handler_sync_handler.clone();
        sync.store_player(player.clone());
        player
            .player_screen_handler
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get_behaviour_mut()
            .sync_handler = Some(sync);
        player.resync_active_inventory_screen();

        let listener = Arc::new(SupersedeInventoryPacket {
            player: player.clone(),
            changed: AtomicBool::new(false),
            cursor,
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(listener.clone(), EventPriority::Normal, None);
        if cursor {
            let handler = player
                .player_screen_handler
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            *handler
                .get_behaviour()
                .cursor_stack
                .lock()
                .unwrap_or_else(PoisonError::into_inner) = ItemStack::new(1, &Item::STONE);
        } else {
            player
                .inventory()
                .set_held_item(ItemStack::new(1, &Item::STONE));
            player
                .inventory()
                .set_slot(1, ItemStack::new(1, &Item::DIRT));
        }
        player.sync_inventory_to_client();
        assert!(listener.changed.load(Ordering::Relaxed));
        if !cursor {
            player.sync_inventory_to_client();
        }
        assert!(
            player
                .client
                .java()
                .ok_or("Java player")?
                .try_send_packet(&CKeepAlive::new(73))
        );
        let observed = read_inventory_screen(peer).await?;
        if cursor {
            assert_eq!(observed.cursors, [Item::DIAMOND.id]);
        } else {
            assert_eq!(observed.content.get(36), Some(&Item::STONE.id));
            assert_eq!(observed.content.get(37), Some(&Item::DIRT.id));
            assert_eq!(observed.content.get(45), Some(&Item::DIAMOND.id));
        }
        world.level.shutdown().await;
    }
    Ok(())
}

struct ReplaceDuringHandUpdate {
    player: Arc<crate::entity::player::Player>,
    direct: bool,
    called: AtomicBool,
}

impl EventRwHandler<PacketSentEvent> for ReplaceDuringHandUpdate {
    fn handle<'a>(
        &'a self,
        _: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_data::{item::Item, item_stack::ItemStack};
        use pumpkin_protocol::{
            ClientPacket,
            codec::item_stack_seralizer::ItemStackSerializer,
            java::client::play::{CSetContainerSlot, CSetPlayerInventory},
        };
        Box::pin(async move {
            let targeted = if self.direct {
                event.packet_id == CSetPlayerInventory::to_id(CURRENT_MC_VERSION)
            } else {
                let mut payload = event.payload.as_ref();
                event.packet_id == CSetContainerSlot::to_id(CURRENT_MC_VERSION)
                    && payload
                        .get_container_id(&CURRENT_MC_VERSION)
                        .is_ok_and(|id| id.0 == 1)
                    && payload.get_var_int().is_ok()
                    && payload.get_i16_be().is_ok_and(|slot| slot == 54)
            };
            if !targeted || self.called.swap(true, Ordering::Relaxed) {
                return;
            }
            let new = ItemStack::new(1, &Item::DIAMOND);
            self.player.inventory().set_held_item(new.clone());
            self.player.sync_hand_slot(0, new);
            let marker = ItemStackSerializer::from(ItemStack::new(1, &Item::DIRT));
            let mut physical = Vec::new();
            let mut personal = Vec::new();
            let mut unrelated = Vec::new();
            assert!(
                CSetPlayerInventory::new(0.into(), &marker)
                    .write_packet_data(&mut physical, &CURRENT_MC_VERSION)
                    .is_ok()
            );
            assert!(
                CSetContainerSlot::new(0, 1, 36, &marker)
                    .write_packet_data(&mut personal, &CURRENT_MC_VERSION)
                    .is_ok()
            );
            assert!(
                CSetPlayerInventory::new(
                    1.into(),
                    &ItemStackSerializer::from(ItemStack::new(1, &Item::STICK))
                )
                .write_packet_data(&mut unrelated, &CURRENT_MC_VERSION)
                .is_ok()
            );
            event.extra_packets.extend([
                (
                    CSetPlayerInventory::to_id(CURRENT_MC_VERSION),
                    physical.into(),
                ),
                (
                    CSetContainerSlot::to_id(CURRENT_MC_VERSION),
                    personal.into(),
                ),
                (
                    CSetPlayerInventory::to_id(CURRENT_MC_VERSION),
                    unrelated.into(),
                ),
            ]);
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_container_and_direct_hand_updates_filter_stale_physical_aliases() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::{
        generic_container_screen_handler::create_generic_9x3,
        inventory::SimpleInventory,
        screen_handler::{ScreenHandler, SharedScreenHandler},
    };
    use pumpkin_protocol::java::client::play::CKeepAlive;

    for direct in [false, true] {
        let folder = tempfile::tempdir()?;
        let (server, world) = inventory_world(&folder).await?;
        let (player, peer) = java_equipment_watcher(&server, &world)?;
        let sync = player.screen_handler_sync_handler.clone();
        sync.store_player(player.clone());
        player
            .player_screen_handler
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get_behaviour_mut()
            .sync_handler = Some(sync.clone());
        if !direct {
            let mut chest = create_generic_9x3(
                1,
                player.inventory(),
                Arc::new(SimpleInventory::new(27)),
                player.as_ref(),
            );
            chest.get_behaviour_mut().sync_handler = Some(sync);
            let chest: SharedScreenHandler = Arc::new(Mutex::new(chest));
            *player
                .current_screen_handler
                .lock()
                .unwrap_or_else(PoisonError::into_inner) = chest;
        }
        player.resync_player_inventory_screen();
        if !direct {
            player.resync_active_inventory_screen();
        }
        let observer = Arc::new(ReplaceDuringHandUpdate {
            player: player.clone(),
            direct,
            called: AtomicBool::new(false),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
        let old = ItemStack::new(1, &Item::STONE);
        player.inventory().set_held_item(old.clone());
        if direct {
            player.sync_hand_slot(0, old);
        } else {
            player.sync_inventory_to_client();
        }
        assert!(observer.called.load(Ordering::Relaxed));
        assert_eq!(player.inventory().held_item().item, &Item::DIAMOND);
        assert!(
            player
                .client
                .java()
                .ok_or("Java player")?
                .try_send_packet(&CKeepAlive::new(73))
        );
        let observed = read_inventory_screen(peer).await?;
        assert!(observed.slots.contains(&Item::DIAMOND.id));
        assert!(observed.slots.contains(&Item::STICK.id));
        assert!(!observed.slots.contains(&Item::DIRT.id));
        if direct {
            assert!(!observed.slots.contains(&Item::STONE.id));
        }
        world.level.shutdown().await;
    }
    Ok(())
}

struct DropOnSharedInventory(Arc<crate::entity::player::Player>, AtomicBool);
impl EventRwHandler<PacketSentEvent> for DropOnSharedInventory {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        use pumpkin_data::item::Item;
        use pumpkin_protocol::{Packet, bedrock::client::inventory_content::CInventoryContent};
        Box::pin(async move {
            if event.packet_id == CInventoryContent::PACKET_ID
                && !self.1.swap(true, Ordering::Relaxed)
            {
                let stick = self.0.inventory().held_item();
                assert_eq!(stick.item, &Item::STICK);
                let mut request = drop_and_armor_requests(&stick, &stick);
                request.requests.truncate(1);
                if let Some(client) = self.0.client.bedrock() {
                    client.handle_item_stack_request(&self.0, request);
                }
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_container_ack_retires_a_personal_snapshot() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::{
        generic_container_screen_handler::create_generic_9x3,
        inventory::SimpleInventory,
        screen_handler::{ScreenHandler, SharedScreenHandler},
    };
    use pumpkin_protocol::{
        Packet,
        bedrock::{
            client::{
                inventory_content::CInventoryContent, item_stack_response::CItemStackResponse,
            },
            network_item::NetworkItemStackDescriptor,
        },
        codec::var_uint::VarUInt,
        serial::PacketRead,
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    let sync = player.screen_handler_sync_handler.clone();
    sync.store_player(player.clone());
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour_mut()
        .sync_handler = Some(sync.clone());
    let mut chest = create_generic_9x3(
        1,
        player.inventory(),
        Arc::new(SimpleInventory::new(27)),
        player.as_ref(),
    );
    chest.get_behaviour_mut().sync_handler = Some(sync);
    let cursor = ItemStack::new(1, &Item::DIAMOND);
    *chest
        .get_behaviour()
        .cursor_stack
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = cursor.clone();
    let chest: SharedScreenHandler = Arc::new(Mutex::new(chest));
    *player
        .current_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = chest;

    let stick = ItemStack::new(1, &Item::STICK);
    player.inventory().set_held_item(stick.clone());

    player.resync_active_inventory_screen();
    while packets.try_recv().is_ok() {}
    let observer = Arc::new(DropOnSharedInventory(
        player.clone(),
        AtomicBool::new(false),
    ));
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    player.resync_player_inventory_screen();
    assert!(observer.1.load(Ordering::Relaxed));
    assert!(player.inventory().held_item().is_empty());
    player.send_player_screen_updates();
    let mut acknowledged = false;
    let mut main = Vec::new();
    while let Ok(packet) = packets.try_recv() {
        if let Some((id, bytes)) = crate::net::bedrock::split_raw_packet(&packet.data) {
            let mut data = bytes.as_ref();
            if i32::from(id) == CItemStackResponse::PACKET_ID {
                acknowledged = true;
            } else if i32::from(id) == CInventoryContent::PACKET_ID
                && VarUInt::read(&mut data)?.0 == 0
            {
                assert_eq!(VarUInt::read(&mut data)?.0, 36);
                let held = NetworkItemStackDescriptor::read(&mut data)?;
                main.push((
                    acknowledged,
                    held.id,
                    held.net_id.map(std::num::NonZero::get),
                ));
            }
        }
    }
    assert!(acknowledged);
    assert_eq!(main.last(), Some(&(true, 0, None)));
    assert!(
        main.iter()
            .all(|(after, _, uid)| !after || *uid != Some(stick.uid.get())),
        "older personal snapshot restored the dropped item after chest acknowledgement"
    );
    world.level.shutdown().await;
    Ok(())
}

const LEGACY_DROP_CHEST: usize = 38;
struct RefillArmor(
    Arc<pumpkin_inventory::player::player_inventory::PlayerInventory>,
    AtomicBool,
);
impl EventRwHandler<crate::plugin::api::events::entity::entity_spawn::EntitySpawnEvent>
    for RefillArmor
{
    fn handle<'a>(
        &'a self,
        _: &'a Arc<Server>,
        event: &'a mut crate::plugin::api::events::entity::entity_spawn::EntitySpawnEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_type == EntityType::ITEM.id.to_string() {
                assert!(self.0.get_slot(LEGACY_DROP_CHEST).is_empty());
                self.0.set_slot(
                    LEGACY_DROP_CHEST,
                    pumpkin_data::item_stack::ItemStack::new(
                        1,
                        &pumpkin_data::item::Item::IRON_CHESTPLATE,
                    ),
                );
                self.1.store(true, Ordering::Relaxed);
            }
        })
    }
}

fn read_legacy_armor_drop_reply(
    refill: bool,
    packets: impl Iterator<Item = bytes::Bytes>,
) -> Result<(usize, Option<i32>), Box<dyn std::error::Error>> {
    use pumpkin_protocol::{
        Packet,
        bedrock::{
            client::{
                inventory_content::CInventoryContent, item_stack_response::CItemStackResponse,
            },
            network_item::{ContainerName, FullContainerName, NetworkItemStackDescriptor},
            server::inventory_transaction::WINDOW_ID_ARMOUR,
        },
        codec::{var_int::VarInt, var_uint::VarUInt},
        serial::PacketRead,
    };

    let mut replies = 0;
    let mut final_uid = None;
    for packet in packets {
        let Some((id, bytes)) = crate::net::bedrock::split_raw_packet(&packet) else {
            continue;
        };
        let mut data = bytes.as_ref();
        if i32::from(id) == CItemStackResponse::PACKET_ID {
            replies += 1;
            assert_eq!(VarUInt::read(&mut data)?.0, 1);
            assert_eq!(u8::read(&mut data)?, u8::from(refill));
            assert_eq!(VarInt::read(&mut data)?.0, -1);
            if bool::read(&mut data)? {
                assert!(!refill);
                assert_eq!(VarUInt::read(&mut data)?.0, 1);
                assert_eq!(
                    FullContainerName::read(&mut data)?.container_name,
                    ContainerName::Armor
                );
            } else {
                assert!(refill);
            }
        } else if i32::from(id) == CInventoryContent::PACKET_ID
            && VarUInt::read(&mut data)?.0 == WINDOW_ID_ARMOUR as u32
        {
            assert_eq!(VarUInt::read(&mut data)?.0, 4);
            NetworkItemStackDescriptor::read(&mut data)?;
            final_uid = NetworkItemStackDescriptor::read(&mut data)?
                .net_id
                .map(std::num::NonZero::get);
        }
    }
    Ok((replies, final_uid))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_player_inventory_legacy_drop_checks_armor_and_reentry() -> TestResult {
    use crate::plugin::api::events::entity::entity_spawn::EntitySpawnEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::screen_handler::ScreenHandler;
    use pumpkin_protocol::bedrock::server::inventory_transaction::{
        LegacySetItemSlot, NormalTransactionData, SInventoryTransaction, TransactionData,
        WINDOW_ID_ARMOUR,
    };

    for refill in [false, true] {
        let folder = tempfile::tempdir()?;
        let (server, world) = inventory_world(&folder).await?;
        let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
        let listener = Arc::new(RefillArmor(
            player.inventory().clone(),
            AtomicBool::new(false),
        ));
        if refill {
            server
                .plugin_manager
                .register_blocking::<EntitySpawnEvent, _>(
                    listener.clone(),
                    EventPriority::Normal,
                    None,
                );
        }
        let sync = player.screen_handler_sync_handler.clone();
        sync.store_player(player.clone());
        player
            .player_screen_handler
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get_behaviour_mut()
            .sync_handler = Some(sync);
        player.inventory().set_slot(
            LEGACY_DROP_CHEST,
            ItemStack::new(1, &Item::DIAMOND_CHESTPLATE),
        );
        player
            .client
            .bedrock()
            .ok_or("Bedrock client")?
            .handle_inventory_action(
                &player,
                SInventoryTransaction {
                    legacy_request_id: (-1).into(),
                    legacy_set_item_slots: vec![LegacySetItemSlot {
                        container_id: WINDOW_ID_ARMOUR as u8,
                        slots: vec![1],
                    }],
                    has_value: false,
                    actions: vec![],
                    transaction_type: 0u32.into(),
                    transaction_data: TransactionData::Normal(NormalTransactionData),
                },
            );
        assert_eq!(listener.1.load(Ordering::Relaxed), refill);
        let (replies, final_uid) = read_legacy_armor_drop_reply(
            refill,
            std::iter::from_fn(|| packets.try_recv().ok().map(|packet| packet.data)),
        )?;
        assert_eq!(replies, 1);
        assert_eq!(
            final_uid,
            refill.then(|| player.inventory().get_slot(LEGACY_DROP_CHEST).uid.get())
        );
        world.level.shutdown().await;
    }
    Ok(())
}

struct ReplaceFallbackSource(AtomicBool);

impl EventRwHandler<crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent> for ReplaceFallbackSource {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if !self.0.swap(true, Ordering::Relaxed) {
                event
                    .player
                    .inventory()
                    .set_slot(0, pumpkin_data::item_stack::ItemStack::new(1, &pumpkin_data::item::Item::STICK));
            }
        })
    }
}

fn bedrock_click_block(
    position: pumpkin_util::math::position::BlockPos,
    source: &pumpkin_data::item_stack::ItemStack,
    hotbar: i32,
) -> pumpkin_protocol::bedrock::server::inventory_transaction::SInventoryTransaction {
    use pumpkin_protocol::bedrock::{
        network_item::NetworkItemDescriptor,
        server::inventory_transaction::{
            HandSlot, SInventoryTransaction, TransactionData, UseItemTransactionData,
        },
    };
    SInventoryTransaction {
        legacy_request_id: 0.into(),
        legacy_set_item_slots: Vec::new(),
        has_value: false,
        actions: Vec::new(),
        transaction_type: 2u32.into(),
        transaction_data: TransactionData::UseItem(UseItemTransactionData {
            action_type: 0.into(),
            trigger_type: 0,
            block_position: position,
            block_face: 1,
            hot_bar_slot: hotbar.into(),
            hand: HandSlot::Mainhand,
            item_in_hand: NetworkItemDescriptor::from(source),
            item_in_hand_network_id: Some(source.uid),
            player_position: CENTER.to_f32_lossy(),
            click_position: Vector3::new(0.5, 1.0, 0.5),
            block_runtime_id: 0u32.into(),
            client_prediction: 0,
            client_cooldown_state: 0,
        }),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_block_use_reports_a_break_in_the_offhand() -> TestResult {
    use pumpkin_data::{
        Block, data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack,
    };
    use pumpkin_protocol::{
        bedrock::server::inventory_transaction::{HandSlot, TransactionData},
        java::client::play::CEntityStatus,
    };
    use pumpkin_util::math::position::BlockPos;
    use std::sync::atomic::AtomicU8;

    struct BreakStatus(i32, AtomicU8);
    impl EventRwHandler<PacketSentEvent> for BreakStatus {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if event.packet_id == CEntityStatus::to_id(CURRENT_MC_VERSION) {
                    let mut payload = event.payload.as_ref();
                    if payload.get_i32_be().ok() == Some(self.0)
                        && let Ok(status) = payload.get_u8()
                    {
                        self.1.store(status, Ordering::Relaxed);
                    }
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (watcher, _peer) = java_equipment_watcher(&server, &world)?;
    let (player, _packets) = bedrock_player(&server, &world, "jeb_").await?;
    let actor: Arc<dyn EntityBase> = player.clone();
    world.entity_tracker.add_entity(&actor, &world);
    world
        .entity_tracker
        .get_tracked_entity(player.entity_id())
        .ok_or("Player tracker")?
        .add_pairing(&watcher);
    let pos = BlockPos::new(8, 63, 8);
    world.set_block_state(
        &pos,
        Block::JUKEBOX.default_state.id,
        pumpkin_world::world::BlockFlags::NOTIFY_ALL,
    );

    let mut tool = ItemStack::new(1, &Item::FLINT_AND_STEEL);
    tool.set_damage(tool.get_max_damage().ok_or("Flint and steel durability")? - 1);
    let inventory = player.inventory();
    let offhand = inventory
        .equipment_slot_index(&EquipmentSlot::OFF_HAND)
        .ok_or("Offhand slot")?;
    inventory.set_slot(offhand, tool.clone());
    let observer = Arc::new(BreakStatus(player.entity_id(), AtomicU8::new(0)));
    server
        .plugin_manager
        .register_blocking::<PacketSentEvent, _>(observer.clone(), EventPriority::Normal, None);
    let mut request = bedrock_click_block(pos, &tool, 0);
    let TransactionData::UseItem(data) = &mut request.transaction_data else {
        return Err("Expected click-block transaction".into());
    };
    data.hand = HandSlot::Offhand;
    player
        .client
        .bedrock()
        .ok_or("Bedrock player")?
        .handle_inventory_action(&player, request);

    assert!(inventory.get_slot(offhand).is_empty());
    assert_eq!(
        observer.1.load(Ordering::Relaxed),
        crate::entity::equipment_break_status(&EquipmentSlot::OFF_HAND) as u8,
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_block_insertions_reject_a_replaced_source_stack() -> TestResult {
    use crate::block::entities::{
        decorated_pot::DecoratedPotBlockEntity, jukebox::JukeboxBlockEntity,
    };
    use crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent;
    use pumpkin_data::{Block, item::Item, item_stack::ItemStack};
    use pumpkin_inventory::Inventory;
    use pumpkin_protocol::{Packet, bedrock::client::inventory_slot::CInventorySlot};
    use pumpkin_util::math::position::BlockPos;

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::JUKEBOX.default_state.id);
    chunk.set_block_absolute_y(9, 63, 8, Block::DECORATED_POT.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let jukebox_pos = BlockPos::new(8, 63, 8);
    let pot_pos = BlockPos::new(9, 63, 8);
    let jukebox = Arc::new(JukeboxBlockEntity::new(jukebox_pos));
    let pot = Arc::new(DecoratedPotBlockEntity::new(pot_pos));
    world.add_block_entity(jukebox.clone());
    world.add_block_entity(pot.clone());
    let (player, mut packets) = bedrock_player(&server, &world, "Notch").await?;
    let client = player.client.bedrock().ok_or("Bedrock client")?;
    let inventory = player.inventory();
    inventory.set_selected_slot(0);

    let listener = Arc::new(ReplaceFallbackSource(AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PlayerStatisticIncrementEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            None,
        );
    let apple = ItemStack::new(1, &Item::APPLE);
    inventory.set_slot(0, apple.clone());
    client.handle_inventory_action(&player, bedrock_click_block(jukebox_pos, &apple, 0));
    assert!(listener.0.load(Ordering::Relaxed));
    assert_eq!(inventory.get_slot(0).item, &Item::STICK);

    for (position, item) in [
        (jukebox_pos, &Item::MUSIC_DISC_CAT),
        (pot_pos, &Item::STONE),
    ] {
        let stale = ItemStack::new(1, item);
        let replacement = ItemStack::new(1, item);
        assert_ne!(stale.uid, replacement.uid);
        inventory.set_slot(0, replacement.clone());
        while packets.try_recv().is_ok() {}
        client.handle_inventory_action(&player, bedrock_click_block(position, &stale, 0));
        assert!(
            std::iter::from_fn(|| packets.try_recv().ok()).any(|packet| {
                crate::net::bedrock::split_raw_packet(&packet.data)
                    .is_some_and(|(id, _)| i32::from(id) == CInventorySlot::PACKET_ID)
            })
        );
        client.handle_inventory_action(&player, bedrock_click_block(position, &replacement, 1));
        assert_eq!(inventory.get_slot(0).uid, replacement.uid);
        assert_eq!(inventory.get_slot(0).item_count, 1);
        client.handle_inventory_action(&player, bedrock_click_block(position, &replacement, 0));
        assert!(inventory.get_slot(0).is_empty());
    }
    assert_eq!(jukebox.get_record().item, &Item::MUSIC_DISC_CAT);
    assert_eq!(pot.get_stack(0).item, &Item::STONE);
    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_placement_debits_when_a_callback_changes_the_clicked_block() -> TestResult {
    use crate::plugin::api::events::block::block_place::BlockPlaceEvent;
    use crate::world::BlockFlags;
    use pumpkin_data::{Block, item::Item, item_stack::ItemStack};
    use pumpkin_util::math::position::BlockPos;

    struct ReplaceSupport(BlockPos, AtomicBool);

    impl EventRwHandler<BlockPlaceEvent> for ReplaceSupport {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut BlockPlaceEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.1.store(true, Ordering::Relaxed);
                event.player.world().set_block_state(
                    &self.0,
                    Block::STONE.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let support = BlockPos::new(10, 63, 8);
    let target = BlockPos::new(10, 64, 8);
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(10, 63, 8, Block::JUKEBOX.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (player, _packets) = bedrock_player(&server, &world, "Notch").await?;
    let client = player.client.bedrock().ok_or("Bedrock client")?;
    let inventory = player.inventory();
    inventory.set_selected_slot(0);
    assert_eq!(world.get_block(&target), &Block::AIR);
    let listener = Arc::new(ReplaceSupport(support, AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<BlockPlaceEvent, _>(listener.clone(), EventPriority::Normal, None);
    // The generated spawn can overlap these coordinates on some seeds.
    player
        .permission_lvl
        .store(pumpkin_util::permission::PermissionLvl::Four);
    let dirt = ItemStack::new(2, &Item::DIRT);
    inventory.set_slot(0, dirt.clone());
    client.handle_inventory_action(&player, bedrock_click_block(support, &dirt, 0));
    assert!(listener.1.load(Ordering::Relaxed));
    assert_eq!(world.get_block(&support), &Block::STONE);
    assert_eq!(world.get_block(&target), &Block::DIRT);
    assert_eq!(inventory.get_slot(0).item_count, 1);
    assert_eq!(inventory.get_slot(0).uid, dirt.uid);

    client.clear_player();
    client.close().await;
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_on_foot_inserts_block_items_from_the_physical_slot() -> TestResult {
    use crate::block::entities::{
        decorated_pot::DecoratedPotBlockEntity, jukebox::JukeboxBlockEntity,
    };
    use pumpkin_data::{Block, item::Item, item_stack::ItemStack};
    use pumpkin_inventory::Inventory;
    use pumpkin_protocol::java::server::play::SUseItemOn;
    use pumpkin_util::math::position::BlockPos;

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let position = BlockPos::new(8, 63, 8);
    let pot_position = BlockPos::new(9, 63, 8);
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::JUKEBOX.default_state.id);
    chunk.set_block_absolute_y(9, 63, 8, Block::DECORATED_POT.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let jukebox = Arc::new(JukeboxBlockEntity::new(position));
    world.add_block_entity(jukebox.clone());
    let pot = Arc::new(DecoratedPotBlockEntity::new(pot_position));
    world.add_block_entity(pot.clone());
    let (player, _peer) = java_player(&server, &world, "Notch")?;
    player.get_entity().set_pos(Vector3::new(8.5, 64.0, 7.5));
    let inventory = player.inventory();
    inventory.set_selected_slot(0);
    inventory.set_slot(0, ItemStack::new(1, &Item::MUSIC_DISC_CAT));
    let other = ItemStack::new(1, &Item::STICK);
    inventory.set_slot(1, other.clone());
    let client = player.client.java().ok_or("Java client")?;
    let click = SUseItemOn {
        hand: 0.into(),
        position,
        face: 1.into(),
        cursor_pos: Vector3::new(0.5, 1.0, 0.5),
        inside_block: false,
        is_against_world_border: false,
        sequence: 1.into(),
    };
    client.handle_use_item_on(&player, &click, &server)?;
    assert!(inventory.get_slot(0).is_empty());
    assert_eq!(inventory.get_slot(1).uid, other.uid);
    assert_eq!(jukebox.get_record().item, &Item::MUSIC_DISC_CAT);
    assert!(jukebox.is_playing());

    let stone = ItemStack::new(2, &Item::STONE);
    inventory.set_slot(0, stone.clone());
    let pot_click = SUseItemOn {
        position: pot_position,
        sequence: 2.into(),
        ..click
    };
    client.handle_use_item_on(&player, &pot_click, &server)?;
    let remainder = inventory.get_slot(0);
    assert_eq!(remainder.uid, stone.uid);
    assert_eq!(remainder.item, &Item::STONE);
    assert_eq!(remainder.item_count, 1);
    assert_eq!(pot.get_stack(0).item, &Item::STONE);
    assert_eq!(pot.get_stack(0).item_count, 1);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_air_use_rechecks_the_stack_after_the_statistic_listener() -> TestResult {
    use crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::server::play::SUseItem;

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    player.inventory().set_selected_slot(0);
    player
        .inventory()
        .set_slot(0, ItemStack::new(1, &Item::APPLE));
    player.hunger_manager.level.store(10);
    let listener = Arc::new(ReplaceFallbackSource(AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PlayerStatisticIncrementEvent, _>(
            listener.clone(),
            EventPriority::Normal,
            None,
        );
    player.client.java().ok_or("Java client")?.handle_use_item(
        &player,
        &SUseItem {
            hand: 0.into(),
            sequence: 1.into(),
            yaw: 0.0,
            pitch: 0.0,
        },
        &server,
    );
    assert!(listener.0.load(Ordering::Relaxed));
    assert_eq!(player.inventory().get_slot(0).item, &Item::STICK);
    assert!(
        player
            .living_entity
            .item_in_use
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_none()
    );
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_click_cannot_commit_to_a_reopened_personal_screen() -> TestResult {
    use crate::plugin::player::inventory_interact::InventoryClickEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::{
        generic_container_screen_handler::create_generic_9x3, inventory::SimpleInventory,
    };
    use pumpkin_protocol::java::server::play::SlotActionType;
    use pumpkin_util::text::TextComponent;

    struct OpenAndClose(Arc<crate::entity::player::Player>, AtomicBool);

    impl EventRwHandler<InventoryClickEvent> for OpenAndClose {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            _event: &'a mut InventoryClickEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if !self.1.swap(true, Ordering::Relaxed) {
                    self.0.open_handled_screen_direct(
                        Arc::new(Mutex::new(create_generic_9x3(
                            1,
                            self.0.inventory(),
                            Arc::new(SimpleInventory::new(27)),
                            self.0.as_ref(),
                        ))),
                        &TextComponent::text("Chest"),
                    );
                    self.0.close_handled_screen();
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, _peer) = java_equipment_watcher(&server, &world)?;
    player
        .inventory()
        .set_held_item(ItemStack::new(1, &Item::DIRT));
    let observer = Arc::new(OpenAndClose(player.clone(), AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<InventoryClickEvent, _>(observer.clone(), EventPriority::Normal, None);

    player.on_slot_click(player_inventory_click(36, SlotActionType::Pickup), &server);
    assert!(observer.1.load(Ordering::Relaxed));
    assert_eq!(player.inventory().held_item().item_count, 1);
    assert_eq!(player.inventory().held_item().item, &Item::DIRT);

    player.on_slot_click(player_inventory_click(36, SlotActionType::Pickup), &server);
    assert!(player.inventory().held_item().is_empty());
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[expect(
    clippy::too_many_lines,
    reason = "Exercises a partial batch and decodes both acknowledgements and the recovery."
)]
async fn bedrock_partial_batch_preserves_the_earlier_success() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::screen_handler::ScreenHandler;
    use pumpkin_protocol::{
        Packet,
        bedrock::{
            client::{
                inventory_content::CInventoryContent, item_stack_response::CItemStackResponse,
            },
            network_item::{ContainerName, FullContainerName, NetworkItemStackDescriptor},
            server::item_stack_request::{
                ItemStackRequest, ItemStackRequestAction, ItemStackRequestSlotInfo,
                SItemStackRequest,
            },
        },
        codec::{var_int::VarInt, var_uint::VarUInt},
        serial::PacketRead,
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    let stick = ItemStack::new(1, &Item::STICK);
    player.inventory().set_slot(1, stick.clone());
    let sync = player.screen_handler_sync_handler.clone();
    sync.store_player(player.clone());
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour_mut()
        .sync_handler = Some(sync);
    player.resync_active_inventory_screen();
    while packets.try_recv().is_ok() {}

    let slot = |name, slot_id, stack_id| ItemStackRequestSlotInfo {
        container_name: FullContainerName {
            container_name: name,
            dynamic_id: None,
        },
        slot_id,
        stack_id,
    };
    let request = |id: i32, actions| ItemStackRequest {
        request_id: id.into(),
        actions,
        filter_strings: Vec::new(),
        filter_cause: 0,
    };
    let hotbar = || slot(ContainerName::HotBar, 0, stick.uid.get());
    let client = player.client.bedrock().ok_or("Bedrock player")?;
    client.handle_item_stack_request(
        &player,
        SItemStackRequest {
            requests: vec![
                request(
                    -3,
                    vec![ItemStackRequestAction::Swap {
                        slot1: slot(ContainerName::HotBar, 1, stick.uid.get()),
                        slot2: slot(ContainerName::HotBar, 0, 0),
                    }],
                ),
                request(
                    -1,
                    vec![
                        ItemStackRequestAction::Swap {
                            slot1: hotbar(),
                            slot2: slot(ContainerName::Cursor, 0, 0),
                        },
                        ItemStackRequestAction::Destroy {
                            count: 1,
                            source: hotbar(),
                        },
                    ],
                ),
            ],
        },
    );

    assert!(player.inventory().get_slot(0).is_empty());
    assert!(player.inventory().get_slot(1).is_empty());
    assert_eq!(
        player
            .player_screen_handler
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get_behaviour()
            .cursor_stack
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .uid,
        stick.uid,
    );
    let mut acknowledged = false;
    let mut hotbar_recovered = false;
    let mut cursor_recovered = false;
    while let Ok(packet) = packets.try_recv() {
        let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data) else {
            continue;
        };
        if i32::from(id) == CItemStackResponse::PACKET_ID {
            let mut payload = payload.as_ref();
            assert_eq!(VarUInt::read(&mut payload)?.0, 2);
            for (status, request_id) in [(0, -3), (1, -1)] {
                assert_eq!(u8::read(&mut payload)?, status);
                assert_eq!(VarInt::read(&mut payload)?.0, request_id);
                assert!(!bool::read(&mut payload)?);
            }
            acknowledged = true;
        } else if i32::from(id) == CInventoryContent::PACKET_ID && acknowledged {
            let mut payload = payload.as_ref();
            match VarUInt::read(&mut payload)?.0 {
                0 => {
                    assert_eq!(VarUInt::read(&mut payload)?.0, 36);
                    for _ in 0..2 {
                        assert_eq!(NetworkItemStackDescriptor::read(&mut payload)?.id, 0);
                    }
                    hotbar_recovered = true;
                }
                59 => {
                    assert_eq!(VarUInt::read(&mut payload)?.0, 1);
                    let cursor = NetworkItemStackDescriptor::read(&mut payload)?;
                    assert_eq!(cursor.net_id, Some(stick.uid));
                    assert_eq!(cursor.stack_size, 1);
                    cursor_recovered = true;
                }
                _ => {}
            }
        }
    }
    assert!(hotbar_recovered && cursor_recovered);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[expect(
    clippy::too_many_lines,
    reason = "Drives the request handler and checks both client queues."
)]
async fn bedrock_mine_block_does_not_republish_a_direct_hand_write() -> TestResult {
    use crate::entity::player::Player;
    use pumpkin_data::{data_component_impl::EquipmentSlot, item::Item, item_stack::ItemStack};
    use pumpkin_inventory::{
        inventory::Inventory,
        screen_handler::ScreenHandler,
        slot::{NormalSlot, Slot},
    };
    use pumpkin_protocol::{
        Packet,
        bedrock::{
            client::item_stack_response::CItemStackResponse,
            network_item::{ContainerName, FullContainerName},
            server::item_stack_request::{
                ItemStackRequest, ItemStackRequestAction, SItemStackRequest,
            },
        },
        codec::{var_int::VarInt, var_uint::VarUInt},
        java::client::play::CKeepAlive,
        serial::PacketRead,
    };

    struct ChangeOnRead(NormalSlot, std::sync::Weak<Player>, ItemStack, AtomicBool);
    impl Slot for ChangeOnRead {
        fn get_inventory(&self) -> Arc<dyn Inventory> {
            self.0.get_inventory()
        }

        fn get_index(&self) -> usize {
            self.0.get_index()
        }

        fn set_id(&self, id: usize) {
            self.0.set_id(id);
        }

        fn mark_dirty(&self) {
            self.0.mark_dirty();
        }

        fn get_cloned_stack(&self) -> ItemStack {
            if !self.3.swap(true, Ordering::Relaxed)
                && let Some(player) = self.1.upgrade()
            {
                player
                    .inventory()
                    .set_slot(self.get_index(), self.2.clone());
                player
                    .living_entity
                    .send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, self.2.clone())]);
            }
            self.0.get_cloned_stack()
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    let (watcher, peer) = java_equipment_watcher(&server, &world)?;
    let actor: Arc<dyn EntityBase> = player.clone();
    world.entity_tracker.add_entity(&actor, &world);
    world
        .entity_tracker
        .get_tracked_entity(player.entity_id())
        .ok_or("Player tracker")?
        .add_pairing(&watcher);
    let original = ItemStack::new(1, &Item::STICK);
    let replacement = ItemStack::new(2, &Item::CARROT);
    player.inventory().set_selected_slot(0);
    player.inventory().set_slot(0, original.clone());
    let slot = Arc::new(ChangeOnRead(
        NormalSlot::new(player.inventory().clone(), 0),
        Arc::downgrade(&player),
        replacement.clone(),
        AtomicBool::new(false),
    ));
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour_mut()
        .slots[36] = slot.clone();
    assert_eq!(player.inventory().held_item().uid, original.uid);
    while packets.try_recv().is_ok() {}
    player
        .client
        .bedrock()
        .ok_or("Bedrock client")?
        .handle_item_stack_request(
            &player,
            SItemStackRequest {
                requests: vec![ItemStackRequest {
                    request_id: (-3).into(),
                    actions: vec![ItemStackRequestAction::MineBlock {
                        hotbar_slot: 0.into(),
                        predicted_durability: 0.into(),
                        stack_id: original.uid.get().into(),
                    }],
                    filter_strings: Vec::new(),
                    filter_cause: 0,
                }],
            },
        );
    assert!(slot.3.load(Ordering::Relaxed));
    let response = std::iter::from_fn(|| packets.try_recv().ok())
        .filter_map(|packet| crate::net::bedrock::split_raw_packet(&packet.data))
        .find(|(id, _)| i32::from(*id) == CItemStackResponse::PACKET_ID)
        .ok_or("MineBlock response")?
        .1;
    let mut payload = response.as_ref();
    assert_eq!(VarUInt::read(&mut payload)?.0, 1);
    assert_eq!(u8::read(&mut payload)?, 0);
    assert_eq!(VarInt::read(&mut payload)?.0, -3);
    assert!(bool::read(&mut payload)?);
    assert_eq!(VarUInt::read(&mut payload)?.0, 1);
    assert_eq!(
        FullContainerName::read(&mut payload)?.container_name,
        ContainerName::HotBar
    );
    assert_eq!(VarUInt::read(&mut payload)?.0, 1);
    assert_eq!(u8::read(&mut payload)?, 0);
    assert_eq!(u8::read(&mut payload)?, 0);
    assert_eq!(u8::read(&mut payload)?, 2);
    assert!(bool::read(&mut payload)?);
    assert_eq!(VarInt::read(&mut payload)?.0, replacement.uid.get());

    let client = watcher.client.java().ok_or("Java watcher")?;
    assert!(client.try_send_packet(&CKeepAlive::new(73)));
    let (updates, _) = read_equipment_updates(peer, player.entity_id()).await?;
    let hands = updates
        .into_iter()
        .filter(|(slot, stack)| {
            *slot == EquipmentSlot::MAIN_HAND.discriminant() && stack.item == replacement.item
        })
        .count();
    assert_eq!(hands, 1);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_failed_stack_request_resends_its_partial_writes() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::screen_handler::ScreenHandler;
    use pumpkin_protocol::{
        Packet,
        bedrock::{
            client::{
                inventory_content::CInventoryContent, item_stack_response::CItemStackResponse,
            },
            network_item::{FullContainerName, NetworkItemStackDescriptor},
            server::inventory_transaction::WINDOW_ID_ARMOUR,
        },
        codec::{var_int::VarInt, var_uint::VarUInt},
        serial::PacketRead,
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    let stick = ItemStack::new(1, &Item::STICK);
    let chestplate = ItemStack::new(1, &Item::DIAMOND_CHESTPLATE);
    player.inventory().set_held_item(stick.clone());
    let sync = player.screen_handler_sync_handler.clone();
    sync.store_player(player.clone());
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_slot(6)
        .set_stack(chestplate.clone());
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour_mut()
        .sync_handler = Some(sync);
    player.resync_active_inventory_screen();
    while packets.try_recv().is_ok() {}

    let mut batch = drop_and_armor_requests(&stick, &chestplate);
    let mut duplicate = drop_and_armor_requests(&stick, &chestplate);
    let duplicate_destroy = duplicate.requests[1].actions.remove(0);
    batch.requests[1].actions.push(duplicate_destroy);
    player
        .client
        .bedrock()
        .ok_or("Bedrock player")?
        .handle_item_stack_request(&player, batch);

    assert!(player.inventory().held_item().is_empty());
    assert!(
        player
            .player_screen_handler
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get_slot(6)
            .get_stack()
            .is_empty()
    );
    let mut acknowledged_failure = false;
    let mut resynced_armor = false;
    while let Ok(packet) = packets.try_recv() {
        let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data) else {
            continue;
        };
        let mut payload = payload.as_ref();
        if i32::from(id) == CItemStackResponse::PACKET_ID {
            assert_eq!(VarUInt::read(&mut payload)?.0, 2);
            assert_eq!(u8::read(&mut payload)?, 0);
            assert_eq!(VarInt::read(&mut payload)?.0, -3);
            assert!(bool::read(&mut payload)?);
            assert_eq!(VarUInt::read(&mut payload)?.0, 1);
            FullContainerName::read(&mut payload)?;
            assert_eq!(VarUInt::read(&mut payload)?.0, 1);
            assert_eq!(u8::read(&mut payload)?, 0);
            assert_eq!(u8::read(&mut payload)?, 0);
            assert_eq!(u8::read(&mut payload)?, 0);
            assert!(!bool::read(&mut payload)?);
            assert!(String::read(&mut payload)?.is_empty());
            assert!(String::read(&mut payload)?.is_empty());
            assert_eq!(VarInt::read(&mut payload)?.0, 0);
            assert_eq!(u8::read(&mut payload)?, 1);
            assert_eq!(VarInt::read(&mut payload)?.0, -1);
            assert!(!bool::read(&mut payload)?);
            assert!(payload.is_empty());
            acknowledged_failure = true;
        } else if i32::from(id) == CInventoryContent::PACKET_ID
            && VarUInt::read(&mut payload)?.0 == WINDOW_ID_ARMOUR as u32
        {
            assert!(acknowledged_failure);
            assert_eq!(VarUInt::read(&mut payload)?.0, 4);
            NetworkItemStackDescriptor::read(&mut payload)?;
            assert_eq!(NetworkItemStackDescriptor::read(&mut payload)?.id, 0);
            resynced_armor = true;
        }
    }
    assert!(resynced_armor);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_drop_resends_a_superseded_cursor() -> TestResult {
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::{
        screen_handler::SharedScreenHandler, sync_handler::ScreenUpdateContext,
    };
    use pumpkin_protocol::{
        Packet,
        bedrock::{
            client::inventory_content::CInventoryContent, network_item::NetworkItemStackDescriptor,
        },
        codec::var_uint::VarUInt,
        serial::PacketRead,
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    player
        .inventory()
        .set_held_item(ItemStack::new(2, &Item::STICK));
    let sync = player.screen_handler_sync_handler.clone();
    sync.store_player(player.clone());
    let screen: SharedScreenHandler = player.player_screen_handler.clone();
    screen
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour_mut()
        .sync_handler = Some(sync);
    player.resync_active_inventory_screen();
    while packets.try_recv().is_ok() {}

    let cursor = ItemStack::new(1, &Item::DIAMOND);
    let expected = NetworkItemStackDescriptor::from(&cursor).id;
    let epoch = {
        let mut handler = screen.lock().unwrap_or_else(PoisonError::into_inner);
        let behaviour = handler.get_behaviour_mut();
        *behaviour
            .cursor_stack
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = cursor;
        behaviour.sync_epoch()
    };
    assert!(
        ScreenUpdateContext {
            screen: &screen,
            active: None,
            epoch: epoch.wrapping_add(1)
        }
        .lock_if_current()
        .is_none()
    );

    player.drop_held_item(false);
    assert_eq!(player.inventory().held_item().item_count, 1);
    let mut cursors = Vec::new();
    while let Ok(packet) = packets.try_recv() {
        if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
            && i32::from(id) == CInventoryContent::PACKET_ID
        {
            let mut payload = payload.as_ref();
            if VarUInt::read(&mut payload)?.0 == 59 {
                assert_eq!(VarUInt::read(&mut payload)?.0, 1);
                cursors.push(NetworkItemStackDescriptor::read(&mut payload)?.id);
            }
        }
    }
    assert_eq!(cursors, [expected]);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_block_fallback_rejects_a_hotbar_change_in_the_interact_callback() -> TestResult {
    use crate::plugin::player::player_interact_event::PlayerInteractEvent;
    use pumpkin_data::{Block, item::Item, item_stack::ItemStack};
    use pumpkin_protocol::java::server::play::SUseItemOn;
    use pumpkin_util::math::position::BlockPos;

    struct SelectOtherSlot(AtomicBool);
    impl EventRwHandler<PlayerInteractEvent> for SelectOtherSlot {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PlayerInteractEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                if !self.0.swap(true, Ordering::Relaxed) {
                    event.player.inventory().set_selected_slot(1);
                }
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let support = BlockPos::new(9, 63, 8);
    let target = BlockPos::new(9, 64, 8);
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(9, 63, 8, Block::STONE.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (player, _peer) = java_player(&server, &world, "Notch")?;
    player.get_entity().set_pos(Vector3::new(8.5, 64.0, 7.5));
    let inventory = player.inventory();
    inventory.set_selected_slot(0);
    // The generated spawn can overlap these coordinates on some seeds.
    player
        .permission_lvl
        .store(pumpkin_util::permission::PermissionLvl::Four);
    let dirt = ItemStack::new(2, &Item::DIRT);
    let stick = ItemStack::new(1, &Item::STICK);
    inventory.set_slot(0, dirt.clone());
    inventory.set_slot(1, stick.clone());
    let listener = Arc::new(SelectOtherSlot(AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(listener.clone(), EventPriority::Normal, None);

    let client = player.client.java().ok_or("Java client")?;
    let mut click = SUseItemOn {
        hand: 0.into(),
        position: support,
        face: 1.into(),
        cursor_pos: Vector3::new(0.5, 1.0, 0.5),
        inside_block: false,
        is_against_world_border: false,
        sequence: 1.into(),
    };
    client.handle_use_item_on(&player, &click, &server)?;
    assert!(listener.0.load(Ordering::Relaxed));
    assert_eq!(inventory.get_selected_slot(), 1);
    assert_eq!(inventory.get_slot(0).uid, dirt.uid);
    assert_eq!(inventory.get_slot(0).item_count, 2);
    assert_eq!(inventory.get_slot(1).uid, stick.uid);
    assert_eq!(world.get_block(&target), &Block::AIR);

    inventory.set_selected_slot(0);
    click.sequence = 2.into();
    client.handle_use_item_on(&player, &click, &server)?;
    assert_eq!(inventory.get_slot(0).item_count, 1);
    assert_eq!(inventory.get_slot(1).uid, stick.uid);
    assert_eq!(world.get_block(&target), &Block::DIRT);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_block_use_keeps_the_block_replaced_by_the_interact_callback() -> TestResult {
    use crate::plugin::player::player_interact_event::PlayerInteractEvent;
    use pumpkin_data::Block;
    use pumpkin_protocol::java::server::play::SUseItemOn;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_world::world::BlockFlags;

    struct ReplaceDetector(BlockPos, AtomicBool);
    impl EventRwHandler<PlayerInteractEvent> for ReplaceDetector {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PlayerInteractEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.1.store(true, Ordering::Relaxed);
                event.player.world().set_block_state(
                    &self.0,
                    Block::STONE.default_state.id,
                    BlockFlags::NOTIFY_ALL,
                );
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let position = BlockPos::new(9, 63, 8);
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(9, 63, 8, Block::DAYLIGHT_DETECTOR.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let (player, _peer) = java_player(&server, &world, "Notch")?;
    player.get_entity().set_pos(Vector3::new(8.5, 64.0, 7.5));
    let listener = Arc::new(ReplaceDetector(position, AtomicBool::new(false)));
    server
        .plugin_manager
        .register_blocking::<PlayerInteractEvent, _>(listener.clone(), EventPriority::Normal, None);

    assert_eq!(world.get_block(&position), &Block::DAYLIGHT_DETECTOR);
    player
        .client
        .java()
        .ok_or("Java client")?
        .handle_use_item_on(
            &player,
            &SUseItemOn {
                hand: 0.into(),
                position,
                face: 1.into(),
                cursor_pos: Vector3::new(0.5, 1.0, 0.5),
                inside_block: false,
                is_against_world_border: false,
                sequence: 1.into(),
            },
            &server,
        )?;
    assert!(listener.1.load(Ordering::Relaxed));
    assert_eq!(world.get_block(&position), &Block::STONE);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_drop_updates_the_menu_opened_by_its_listener() -> TestResult {
    use crate::entity::player::Player;
    use crate::plugin::api::events::player::player_drop_item::PlayerDropItemEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::{
        generic_container_screen_handler::create_generic_9x3, inventory::SimpleInventory,
        screen_handler::SharedScreenHandler,
    };
    use pumpkin_protocol::java::client::play::CKeepAlive;
    use pumpkin_util::text::TextComponent;

    struct OpenChest(Arc<Player>, SharedScreenHandler, AtomicBool);
    impl EventRwHandler<PlayerDropItemEvent> for OpenChest {
        fn handle<'a>(
            &'a self,
            _: &'a Arc<Server>,
            _: &'a mut PlayerDropItemEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async move {
                self.0
                    .open_handled_screen_direct(self.1.clone(), &TextComponent::text("Chest"));
                self.2.store(true, Ordering::Relaxed);
            })
        }
    }

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, peer) = java_equipment_watcher(&server, &world)?;
    player
        .screen_handler_sync_handler
        .store_player(player.clone());
    player.inventory().set_selected_slot(0);
    player
        .inventory()
        .set_held_item(ItemStack::new(1, &Item::STICK));
    let chest: SharedScreenHandler = Arc::new(Mutex::new(create_generic_9x3(
        1,
        player.inventory(),
        Arc::new(SimpleInventory::new(27)),
        player.as_ref(),
    )));
    let observer = Arc::new(OpenChest(
        player.clone(),
        chest.clone(),
        AtomicBool::new(false),
    ));
    server
        .plugin_manager
        .register_blocking::<PlayerDropItemEvent, _>(observer.clone(), EventPriority::Normal, None);

    player.drop_held_item(false);
    assert!(observer.2.load(Ordering::Relaxed));
    assert!(player.is_active_screen(&chest));
    assert!(player.inventory().held_item().is_empty());
    assert!(
        player
            .client
            .java()
            .ok_or("Java player")?
            .try_send_packet(&CKeepAlive::new(73))
    );
    let observed = read_inventory_screen(peer).await?;
    assert_eq!(observed.content.get(54), Some(&Item::STICK.id));
    assert_eq!(observed.slots, [Item::AIR.id]);
    assert!(matches!(
        observed.slot_destinations.as_slice(),
        [(Some(1), 54) | (None, 0)]
    ));
    world.level.shutdown().await;
    Ok(())
}

struct ReplaceDropStack(pumpkin_data::item_stack::ItemStack);
impl EventRwHandler<crate::plugin::api::events::player::player_drop_item::PlayerDropItemEvent>
    for ReplaceDropStack
{
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut crate::plugin::api::events::player::player_drop_item::PlayerDropItemEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            event.player.inventory().set_held_item(self.0.clone());
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_rejected_drop_forces_a_hotbar_correction() -> TestResult {
    use crate::plugin::api::events::player::player_drop_item::PlayerDropItemEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::screen_handler::ScreenHandler;
    use pumpkin_protocol::{
        codec::item_stack_seralizer::ItemStackSerializer,
        java::{
            client::play::{CKeepAlive, CSetContainerContent},
            packet_decoder::TCPNetworkDecoder,
        },
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, peer) = java_equipment_watcher(&server, &world)?;
    let sync = player.screen_handler_sync_handler.clone();
    sync.store_player(player.clone());
    let original = ItemStack::new(2, &Item::STICK);
    let replacement = ItemStack::new(2, &Item::STICK);
    assert_ne!(original.uid, replacement.uid);
    player.inventory().set_selected_slot(0);
    player.inventory().set_held_item(original.clone());
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_behaviour_mut()
        .sync_handler = Some(sync);
    player
        .player_screen_handler
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .set_received_stack(36, original);
    server
        .plugin_manager
        .register_blocking::<PlayerDropItemEvent, _>(
            Arc::new(ReplaceDropStack(replacement.clone())),
            EventPriority::Normal,
            None,
        );

    player.drop_held_item(false);
    assert_eq!(player.inventory().held_item().uid, replacement.uid);
    assert!(
        player
            .client
            .java()
            .ok_or("Java player")?
            .try_send_packet(&CKeepAlive::new(73))
    );
    let mut decoder = TCPNetworkDecoder::new(tokio::io::BufReader::new(peer));
    let correction: Option<ItemStack> =
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            let mut hotbar = None;
            loop {
                let packet = decoder.get_raw_packet().await?;
                if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                    return Ok::<_, Box<dyn std::error::Error>>(hotbar);
                }
                if packet.id == CSetContainerContent::to_id(CURRENT_MC_VERSION) {
                    let mut payload = packet.payload.as_ref();
                    assert_eq!(payload.get_container_id(&CURRENT_MC_VERSION)?.0, 0);
                    let _revision = payload.get_var_int()?;
                    assert!(payload.get_var_int()?.0 > 36);
                    for _ in 0..=36 {
                        hotbar = Some(
                            ItemStackSerializer::read_with_version(
                                &mut payload,
                                &CURRENT_MC_VERSION,
                            )?
                            .0
                            .into_owned(),
                        );
                    }
                }
            }
        })
        .await??;
    let correction = correction.ok_or("Missing forced window-0 correction")?;
    assert_eq!(correction.item, &Item::STICK);
    assert_eq!(correction.item_count, 2);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rejected_drop_resends_the_replacement_stack() -> TestResult {
    use crate::plugin::api::events::player::player_drop_item::PlayerDropItemEvent;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_protocol::{
        Packet,
        bedrock::{
            client::inventory_slot::CInventorySlot,
            network_item::{FullContainerName, NetworkItemStackDescriptor},
        },
        codec::var_uint::VarUInt,
        serial::PacketRead,
    };

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    let original = ItemStack::new(2, &Item::STICK);
    let replacement = ItemStack::new(2, &Item::STICK);
    assert_ne!(original.uid, replacement.uid);
    assert!(original.are_equal(&replacement));
    player.inventory().set_selected_slot(0);
    player.inventory().set_held_item(original);
    server
        .plugin_manager
        .register_blocking::<PlayerDropItemEvent, _>(
            Arc::new(ReplaceDropStack(replacement.clone())),
            EventPriority::Normal,
            None,
        );
    while packets.try_recv().is_ok() {}

    player.drop_held_item(false);
    assert_eq!(player.inventory().held_item().uid, replacement.uid);
    let mut corrections = Vec::new();
    while let Ok(packet) = packets.try_recv() {
        if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
            && i32::from(id) == CInventorySlot::PACKET_ID
        {
            let mut payload = payload.as_ref();
            if VarUInt::read(&mut payload)?.0 == 0 && VarUInt::read(&mut payload)?.0 == 0 {
                Option::<FullContainerName>::read(&mut payload)?;
                Option::<NetworkItemStackDescriptor>::read(&mut payload)?;
                let stack = NetworkItemStackDescriptor::read(&mut payload)?;
                corrections.push((stack.net_id, stack.stack_size));
            }
        }
    }
    assert_eq!(corrections, [(Some(replacement.uid), 2)]);
    world.level.shutdown().await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_beacon_sync_uses_the_final_payment_in_the_batch() -> TestResult {
    use crate::block::entities::beacon::BeaconBlockEntity;
    use pumpkin_data::{item::Item, item_stack::ItemStack};
    use pumpkin_inventory::{
        Inventory, beacon_screen_handler::create_beacon_handler, screen_handler::ScreenHandler,
    };
    use pumpkin_protocol::{
        Packet,
        bedrock::{
            client::inventory_slot::CInventorySlot,
            network_item::{ContainerName, FullContainerName, NetworkItemStackDescriptor},
            server::item_stack_request::{
                ItemStackRequest, ItemStackRequestAction, ItemStackRequestSlotInfo,
                SItemStackRequest,
            },
        },
        codec::var_uint::VarUInt,
        serial::PacketRead,
    };
    use pumpkin_util::{math::position::BlockPos, text::TextComponent};

    let folder = tempfile::tempdir()?;
    let (server, world) = inventory_world(&folder).await?;
    let (player, mut packets) = bedrock_player(&server, &world, "jeb_").await?;
    let client = player.client.bedrock().ok_or("Bedrock player")?;
    let beacon = Arc::new(BeaconBlockEntity::new(BlockPos::new(8, 63, 8)));
    beacon.set_stack(0, ItemStack::new(1, &Item::GOLD_INGOT));
    let refill = ItemStack::new(1, &Item::IRON_INGOT);
    let menu = create_beacon_handler(1, player.inventory(), beacon.clone());
    *menu
        .get_behaviour()
        .cursor_stack
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = refill.clone();
    player
        .screen_handler_sync_handler
        .store_player(player.clone());
    player.open_handled_screen_direct(Arc::new(Mutex::new(menu)), &TextComponent::text("Beacon"));
    while packets.try_recv().is_ok() {}

    let slot = |container_name, stack_id| ItemStackRequestSlotInfo {
        container_name: FullContainerName {
            container_name,
            dynamic_id: None,
        },
        slot_id: 0,
        stack_id,
    };
    client.handle_item_stack_request(
        &player,
        SItemStackRequest {
            requests: vec![ItemStackRequest {
                request_id: (-3).into(),
                actions: vec![
                    ItemStackRequestAction::BeaconPayment {
                        primary_effect_id: 0.into(),
                        secondary_effect_id: 0.into(),
                    },
                    ItemStackRequestAction::Place {
                        count: 1,
                        source: slot(ContainerName::Cursor, refill.uid.get()),
                        destination: slot(ContainerName::BeaconPayment, 0),
                    },
                ],
                filter_strings: Vec::new(),
                filter_cause: 0,
            }],
        },
    );

    let committed_payment = beacon.get_stack(0);
    assert!(committed_payment.are_equal(&refill));
    let mut payment_updates = Vec::new();
    while let Ok(packet) = packets.try_recv() {
        if let Some((id, payload)) = crate::net::bedrock::split_raw_packet(&packet.data)
            && i32::from(id) == CInventorySlot::PACKET_ID
        {
            let mut payload = payload.as_ref();
            let _window = VarUInt::read(&mut payload)?;
            let _slot = VarUInt::read(&mut payload)?;
            if Option::<FullContainerName>::read(&mut payload)?
                .is_some_and(|name| name.container_name == ContainerName::BeaconPayment)
            {
                Option::<NetworkItemStackDescriptor>::read(&mut payload)?;
                payment_updates.push(NetworkItemStackDescriptor::read(&mut payload)?.net_id);
            }
        }
    }
    assert_eq!(payment_updates, [Some(committed_payment.uid)]);
    world.level.shutdown().await;
    Ok(())
}
