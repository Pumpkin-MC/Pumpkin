use super::{Controls, Goal, wander_around::WanderAroundGoal};
use crate::entity::ai::util::{air_and_water_random_pos, hover_random_pos};
use crate::entity::mob::Mob;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

pub struct WaterAvoidingRandomFlyingGoal {
    wander: WanderAroundGoal,
}

impl WaterAvoidingRandomFlyingGoal {
    #[must_use]
    pub const fn new(speed: f64) -> Self {
        Self {
            wander: WanderAroundGoal::new(speed).with_position(Self::get_position, 0.0),
        }
    }

    #[must_use]
    pub const fn parrot(speed: f64) -> Self {
        Self {
            wander: WanderAroundGoal::new(speed).with_position(Self::parrot_position, 0.0),
        }
    }

    fn parrot_position(mob: &dyn Mob) -> Option<Vector3<f64>> {
        use pumpkin_data::tag::{self, Taggable};
        use pumpkin_util::math::position::BlockPos;
        let entity = mob.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        let mut target = if entity.is_in_water() {
            crate::entity::ai::util::land_random_pos::get_pos(mob, 15, 15)
        } else {
            None
        };
        if mob.get_random().random::<f32>() >= super::wander_around::PROBABILITY {
            target = None;
            // BlockPos.betweenClosed iterates x fastest, then y, then z.
            'search: for z in (pos.z - 3.0).floor() as i32..=(pos.z + 3.0).floor() as i32 {
                for y in (pos.y - 6.0).floor() as i32..=(pos.y + 6.0).floor() as i32 {
                    for x in (pos.x - 3.0).floor() as i32..=(pos.x + 3.0).floor() as i32 {
                        let at = BlockPos::new(x, y, z);
                        if at == entity.block_pos.load() {
                            continue;
                        }
                        let below = world.get_block(&at.down());
                        if (below.has_tag(&tag::Block::MINECRAFT_LEAVES)
                            || below.has_tag(&tag::Block::MINECRAFT_LOGS))
                            && world.get_block_state(&at).is_air()
                            && world.get_block_state(&at.up()).is_air()
                        {
                            target = Some(Vector3::new(
                                f64::from(x) + 0.5,
                                f64::from(y),
                                f64::from(z) + 0.5,
                            ));
                            break 'search;
                        }
                    }
                }
            }
        }
        target.or_else(|| Self::get_position(mob))
    }

    /// Prefers a spot to hover over the ground, and falls back to open air or water.
    fn get_position(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let wander_direction = mob.get_looking_vector();

        hover_random_pos::get_pos(
            mob,
            8,
            7,
            wander_direction.x,
            wander_direction.z,
            std::f64::consts::FRAC_PI_2,
            3,
            1,
        )
        .or_else(|| {
            air_and_water_random_pos::get_pos(
                mob,
                8,
                4,
                -2,
                wander_direction.x,
                wander_direction.z,
                std::f64::consts::FRAC_PI_2,
            )
        })
    }
}

impl Goal for WaterAvoidingRandomFlyingGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.wander.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.wander.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.wander.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.wander.stop(mob);
    }

    fn controls(&self) -> Controls {
        self.wander.controls()
    }
}
