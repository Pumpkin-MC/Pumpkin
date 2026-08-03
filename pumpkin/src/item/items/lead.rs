use std::pin::Pin;
use std::sync::Arc;

use crate::entity::EntityBase;
use crate::entity::decoration::leash_knot::LeashKnotEntity;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use crate::server::Server;
use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

pub struct LeadItem;

impl ItemMetadata for LeadItem {
    fn ids() -> Box<[u16]> {
        [Item::LEAD.id].into()
    }
}

impl ItemBehaviour for LeadItem {
    fn use_on_block<'a>(
        &'a self,
        _item: &'a mut ItemStack,
        player: &'a Player,
        location: BlockPos,
        _face: BlockDirection,
        _cursor_pos: Vector3<f32>,
        block: &'a Block,
        _server: &'a Server,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            if !block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_FENCES) {
                return;
            }

            let world = player.world();
            let center = Vector3::new(
                f64::from(location.0.x) + 0.5,
                f64::from(location.0.y) + 0.5,
                f64::from(location.0.z) + 0.5,
            );

            let search_dim = EntityDimensions {
                width: 32.0,
                height: 32.0,
                eye_height: 16.0,
            };

            let search_box = BoundingBox::new_from_pos(center.x, center.y, center.z, &search_dim);

            let player_id = player.entity_id();
            let entities = world.get_entities_at_box(&search_box);

            let mut any_leashed = false;
            let mut knot: Option<Arc<LeashKnotEntity>> = None;

            for entity_base in entities {
                let ent = entity_base.get_entity();
                let is_leashed_to_player = ent
                    .leashed_to
                    .try_lock()
                    .ok()
                    .and_then(|guard| {
                        guard
                            .as_ref()
                            .map(|holder| holder.get_entity().entity_id == player_id)
                    })
                    .unwrap_or(false);

                if is_leashed_to_player {
                    if knot.is_none() {
                        knot = Some(LeashKnotEntity::get_or_create(&world, location).await);
                    }
                    if let Some(k) = &knot {
                        ent.leash_to(k.clone() as Arc<dyn EntityBase>).await;
                        any_leashed = true;
                    }
                }
            }

            if any_leashed {
                world.play_sound(Sound::ItemLeadTied, SoundCategory::Neutral, &center);
                if let Some(player_arc) = world.get_player_by_id(player.get_entity().entity_id) {
                    crate::world::game_event::emit_game_event(
                        &world,
                        pumpkin_data::game_event::GameEvent::BlockAttach,
                        center,
                        crate::world::game_event::GameEventContext::of_entity(player_arc),
                    )
                    .await;
                }
            }
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
