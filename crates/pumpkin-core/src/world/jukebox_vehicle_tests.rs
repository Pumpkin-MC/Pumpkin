use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use pumpkin_data::{
    Block, block_properties::JukeboxLikeProperties, data_component_impl::JukeboxPlayableImpl,
    game_event::GameEvent, item::Item, item_stack::ItemStack,
};
use pumpkin_protocol::{
    bedrock::server::inventory_transaction::TransactionData, java::server::play::SUseItemOn,
};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2, vector3::Vector3};
use pumpkin_world::chunk::ChunkData;

use crate::{
    block::entities::jukebox::JukeboxBlockEntity,
    entity::{EntityBase, player::Player},
    plugin::{
        BoxFuture, EventPriority, EventRwHandler,
        api::events::world::generic_game::GenericGameEvent,
    },
    server::Server,
};

use super::boat_vehicle_tests::bedrock_use_item_transaction;
use super::vehicle_tests::{TestResult, bedrock_rider, java_rider, vehicle_world};

struct MoveVehicleSource {
    jukebox: Arc<JukeboxBlockEntity>,
    position: BlockPos,
    player: Arc<Player>,
    called: AtomicBool,
    source_was_spent: AtomicBool,
}

impl EventRwHandler<GenericGameEvent> for MoveVehicleSource {
    fn handle<'a>(
        &'a self,
        _: &'a Arc<Server>,
        event: &'a mut GenericGameEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.event_key != GameEvent::BlockChange.name()
                || event.position != self.position.to_centered_f64()
            {
                return;
            }
            let inventory = self.player.inventory();
            let original = inventory.get_slot(0);
            self.source_was_spent
                .store(original.is_empty(), Ordering::Relaxed);
            self.called.store(true, Ordering::Relaxed);
            assert_eq!(self.jukebox.get_record().item, &Item::MINECART);
            if self.player.client.bedrock().is_some() {
                inventory.set_slot(2, original);
                inventory.set_slot(0, ItemStack::new(1, &Item::STONE));
            } else {
                inventory.set_selected_slot(1);
            }
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn java_vehicle_jukebox_commits_before_block_change() -> TestResult {
    vehicle_jukebox_commits_before_block_change(false).await
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_vehicle_jukebox_commits_before_block_change() -> TestResult {
    vehicle_jukebox_commits_before_block_change(true).await
}

async fn vehicle_jukebox_commits_before_block_change(bedrock: bool) -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, world) = vehicle_world(&folder).await?;
    let position = BlockPos::new(8, 63, 8);
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 63, 8, Block::JUKEBOX.default_state.id);
    world.level.loaded_chunks.insert(Vector2::new(0, 0), chunk);
    let jukebox = Arc::new(JukeboxBlockEntity::new(position));
    world.add_block_entity(jukebox.clone());

    let (player, _java_peer, _bedrock_queue) = if bedrock {
        let (player, packets) = bedrock_rider(&server, &world, "jeb_").await?;
        (player, None, Some(packets))
    } else {
        let (player, peer) = java_rider(&server, &world, "Notch")?;
        (player, Some(peer), None)
    };
    player.get_entity().set_pos(Vector3::new(8.9, 64.0, 7.0));
    let inventory = player.inventory();
    inventory.set_selected_slot(0);
    let mut cart = ItemStack::new(1, &Item::MINECART);
    cart.set_data_component(JukeboxPlayableImpl {
        song: "minecraft:cat",
    });
    inventory.set_slot(0, cart.clone());
    let stick = ItemStack::new(1, &Item::STICK);
    inventory.set_slot(1, stick.clone());
    let observer = Arc::new(MoveVehicleSource {
        jukebox: jukebox.clone(),
        position,
        player: player.clone(),
        called: AtomicBool::new(false),
        source_was_spent: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register_blocking::<GenericGameEvent, _>(observer.clone(), EventPriority::Normal, None);

    if let Some(client) = player.client.bedrock() {
        let mut click = bedrock_use_item_transaction(0, &cart, 0);
        if let TransactionData::UseItem(data) = &mut click.transaction_data {
            data.item_in_hand_network_id = Some(cart.uid);
        }
        client.handle_inventory_action(&player, click);
        client.clear_player();
        client.close().await;
    } else {
        let client = player.client.java().ok_or("Java player")?;
        let click = SUseItemOn {
            hand: 0.into(),
            position,
            face: 1.into(),
            cursor_pos: Vector3::new(0.5, 0.75, 0.5),
            inside_block: false,
            is_against_world_border: false,
            sequence: 1.into(),
        };
        client.handle_use_item_on(&player, &click, &server)?;
    }

    assert!(observer.called.load(Ordering::Relaxed));
    assert_eq!(inventory.get_slot(1).uid, stick.uid);
    assert!(inventory.get_slot(2).is_empty());
    assert!(observer.source_was_spent.load(Ordering::Relaxed));
    if bedrock {
        assert_eq!(inventory.get_slot(0).item, &Item::STONE);
    } else {
        assert!(inventory.get_slot(0).is_empty());
        assert_eq!(inventory.get_selected_slot(), 1);
    }
    assert!(jukebox.get_record().are_equal(&cart));
    assert!(JukeboxLikeProperties::from_state_id(world.get_block_state_id(&position)).has_record);
    assert!(jukebox.is_playing());
    world.level.shutdown().await;
    Ok(())
}
