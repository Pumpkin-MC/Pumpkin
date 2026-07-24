use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

use pumpkin_protocol::{
    bedrock::client::remove_actor::CRemoveActor,
    codec::{var_int::VarInt, var_long::VarLong},
    java::client::play::CRemoveEntities,
};
use pumpkin_util::math::{get_section_cord, vector2::Vector2};
use rustc_hash::FxHashSet;
use uuid::Uuid;

use crate::{
    entity::{EntityBase, player::Player},
    net::ClientPlatform,
};

use super::World;

pub(super) struct EntityTracker {
    entity: Arc<dyn EntityBase>,
    tracked_chunk: AtomicU64,
    viewers: Mutex<FxHashSet<Uuid>>,
}

impl EntityTracker {
    fn new(entity: Arc<dyn EntityBase>) -> Self {
        let chunk = entity_chunk(entity.as_ref());
        Self {
            entity,
            tracked_chunk: AtomicU64::new(chunk_key(chunk)),
            viewers: Mutex::new(FxHashSet::default()),
        }
    }
}

impl World {
    pub fn register_entity_tracking(&self, entity: Arc<dyn EntityBase>) {
        let entity_id = entity.get_entity().entity_id;
        self.entity_trackers
            .insert(entity_id, EntityTracker::new(entity));
        self.update_entity_tracking_for_entity_id(entity_id, true);
    }

    pub fn update_entity_tracking_for_player(&self, player: &Arc<Player>) {
        for tracker in &self.entity_trackers {
            Self::update_pairing(tracker.value(), player);
        }
    }

    pub fn update_entity_tracking_for_entity(&self, entity: &Arc<dyn EntityBase>) {
        self.update_entity_tracking_for_entity_id(entity.get_entity().entity_id, false);
    }

    fn update_entity_tracking_for_entity_id(&self, entity_id: i32, force: bool) {
        let Some(tracker) = self.entity_trackers.get(&entity_id) else {
            return;
        };

        let chunk = entity_chunk(tracker.entity.as_ref());
        tracker.entity.get_entity().chunk_pos.store(chunk);
        let previous_chunk = tracker
            .tracked_chunk
            .swap(chunk_key(chunk), Ordering::Relaxed);
        if !force && previous_chunk == chunk_key(chunk) {
            return;
        }

        for player in self.players.load().iter() {
            Self::update_pairing(tracker.value(), player);
        }
    }

    fn update_pairing(tracker: &EntityTracker, player: &Arc<Player>) {
        let chunk = entity_chunk(tracker.entity.as_ref());
        let entity = tracker.entity.get_entity();
        let entity_position = entity.pos.load();
        let player_position = player.position();
        let tracking_range = f64::from(client_tracking_range(entity.entity_type)) * 16.0;
        let should_track = tracking_range > 0.0
            && player.delivered_chunks.contains(&chunk)
            && (entity_position.x - player_position.x).abs() <= tracking_range
            && (entity_position.z - player_position.z).abs() <= tracking_range;
        let player_id = player.gameprofile.id;
        let mut viewers = tracker.viewers.lock().unwrap();

        if should_track {
            if viewers.insert(player_id) {
                player.client.try_enqueue_spawn_packet(&tracker.entity);
            }
        } else if viewers.remove(&player_id) {
            send_remove_entity(player, tracker.entity.get_entity().entity_id);
        }
    }

    pub fn remove_entity_tracking(&self, entity_id: i32) {
        let Some((_, tracker)) = self.entity_trackers.remove(&entity_id) else {
            return;
        };

        let viewers = tracker.viewers.into_inner().unwrap();
        for player_id in viewers {
            if let Some(player) = self.get_player_by_uuid(player_id) {
                send_remove_entity(&player, entity_id);
            }
        }
    }

    pub fn remove_player_from_entity_tracking(&self, player: &Player) {
        let player_id = player.gameprofile.id;
        for tracker in &self.entity_trackers {
            tracker.viewers.lock().unwrap().remove(&player_id);
        }
    }
}

/// Vanilla 26.2 client tracking ranges, measured in chunks.
fn client_tracking_range(entity_type: &pumpkin_data::entity::EntityType) -> i32 {
    match entity_type.resource_name {
        "marker" => 0,
        "arrow" | "breeze_wind_charge" | "cod" | "dragon_fireball" | "egg" | "ender_pearl"
        | "experience_bottle" | "eye_of_ender" | "fireball" | "firework_rocket"
        | "fishing_bobber" | "lingering_potion" | "llama_spit" | "pufferfish" | "salmon"
        | "small_fireball" | "snowball" | "spectral_arrow" | "splash_potion" | "trident"
        | "tropical_fish" | "wind_charge" | "wither_skull" => 4,
        "bat" | "dolphin" => 5,
        "evoker_fangs" | "experience_orb" | "item" => 6,
        "allay"
        | "bee"
        | "blaze"
        | "bogged"
        | "cat"
        | "cave_spider"
        | "chest_minecart"
        | "command_block_minecart"
        | "creaking"
        | "creeper"
        | "drowned"
        | "enderman"
        | "endermite"
        | "evoker"
        | "fox"
        | "furnace_minecart"
        | "guardian"
        | "hoglin"
        | "hopper_minecart"
        | "husk"
        | "illusioner"
        | "magma_cube"
        | "minecart"
        | "mule"
        | "ominous_item_spawner"
        | "parched"
        | "parrot"
        | "phantom"
        | "piglin"
        | "piglin_brute"
        | "pillager"
        | "rabbit"
        | "shulker_bullet"
        | "silverfish"
        | "skeleton"
        | "snow_golem"
        | "spawner_minecart"
        | "spider"
        | "squid"
        | "stray"
        | "tnt_minecart"
        | "vex"
        | "vindicator"
        | "witch"
        | "wither_skeleton"
        | "zoglin"
        | "zombie"
        | "zombie_villager"
        | "zombified_piglin" => 8,
        "end_crystal" | "lightning_bolt" | "warden" => 16,
        "mannequin" | "player" => 32,
        _ => 10,
    }
}

fn entity_chunk(entity: &dyn EntityBase) -> Vector2<i32> {
    let position = entity.get_entity().pos.load();
    Vector2::new(
        get_section_cord(position.x.floor() as i32),
        get_section_cord(position.z.floor() as i32),
    )
}

fn chunk_key(chunk: Vector2<i32>) -> u64 {
    (u64::from(chunk.x as u32) << 32) | u64::from(chunk.y as u32)
}

fn send_remove_entity(player: &Player, entity_id: i32) {
    match player.client.as_ref() {
        ClientPlatform::Java(java) => {
            java.try_enqueue_packet(&CRemoveEntities::new(&[VarInt(entity_id)]));
        }
        ClientPlatform::Bedrock(bedrock) => {
            bedrock.try_enqueue_packet(&CRemoveActor::new(VarLong(i64::from(entity_id))));
        }
    }
}
