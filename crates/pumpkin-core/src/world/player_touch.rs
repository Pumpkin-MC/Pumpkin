use std::sync::Arc;

use pumpkin_util::math::{boundingbox::BoundingBox, get_section_cord, vector2::Vector2};

use crate::entity::{EntityBase, player::Player};

/// One player's touch state, taken once per tick after players move.
pub(super) struct PlayerTouch<'a> {
    player: &'a Arc<Player>,
    area: BoundingBox,
    chunk: Vector2<i32>,
}

impl<'a> PlayerTouch<'a> {
    /// `None` for dead or spectating players
    pub(super) fn new(player: &'a Arc<Player>) -> Option<Self> {
        if player.living_entity.health.load() <= 0.0 || player.is_spectator() {
            return None;
        }
        let entity = player.get_entity();
        let pos = entity.pos.load();
        let hitbox = entity.bounding_box.load();
        // Vanilla `Player.aiStep` pickup volume. Push uses stored hitboxes elsewhere.
        let area = match entity.get_vehicle() {
            Some(vehicle) if !vehicle.get_entity().is_removed() => hitbox
                .minmax(&vehicle.get_entity().bounding_box.load())
                .expand(1.0, 0.0, 1.0),
            _ => hitbox.expand(1.0, 0.5, 1.0),
        };
        let chunk = Vector2::new(
            get_section_cord(pos.x.floor() as i32),
            get_section_cord(pos.z.floor() as i32),
        );
        Some(Self {
            player,
            area,
            chunk,
        })
    }

    fn touches(&self, entity_bb: &BoundingBox, chunk: Vector2<i32>) -> bool {
        (self.chunk.x - chunk.x).abs() <= 1
            && (self.chunk.y - chunk.y).abs() <= 1
            && self.area.intersects(entity_bb)
    }
}

/// Vanilla `Entity.playerTouch`. First overlapping player in list order wins.
pub(super) fn touch_players(
    entity: &Arc<dyn EntityBase>,
    entity_chunk: Vector2<i32>,
    players: &[PlayerTouch<'_>],
) {
    if players.is_empty() || !entity.receives_player_touch() {
        return;
    }
    let entity_inner = entity.get_entity();
    if entity_inner.is_removed() {
        return;
    }
    let entity_bb = entity_inner.bounding_box.load();

    if let Some(touch) = players
        .iter()
        .find(|touch| touch.touches(&entity_bb, entity_chunk))
    {
        entity.on_player_collision(touch.player);
    }
}
