use pumpkin_data::entity::EntityType;
use rand::RngExt;

use super::super::memory::{MemoryStatus, types};
use super::{OneShot, utils};

#[must_use]
pub fn create(
    entity_type: &'static EntityType,
    max_distance: f32,
    min_interval: i32,
    max_interval: i32,
) -> OneShot {
    let max_distance_squared = f64::from(max_distance * max_distance);
    let mut ticker = Ticker {
        min_interval,
        max_interval,
        ticks_until_next_start: 0,
    };
    OneShot::new(
        "SetEntityLookTargetSometimes",
        vec![
            (types::LOOK_TARGET.id(), MemoryStatus::ValueAbsent),
            (
                types::NEAREST_VISIBLE_LIVING_ENTITIES.id(),
                MemoryStatus::ValuePresent,
            ),
        ],
        move |tick| {
            let position = tick.mob.get_entity().pos.load();
            let target = tick
                .brain
                .get(types::NEAREST_VISIBLE_LIVING_ENTITIES)
                .and_then(|nearby| {
                    nearby.find_closest(&tick.visibility(), |other| {
                        other.get_entity().entity_type == entity_type
                            && other
                                .get_entity()
                                .pos
                                .load()
                                .squared_distance_to_vec(&position)
                                <= max_distance_squared
                    })
                });
            let Some(target) = target else {
                return false;
            };
            if !ticker.tick_down_and_check(&mut tick.mob.get_random()) {
                return false;
            }
            utils::look_at_entity(tick.brain, target);
            true
        },
    )
}

struct Ticker {
    min_interval: i32,
    max_interval: i32,
    ticks_until_next_start: i32,
}

impl Ticker {
    fn tick_down_and_check(&mut self, random: &mut impl RngExt) -> bool {
        if self.ticks_until_next_start == 0 {
            self.ticks_until_next_start =
                random.random_range(self.min_interval..=self.max_interval) - 1;
            false
        } else {
            self.ticks_until_next_start -= 1;
            self.ticks_until_next_start == 0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Ticker;

    #[test]
    fn sampling_tick_counts_toward_the_look_interval() {
        let mut ticker = Ticker {
            min_interval: 3,
            max_interval: 3,
            ticks_until_next_start: 0,
        };
        let mut random = rand::rng();
        let starts: Vec<bool> = (0..6)
            .map(|_| ticker.tick_down_and_check(&mut random))
            .collect();

        assert_eq!(starts, [false, false, true, false, false, true]);
    }
}
