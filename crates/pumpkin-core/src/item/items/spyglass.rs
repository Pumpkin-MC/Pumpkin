use std::any::Any;

use crate::entity::EntityBase;
use crate::entity::player::Player;
use crate::item::{ItemBehaviour, ItemMetadata};
use pumpkin_data::item::Item;
use pumpkin_data::sound::{Sound, SoundCategory};

pub struct SpyglassItem;

impl ItemMetadata for SpyglassItem {
    fn ids() -> Box<[u16]> {
        Box::new([Item::SPYGLASS.id])
    }
}

impl ItemBehaviour for SpyglassItem {
    fn normal_use(&self, _item: &Item, player: &Player) {
        player.world().play_sound(
            Sound::ItemSpyglassUse,
            SoundCategory::Players,
            &player.position(),
        );
        let stack = player.inventory().held_item();
        player
            .living_entity
            .set_active_hand(pumpkin_util::Hand::Right, stack, Self::USE_DURATION);
    }

    fn on_stopped_using(&self, _stack: &pumpkin_data::item_stack::ItemStack, player: &Player) {
        player.world().play_sound(
            Sound::ItemSpyglassStopUsing,
            SoundCategory::Players,
            &player.position(),
        );
        player.increment_stat(
            pumpkin_data::statistic::StatisticCategory::Used,
            Item::SPYGLASS.id as i32,
            1,
        );
        Self::check_spyglass_target(player);
    }

    fn get_use_duration(&self) -> i32 {
        Self::USE_DURATION
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn ray_intersects_aabb(
    origin: pumpkin_util::math::vector3::Vector3<f64>,
    dir: pumpkin_util::math::vector3::Vector3<f64>,
    min: pumpkin_util::math::vector3::Vector3<f64>,
    max: pumpkin_util::math::vector3::Vector3<f64>,
    max_dist: f64,
) -> bool {
    let mut tmin = 0.0f64;
    let mut tmax = max_dist;

    for axis in 0..3 {
        let (p, d, bmin, bmax) = match axis {
            0 => (origin.x, dir.x, min.x, max.x),
            1 => (origin.y, dir.y, min.y, max.y),
            _ => (origin.z, dir.z, min.z, max.z),
        };
        if d.abs() < 1e-9 {
            if p < bmin || p > bmax {
                return false;
            }
        } else {
            let inv_d = 1.0 / d;
            let mut t1 = (bmin - p) * inv_d;
            let mut t2 = (bmax - p) * inv_d;
            if t1 > t2 {
                std::mem::swap(&mut t1, &mut t2);
            }
            tmin = tmin.max(t1);
            tmax = tmax.min(t2);
            if tmin > tmax {
                return false;
            }
        }
    }
    tmax >= 0.0
}

impl SpyglassItem {
    pub const USE_DURATION: i32 = 1200;

    pub fn check_spyglass_target(player: &Player) {
        let eye_pos = player
            .position()
            .add_raw(0.0, f64::from(player.get_entity().entity_type.eye_height), 0.0);
        let yaw = player.get_entity().yaw.load();
        let pitch = player.get_entity().pitch.load();
        let yaw_rad = f64::from(yaw + 90.0).to_radians();
        let pitch_rad = f64::from(-pitch).to_radians();
        let dir = pumpkin_util::math::vector3::Vector3::new(
            pitch_rad.cos() * yaw_rad.cos(),
            pitch_rad.sin(),
            pitch_rad.cos() * yaw_rad.sin(),
        );

        let world = player.world();
        let nearby_entities = world.get_nearby_entities(eye_pos, 128.0);
        for entity in nearby_entities.values() {
            if entity.get_entity().entity_id == player.entity_id() {
                continue;
            }
            let ent_type = entity.get_entity().entity_type;
            let target_name = if ent_type == &pumpkin_data::entity::EntityType::PARROT {
                "minecraft:parrot"
            } else if ent_type == &pumpkin_data::entity::EntityType::GHAST {
                "minecraft:ghast"
            } else if ent_type == &pumpkin_data::entity::EntityType::ENDER_DRAGON {
                "minecraft:ender_dragon"
            } else {
                continue;
            };

            let bb = entity.get_entity().bounding_box.load();
            let min = bb.min - pumpkin_util::math::vector3::Vector3::new(0.5, 0.5, 0.5);
            let max = bb.max + pumpkin_util::math::vector3::Vector3::new(0.5, 0.5, 0.5);
            if ray_intersects_aabb(eye_pos, dir, min, max, 128.0) {
                player.trigger_advancement(
                    crate::entity::player::advancement::trigger::AdvancementTrigger::UsingItem {
                        item_id: "minecraft:spyglass".to_string(),
                        target_entity: Some(target_name.to_string()),
                    },
                );
                break;
            }
        }
    }
}
