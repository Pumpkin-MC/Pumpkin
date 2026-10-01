//! Component payload regressions for block entity drops.

use super::*;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use std::io::Cursor;

/// Profile payloads identify the optional-name/UUID representation as a partial profile.
#[test]
fn profile_uses_partial_discriminator() -> Result<(), Box<dyn std::error::Error>> {
    let original = ProfileImpl {
        name: Some("fixture".into()),
        id: Some([1, 2, 3, 4]),
        properties: vec![ProfileProperty {
            name: "textures".into(),
            value: "payload".into(),
            signature: Some("signature".into()),
        }],
        model: Some("slim".into()),
        ..Default::default()
    };
    let mut bytes = Vec::new();
    original.serialize(&mut bytes)?;
    assert_eq!(
        bytes.first(),
        Some(&0),
        "partial profile discriminator must be false"
    );
    assert_eq!(ProfileImpl::deserialize(&mut Cursor::new(bytes))?, original);
    Ok(())
}

/// Complete client profiles use the true discriminator and an unwrapped UUID and name.
#[test]
fn profile_reads_complete_client_identity() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    bytes.write_bool(true)?;
    bytes.write_uuid(&uuid::Uuid::from_u128(1))?;
    bytes.write_string("Fixture")?;
    bytes.write_var_int(&VarInt(0))?;
    bytes.write_bool(false)?;
    bytes.write_bool(false)?;
    bytes.write_bool(false)?;
    bytes.write_bool(true)?;
    bytes.write_bool(true)?;
    let decoded = ProfileImpl::deserialize(&mut Cursor::new(bytes))?;
    assert_eq!(decoded.name.as_deref(), Some("Fixture"));
    assert_eq!(decoded.id, Some([0, 0, 0, 1]));
    assert_eq!(decoded.model.as_deref(), Some("slim"));
    Ok(())
}

/// Banner layers encode a registered holder instead of an incomplete inline holder.
#[test]
fn banner_patterns_preserve_registry_names() -> Result<(), Box<dyn std::error::Error>> {
    let original = BannerPatternsImpl {
        layers: vec![BannerPatternLayer {
            pattern: "minecraft:creeper".into(),
            color: pumpkin_data::dye_color::DyeColor::Red,
        }],
    };
    let mut bytes = Vec::new();
    original.serialize(&mut bytes)?;
    assert_eq!(
        BannerPatternsImpl::deserialize(&mut Cursor::new(bytes))?,
        original
    );
    Ok(())
}

/// Sparse shulker contents retain their slot indices in the dense wire representation.
#[test]
fn container_preserves_sparse_slots() -> Result<(), Box<dyn std::error::Error>> {
    let original = ContainerImpl {
        items: vec![
            (0, ItemStack::new(1, &Item::DIAMOND)),
            (10, ItemStack::new(2, &Item::EMERALD)),
            (26, ItemStack::new(3, &Item::GOLD_INGOT)),
        ],
    };
    let mut bytes = Vec::new();
    original.serialize(&mut bytes)?;
    let decoded = ContainerImpl::deserialize(&mut Cursor::new(bytes))?;
    assert_eq!(
        decoded
            .items
            .iter()
            .map(|(slot, stack)| (*slot, stack.item.id, stack.item_count))
            .collect::<Vec<_>>(),
        vec![
            (0, Item::DIAMOND.id, 1),
            (10, Item::EMERALD.id, 2),
            (26, Item::GOLD_INGOT.id, 3)
        ]
    );
    Ok(())
}

/// Styled custom names retain both their text and boolean formatting on the wire.
#[test]
fn custom_names_preserve_wire_style() -> Result<(), Box<dyn std::error::Error>> {
    let original = CustomNameImpl {
        name: pumpkin_util::text::TextComponent::text("Styled name").bold(),
    };
    let mut bytes = Vec::new();
    original.serialize(&mut bytes)?;
    assert_eq!(
        CustomNameImpl::deserialize(&mut Cursor::new(bytes))?,
        original
    );
    Ok(())
}

/// An occupant's registry type precedes its untyped compound and both residence timers.
#[test]
fn bees_preserve_wire_entity_and_timers() -> Result<(), Box<dyn std::error::Error>> {
    let mut entity = pumpkin_nbt::compound::NbtCompound::new();
    entity.put_string("id", "minecraft:bee".into());
    entity.put_bool("HasNectar", true);
    let original = BeesImpl {
        bees: vec![BeeData {
            entity_data: entity,
            ticks_in_hive: 37,
            min_ticks_in_hive: 2400,
        }],
    };
    let mut bytes = Vec::new();
    original.serialize(&mut bytes)?;
    let mut reader = Cursor::new(&bytes);
    assert_eq!(reader.get_var_int()?.0, 1);
    assert_eq!(reader.get_var_int()?.0, i32::from(EntityType::BEE.id));
    let Some(NbtTag::Compound(data)) =
        reader.get_nbt_with_version(&JavaMinecraftVersion::V_26_3)?
    else {
        return Err("Missing occupant compound".into());
    };
    assert!(data.get_string("id").is_none());
    assert_eq!(data.get_bool("HasNectar"), Some(true));
    assert_eq!(reader.get_var_int()?.0, 37);
    assert_eq!(reader.get_var_int()?.0, 2400);
    assert_eq!(BeesImpl::deserialize(&mut Cursor::new(bytes))?, original);
    Ok(())
}

/// Four distinct pottery faces retain the documented back, left, right, front wire order.
#[test]
fn pot_decorations_preserve_wire_face_order() -> Result<(), Box<dyn std::error::Error>> {
    let original = PotDecorationsImpl {
        decorations: [
            &Item::ANGLER_POTTERY_SHERD,
            &Item::ARCHER_POTTERY_SHERD,
            &Item::ARMS_UP_POTTERY_SHERD,
            &Item::BLADE_POTTERY_SHERD,
        ]
        .map(|item| {
            Some(PotDecoration {
                item,
                count: 1,
                patch: Vec::new(),
            })
        }),
    };
    let mut bytes = Vec::new();
    original.serialize(&mut bytes)?;
    let mut reader = Cursor::new(&bytes);
    for expected in [
        Item::ANGLER_POTTERY_SHERD.id,
        Item::ARCHER_POTTERY_SHERD.id,
        Item::ARMS_UP_POTTERY_SHERD.id,
        Item::BLADE_POTTERY_SHERD.id,
    ] {
        assert!(reader.get_bool()?);
        assert_eq!(reader.get_var_int()?.0, i32::from(expected));
        assert_eq!(reader.get_var_int()?.0, 1);
        assert_eq!(reader.get_var_int()?.0, 0);
        assert_eq!(reader.get_var_int()?.0, 0);
    }
    assert_eq!(reader.position(), bytes.len() as u64);
    assert_eq!(
        PotDecorationsImpl::deserialize(&mut Cursor::new(bytes))?,
        original
    );
    Ok(())
}

/// Optional pot templates decode item-first counts and unprefixed component patches without losing faces.
#[test]
fn pot_decorations_decode_optional_item_templates() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    bytes.write_bool(true)?;
    bytes.write_var_int(&VarInt(i32::from(Item::ANGLER_POTTERY_SHERD.id)))?;
    bytes.write_var_int(&VarInt(3))?;
    bytes.write_var_int(&VarInt(1))?;
    bytes.write_var_int(&VarInt(1))?;
    bytes.write_var_int(&VarInt(i32::from(DataComponent::RepairCost.to_id())))?;
    bytes.write_var_int(&VarInt(7))?;
    bytes.write_var_int(&VarInt(i32::from(DataComponent::CustomName.to_id())))?;
    bytes.write_bool(false)?;
    bytes.write_bool(false)?;
    bytes.write_bool(true)?;
    bytes.write_var_int(&VarInt(i32::from(Item::BRICK.id)))?;
    bytes.write_var_int(&VarInt(1))?;
    bytes.write_var_int(&VarInt(0))?;
    bytes.write_var_int(&VarInt(0))?;
    let mut reader = Cursor::new(&bytes);
    let decoded = PotDecorationsImpl::deserialize(&mut reader)?;
    assert_eq!(reader.position(), bytes.len() as u64);
    let mut patch = pumpkin_nbt::compound::NbtCompound::new();
    patch.put_int("minecraft:repair_cost", 7);
    patch.put_compound(
        "!minecraft:custom_name",
        pumpkin_nbt::compound::NbtCompound::new(),
    );
    let mut back = pumpkin_nbt::compound::NbtCompound::new();
    back.put_string("id", "minecraft:angler_pottery_sherd".into());
    back.put_int("count", 3);
    back.put_compound("components", patch);
    let mut front = pumpkin_nbt::compound::NbtCompound::new();
    front.put_string("id", "minecraft:brick".into());
    let mut expected = pumpkin_nbt::compound::NbtCompound::new();
    expected.put_compound("back", back);
    expected.put_compound("front", front);
    assert_eq!(decoded.write_data(), NbtTag::Compound(expected));
    let mut encoded = Vec::new();
    decoded.serialize(&mut encoded)?;
    assert_eq!(encoded, bytes);
    Ok(())
}

/// Wire template counts remain full positive integers instead of wrapping at the runtime stack's byte limit.
#[test]
fn pot_decorations_preserve_large_wire_counts() -> Result<(), Box<dyn std::error::Error>> {
    for count in [256, i32::MAX] {
        let mut bytes = Vec::new();
        bytes.write_bool(true)?;
        bytes.write_var_int(&VarInt(i32::from(Item::BRICK.id)))?;
        bytes.write_var_int(&VarInt(count))?;
        bytes.write_var_int(&VarInt(0))?;
        bytes.write_var_int(&VarInt(0))?;
        bytes.extend_from_slice(&[0, 0, 0]);
        let mut reader = Cursor::new(&bytes);
        let decoded = PotDecorationsImpl::deserialize(&mut reader)?;
        assert_eq!(reader.position(), bytes.len() as u64);
        assert_eq!(
            decoded.decorations[0].as_ref().map(|face| face.count),
            Some(count)
        );
        let mut encoded = Vec::new();
        decoded.serialize(&mut encoded)?;
        assert_eq!(encoded, bytes);
    }
    Ok(())
}

/// Invalid item IDs, nonpositive counts, component lengths, and truncated templates fail explicitly.
#[test]
fn pot_decorations_reject_invalid_wire_templates() -> Result<(), Box<dyn std::error::Error>> {
    for values in [
        vec![-1],
        vec![i32::MAX],
        vec![i32::from(u16::MAX)],
        vec![i32::from(Item::AIR.id)],
        vec![i32::from(Item::BRICK.id), 0],
        vec![i32::from(Item::BRICK.id), -1],
        vec![i32::from(Item::BRICK.id), 1, -1, 0],
        vec![i32::from(Item::BRICK.id), 1, 257, 0],
        vec![i32::from(Item::BRICK.id), 1, i32::MAX, 1],
        vec![i32::from(Item::BRICK.id), 1, 1, 0, 256],
        vec![i32::from(Item::BRICK.id), 1, 0, 1, -1],
        vec![
            i32::from(Item::BRICK.id),
            1,
            0,
            2,
            i32::from(DataComponent::CustomName.to_id()),
            i32::from(DataComponent::CustomName.to_id()),
        ],
    ] {
        let mut bytes = vec![1];
        for value in values {
            bytes.write_var_int(&VarInt(value))?;
        }
        assert!(PotDecorationsImpl::deserialize(&mut Cursor::new(bytes)).is_err());
    }
    for truncated in [vec![], vec![0], vec![0, 0], vec![0, 0, 0], vec![1]] {
        assert!(PotDecorationsImpl::deserialize(&mut Cursor::new(truncated)).is_err());
    }
    assert_eq!(
        PotDecorationsImpl::deserialize(&mut Cursor::new([0; 4]))?,
        PotDecorationsImpl::EMPTY
    );
    Ok(())
}

/// Outbound templates reject empty items and invalid patches before dispatching typed payload codecs.
#[test]
fn pot_decorations_reject_invalid_outbound_templates() {
    for face in [
        PotDecoration {
            item: &Item::AIR,
            count: 1,
            patch: Vec::new(),
        },
        PotDecoration {
            item: &Item::BRICK,
            count: 0,
            patch: Vec::new(),
        },
        PotDecoration {
            item: &Item::BRICK,
            count: 1,
            patch: vec![(DataComponent::CustomName, None); 2],
        },
        PotDecoration {
            item: &Item::BRICK,
            count: 1,
            patch: vec![(
                DataComponent::CustomName,
                Some(Box::new(RepairCostImpl { cost: 7 })),
            )],
        },
    ] {
        let original = PotDecorationsImpl {
            decorations: [Some(face), None, None, None],
        };
        assert!(original.serialize(&mut Vec::new()).is_err());
    }
}

/// Network-only component values remain in the pot wire patch even though NBT excludes them.
#[test]
fn pot_decorations_preserve_transient_wire_components() -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = vec![1];
    for value in [
        i32::from(Item::BRICK.id),
        1,
        1,
        1,
        i32::from(DataComponent::CreativeSlotLock.to_id()),
        i32::from(DataComponent::MapPostProcessing.to_id()),
    ] {
        bytes.write_var_int(&VarInt(value))?;
    }
    bytes.extend_from_slice(&[0, 0, 0]);
    let mut reader = Cursor::new(&bytes);
    let decoded = PotDecorationsImpl::deserialize(&mut reader)?;
    assert_eq!(reader.position(), bytes.len() as u64);
    let face = decoded.decorations[0].as_ref().ok_or("Missing pot face")?;
    assert_eq!(face.patch.len(), 2);
    assert_eq!(face.patch[0].0, DataComponent::CreativeSlotLock);
    assert!(face.patch[0].1.is_some());
    assert_eq!(face.patch[1].0, DataComponent::MapPostProcessing);
    assert!(face.patch[1].1.is_none());
    let mut encoded = Vec::new();
    decoded.serialize(&mut encoded)?;
    assert_eq!(encoded, bytes);
    Ok(())
}

/// A dense list may reach slot 255, while invalid lengths fail before reading or allocating entries.
#[test]
fn container_enforces_256_slot_bound() -> Result<(), Box<dyn std::error::Error>> {
    let original = ContainerImpl {
        items: vec![(255, ItemStack::new(1, &Item::DIAMOND))],
    };
    let mut bytes = Vec::new();
    original.serialize(&mut bytes)?;
    let decoded = ContainerImpl::deserialize(&mut Cursor::new(bytes))?;
    assert_eq!(decoded.items.first().map(|(slot, _)| *slot), Some(255));
    for length in [-1, 257, i32::MAX] {
        let mut bytes = Vec::new();
        bytes.write_var_int(&VarInt(length))?;
        assert!(matches!(
            ContainerImpl::deserialize(&mut Cursor::new(bytes)),
            Err(ReadingError::TooLarge(_))
        ));
    }
    Ok(())
}

/// Creates a named item with a lock and deferred loot for fallback-codec coverage.
fn locked_loot_stack(name: &CustomNameImpl) -> ItemStack {
    ItemStack::new_with_component(
        1,
        &Item::SHULKER_BOX,
        vec![
            (
                DataComponent::Lock,
                Some(
                    LockImpl {
                        predicate: pumpkin_nbt::compound::NbtCompound::new(),
                    }
                    .to_dyn(),
                ),
            ),
            (
                DataComponent::ContainerLoot,
                Some(
                    ContainerLootImpl {
                        loot_table: "minecraft:chests/simple_dungeon".into(),
                        seed: 17,
                    }
                    .to_dyn(),
                ),
            ),
            (DataComponent::CustomName, Some(name.clone().to_dyn())),
        ],
    )
}

/// Locks and deferred loot use the fallback NBT codec in both item packet formats.
#[test]
fn lock_and_container_loot_preserve_network_payloads() -> Result<(), Box<dyn std::error::Error>> {
    use crate::codec::item_stack_seralizer::ItemStackSerializer;
    let name = CustomNameImpl {
        name: pumpkin_util::text::TextComponent::text("Retained"),
    };
    let stack = locked_loot_stack(&name);
    for prefixed in [false, true] {
        let mut bytes = Vec::new();
        let serializer = ItemStackSerializer(Cow::Borrowed(&stack));
        if prefixed {
            serializer
                .write_length_prefixed_with_version(&mut bytes, &JavaMinecraftVersion::V_26_3)?;
        } else {
            serializer.write(&mut bytes)?;
        }
        let mut reader = Cursor::new(bytes);
        assert_eq!(reader.get_var_int()?.0, 1);
        assert_eq!(reader.get_var_int()?.0, i32::from(Item::SHULKER_BOX.id));
        assert_eq!(reader.get_var_int()?.0, 3);
        assert_eq!(reader.get_var_int()?.0, 0);
        assert_eq!(
            reader.get_var_int()?.0,
            i32::from(DataComponent::Lock.to_id())
        );
        if prefixed {
            assert!(reader.get_var_int()?.0 > 0);
        }
        assert_eq!(
            LockImpl::deserialize(&mut reader)?.predicate,
            pumpkin_nbt::compound::NbtCompound::new()
        );
        assert_eq!(
            reader.get_var_int()?.0,
            i32::from(DataComponent::ContainerLoot.to_id())
        );
        if prefixed {
            assert!(reader.get_var_int()?.0 > 0);
        }
        assert_eq!(
            ContainerLootImpl::deserialize(&mut reader)?,
            ContainerLootImpl {
                loot_table: "minecraft:chests/simple_dungeon".into(),
                seed: 17
            }
        );
        assert_eq!(
            reader.get_var_int()?.0,
            i32::from(DataComponent::CustomName.to_id())
        );
        if prefixed {
            assert!(reader.get_var_int()?.0 > 0);
        }
        assert_eq!(CustomNameImpl::deserialize(&mut reader)?, name);
        assert_eq!(reader.position() as usize, reader.get_ref().len());
    }
    let mut nested = Vec::new();
    serialize_item_stack_template(&stack, &mut nested)?;
    let mut reader = Cursor::new(nested);
    assert_eq!(reader.get_var_int()?.0, i32::from(Item::SHULKER_BOX.id));
    assert_eq!(reader.get_var_int()?.0, 1);
    assert_eq!(reader.get_var_int()?.0, 3);
    assert_eq!(reader.get_var_int()?.0, 0);
    assert_eq!(
        reader.get_var_int()?.0,
        i32::from(DataComponent::Lock.to_id())
    );
    assert_eq!(
        LockImpl::deserialize(&mut reader)?.predicate,
        pumpkin_nbt::compound::NbtCompound::new()
    );
    assert_eq!(
        reader.get_var_int()?.0,
        i32::from(DataComponent::ContainerLoot.to_id())
    );
    assert_eq!(
        ContainerLootImpl::deserialize(&mut reader)?,
        ContainerLootImpl {
            loot_table: "minecraft:chests/simple_dungeon".into(),
            seed: 17
        }
    );
    assert_eq!(
        reader.get_var_int()?.0,
        i32::from(DataComponent::CustomName.to_id())
    );
    assert_eq!(CustomNameImpl::deserialize(&mut reader)?, name);
    assert_eq!(reader.position() as usize, reader.get_ref().len());
    assert!(deserialize(DataComponent::Lock, &mut Cursor::new([])).is_err());
    Ok(())
}

/// Tooltip flags and selected hidden component IDs survive wire transport without reordering.
#[test]
fn tooltip_display_preserves_wire_selection() -> Result<(), Box<dyn std::error::Error>> {
    let original = TooltipDisplayImpl {
        hide_tooltip: true,
        hidden_components: vec![DataComponent::BannerPatterns, DataComponent::Rarity],
    };
    let mut bytes = Vec::new();
    original.serialize(&mut bytes)?;
    assert_eq!(
        TooltipDisplayImpl::deserialize(&mut Cursor::new(bytes))?,
        original
    );
    Ok(())
}

/// Styled item names encode as structured NBT and survive length-prefixed creative item decoding.
#[test]
fn item_names_preserve_structured_packet_text() -> Result<(), Box<dyn std::error::Error>> {
    use crate::codec::item_stack_seralizer::ItemStackSerializer;
    let name =
        ItemNameImpl::Component(pumpkin_util::text::TextComponent::text("Named banner").bold());
    let stack = ItemStack::new_with_component(
        1,
        &Item::WHITE_BANNER,
        vec![(DataComponent::ItemName, Some(name.clone().to_dyn()))],
    );
    let mut bytes = Vec::new();
    ItemStackSerializer(Cow::Borrowed(&stack))
        .write_length_prefixed_with_version(&mut bytes, &JavaMinecraftVersion::V_26_3)?;
    let decoded =
        ItemStackSerializer::read_length_prefixed_optional(&mut Cursor::new(bytes))?.to_stack();
    assert_eq!(decoded.get_data_component::<ItemNameImpl>(), Some(&name));
    Ok(())
}
