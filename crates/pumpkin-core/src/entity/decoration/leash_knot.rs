use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, living::LivingEntity};
use crate::world::World;
use pumpkin_data::damage::DamageType;
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
}

impl LeashKnotEntity {
    pub const OFFSET_Y: f64 = 0.375;

    pub const fn new(entity: Entity) -> Self {
        Self { entity }
    }

    /// The fence block this knot is attached to. Derived from the entity position so a knot loaded
    /// from disk (whose position is only known after the entity NBT is read) reports the right block.
    pub fn block_pos(&self) -> BlockPos {
        self.entity.block_pos.load()
    }

    pub fn get_or_create(world: &Arc<World>, pos: BlockPos) -> Arc<dyn EntityBase> {
        Self::find_knot(world, pos)
            .unwrap_or_else(|| Self::create_knot(world, pos) as Arc<dyn EntityBase>)
    }

    /// Finds a knot placed at `pos`, returned as a trait object so it can be used directly as a
    /// leash holder.
    pub fn find_knot(world: &Arc<World>, pos: BlockPos) -> Option<Arc<dyn EntityBase>> {
        for entity_base in world.get_entities_at_box(&Self::search_box(pos)) {
            if entity_base.get_entity().entity_type == &EntityType::LEASH_KNOT
                && let Some(knot) = entity_base.cast_any().downcast_ref::<Self>()
                && knot.block_pos() == pos
            {
                return Some(entity_base);
            }
        }
        None
    }

    fn search_box(pos: BlockPos) -> BoundingBox {
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

        BoundingBox::new_from_pos(center.x, center.y, center.z, &search_dim)
    }

    pub fn create_knot(world: &Arc<World>, pos: BlockPos) -> Arc<Self> {
        let raw_pos = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + Self::OFFSET_Y,
            f64::from(pos.0.z) + 0.5,
        );

        let entity = Entity::new(world.clone(), raw_pos, &EntityType::LEASH_KNOT);
        let knot = Arc::new(Self::new(entity));
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

    fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {
        let world = self.entity.world.load();
        let knot_pos = self.block_pos();
        let block = world.get_block(&knot_pos);
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
                    ent.drop_leash_with_item();
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

        if let Some(self_knot) = Self::find_knot(&world, self.block_pos()) {
            for mob in player_leashed_mobs {
                mob.leash_to(self_knot.clone());
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

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        // Vanilla `BlockAttachedEntity.hurtServer` checks `isInvulnerableToBase` first, and a
        // creative player bypasses invulnerability there.
        let creative_player = source.is_some_and(|source| {
            source
                .cast_any()
                .downcast_ref::<Player>()
                .is_some_and(Player::is_creative)
        });
        if self.entity.is_removed()
            || (!creative_player && self.entity.is_invulnerable_to(&damage_type))
        {
            return false;
        }
        // `BlockAttachedEntity.hurtServer`: no damage from a mob while `mob_griefing` is off.
        let world = self.entity.world.load();
        if !world.level_info.load().game_rules.mob_griefing
            && cause.or(source).is_some_and(|atk| atk.get_mob().is_some())
        {
            return false;
        }
        // Hitting the knot destroys it. The mobs leashed to it drop their lead on their next tick,
        // once they see the holder is gone.
        world.play_sound(
            Sound::ItemLeadUntied,
            SoundCategory::Neutral,
            &self.entity.pos.load(),
        );
        self.entity.remove();
        true
    }
}
