use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
};

use pumpkin_core::{
    entity::{
        Entity, EntityBase, decoration::leash_knot::LeashKnotEntity, item::ItemEntity,
        passive::snow_golem::SnowGolemEntity, player::Player,
    },
    net::ClientPlatform,
    plugin::{
        BoxFuture, EventHandler,
        api::events::{
            EventPriority,
            block::entity_block_form::EntityBlockFormEvent,
            entity::entity_unleash::EntityUnleashEvent,
            player::{
                player_leash_entity::PlayerLeashEntityEvent,
                player_shear_entity::PlayerShearEntityEvent,
                player_unleash_entity::PlayerUnleashEntityEvent,
            },
        },
    },
    server::Server,
};
use pumpkin_data::{
    entity::EntityType, item::Item, item_stack::ItemStack, statistic::StatisticCategory,
};
use pumpkin_protocol::{
    codec::var_int::VarInt,
    java::server::play::{ActionType, SInteract},
};
use pumpkin_util::{GameMode, Hand};

struct ShearEvents {
    entity_id: i32,
    formations: AtomicUsize,
    hand: AtomicU8,
    count: AtomicUsize,
    cancel: AtomicBool,
    cancel_unleash: AtomicBool,
    cancel_leash: AtomicBool,
}

impl EventHandler<PlayerLeashEntityEvent> for ShearEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerLeashEntityEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                event.cancelled = self.cancel_leash.load(Ordering::Relaxed);
            }
        })
    }
}

impl EventHandler<EntityUnleashEvent> for ShearEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityUnleashEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                event.cancelled = self.cancel_unleash.load(Ordering::Relaxed);
            }
        })
    }
}

impl EventHandler<PlayerShearEntityEvent> for ShearEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerShearEntityEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                self.hand.store(event.hand, Ordering::Relaxed);
                self.count.fetch_add(1, Ordering::Relaxed);
                event.cancelled = self.cancel.load(Ordering::Relaxed);
            }
        })
    }
}

impl EventHandler<EntityBlockFormEvent> for ShearEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityBlockFormEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity.get_entity().entity_id == self.entity_id {
                self.formations.fetch_add(1, Ordering::Relaxed);
                event.cancelled = self.cancel.load(Ordering::Relaxed);
                event.new_state_id = pumpkin_data::Block::MOSS_CARPET.default_state.id;
            }
        })
    }
}

impl EventHandler<PlayerUnleashEntityEvent> for ShearEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerUnleashEntityEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                event.cancelled = self.cancel_unleash.load(Ordering::Relaxed);
            }
        })
    }
}

pub fn run_snow_golem_shearing_test(
    server: &Arc<Server>,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let player = server
        .get_player_by_name("TestBot1")
        .ok_or("Missing TestBot1")?;
    let world = player.world();
    let original_mode = player.gamemode.load();
    let original_main = player.inventory.get_stack_in_hand(Hand::Right);
    let original_off = player.inventory.get_stack_in_hand(Hand::Left);
    let golem = SnowGolemEntity::new(Entity::new(
        world.clone(),
        player.position().add_raw(2.0, 0.0, 0.0),
        &EntityType::SNOW_GOLEM,
    ));
    golem.mob_entity.set_no_ai(true);
    player.set_gamemode(GameMode::Survival);
    let result = if world.spawn_entity(golem.clone()) {
        check_player_shearing(server, &player, &golem)
    } else {
        Err("Could not spawn the snow golem".into())
    };
    golem.get_entity().remove();
    player
        .inventory
        .set_stack_in_hand(Hand::Right, original_main);
    player.inventory.set_stack_in_hand(Hand::Left, original_off);
    player.set_gamemode(original_mode);
    result?;
    summary_lines.push("[PASS] bot:snow_golem_offhand_shearing_and_cancellation".into());
    Ok(())
}

fn check_player_shearing(
    server: &Arc<Server>,
    player: &Arc<Player>,
    golem: &SnowGolemEntity,
) -> Result<(), String> {
    let ClientPlatform::Java(client) = player.client.as_ref() else {
        return Err("Expected a Java player for the snow golem test".into());
    };
    let hooks = Arc::new(ShearEvents {
        entity_id: golem.get_entity().entity_id,
        formations: AtomicUsize::new(0),
        hand: AtomicU8::new(0),
        count: AtomicUsize::new(0),
        cancel: AtomicBool::new(true),
        cancel_unleash: AtomicBool::new(false),
        cancel_leash: AtomicBool::new(false),
    });
    server.plugin_manager.register::<PlayerShearEntityEvent, _>(
        hooks.clone(),
        EventPriority::Normal,
        true,
    );
    let mut tool = ItemStack::new(1, &Item::SHEARS);
    let last_damage = tool.get_max_damage().ok_or("Shears have no durability")? - 1;
    tool.set_damage(last_damage);
    player
        .inventory
        .set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::DIAMOND));
    player.inventory.set_stack_in_hand(Hand::Left, tool);
    let broken_before = shears_stat(player, StatisticCategory::Broken);
    let used_before = shears_stat(player, StatisticCategory::Used);
    let interact = SInteract {
        entity_id: VarInt(golem.get_entity().entity_id),
        r#type: VarInt(ActionType::Interact as i32),
        target_position: None,
        hand: Some(VarInt(1)),
        sneaking: false,
    };
    client.handle_interact(player, &interact, server);
    if !golem.has_pumpkin()
        || player.inventory.get_stack_in_hand(Hand::Left).get_damage() != last_damage
        || hooks.hand.load(Ordering::Relaxed) != 1
    {
        return Err(
            "Cancelled offhand shearing changed the golem or tool, or reported the wrong hand"
                .into(),
        );
    }
    hooks.cancel.store(false, Ordering::Relaxed);
    client.handle_interact(player, &interact, server);
    let broken_after = shears_stat(player, StatisticCategory::Broken);
    let used_after = shears_stat(player, StatisticCategory::Used);
    if golem.has_pumpkin()
        || !player.inventory.get_stack_in_hand(Hand::Left).is_empty()
        || player.inventory.held_item().item != &Item::DIAMOND
        || broken_after != broken_before + 1
        || used_after != used_before + 1
        || hooks.count.load(Ordering::Relaxed) != 2
    {
        return Err(
        "Offhand shearing did not remove the pumpkin, update tool statistics and break only the offhand tool".into(),
    );
    }
    check_shearing_drop(golem)?;
    player
        .inventory
        .set_stack_in_hand(Hand::Left, ItemStack::new(1, &Item::SHEARS));
    client.handle_interact(player, &interact, server);
    if player.inventory.get_stack_in_hand(Hand::Left).get_damage() != 0
        || hooks.count.load(Ordering::Relaxed) != 2
    {
        return Err("A golem without a pumpkin was sheared again".into());
    }
    check_main_hand_equipment_order(server, player, golem, &hooks, last_damage)?;
    check_cancelled_equipment_with_pumpkin(server, player, golem, &hooks)?;
    check_offhand_equipment_without_pumpkin(server, player, golem, &hooks)?;
    check_leash_precedes_pumpkin(server, player, golem, &hooks)?;
    check_leash_replacement_and_knot_cleanup(server, player, golem, &hooks)?;
    check_snow_formation_events(server, golem, &hooks)?;
    check_normal_leashing_and_dead_interaction(server, player, golem, &hooks)?;
    Ok(())
}

fn check_cancelled_equipment_with_pumpkin(
    server: &Arc<Server>,
    player: &Arc<Player>,
    golem: &SnowGolemEntity,
    hooks: &ShearEvents,
) -> Result<(), String> {
    use pumpkin_data::data_component_impl::EquipmentSlot;
    let ClientPlatform::Java(client) = player.client.as_ref() else {
        return Err("Expected a Java player".into());
    };
    golem.set_has_pumpkin(true);
    golem
        .mob_entity
        .set_item_slot(&EquipmentSlot::BODY, ItemStack::new(1, &Item::BLACK_CARPET));
    hooks.cancel.store(true, Ordering::Relaxed);
    for (hand, packet_hand) in [(Hand::Right, 0), (Hand::Left, 1)] {
        player
            .inventory
            .set_stack_in_hand(hand, ItemStack::new(1, &Item::SHEARS));
        let count = hooks.count.load(Ordering::Relaxed);
        let used_before = shears_stat(player, StatisticCategory::Used);
        client.handle_interact(
            player,
            &SInteract {
                entity_id: VarInt(golem.get_entity().entity_id),
                r#type: VarInt(ActionType::Interact as i32),
                target_position: None,
                hand: Some(VarInt(packet_hand)),
                sneaking: false,
            },
            server,
        );
        let used_after = shears_stat(player, StatisticCategory::Used);
        if !golem.has_pumpkin()
            || golem.mob_entity.is_slot_empty(&EquipmentSlot::BODY)
            || player.inventory.get_stack_in_hand(hand).get_damage() != 0
            || hooks.count.load(Ordering::Relaxed) != count + 1
            || hooks.hand.load(Ordering::Relaxed) != packet_hand as u8
            || used_after != used_before
        {
            return Err("Cancelled equipment shearing must retain the equipment and pumpkin without a second shear event or tool use".into());
        }
    }
    golem
        .mob_entity
        .set_item_slot(&EquipmentSlot::BODY, ItemStack::EMPTY.clone());
    golem.set_has_pumpkin(false);
    hooks.cancel.store(false, Ordering::Relaxed);
    Ok(())
}

fn check_offhand_equipment_without_pumpkin(
    server: &Arc<Server>,
    player: &Arc<Player>,
    golem: &SnowGolemEntity,
    hooks: &ShearEvents,
) -> Result<(), String> {
    use pumpkin_data::data_component_impl::EquipmentSlot;
    let ClientPlatform::Java(client) = player.client.as_ref() else {
        return Err("Expected a Java player".into());
    };
    golem
        .mob_entity
        .set_item_slot(&EquipmentSlot::BODY, ItemStack::new(1, &Item::BLACK_CARPET));
    player
        .inventory
        .set_stack_in_hand(Hand::Right, ItemStack::new(1, &Item::DIAMOND));
    player
        .inventory
        .set_stack_in_hand(Hand::Left, ItemStack::new(1, &Item::SHEARS));
    let interact = SInteract {
        entity_id: VarInt(golem.get_entity().entity_id),
        r#type: VarInt(ActionType::Interact as i32),
        target_position: None,
        hand: Some(VarInt(1)),
        sneaking: false,
    };
    let count = hooks.count.load(Ordering::Relaxed);
    for (cancel, expected_damage) in [(true, 0), (false, 1)] {
        hooks.cancel.store(cancel, Ordering::Relaxed);
        client.handle_interact(player, &interact, server);
        if golem.has_pumpkin()
            || player.inventory.get_stack_in_hand(Hand::Left).get_damage() != expected_damage
            || player.inventory.held_item().item != &Item::DIAMOND
            || hooks.hand.load(Ordering::Relaxed) != 1
        {
            return Err("Offhand equipment shearing without a pumpkin used the wrong hand or ignored cancellation".into());
        }
    }
    if hooks.count.load(Ordering::Relaxed) != count + 2 {
        return Err("Offhand equipment shearing did not fire the expected events".into());
    }
    Ok(())
}

fn check_leash_precedes_pumpkin(
    server: &Arc<Server>,
    player: &Arc<Player>,
    golem: &SnowGolemEntity,
    hooks: &Arc<ShearEvents>,
) -> Result<(), String> {
    let ClientPlatform::Java(client) = player.client.as_ref() else {
        return Err("Expected a Java player".into());
    };
    golem.set_has_pumpkin(true);
    let holder: Arc<dyn EntityBase> = player.clone();
    golem.get_entity().attach_leash(&holder);
    player
        .inventory
        .set_stack_in_hand(Hand::Left, ItemStack::new(1, &Item::SHEARS));
    let interact = SInteract {
        entity_id: VarInt(golem.get_entity().entity_id),
        r#type: VarInt(ActionType::Interact as i32),
        target_position: None,
        hand: Some(VarInt(1)),
        sneaking: false,
    };
    server
        .plugin_manager
        .register::<PlayerUnleashEntityEvent, _>(hooks.clone(), EventPriority::Normal, true);
    hooks.cancel.store(false, Ordering::Relaxed);
    hooks.cancel_unleash.store(true, Ordering::Relaxed);
    client.handle_interact(player, &interact, server);
    if golem.get_entity().get_leash_holder().is_none()
        || !golem.has_pumpkin()
        || player.inventory.get_stack_in_hand(Hand::Left).get_damage() != 0
    {
        return Err("Cancelled leash cutting changed the leash, pumpkin or shears".into());
    }
    let entity = golem.get_entity();
    let original_pos = entity.pos.load();
    let world = entity.world.load();
    entity.set_pos(
        player
            .position()
            .add_raw(Entity::LEASH_SNAP_DISTANCE + 2.0, 0.0, 0.0),
    );
    let entities_before = world.entities.load().len();
    let retained = (0..3).all(|_| {
        matches!(
            entity.tick_leash(golem),
            pumpkin_core::entity::LeashTick::NotLeashed
        ) && entity.is_leashed()
            && world.entities.load().len() == entities_before
    });
    entity.set_pos(original_pos);
    if !retained {
        return Err(
            "Cancelled leash snapping must retain the connection without dropping leads".into(),
        );
    }
    hooks.cancel_unleash.store(false, Ordering::Relaxed);
    client.handle_interact(player, &interact, server);
    if golem.get_entity().get_leash_holder().is_some()
        || !golem.has_pumpkin()
        || player.inventory.get_stack_in_hand(Hand::Left).get_damage() != 1
    {
        return Err("Shearing must cut the leash before removing the pumpkin".into());
    }
    Ok(())
}

fn check_leash_replacement_and_knot_cleanup(
    server: &Arc<Server>,
    player: &Arc<Player>,
    golem: &SnowGolemEntity,
    hooks: &Arc<ShearEvents>,
) -> Result<(), String> {
    use pumpkin_core::world::BlockFlags;
    use pumpkin_data::Block;

    let ClientPlatform::Java(client) = player.client.as_ref() else {
        return Err("Expected a Java player".into());
    };
    let entity = golem.get_entity();
    let world = entity.world.load();
    let pos = entity.block_pos.load();
    let previous_block = world.get_block_state_id(&pos);
    world.set_block_state(
        &pos,
        Block::OAK_FENCE.default_state.id,
        BlockFlags::NOTIFY_ALL,
    );
    let knot = LeashKnotEntity::create_knot(&world, pos);
    entity.attach_leash(&knot);
    server.plugin_manager.register::<PlayerLeashEntityEvent, _>(
        hooks.clone(),
        EventPriority::Normal,
        true,
    );
    server.plugin_manager.register::<EntityUnleashEvent, _>(
        hooks.clone(),
        EventPriority::Normal,
        true,
    );
    let result = (|| {
        player
            .inventory
            .set_held_item(ItemStack::new(2, &Item::LEAD));
        let packet = SInteract {
            entity_id: VarInt(entity.entity_id),
            r#type: VarInt(ActionType::Interact as i32),
            target_position: None,
            hand: Some(VarInt(0)),
            sneaking: false,
        };
        hooks.cancel_leash.store(true, Ordering::Relaxed);
        let before = count_dropped_leads(&world);
        client.handle_interact(player, &packet, server);
        if entity
            .get_leash_holder()
            .is_none_or(|holder| holder.get_entity().entity_id != knot.get_entity().entity_id)
            || player.inventory.held_item().item_count != 2
            || count_dropped_leads(&world) != before
        {
            return Err(
                "Cancelled leash replacement must retain the knot connection and both leads".into(),
            );
        }
        hooks.cancel_leash.store(false, Ordering::Relaxed);
        client.handle_interact(player, &packet, server);
        if entity
            .get_leash_holder()
            .is_none_or(|holder| holder.get_entity().entity_id != player.entity_id())
            || player.inventory.held_item().item_count != 1
            || count_dropped_leads(&world) != before + 1
        {
            return Err("Approved leash replacement must attach to the player and drop exactly one old lead".into());
        }

        entity.attach_leash(&knot);
        world.set_block_state(&pos, Block::AIR.default_state.id, BlockFlags::NOTIFY_ALL);
        hooks.cancel_unleash.store(true, Ordering::Relaxed);
        let before = count_dropped_leads(&world);
        for _ in 0..3 {
            knot.tick(knot.as_ref(), server);
            if knot.get_entity().is_removed()
                || !entity.is_leashed()
                || count_dropped_leads(&world) != before
            {
                return Err(
                    "Cancelled knot cleanup must retain the holder without dropping leads".into(),
                );
            }
        }
        hooks.cancel_unleash.store(false, Ordering::Relaxed);
        knot.tick(knot.as_ref(), server);
        if !knot.get_entity().is_removed()
            || entity.is_leashed()
            || count_dropped_leads(&world) != before + 1
        {
            return Err("Approved knot cleanup must remove the holder and drop one lead".into());
        }
        Ok(())
    })();
    hooks.cancel_leash.store(false, Ordering::Relaxed);
    hooks.cancel_unleash.store(false, Ordering::Relaxed);
    entity.remove_leash();
    knot.get_entity().remove();
    world.set_block_state(&pos, previous_block, BlockFlags::NOTIFY_ALL);
    result
}

fn count_dropped_leads(world: &pumpkin_core::world::World) -> usize {
    world
        .entities
        .load()
        .iter()
        .filter_map(|entity| entity.cast_any().downcast_ref::<ItemEntity>())
        .map(|item| {
            let stack = item
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if stack.item == &Item::LEAD {
                usize::from(stack.item_count)
            } else {
                0
            }
        })
        .sum()
}

fn check_shearing_drop(golem: &SnowGolemEntity) -> Result<(), String> {
    let eye_position =
        golem
            .get_entity()
            .pos
            .load()
            .add_raw(0.0, golem.get_entity().get_eye_height(), 0.0);
    let drops: usize = golem
        .get_entity()
        .world
        .load()
        .entities
        .load()
        .iter()
        .filter_map(|entity| entity.cast_any().downcast_ref::<ItemEntity>())
        .filter(|entity| entity.get_entity().pos.load() == eye_position)
        .map(|entity| {
            let stack = entity
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if stack.item == &Item::CARVED_PUMPKIN {
                usize::from(stack.item_count)
            } else {
                0
            }
        })
        .sum();
    if drops != 1 {
        return Err(format!(
            "Expected one shearing loot drop at eye height, got {drops}"
        ));
    }
    Ok(())
}

fn check_main_hand_equipment_order(
    server: &Arc<Server>,
    player: &Arc<Player>,
    golem: &SnowGolemEntity,
    hooks: &ShearEvents,
    last_damage: i32,
) -> Result<(), String> {
    use pumpkin_core::entity::mob::Mob;
    use pumpkin_data::data_component_impl::EquipmentSlot;
    let ClientPlatform::Java(client) = player.client.as_ref() else {
        return Err("Expected a Java player".into());
    };
    let broken_before = shears_stat(player, StatisticCategory::Broken);
    golem.set_has_pumpkin(true);
    golem
        .get_mob_entity()
        .set_item_slot(&EquipmentSlot::BODY, ItemStack::new(1, &Item::BLACK_CARPET));
    let interact = SInteract {
        entity_id: VarInt(golem.get_entity().entity_id),
        r#type: VarInt(ActionType::Interact as i32),
        target_position: None,
        hand: Some(VarInt(0)),
        sneaking: false,
    };
    for (pumpkin, broken, count) in [(true, broken_before + 1, 3), (false, broken_before + 2, 4)] {
        let mut tool = ItemStack::new(1, &Item::SHEARS);
        tool.set_damage(last_damage);
        player.inventory.set_held_item(tool);
        let used_before = shears_stat(player, StatisticCategory::Used);
        client.handle_interact(player, &interact, server);
        let actual_broken = shears_stat(player, StatisticCategory::Broken);
        let used_after = shears_stat(player, StatisticCategory::Used);
        if golem.has_pumpkin() != pumpkin
            || !player.inventory.held_item().is_empty()
            || player.inventory.get_stack_in_hand(Hand::Left).get_damage() != 0
            || actual_broken != broken
            || used_after != used_before + 1
            || hooks.hand.load(Ordering::Relaxed) != 0
            || hooks.count.load(Ordering::Relaxed) != count
        {
            return Err("Shearable equipment must precede the pumpkin, with one main-hand break per interaction".into());
        }
    }
    Ok(())
}

fn shears_stat(player: &Player, category: StatisticCategory) -> i32 {
    player
        .stats
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(category, i32::from(Item::SHEARS.id))
}

fn check_normal_leashing_and_dead_interaction(
    server: &Arc<Server>,
    player: &Arc<Player>,
    golem: &SnowGolemEntity,
    hooks: &ShearEvents,
) -> Result<(), String> {
    let ClientPlatform::Java(client) = player.client.as_ref() else {
        return Err("Expected a Java player".into());
    };
    let packet = SInteract {
        entity_id: VarInt(golem.get_entity().entity_id),
        r#type: VarInt(ActionType::Interact as i32),
        target_position: None,
        hand: Some(VarInt(0)),
        sneaking: false,
    };
    player
        .inventory
        .set_held_item(ItemStack::new(2, &Item::LEAD));
    client.handle_interact(player, &packet, server);
    if golem
        .get_entity()
        .get_leash_holder()
        .is_none_or(|holder| holder.get_entity().entity_id != player.entity_id())
        || player.inventory.held_item().item_count != 1
    {
        return Err("A lead must attach to the snow golem and consume one item".into());
    }
    let mut name_tag = ItemStack::new(1, &Item::NAME_TAG);
    name_tag.set_data_component(pumpkin_data::data_component_impl::CustomNameImpl {
        name: pumpkin_util::text::TextComponent::text("Snowy"),
    });
    player.inventory.set_held_item(name_tag);
    client.handle_interact(player, &packet, server);
    if !golem.get_entity().is_leashed() || golem.get_entity().custom_name.load().is_none() {
        return Err("Naming a leashed golem must retain its leash".into());
    }
    player.inventory.set_held_item(ItemStack::EMPTY.clone());
    client.handle_interact(player, &packet, server);
    if golem.get_entity().is_leashed() || !golem.has_pumpkin() {
        return Err("Empty-hand interaction must remove the lead and retain the pumpkin".into());
    }
    golem.mob_entity.living_entity.set_health(0.0);
    player
        .inventory
        .set_held_item(ItemStack::new(1, &Item::SHEARS));
    let events_before = hooks.count.load(Ordering::Relaxed);
    client.handle_interact(player, &packet, server);
    if !golem.has_pumpkin()
        || player.inventory.held_item().get_damage() != 0
        || hooks.count.load(Ordering::Relaxed) != events_before
    {
        return Err("A dying golem must not be sheared".into());
    }
    Ok(())
}

fn check_snow_formation_events(
    server: &Arc<Server>,
    golem: &SnowGolemEntity,
    hooks: &Arc<ShearEvents>,
) -> Result<(), String> {
    use pumpkin_core::entity::mob::Mob;
    use pumpkin_core::world::BlockFlags;
    use pumpkin_data::Block;
    use pumpkin_util::math::vector3::Vector3;

    server.plugin_manager.register::<EntityBlockFormEvent, _>(
        hooks.clone(),
        EventPriority::Normal,
        true,
    );
    let entity = golem.get_entity();
    let world = entity.world.load();
    let original_pos = entity.pos.load();
    let base = entity.block_pos.load();
    entity.set_pos(Vector3::new(
        f64::from(base.0.x),
        f64::from(base.0.y),
        f64::from(base.0.z),
    ));
    let positions = [base, base.west(), base.north(), base.west().north()];
    let saved: Vec<_> = positions
        .iter()
        .flat_map(|pos| [*pos, pos.down()])
        .map(|pos| (pos, world.get_block_state_id(&pos)))
        .collect();
    let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
    let result = (|| {
        for pos in positions {
            world.set_block_state(&pos.down(), Block::STONE.default_state.id, flags);
            world.set_block_state(&pos, Block::AIR.default_state.id, flags);
        }
        hooks.cancel.store(true, Ordering::Relaxed);
        golem.mob_ai_step();
        if hooks.formations.load(Ordering::Relaxed) != 4
            || positions
                .iter()
                .any(|pos| world.get_block(pos) != &Block::AIR)
        {
            return Err("Cancelled snow formation must leave all four positions unchanged".into());
        }
        hooks.cancel.store(false, Ordering::Relaxed);
        golem.mob_ai_step();
        if hooks.formations.load(Ordering::Relaxed) != 8
            || positions
                .iter()
                .any(|pos| world.get_block(pos) != &Block::MOSS_CARPET)
        {
            return Err("Snow formation must honor a plugin's replacement block state".into());
        }
        Ok(())
    })();
    for (pos, state) in saved {
        world.set_block_state(&pos, state, flags);
    }
    entity.set_pos(original_pos);
    hooks.cancel.store(false, Ordering::Relaxed);
    result
}
