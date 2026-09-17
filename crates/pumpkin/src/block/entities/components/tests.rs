use super::*;
use crate::block::entities::{BlockEntity, block_entity_from_nbt, create_block_entity};
use pumpkin_data::data_component_impl::{ContainerImpl, CustomNameImpl, LockImpl, RepairCostImpl};
use pumpkin_data::{Block, item::Item};
use pumpkin_inventory::Inventory;
use pumpkin_util::{math::position::BlockPos, text::TextComponent};

/// Recreates an entity using its persisted registry identifier and position.
fn reload(entity: &dyn BlockEntity) -> Result<std::sync::Arc<dyn BlockEntity>, String> {
    let mut nbt = NbtCompound::new();
    entity.write_internal(&mut nbt);
    block_entity_from_nbt(&nbt)
        .ok_or_else(|| format!("Cannot reload {}", entity.resource_location()))
}

/// Encodes a collected value for comparisons independent of trait-object identity.
fn encoded(entity: &dyn BlockEntity, id: DataComponent) -> Option<NbtTag> {
    entity
        .collect_components()
        .into_iter()
        .find(|(key, _)| *key == id)
        .map(|(_, value)| value.write_data())
}

/// Every block covered by bundled copy-components tables preserves placement additions through both save formats.
#[test]
fn bundled_table_entities_preserve_placement_and_replacement()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/datapacks/26_2/data/minecraft/loot_table/blocks");
    let mut tested = 0;
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "json") {
            continue;
        }
        let table = std::fs::read_to_string(&path)?;
        if !table.contains("minecraft:copy_components") {
            continue;
        }
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or("Invalid table filename")?;
        let block = Block::from_name(name).ok_or_else(|| format!("Missing block {name}"))?;
        let item = Item::from_registry_key(name).ok_or_else(|| format!("Missing item {name}"))?;
        let mut stack = ItemStack::new(1, item);
        let name_component = CustomNameImpl {
            name: TextComponent::text("Archive").bold(),
        };
        stack.set_data_component(name_component.clone());
        stack.set_data_component(RepairCostImpl { cost: 7 });
        let entity = create_block_entity(
            block.default_state.block_entity_type,
            BlockPos::new(3, 64, 9),
        )
        .ok_or_else(|| format!("Missing entity for {name}"))?;
        entity.apply_item_components(&stack);
        let reloaded = reload(entity.as_ref())?;
        assert_eq!(
            encoded(reloaded.as_ref(), DataComponent::CustomName),
            Some(name_component.write_data()),
            "{name}"
        );
        assert_eq!(
            encoded(reloaded.as_ref(), DataComponent::RepairCost),
            Some(NbtTag::Int(7)),
            "{name}"
        );

        let mut collected = ItemStack::new(1, item);
        collected.patch = reloaded
            .collect_components()
            .into_iter()
            .map(|(id, value)| (id, Some(value)))
            .collect();
        let mut item_nbt = NbtCompound::new();
        collected.write_item_stack(&mut item_nbt);
        let collected = ItemStack::read_item_stack(&item_nbt)
            .ok_or_else(|| format!("Cannot reload dropped {name}"))?;
        let replacement = create_block_entity(
            block.default_state.block_entity_type,
            BlockPos::new(8, 64, 9),
        )
        .ok_or_else(|| format!("Missing replacement for {name}"))?;
        replacement.apply_item_components(&collected);
        let replacement = reload(replacement.as_ref())?;
        assert_eq!(
            encoded(replacement.as_ref(), DataComponent::CustomName),
            Some(name_component.write_data()),
            "{name}"
        );
        assert_eq!(
            encoded(replacement.as_ref(), DataComponent::RepairCost),
            Some(NbtTag::Int(7)),
            "{name}"
        );
        assert_eq!(
            replacement.resource_location(),
            entity.resource_location(),
            "{name}"
        );
        tested += 1;
    }
    assert!(
        tested >= 70,
        "The bundled table coverage inventory unexpectedly shrank: {tested}"
    );
    Ok(())
}

/// Live slots supersede retained data and survive item and block persistence without compaction.
#[test]
fn shulker_slots_preserve_positions_and_current_values() -> Result<(), Box<dyn std::error::Error>> {
    use crate::block::entities::shulker_box::ShulkerBoxBlockEntity;
    let shulker = ShulkerBoxBlockEntity::new(BlockPos::new(0, 64, 0));
    let mut source = ItemStack::new(1, &Item::SHULKER_BOX);
    source.set_data_component(ContainerImpl {
        items: vec![
            (0, ItemStack::new(2, &Item::DIAMOND)),
            (10, ItemStack::new(3, &Item::EMERALD)),
            (26, ItemStack::new(4, &Item::GOLD_INGOT)),
        ],
    });
    shulker.apply_item_components(&source);
    shulker.set_stack(10, ItemStack::new(5, &Item::IRON_INGOT));
    let reloaded = reload(&shulker)?;
    let mut collected = ItemStack::new(1, &Item::SHULKER_BOX);
    collected.patch = reloaded
        .collect_components()
        .into_iter()
        .map(|(id, value)| (id, Some(value)))
        .collect();
    let mut nbt = NbtCompound::new();
    collected.write_item_stack(&mut nbt);
    let collected = ItemStack::read_item_stack(&nbt).ok_or("Cannot decode shulker item")?;
    let replacement = ShulkerBoxBlockEntity::new(BlockPos::new(1, 64, 0));
    replacement.apply_item_components(&collected);
    for (slot, item, count) in [
        (0, &Item::DIAMOND, 2),
        (10, &Item::IRON_INGOT, 5),
        (26, &Item::GOLD_INGOT, 4),
    ] {
        let actual = replacement.get_stack(slot);
        assert_eq!(actual.item.id, item.id);
        assert_eq!(actual.item_count, count);
    }
    assert!(replacement.get_stack(1).is_empty());
    Ok(())
}

/// Deferred loot and lock predicates persist without being consumed or added to container chunk updates.
#[test]
fn deferred_loot_and_lock_survive_collection_without_unpacking()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::block::entities::shulker_box::ShulkerBoxBlockEntity;
    let mut source = ItemStack::new(1, &Item::SHULKER_BOX);
    let mut predicate = NbtCompound::new();
    predicate.put_string("items", "minecraft:tripwire_hook".to_owned());
    let lock = LockImpl { predicate };
    let loot = ContainerLootImpl {
        loot_table: "minecraft:chests/simple_dungeon".to_owned(),
        seed: 8123,
    };
    source.set_data_component(lock.clone());
    source.set_data_component(loot.clone());
    let shulker = ShulkerBoxBlockEntity::new(BlockPos::new(0, 64, 0));
    shulker.apply_item_components(&source);
    let reloaded = reload(&shulker)?;
    assert_eq!(
        encoded(reloaded.as_ref(), DataComponent::Lock),
        Some(lock.write_data())
    );
    assert_eq!(
        encoded(reloaded.as_ref(), DataComponent::ContainerLoot),
        Some(loot.write_data())
    );
    assert!(reloaded.has_loot_table());
    let visible = reloaded.chunk_data_nbt().ok_or("Missing chunk NBT")?;
    assert!(visible.get("lock").is_none());
    assert!(visible.get("LootTable").is_none());
    assert!(visible.get("LootTableSeed").is_none());
    assert_eq!(
        reloaded.take_loot_table(),
        Some((loot.loot_table, loot.seed))
    );
    assert!(!reloaded.has_loot_table());
    assert!(encoded(reloaded.as_ref(), DataComponent::ContainerLoot).is_none());
    Ok(())
}

/// Implicit scalar fields override stored additions and placement ignores consumed block-data patches.
#[test]
fn implicit_fields_override_retained_additions() {
    let mut nbt = NbtCompound::new();
    nbt.put_string("CustomName", "Current".to_owned());
    let mut retained = NbtCompound::new();
    retained.put_string("minecraft:custom_name", "Stale".to_owned());
    retained.put_int("minecraft:repair_cost", 3);
    nbt.put_compound("components", retained);
    let state = BlockEntityComponents::from_nbt(&nbt, CONTAINER_FIELDS);
    assert_eq!(
        state.with_component::<CustomNameImpl, _>(|name| name.name.clone().get_text()),
        Some("Current".to_owned())
    );
    let mut stack = ItemStack::new(1, &Item::CHEST);
    stack.set_data_component(pumpkin_data::data_component_impl::BlockEntityDataImpl { nbt });
    stack.set_data_component(RepairCostImpl { cost: 9 });
    state.apply(&stack, &[]);
    assert!(state.with_component::<CustomNameImpl, _>(|_| ()).is_none());
    assert_eq!(
        state.with_component::<RepairCostImpl, _>(|repair| repair.cost),
        Some(9)
    );
    assert!(
        !state
            .collect()
            .iter()
            .any(|(id, _)| *id == DataComponent::BlockEntityData)
    );
}

/// Preserves a version-matched implicit payload from block NBT through an item and replacement block.
fn round_trip_payload(
    block_name: &str,
    entity_id: &str,
    field: &str,
    id: DataComponent,
    expected: NbtTag,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut nbt = NbtCompound::new();
    nbt.put_string("id", entity_id.to_owned());
    nbt.put_int("x", 0);
    nbt.put_int("y", 64);
    nbt.put_int("z", 0);
    nbt.put(field, expected.clone());
    let entity = block_entity_from_nbt(&nbt).ok_or("Cannot decode payload entity")?;
    let entity = reload(entity.as_ref())?;
    assert_eq!(encoded(entity.as_ref(), id), Some(expected.clone()));
    let item = Item::from_registry_key(block_name).ok_or("Missing payload item")?;
    let mut stack = ItemStack::new(1, item);
    stack.patch = entity
        .collect_components()
        .into_iter()
        .map(|(id, value)| (id, Some(value)))
        .collect();
    let mut item_nbt = NbtCompound::new();
    stack.write_item_stack(&mut item_nbt);
    let stack = ItemStack::read_item_stack(&item_nbt).ok_or("Cannot decode payload item")?;
    let block = Block::from_name(block_name).ok_or("Missing payload block")?;
    let replacement = create_block_entity(
        block.default_state.block_entity_type,
        BlockPos::new(1, 64, 0),
    )
    .ok_or("Cannot create payload replacement")?;
    replacement.apply_item_components(&stack);
    let replacement = reload(replacement.as_ref())?;
    assert_eq!(encoded(replacement.as_ref(), id), Some(expected));
    Ok(())
}

/// Skull profiles retain explicit identities, signed texture properties, and note-block sounds.
#[test]
fn skull_profile_and_sound_survive_replacement() -> Result<(), Box<dyn std::error::Error>> {
    let mut profile = NbtCompound::new();
    profile.put_string("name", "ArchiveOwner".to_owned());
    profile.put("id", NbtTag::IntArray(vec![1, 2, 3, 4]));
    let mut property = NbtCompound::new();
    property.put_string("name", "textures".to_owned());
    property.put_string("value", "texture-payload".to_owned());
    property.put_string("signature", "texture-signature".to_owned());
    profile.put_list("properties", vec![NbtTag::Compound(property)]);
    round_trip_payload(
        "player_head",
        "minecraft:skull",
        "profile",
        DataComponent::Profile,
        NbtTag::Compound(profile),
    )?;
    round_trip_payload(
        "player_head",
        "minecraft:skull",
        "note_block_sound",
        DataComponent::NoteBlockSound,
        NbtTag::String("minecraft:block.note_block.bell".into()),
    )
}

/// All four pot faces keep their documented back, left, right, front order through replacement.
#[test]
fn pot_faces_survive_replacement() -> Result<(), Box<dyn std::error::Error>> {
    let faces = ["angler", "archer", "arms_up", "blade"]
        .into_iter()
        .map(|face| NbtTag::String(format!("minecraft:{face}_pottery_sherd").into()))
        .collect();
    round_trip_payload(
        "decorated_pot",
        "minecraft:decorated_pot",
        "sherds",
        DataComponent::PotDecorations,
        NbtTag::List(faces),
    )
}

/// Bee entity data and both timers survive collection without advancing or spawning the bee.
#[test]
fn bee_payloads_survive_replacement() -> Result<(), Box<dyn std::error::Error>> {
    let mut entity_data = NbtCompound::new();
    entity_data.put_string("id", "minecraft:bee".to_owned());
    entity_data.put_bool("HasNectar", true);
    entity_data.put_int("Age", -1200);
    let mut bee = NbtCompound::new();
    bee.put_compound("entity_data", entity_data);
    bee.put_int("ticks_in_hive", 123);
    bee.put_int("min_ticks_in_hive", 2400);
    round_trip_payload(
        "beehive",
        "minecraft:beehive",
        "bees",
        DataComponent::Bees,
        NbtTag::List(vec![NbtTag::Compound(bee)]),
    )
}

/// Banner patterns retain both their pattern identifiers and colors across item placement.
#[test]
fn banner_patterns_survive_replacement() -> Result<(), Box<dyn std::error::Error>> {
    let mut pattern = NbtCompound::new();
    pattern.put_string("pattern", "minecraft:stripe_bottom".to_owned());
    pattern.put_string("color", "red".to_owned());
    round_trip_payload(
        "white_banner",
        "minecraft:banner",
        "patterns",
        DataComponent::BannerPatterns,
        NbtTag::List(vec![NbtTag::Compound(pattern)]),
    )
}

/// Styled banner item labels, rarity, and tooltip visibility remain stored additions rather than being lost on placement.
#[test]
fn banner_retained_metadata_survives_replacement() -> Result<(), Box<dyn std::error::Error>> {
    use crate::block::entities::banner::BannerBlockEntity;
    let mut item_name = NbtCompound::new();
    item_name.put_string("text", "Pattern catalogue".to_owned());
    item_name.put_bool("italic", true);
    let mut tooltip = NbtCompound::new();
    tooltip.put_bool("hide_tooltip", true);
    tooltip.put_list(
        "hidden_components",
        vec![NbtTag::String("minecraft:banner_patterns".into())],
    );
    let additions = [
        (DataComponent::ItemName, NbtTag::Compound(item_name)),
        (DataComponent::Rarity, NbtTag::String("epic".into())),
        (DataComponent::TooltipDisplay, NbtTag::Compound(tooltip)),
    ];
    let mut item = ItemStack::new(1, &Item::WHITE_BANNER);
    for (id, tag) in &additions {
        item.patch.push((
            *id,
            Some(read_data(*id, tag).ok_or("Cannot decode banner metadata")?),
        ));
    }
    let banner = BannerBlockEntity::new(BlockPos::new(0, 64, 0));
    banner.apply_item_components(&item);
    let reloaded = reload(&banner)?;
    let mut item = ItemStack::new(1, &Item::WHITE_BANNER);
    item.patch = reloaded
        .collect_components()
        .into_iter()
        .map(|(id, value)| (id, Some(value)))
        .collect();
    let mut item_nbt = NbtCompound::new();
    item.write_item_stack(&mut item_nbt);
    let item = ItemStack::read_item_stack(&item_nbt).ok_or("Cannot reload banner metadata")?;
    let replacement = BannerBlockEntity::new(BlockPos::new(1, 64, 0));
    replacement.apply_item_components(&item);
    let reloaded = reload(&replacement)?;
    for (id, expected) in additions {
        assert_eq!(encoded(reloaded.as_ref(), id), Some(expected));
    }
    Ok(())
}

/// Retained conditional components survive absent implicit fields and do not become active deferred loot.
#[test]
fn retained_lock_and_loot_remain_distinct_from_implicit_fields() {
    let mut retained = NbtCompound::new();
    let loot = ContainerLootImpl {
        loot_table: "minecraft:chests/simple_dungeon".to_owned(),
        seed: 42,
    };
    retained.put("minecraft:container_loot", loot.write_data());
    let mut lock = NbtCompound::new();
    lock.put_string("items", "minecraft:tripwire_hook".to_owned());
    retained.put_compound("minecraft:lock", lock.clone());
    let mut nbt = NbtCompound::new();
    nbt.put_compound("components", retained.clone());
    let state = BlockEntityComponents::from_nbt(&nbt, RANDOMIZABLE_CONTAINER_FIELDS);
    assert!(!state.has_loot_table());
    assert_eq!(state.take_loot_table(), None);
    let collected = state.collect();
    assert!(
        collected
            .iter()
            .any(|(id, value)| *id == DataComponent::ContainerLoot
                && value.write_data() == loot.write_data())
    );
    assert!(
        collected
            .iter()
            .any(|(id, value)| *id == DataComponent::Lock
                && value.write_data() == NbtTag::Compound(lock.clone()))
    );
    let mut saved = NbtCompound::new();
    state.write_nbt(&mut saved);
    assert_eq!(saved.get_compound("components"), Some(&retained));
    assert!(saved.get("LootTable").is_none());
    assert!(saved.get("lock").is_none());
}

/// A pot's pending loot stays in block NBT and is not exposed as an implicit item component.
#[test]
fn pot_deferred_loot_is_persistent_without_becoming_an_item_component() {
    use crate::block::entities::decorated_pot::DecoratedPotBlockEntity;
    let mut nbt = NbtCompound::new();
    nbt.put_string("LootTable", "minecraft:chests/simple_dungeon".to_owned());
    nbt.put_long("LootTableSeed", 901);
    let pot = DecoratedPotBlockEntity::from_nbt(&nbt, BlockPos::new(0, 64, 0));
    assert!(pot.has_loot_table());
    assert!(encoded(&pot, DataComponent::ContainerLoot).is_none());
    let mut saved = NbtCompound::new();
    pot.write_nbt(&mut saved);
    assert_eq!(
        saved.get_string("LootTable"),
        Some("minecraft:chests/simple_dungeon")
    );
    assert_eq!(saved.get_long("LootTableSeed"), Some(901));
}

/// Custom update tags retain their mapped fields while stored additions survive only in saved NBT.
#[test]
fn custom_chunk_tags_omit_retained_additions() -> Result<(), Box<dyn std::error::Error>> {
    let name = CustomNameImpl {
        name: TextComponent::text("Archive").bold(),
    };
    let mut retained = NbtCompound::new();
    retained.put_int("minecraft:repair_cost", 7);
    let mut lock = NbtCompound::new();
    lock.put_string("items", "minecraft:tripwire_hook".to_owned());
    retained.put_compound("minecraft:lock", lock);
    retained.put(
        "minecraft:container_loot",
        ContainerLootImpl {
            loot_table: "minecraft:chests/simple_dungeon".to_owned(),
            seed: 42,
        }
        .write_data(),
    );
    let pot_decorations = NbtTag::List(
        ["angler", "archer", "arms_up", "blade"]
            .map(|name| NbtTag::String(format!("minecraft:{name}_pottery_sherd").into()))
            .into(),
    );
    for (id, field, expected) in [
        ("beacon", "CustomName", name.write_data()),
        ("skull", "custom_name", name.write_data()),
        ("enchanting_table", "CustomName", name.write_data()),
        ("copper_golem_statue", "custom_name", name.write_data()),
        ("decorated_pot", "sherds", pot_decorations),
    ] {
        let mut input = NbtCompound::new();
        input.put_string("id", format!("minecraft:{id}"));
        input.put_int("x", 0);
        input.put_int("y", 64);
        input.put_int("z", 0);
        input.put(field, expected.clone());
        input.put_compound("components", retained.clone());
        let entity = block_entity_from_nbt(&input).ok_or("Cannot load update-tag fixture")?;
        let visible = entity.chunk_data_nbt().ok_or("Missing chunk update tag")?;
        assert_eq!(visible.get(field), Some(&expected), "{id}");
        assert!(visible.get("components").is_none(), "{id}");
        let mut saved = NbtCompound::new();
        entity.write_nbt(&mut saved);
        assert_eq!(saved.get_compound("components"), Some(&retained), "{id}");
        let reloaded = reload(entity.as_ref())?;
        assert_eq!(
            encoded(reloaded.as_ref(), DataComponent::RepairCost),
            Some(NbtTag::Int(7)),
            "{id}"
        );
    }
    Ok(())
}

/// Beacon locks and pending pot loot remain in their custom update tags without unpacking the pot.
#[test]
fn custom_chunk_tags_preserve_family_specific_fields() -> Result<(), Box<dyn std::error::Error>> {
    use crate::block::entities::{
        beacon::BeaconBlockEntity, decorated_pot::DecoratedPotBlockEntity,
    };
    let mut input = NbtCompound::new();
    let mut lock = NbtCompound::new();
    lock.put_string("items", "minecraft:tripwire_hook".to_owned());
    input.put_compound("lock", lock.clone());
    let beacon = BeaconBlockEntity::from_nbt(&input, BlockPos::new(0, 64, 0));
    let visible = beacon.chunk_data_nbt().ok_or("Missing beacon update tag")?;
    assert_eq!(visible.get_compound("lock"), Some(&lock));

    input.put_string("LootTable", "minecraft:chests/simple_dungeon".to_owned());
    input.put_long("LootTableSeed", 901);
    let pot = DecoratedPotBlockEntity::from_nbt(&input, BlockPos::new(1, 64, 0));
    let visible = pot.chunk_data_nbt().ok_or("Missing pot update tag")?;
    assert_eq!(
        visible.get_string("LootTable"),
        input.get_string("LootTable")
    );
    assert_eq!(visible.get_long("LootTableSeed"), Some(901));
    assert!(visible.get("item").is_none());
    assert!(pot.has_loot_table());
    Ok(())
}

/// Banner updates use the full component-bearing tag rather than the custom-only tag of skulls and pots.
#[test]
fn banner_chunk_tags_include_retained_additions() -> Result<(), Box<dyn std::error::Error>> {
    use crate::block::entities::banner::BannerBlockEntity;
    let mut input = NbtCompound::new();
    let mut retained = NbtCompound::new();
    retained.put_int("minecraft:repair_cost", 7);
    input.put_compound("components", retained.clone());
    let banner = BannerBlockEntity::from_nbt(&input, BlockPos::new(0, 64, 0));
    let visible = banner.chunk_data_nbt().ok_or("Missing banner update tag")?;
    assert_eq!(visible.get_compound("components"), Some(&retained));
    Ok(())
}
