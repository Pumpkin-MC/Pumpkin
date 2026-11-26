use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, can_have_a_leash_attached_to, living::LivingEntity};
use crate::world::World;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::Taggable;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};

const CHECK_INTERVAL: i32 = 100;

pub struct LeashKnotEntity {
    entity: Entity,
    ticks_since_last_check: AtomicI32,
}

impl LeashKnotEntity {
    pub const OFFSET_Y: f64 = 0.375;

    pub const fn new(entity: Entity) -> Self {
        Self {
            entity,
            ticks_since_last_check: AtomicI32::new(0),
        }
    }

    pub fn get_or_create(world: &Arc<World>, pos: BlockPos) -> Arc<dyn EntityBase> {
        Self::get_knot(world, pos).unwrap_or_else(|| Self::create_knot(world, pos))
    }

    pub fn get_knot(world: &Arc<World>, pos: BlockPos) -> Option<Arc<dyn EntityBase>> {
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
        world
            .get_entities_at_box(&search_box)
            .into_iter()
            .find(|entity| {
                let entity = entity.get_entity();
                entity.entity_type == &EntityType::LEASH_KNOT && entity.block_pos.load() == pos
            })
    }

    pub fn create_knot(world: &Arc<World>, pos: BlockPos) -> Arc<dyn EntityBase> {
        let raw_pos = Vector3::new(
            f64::from(pos.0.x) + 0.5,
            f64::from(pos.0.y) + Self::OFFSET_Y,
            f64::from(pos.0.z) + 0.5,
        );

        let entity = Entity::new(world.clone(), raw_pos, &EntityType::LEASH_KNOT);
        let knot: Arc<dyn EntityBase> = Arc::new(Self::new(entity));
        world.spawn_entity(knot.clone());
        knot
    }

    pub fn play_placement_sound(entity: &Entity) {
        entity.world.load().play_sound(
            Sound::ItemLeadTied,
            SoundCategory::Neutral,
            &entity.pos.load(),
        );
    }

    fn drop_item(&self) {
        self.entity.world.load().play_sound(
            Sound::ItemLeadUntied,
            SoundCategory::Neutral,
            &self.entity.pos.load(),
        );
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

    fn notify_leashee_removed(&self) {
        if self.entity.leashable_leashed_to().is_empty() {
            self.entity.remove();
        }
    }

    fn tick(&self, _caller: &dyn EntityBase, _server: &Server) {
        if self.ticks_since_last_check.fetch_add(1, Ordering::Relaxed) < CHECK_INTERVAL {
            return;
        }
        self.ticks_since_last_check.store(0, Ordering::Relaxed);
        let world = self.entity.world.load();
        let block = world.get_block(&self.entity.block_pos.load());
        if self.entity.is_removed() || block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_FENCES) {
            return;
        }
        self.entity.remove();
        self.drop_item();
    }

    fn interact(&self, player: &Arc<Player>, _item_stack: &mut ItemStack) -> bool {
        let world = player.world();

        let Some(knot) = Self::get_knot(&world, self.entity.block_pos.load()) else {
            return false;
        };

        let player_entity = player.get_entity();
        let mut attached_mob = false;
        for leashable in player_entity.leashable_leashed_to() {
            if can_have_a_leash_attached_to(leashable.as_ref(), knot.as_ref())
                && leashable
                    .get_entity()
                    .leash_to_if_held_by(player_entity.entity_id, &knot)
            {
                attached_mob = true;
            }
        }

        let mut any_dropped = false;
        if !attached_mob && !player_entity.is_sneaking() {
            let player_holder = player.clone() as Arc<dyn EntityBase>;
            for mob in self.entity.leashable_leashed_to() {
                if can_have_a_leash_attached_to(mob.as_ref(), player.as_ref())
                    && mob
                        .get_entity()
                        .leash_to_if_held_by(self.entity.entity_id, &player_holder)
                {
                    any_dropped = true;
                }
            }
        }

        if !attached_mob && !any_dropped {
            return false;
        }

        world.emit_game_event(GameEvent::BlockAttach.name(), self.entity.pos.load());
        Self::play_placement_sound(&self.entity);
        true
    }

    fn damage_with_context(
        &self,
        _caller: &dyn EntityBase,
        _amount: f32,
        damage_type: DamageType,
        _position: Option<Vector3<f64>>,
        _source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        if self.entity.is_invulnerable_to(&damage_type, cause) {
            return false;
        }
        let world = self.entity.world.load();
        if !world.level_info.load().game_rules.mob_griefing
            && cause.is_some_and(|cause| cause.get_mob().is_some())
        {
            return false;
        }
        if !self.entity.is_removed() {
            self.entity.remove();
            world.emit_game_event(GameEvent::EntityDie.name(), self.entity.pos.load());
            self.drop_item();
        }
        true
    }

    fn can_hit(&self) -> bool {
        self.entity.is_alive()
    }

    fn cast_any(&self) -> &dyn std::any::Any {
        self
    }
}
