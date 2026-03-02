use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;

use crate::block::{BlockBehaviour, BlockFuture, OnEntityCollisionArgs, OnLandedUponArgs};
use crate::entity::EntityBase;

#[pumpkin_block("minecraft:powder_snow")]
pub struct PowderSnowBlock;

const FALLING_COLLISION_SHAPE: BoundingBox =
    BoundingBox::new_array([0.0, 0.0, 0.0], [1.0, 0.9, 1.0]);
const WALK_ON_EPSILON: f64 = 1.0e-7;

pub(crate) async fn can_entity_walk_on_powder_snow(entity: &dyn EntityBase) -> bool {
    let base = entity.get_entity();
    if base
        .entity_type
        .has_tag(&tag::EntityType::MINECRAFT_POWDER_SNOW_WALKABLE_MOBS)
    {
        return true;
    }

    let Some(living) = entity.get_living_entity() else {
        return false;
    };

    let equipment = living.entity_equipment.lock().await;
    let boots = equipment.get(&EquipmentSlot::FEET);
    let boots = boots.lock().await;
    boots.item == &Item::LEATHER_BOOTS
}

fn is_entity_above_block(entity: &crate::entity::Entity, position: &BlockPos) -> bool {
    let bb = entity.bounding_box.load();
    let block_top = f64::from(position.0.y) + 1.0;
    bb.min.y >= block_top - WALK_ON_EPSILON
}

fn is_entity_descending(entity: &crate::entity::Entity) -> bool {
    entity.velocity.load().y < 0.0
}

pub(crate) async fn collision_shape_for_entity(
    entity: &dyn EntityBase,
    position: &BlockPos,
) -> Option<BoundingBox> {
    let fall_distance = entity
        .get_living_entity()
        .map(|living| living.fall_distance.load())
        .unwrap_or(0.0);

    if fall_distance > 2.5f32 {
        return Some(FALLING_COLLISION_SHAPE);
    }

    let base = entity.get_entity();
    if base.entity_type == &EntityType::FALLING_BLOCK {
        return Some(BoundingBox::full_block());
    }

    if can_entity_walk_on_powder_snow(entity).await
        && is_entity_above_block(base, position)
        && !is_entity_descending(base)
    {
        return Some(BoundingBox::full_block());
    }

    None
}

pub(crate) async fn inside_collision_shape_for_entity(
    entity: &dyn EntityBase,
    position: &BlockPos,
) -> BoundingBox {
    collision_shape_for_entity(entity, position)
        .await
        .unwrap_or_else(BoundingBox::full_block)
}

impl BlockBehaviour for PowderSnowBlock {
    fn on_landed_upon<'a>(&'a self, args: OnLandedUponArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            if let Some(living) = args.entity.get_living_entity()
                && (args.fall_distance.partial_cmp(&4.0) != Some(std::cmp::Ordering::Less))
            {
                let sound = if args.fall_distance < 7.0 {
                    Sound::EntityGenericSmallFall
                } else {
                    Sound::EntityGenericBigFall
                };

                living.entity.play_sound(sound).await;
            }
        })
    }

    fn on_entity_collision<'a>(&'a self, args: OnEntityCollisionArgs<'a>) -> BlockFuture<'a, ()> {
        Box::pin(async move {
            let entity = args.entity.get_entity();
            entity
                .slow_movement(args.state, Vector3::new(0.9, 1.5, 0.9))
                .await;

            if entity.fire_ticks.load(std::sync::atomic::Ordering::Relaxed) > 0 {
                let can_destroy = args.entity.get_player().is_some()
                    || args.world.level_info.load().game_rules.mob_griefing;
                if can_destroy {
                    let _ = args
                        .world
                        .break_block(
                            args.position,
                            None,
                            BlockFlags::NOTIFY_ALL | BlockFlags::SKIP_DROPS,
                        )
                        .await;
                }
            }

            entity.extinguish();
        })
    }
}
