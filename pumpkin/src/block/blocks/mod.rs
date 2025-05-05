use pumpkin_data::Block;
use pumpkin_data::screen::WindowType;
use pumpkin_inventory::{Container, OpenContainer};
use pumpkin_util::math::position::BlockPos;

use crate::{entity::player::Player, server::Server};

pub mod cactus;
pub mod chest;
pub mod crafting_table;
pub mod dirt_path;
pub mod doors;
pub mod farmland;
pub mod fence_gates;
pub mod fences;
pub mod fire;
pub mod furnace;
pub mod glass_panes;
pub mod iron_bars;
pub mod jukebox;
pub mod logs;
pub mod nether_portal;
pub mod redstone;
pub mod signs;
pub mod slabs;
pub mod stairs;
pub mod sugar_cane;
pub mod tnt;
pub mod torches;
pub mod walls;

/// The standard destroy with container removes the player forcibly from the container,
/// drops items to the floor, and back to the player's inventory if the item stack is in movement.
pub async fn standard_on_broken_with_container(
    block: &Block,
    player: &Player,
    location: BlockPos,
    server: &Server,
) {
    // TODO: drop all items and back to players inventory if in motion
    if let Some(all_container_ids) = server.get_all_container_ids(location, block.clone()).await {
        let mut open_containers = server.open_containers.write().await;
        for individual_id in all_container_ids {
            if let Some(container) = open_containers.get_mut(&u64::from(individual_id)) {
                container.clear_all_slots().await;
                player.open_container.store(None);
                close_all_in_container(player, container).await;
                container.clear_all_players();
            }
        }
    }
}

/// The standard open container creates a new container if a container of the same block
/// type does not exist at the selected block location. If a container of the same type exists, the player
/// is added to the currently connected players to that container.
pub async fn standard_open_container<C: Container + Default + 'static>(
    block: &Block,
    player: &Player,
    location: BlockPos,
    server: &Server,
    window_type: WindowType,
) {
    let entity_id = player.entity_id();
    // If container exists, add player to container, otherwise create new container
    if let Some(container_id) = server.get_container_id(location, block.clone()).await {
        let mut open_containers = server.open_containers.write().await;
        log::debug!("Using previous standard container id: {container_id}");
        if let Some(container) = open_containers.get_mut(&u64::from(container_id)) {
            container.add_player(entity_id);
            player.open_container.store(Some(container_id.into()));
        }
    } else {
        let mut open_containers = server.open_containers.write().await;
        let new_id = server.new_container_id();
        log::debug!("Creating new standard container id: {new_id}");
        let open_container =
            OpenContainer::new_empty_container::<C>(entity_id, Some(location), Some(block.clone()));
        open_containers.insert(new_id.into(), open_container);
        player.open_container.store(Some(new_id.into()));
    }
    player.open_container(server, window_type).await;
}

pub async fn standard_open_container_unique<C: Container + Default + 'static>(
    block: &Block,
    player: &Player,
    location: BlockPos,
    server: &Server,
    window_type: WindowType,
) {
    {
        let entity_id = player.entity_id();
        let mut open_containers = server.open_containers.write().await;
        let mut id_to_use = -1;

        // TODO: we can do better than brute force
        for (id, container) in open_containers.iter() {
            if let Some(a_block) = container.get_block() {
                if a_block.id == block.id && container.all_player_ids().is_empty() {
                    id_to_use = *id as i64;
                }
            }
        }

        if id_to_use == -1 {
            let new_id = server.new_container_id();
            log::debug!("Creating new unique container id: {new_id}");
            let open_container = OpenContainer::new_empty_container::<C>(
                entity_id,
                Some(location),
                Some(block.clone()),
            );

            open_containers.insert(new_id.into(), open_container);

            player.open_container.store(Some(new_id.into()));
        } else {
            log::debug!("Using previous unique container id: {id_to_use}");
            if let Some(unique_container) = open_containers.get_mut(&(id_to_use as u64)) {
                unique_container.set_location(Some(location)).await;
                unique_container.add_player(entity_id);
                player
                    .open_container
                    .store(Some(id_to_use.try_into().unwrap()));
            }
        }
    }
    player.open_container(server, window_type).await;
}

pub async fn close_all_in_container(player: &Player, container: &OpenContainer) {
    for id in container.all_player_ids() {
        if let Some(remote_player) = player.world().await.get_player_by_id(id).await {
            remote_player.close_container().await;
        }
    }
}
