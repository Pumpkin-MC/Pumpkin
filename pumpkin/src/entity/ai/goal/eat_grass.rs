use super::{Controls, Goal, GoalFuture};
use crate::entity::mob::Mob;
use crate::world::game_event::{GameEventContext, emit_game_event};
use pumpkin_data::Block;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

const MAX_TIMER: i32 = 40;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Destroy {
    /// The block the mob stands in (`minecraft:edible_for_sheep`) is removed.
    EdibleBlock,
    /// The grass block underneath the mob is turned to dirt.
    GrassBlockBelow,
}

#[derive(Debug, PartialEq, Eq)]
struct EatOutcome {
    destroy: Option<Destroy>,
    ate: bool,
}

/// Vanilla `EatBlockGoal.tick` (1.21.4, `EatBlockGoal.java:59-81`): both branches perform the
/// world edit only when the `mobGriefing` game rule is on, but call `this.mob.ate()`
/// unconditionally once the corresponding block was found. So with `mobGriefing false` a sheep
/// still regrows its wool and a lamb still ages up, but the grass survives.
const fn eat_outcome(
    mob_griefing: bool,
    standing_in_edible: bool,
    grass_below: bool,
) -> EatOutcome {
    if standing_in_edible {
        EatOutcome {
            destroy: if mob_griefing {
                Some(Destroy::EdibleBlock)
            } else {
                None
            },
            ate: true,
        }
    } else if grass_below {
        EatOutcome {
            destroy: if mob_griefing {
                Some(Destroy::GrassBlockBelow)
            } else {
                None
            },
            ate: true,
        }
    } else {
        EatOutcome {
            destroy: None,
            ate: false,
        }
    }
}

pub struct EatGrassGoal {
    goal_control: Controls,
    timer: i32,
}

impl Default for EatGrassGoal {
    fn default() -> Self {
        Self {
            goal_control: Controls::MOVE | Controls::LOOK | Controls::JUMP,
            timer: 0,
        }
    }
}

impl EatGrassGoal {
    #[must_use]
    pub const fn get_timer(&self) -> i32 {
        self.timer
    }
}

impl Goal for EatGrassGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = &mob.get_mob_entity().living_entity.entity;
            let bound = if entity.age.load(std::sync::atomic::Ordering::Relaxed) < 0 {
                50
            } else {
                1000
            };
            if mob.get_random().random_range(0..bound) != 0 {
                return false;
            }

            let block_pos = entity.block_pos.load();
            let world = entity.world.load();

            let block_at_pos = world.get_block(&block_pos);
            if block_at_pos.has_tag(&tag::Block::MINECRAFT_EDIBLE_FOR_SHEEP) {
                return true;
            }

            let block_below = world.get_block(&block_pos.down());
            block_below.id == Block::GRASS_BLOCK.id
        })
    }

    fn should_continue<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move { self.timer > 0 })
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.timer = MAX_TIMER;
            // Vanilla EatBlockGoal.start(): broadcasts entity-event byte 10, which drives the
            // client-side head-eating animation (Sheep.handleEntityEvent / getHeadEatAngleScale).
            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load();
            world.send_entity_status(entity, pumpkin_data::entity::EntityStatus::EatGrass);
            let mut navigator = mob.get_mob_entity().navigator.lock().unwrap();
            navigator.stop();
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.timer -= 1;

            if self.timer == 4 {
                let entity = &mob.get_mob_entity().living_entity.entity;
                let block_pos = entity.block_pos.load();
                let world = entity.world.load_full();

                let below_pos = block_pos.down();
                let outcome = eat_outcome(
                    world.level_info.load().game_rules.mob_griefing,
                    world
                        .get_block(&block_pos)
                        .has_tag(&tag::Block::MINECRAFT_EDIBLE_FOR_SHEEP),
                    world.get_block(&below_pos).id == Block::GRASS_BLOCK.id,
                );

                match outcome.destroy {
                    Some(Destroy::EdibleBlock) => {
                        world
                            .set_block_state(
                                &block_pos,
                                Block::AIR.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                    }
                    Some(Destroy::GrassBlockBelow) => {
                        world
                            .set_block_state(
                                &below_pos,
                                Block::DIRT.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await;
                    }
                    None => {}
                }

                if outcome.ate {
                    mob.on_eating_grass().await;
                    emit_eat_game_event(&world, &block_pos).await;
                }
            }
        })
    }

    fn stop<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.timer = 0;
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

// Mob.ate() -> gameEvent(GameEvent.EAT); no Arc<dyn EntityBase> available here, so none().
async fn emit_eat_game_event(
    world: &std::sync::Arc<crate::world::World>,
    block_pos: &pumpkin_util::math::position::BlockPos,
) {
    emit_game_event(
        world,
        GameEvent::Eat,
        Vector3::new(
            f64::from(block_pos.0.x) + 0.5,
            f64::from(block_pos.0.y) + 0.5,
            f64::from(block_pos.0.z) + 0.5,
        ),
        GameEventContext::none(),
    )
    .await;
}

#[cfg(test)]
mod eat_outcome_tests {
    use super::{Destroy, eat_outcome};

    #[test]
    fn edible_block_is_removed_only_with_mob_griefing() {
        let on = eat_outcome(true, true, false);
        assert_eq!(on.destroy, Some(Destroy::EdibleBlock));
        assert!(on.ate);

        let off = eat_outcome(false, true, false);
        assert_eq!(off.destroy, None);
        assert!(
            off.ate,
            "vanilla calls mob.ate() outside the mobGriefing check"
        );
    }

    #[test]
    fn grass_below_becomes_dirt_only_with_mob_griefing() {
        let on = eat_outcome(true, false, true);
        assert_eq!(on.destroy, Some(Destroy::GrassBlockBelow));
        assert!(on.ate);

        let off = eat_outcome(false, false, true);
        assert_eq!(off.destroy, None);
        assert!(off.ate);
    }

    #[test]
    fn standing_in_edible_takes_precedence_over_grass_below() {
        assert_eq!(
            eat_outcome(true, true, true).destroy,
            Some(Destroy::EdibleBlock)
        );
    }

    #[test]
    fn nothing_edible_means_no_edit_and_no_ate() {
        let none = eat_outcome(true, false, false);
        assert_eq!(none.destroy, None);
        assert!(!none.ate);
    }
}
