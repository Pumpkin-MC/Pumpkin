use std::{
    io::Cursor,
    sync::{Arc, Weak},
};

use arc_swap::ArcSwap;
use pumpkin_config::world::LevelConfig;
use pumpkin_data::{damage::DamageType, dimension::Dimension, entity::EntityType};
use pumpkin_nbt::{deserializer::NbtReadHelperJava, tag::NbtTag};
use pumpkin_protocol::{
    ClientPacket,
    bedrock::server::text::{SText, TextPacketType},
    java::client::play::CSystemChatMessage,
    serial::{PacketRead, PacketWrite},
};
use pumpkin_util::{
    math::{vector2::Vector2, vector3::Vector3},
    text::{TextComponent, TextComponentBase, TextContent, hover::HoverEvent},
    version::JavaMinecraftVersion,
};
use pumpkin_world::{chunk::ChunkData, level::Level, world_info::LevelData};
use tempfile::TempDir;
use uuid::Uuid;

use super::{
    Entity, EntityBase, combat::FallLocation, living::LivingEntity, passive::wolf::WolfEntity,
};
use crate::{block::registry::default_registry, world::World};

fn test_world() -> std::io::Result<(TempDir, Arc<World>)> {
    let directory = TempDir::new()?;
    let level = Level::from_root_folder(
        &LevelConfig::default(),
        directory.path().to_path_buf(),
        0,
        Dimension::OVERWORLD,
    );
    level
        .loaded_chunks
        .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
    let info = Arc::new(ArcSwap::from_pointee(LevelData::default(level.seed)));
    let world = Arc::new(World::load(
        level,
        info,
        Dimension::OVERWORLD,
        default_registry(),
        Weak::new(),
    ));
    Ok((directory, world))
}

fn assert_death_message(
    victim: &LivingEntity,
    wolf: &WolfEntity,
    expected_java_name: TextComponent,
    expected_parameter: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let attacker_id = {
        let mut tracker = victim
            .combat_tracker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // MobEntity::try_attack supplies the wolf as both source and cause.
        tracker.record_damage(
            0,
            true,
            0.0,
            FallLocation::Generic,
            DamageType::MOB_ATTACK,
            4.0,
            Some(wolf),
            Some(wolf),
        );
        tracker
            .get_killer_entry()
            .and_then(|entry| entry.attacker_id)
    };
    assert_eq!(attacker_id, Some(wolf.get_entity().entity_id));
    let message =
        LivingEntity::get_death_message(victim, DamageType::MOB_ATTACK, Some(wolf), Some(wolf));
    let uuid = wolf.get_entity().entity_uuid.to_string();
    let expected_display = expected_java_name
        .clone()
        .hover_event(HoverEvent::show_entity(
            uuid.clone(),
            "wolf".to_string(),
            Some(expected_java_name),
        ))
        .insertion(uuid);
    let expected_java_message = TextComponent::translate(
        "death.attack.mob",
        [victim.get_display_name(), expected_display],
    );
    let mut actual_java = Vec::new();
    let mut expected_java = Vec::new();
    CSystemChatMessage::new(&message, false)
        .write_packet_data(&mut actual_java, &JavaMinecraftVersion::V_26_3)?;
    CSystemChatMessage::new(&expected_java_message, false)
        .write_packet_data(&mut expected_java, &JavaMinecraftVersion::V_26_3)?;
    let mut actual_input = Cursor::new(actual_java.as_slice());
    let mut expected_input = Cursor::new(expected_java.as_slice());
    // Compound key order is not stable; compare the serialized NBT structure.
    assert_eq!(
        NbtTag::deserialize(&mut NbtReadHelperJava::new(&mut actual_input))?,
        NbtTag::deserialize(&mut NbtReadHelperJava::new(&mut expected_input))?,
        "Java name and metadata must not change"
    );
    assert_eq!(&actual_java[actual_input.position() as usize..], &[0]);

    let TextContent::Translate {
        translate,
        bedrock_translate,
        with,
    } = &*message.0.content
    else {
        return Err("combat must produce a translated death message".into());
    };
    // Exercise the conversion and packet codec used by Player::send_system_message_raw.
    let packet = SText::translation(
        bedrock_translate
            .as_deref()
            .unwrap_or(translate)
            .to_string(),
        with.iter()
            .map(TextComponentBase::to_bedrock_string)
            .collect(),
    );
    let mut bytes = Vec::new();
    packet.write(&mut bytes)?;
    let mut input = bytes.as_slice();
    let decoded = SText::read(&mut input)?;
    assert!(input.is_empty());
    assert!(decoded.needs_translation);
    assert_eq!(decoded.r#type, TextPacketType::Translation);
    assert_eq!(decoded.message, "death.attack.mob");
    assert_eq!(decoded.parameters, ["Victim", expected_parameter]);
    assert_eq!(wolf.get_name().0.to_bedrock_string(), expected_parameter);
    assert_eq!(
        wolf.get_display_name().0.to_bedrock_string(),
        expected_parameter
    );
    Ok(())
}

#[tokio::test]
async fn wolf_names_survive_combat_and_editioned_death_messages()
-> Result<(), Box<dyn std::error::Error>> {
    let (_directory, world) = test_world()?;
    let wolf = WolfEntity::new(Entity::from_uuid(
        Uuid::new_v4(),
        world.clone(),
        Vector3::new(1.0, 0.0, 1.0),
        &EntityType::WOLF,
    ));
    let victim = LivingEntity::new(Entity::from_uuid(
        Uuid::new_v4(),
        world.clone(),
        Vector3::new(2.0, 0.0, 1.0),
        &EntityType::PLAYER,
    ));
    victim
        .entity
        .custom_name
        .store(Arc::new(Some(TextComponent::text("Victim"))));
    // All remaining operations use in-memory names and combat records, not world ticks.
    world.level.shutdown().await;

    assert_death_message(
        &victim,
        &wolf,
        TextComponent::translate("entity.minecraft.wolf", []),
        "%entity.wolf.name",
    )?;
    assert_eq!(
        wolf.get_name()
            .to_json_value_for_version(&JavaMinecraftVersion::V_26_3),
        serde_json::json!({"translate": "entity.minecraft.wolf"})
    );
    assert!(wolf.get_name().0.style.hover_event.is_none());
    assert!(wolf.get_name().0.style.insertion.is_none());

    let custom = TextComponent::text("Luna")
        .bold()
        .add_child(TextComponent::text(" Jr."));
    wolf.get_entity()
        .custom_name
        .store(Arc::new(Some(custom.clone())));
    assert_eq!(wolf.get_name(), custom);
    assert_death_message(&victim, &wolf, custom, "Luna Jr.")?;
    Ok(())
}
