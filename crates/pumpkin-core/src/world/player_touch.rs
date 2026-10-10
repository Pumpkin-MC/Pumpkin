use std::sync::{Arc, Mutex, PoisonError};

use pumpkin_data::entity::EntityType;
use pumpkin_util::math::{boundingbox::BoundingBox, vector2::Vector2};
use rand::RngExt;

use crate::entity::{EntityBase, player::Player};

/// One player's touch state, taken once per tick after players move.
pub(super) struct PlayerTouch<'a> {
    player: &'a Arc<Player>,
    area: BoundingBox,
    chunk: Vector2<i32>,
    /// Orbs in the pickup area
    orbs: Mutex<Vec<Arc<dyn EntityBase>>>,
}

impl<'a> PlayerTouch<'a> {
    /// `None` for dead or spectating players
    pub(super) fn new(player: &'a Arc<Player>) -> Option<Self> {
        if player.living_entity.health.load() <= 0.0 || player.is_spectator() {
            return None;
        }
        let entity = player.get_entity();
        let hitbox = entity.bounding_box.load();
        // Vanilla `Player.aiStep` pickup volume. Push uses stored hitboxes elsewhere.
        let area = match entity.get_vehicle() {
            Some(vehicle) if !vehicle.get_entity().is_removed() => hitbox
                .minmax(&vehicle.get_entity().bounding_box.load())
                .expand(1.0, 0.0, 1.0),
            _ => hitbox.expand(1.0, 0.5, 1.0),
        };
        Some(Self {
            player,
            area,
            chunk: entity.chunk_pos.load(),
            orbs: Mutex::new(Vec::new()),
        })
    }

    fn touches(&self, entity_bb: &BoundingBox, chunk: Vector2<i32>) -> bool {
        (self.chunk.x - chunk.x).abs() <= 1
            && (self.chunk.y - chunk.y).abs() <= 1
            && self.area.intersects(entity_bb)
    }

    /// Vanilla `Player.aiStep`: one random orb per player per tick.
    pub(super) fn touch_random_orb(&self) {
        let mut orbs = self.orbs.lock().unwrap_or_else(PoisonError::into_inner);
        // Earlier players may have taken some since they were collected
        orbs.retain(|orb| !orb.get_entity().is_removed());
        if orbs.is_empty() {
            return;
        }
        let orb = &orbs[rand::rng().random_range(0..orbs.len())];
        orb.on_player_collision(self.player);
    }
}

/// Vanilla `Entity.playerTouch`, run for every player whose pickup area overlaps.
/// Experience orbs are only collected here and touched by `touch_random_orb`.
pub(super) fn touch_players(
    entity: &Arc<dyn EntityBase>,
    entity_chunk: Vector2<i32>,
    players: &[PlayerTouch<'_>],
) {
    if players.is_empty() || !entity.receives_player_touch() {
        return;
    }
    let entity_inner = entity.get_entity();
    let entity_bb = entity_inner.bounding_box.load();
    let is_orb = entity_inner.entity_type == &EntityType::EXPERIENCE_ORB;

    for touch in players {
        // An earlier player may have picked it up
        if entity_inner.is_removed() {
            return;
        }
        if !touch.touches(&entity_bb, entity_chunk) {
            continue;
        }
        if is_orb {
            touch
                .orbs
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(entity.clone());
        } else {
            entity.on_player_collision(touch.player);
        }
    }
}
