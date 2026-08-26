use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, living::LivingEntity};
use crate::world::World;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

pub struct LeashKnotEntity {
    entity: Entity,
    pos: BlockPos,
}

impl LeashKnotEntity {
    pub const OFFSET_Y: f64 = 0.375;

    pub const fn new(entity: Entity, pos: BlockPos) -> Self {
        Self { entity, pos }
    }

    pub const fn block_pos(&self) -> BlockPos {
        self.pos
    }

    pub fn get_or_create(world: &Arc<World>, pos: BlockPos) -> Arc<Self> {
        if let Some(existing) = Self::get_knot(world, pos) {
            return existing;
        }
        Self::create_knot(world, pos)
    }

    pub fn get_knot(world: &Arc<World>, pos: BlockPos) -> Option<Arc<Self>> {
        let center = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + Self::OFFSET_Y,
            f64::from(pos.0.z) + 0.5,
        );

        let search_dim = EntityDimensions {
            width: 2.0,
            height: 2.0,
            eye_height: 1.0,
        };

        let search_box = BoundingBox::new_from_pos(center.x, center.y, center.z, &search_dim);
        let entities = world.get_entities_at_box(&search_box);

        for entity_base in entities {
            if entity_base.get_entity().entity_type == &EntityType::LEASH_KNOT
                && let Some(knot) = entity_base.cast_any().downcast_ref::<Arc<Self>>()
                && knot.pos == pos
            {
                return Some(knot.clone());
            }
        }
        None
    }

    pub fn create_knot(world: &Arc<World>, pos: BlockPos) -> Arc<Self> {
        let raw_pos = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + Self::OFFSET_Y,
            f64::from(pos.0.z) + 0.5,
        );

        let entity = Entity::new(world.clone(), raw_pos, &EntityType::LEASH_KNOT);
        let knot = Arc::new(Self::new(entity, pos));
        world.spawn_entity(knot.clone() as Arc<dyn EntityBase>);

        world.play_sound(Sound::ItemLeadTied, SoundCategory::Neutral, &raw_pos);

        knot
    }

    pub fn play_placement_sound(&self, world: &World) {
        let pos = self.entity.pos.load();
        world.play_sound(Sound::ItemLeadTied, SoundCategory::Neutral, &pos);
    }
}

use crate::server::Server;

impl EntityBase for LeashKnotEntity {
    fn get_entity(&self) -> &Entity {
        &self.entity
    }

    fn get_living_entity(&self) -> Option<&LivingEntity> {
        None
    }

    fn tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>, _server: &'a Server) {
        let world = self.entity.world.load();
        let block = world.get_block(&self.pos);
        if !block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_FENCES) {
            let knot_id = self.entity.entity_id;
            let search_dim = EntityDimensions {
                width: 32.0,
                height: 32.0,
                eye_height: 16.0,
            };
            let pos = self.entity.pos.load();
            let search_box = BoundingBox::new_from_pos(pos.x, pos.y, pos.z, &search_dim);
            let entities = world.get_entities_at_box(&search_box);

            for entity_base in entities {
                let ent = entity_base.get_entity();
                let is_attached_to_knot = ent
                    .leashed_to
                    .try_lock()
                    .ok()
                    .and_then(|guard| {
                        guard
                            .as_ref()
                            .map(|holder| holder.get_entity().entity_id == knot_id)
                    })
                    .unwrap_or(false);

                if is_attached_to_knot {
                    ent.unleash();
                    let lead_item = pumpkin_data::item_stack::ItemStack::new(
                        1,
                        &pumpkin_data::item::Item::LEAD,
                    );
                    world.drop_stack(&ent.block_pos.load(), lead_item);
                }
            }

            world.play_sound(Sound::ItemLeadUntied, SoundCategory::Neutral, &pos);
            self.entity.remove();
        }
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        let world = player.world();
        let knot_id = self.entity.entity_id;
        let player_id = player.entity_id();

        let search_dim = EntityDimensions {
            width: 32.0,
            height: 32.0,
            eye_height: 16.0,
        };
        let pos = self.entity.pos.load();
        let search_box = BoundingBox::new_from_pos(pos.x, pos.y, pos.z, &search_dim);
        let entities = world.get_entities_at_box(&search_box);

        let mut attached_mob = false;
        let mut player_leashed_mobs = Vec::new();

        for entity_base in &entities {
            let ent = entity_base.get_entity();
            if let Ok(guard) = ent.leashed_to.try_lock()
                && let Some(holder) = guard.as_ref()
                && holder.get_entity().entity_id == player_id
            {
                player_leashed_mobs.push(ent);
            }
        }

        if let Some(self_knot) = Self::get_knot(&world, self.pos) {
            for mob in player_leashed_mobs {
                mob.leash_to(self_knot.clone() as Arc<dyn EntityBase>);
                attached_mob = true;
            }
        }

        let mut any_dropped = false;
        if !attached_mob {
            for entity_base in &entities {
                let ent = entity_base.get_entity();
                if let Ok(guard) = ent.leashed_to.try_lock()
                    && let Some(holder) = guard.as_ref()
                    && holder.get_entity().entity_id == knot_id
                {
                    ent.leash_to(player.clone() as Arc<dyn EntityBase>);
                    any_dropped = true;
                }
            }
        }

        if attached_mob || any_dropped {
            self.play_placement_sound(&world);
            true
        } else {
            false
        }
    }
    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
