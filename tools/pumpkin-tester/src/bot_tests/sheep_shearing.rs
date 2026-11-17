use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use pumpkin_core::{
    data::datapack::loot_table_loader::parse_loot_table,
    entity::{Entity, EntityBase, passive::sheep::SheepEntity},
    net::ClientPlatform,
    plugin::{
        BoxFuture, EventHandler,
        api::events::{
            EventPriority,
            player::{
                player_item_break::PlayerItemBreakEvent, player_item_damage::PlayerItemDamageEvent,
            },
        },
    },
    server::Server,
};
use pumpkin_data::{
    dimension::Dimension, dye_color::DyeColor, entity::EntityType, item::Item,
    item_stack::ItemStack,
};
use pumpkin_protocol::{
    codec::var_int::VarInt,
    java::server::play::{ActionType, SInteract},
};
use pumpkin_util::{
    GameMode,
    math::{boundingbox::BoundingBox, vector3::Vector3},
};

#[derive(Default)]
struct ItemEvents {
    damage: AtomicUsize,
    broken: AtomicUsize,
    cancel: AtomicBool,
}

impl EventHandler<PlayerItemDamageEvent> for ItemEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerItemDamageEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.item_name == Item::SHEARS.registry_key {
                self.damage.fetch_add(1, Ordering::Relaxed);
                event.cancelled = self.cancel.load(Ordering::Relaxed);
            }
        })
    }
}

impl EventHandler<PlayerItemBreakEvent> for ItemEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerItemBreakEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.item_name == Item::SHEARS.registry_key {
                self.broken.fetch_add(1, Ordering::Relaxed);
            }
        })
    }
}

pub fn run_sheep_shearing_test(
    server: &Arc<Server>,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    let player = world
        .get_player_by_name("TestBot1")
        .ok_or("Could not find TestBot1 for the sheep shearing test")?;
    let ClientPlatform::Java(client) = player.client.as_ref() else {
        return Err("Expected a Java player for the sheep shearing test".into());
    };
    let table = parse_loot_table(
        r#"{"pools":[{"rolls":1,"entries":[{"type":"minecraft:item","name":"minecraft:diamond","conditions":[{"condition":"minecraft:match_tool","predicate":{"items":"minecraft:shears"}}]}]}]}"#,
    )
    .ok_or("Could not parse the shearing match_tool loot table")?;
    server
        .datapack_manager
        .insert_loot_table("minecraft:shearing/sheep/red".into(), Arc::new(table));
    let hooks = Arc::new(ItemEvents::default());
    server.plugin_manager.register::<PlayerItemDamageEvent, _>(
        hooks.clone(),
        EventPriority::Normal,
        true,
    );
    server.plugin_manager.register::<PlayerItemBreakEvent, _>(
        hooks.clone(),
        EventPriority::Normal,
        true,
    );
    player.set_gamemode(GameMode::Survival);
    let pos = player.position().add_raw(2.0, 0.0, 0.0);
    let sheep = SheepEntity::new(Entity::new(world.clone(), pos, &EntityType::SHEEP));
    sheep.set_color(DyeColor::Red.id());
    if !world.spawn_entity(sheep.clone()) {
        return Err("Could not spawn the test sheep".into());
    }
    let interact = SInteract {
        entity_id: VarInt(sheep.get_entity().entity_id),
        r#type: VarInt(ActionType::InteractAt as i32),
        target_position: Some(Vector3::new(0.0, 0.0, 0.0)),
        hand: Some(VarInt(0)),
        sneaking: false,
    };
    let max_damage = ItemStack::new(1, &Item::SHEARS)
        .get_max_damage()
        .ok_or("Shears have no maximum durability")?;
    for (damage, cancel, expected_damage, expected_events, expected_breaks) in [
        (0, false, Some(1), 1, 0),
        (max_damage - 1, false, None, 2, 1),
        (0, true, Some(0), 3, 1),
    ] {
        let mut tool = ItemStack::new(1, &Item::SHEARS);
        tool.set_damage(damage);
        player.inventory.set_held_item(tool);
        sheep.set_sheared(false);
        hooks.cancel.store(cancel, Ordering::Relaxed);
        client.handle_interact(&player, &interact, server);
        let held = player.inventory.held_item();
        let actual_damage = (!held.is_empty()).then(|| held.get_damage());
        let damage_events = hooks.damage.load(Ordering::Relaxed);
        let break_events = hooks.broken.load(Ordering::Relaxed);
        let diamonds: usize = world
            .get_entities_at_box(
                &BoundingBox::from_block(&sheep.get_entity().block_pos.load()).expand_all(3.0),
            )
            .iter()
            .filter_map(|entity| {
                entity
                    .cast_any()
                    .downcast_ref::<pumpkin_core::entity::item::ItemEntity>()
            })
            .map(|entity| {
                let stack = entity
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if stack.item == &Item::DIAMOND {
                    usize::from(stack.item_count)
                } else {
                    0
                }
            })
            .sum();
        if !sheep.is_sheared()
            || actual_damage != expected_damage
            || damage_events != expected_events
            || break_events != expected_breaks
            || diamonds != expected_events
        {
            return Err(format!(
                "Shearing: damage={actual_damage:?} (expected {expected_damage:?}), damage events={damage_events} (expected {expected_events}), break events={break_events} (expected {expected_breaks}), match_tool drops={diamonds} (expected {expected_events})"
            ));
        }
    }
    hooks.cancel.store(false, Ordering::Relaxed);
    player.set_gamemode(GameMode::Creative);
    summary_lines.push("[PASS] bot:shearing_tool_loot_and_item_events".into());
    Ok(())
}
