use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    BlockProperties, CopperGolemPose, CopperGolemStatueLikeProperties,
};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::block::entities::copper_golem_statue::CopperGolemStatueBlockEntity;
use crate::entity::player::Player;
use crate::entity::{
    Entity, EntityBase, NBTStorage, NbtFuture,
    ai::goal::{
        interact_with_door::InteractWithDoorGoal, look_at_entity::LookAtEntityGoal,
        transport_items::TransportItemsGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::world::World;

/// Copper oxidation stage. Mirrors `WeatheringCopper.WeatherState` (`UNAFFECTED`, `EXPOSED`,
/// `WEATHERED`, `OXIDIZED`) as consulted by `CopperGolem`/`CopperGolemOxidationLevels`.
///
/// This is intentionally a separate type from `copper_weathering.rs`'s block-oxidation
/// helpers: the block side rolls a per-random-tick neighbor-weighted chance
/// (`try_oxidize_copper`), while the mob advances on a fixed game-time schedule
/// (`CopperGolem.updateWeathering`) with no neighbor lookup at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopperWeatherState {
    Unaffected,
    Exposed,
    Weathered,
    Oxidized,
}

impl CopperWeatherState {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Unaffected => Self::Exposed,
            Self::Exposed => Self::Weathered,
            Self::Weathered | Self::Oxidized => Self::Oxidized,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Unaffected | Self::Exposed => Self::Unaffected,
            Self::Weathered => Self::Exposed,
            Self::Oxidized => Self::Weathered,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unaffected => "unaffected",
            Self::Exposed => "exposed",
            Self::Weathered => "weathered",
            Self::Oxidized => "oxidized",
        }
    }

    #[must_use]
    pub fn from_name(s: &str) -> Self {
        match s {
            "exposed" => Self::Exposed,
            "weathered" => Self::Weathered,
            "oxidized" => Self::Oxidized,
            _ => Self::Unaffected,
        }
    }
}

/// `CopperGolem.UNSET_WEATHERING_TICK`: no schedule set yet, roll one on the next tick.
const UNSET_WEATHERING_TICK: i64 = -1;
/// `CopperGolem.IGNORE_WEATHERING_TICK`: waxed, weathering is frozen entirely.
const IGNORE_WEATHERING_TICK: i64 = -2;
/// `CopperGolem.WEATHERING_TICK_FROM` / `_TO`: ticks between weathering stage advances.
const WEATHERING_TICK_FROM: i64 = 504_000;
const WEATHERING_TICK_TO: i64 = 552_000;
/// `CopperGolem.TURN_TO_STATUE_CHANCE`: rolled once per tick while fully oxidized.
const TURN_TO_STATUE_CHANCE: f32 = 0.0058;

pub struct CopperGolemEntity {
    pub mob_entity: MobEntity,
    weather_state: AtomicCell<CopperWeatherState>,
    next_weathering_tick: AtomicCell<i64>,
}

impl CopperGolemEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let golem = Self {
            mob_entity,
            weather_state: AtomicCell::new(CopperWeatherState::Unaffected),
            next_weathering_tick: AtomicCell::new(UNSET_WEATHERING_TICK),
        };
        let mob_arc = Arc::new(golem);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        // Cites `CopperGolemAi.initCoreActivity`/`initIdleActivity`. Deferred: `AnimalPanic`
        // (no per-mob danger-flee goal ported here), `SetEntityLookTargetSometimes`,
        // `CountDownCooldownTicks` bookkeeping -- none change observable behavior beyond
        // what `LookAtEntityGoal`/`WanderAroundGoal` already provide.
        #[expect(
            clippy::semicolon_outside_block,
            reason = "conflicts with semicolon_if_nothing_returned for a bare block statement"
        )]
        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            goal_selector.add_goal(1, Box::new(InteractWithDoorGoal::new(true)));
            goal_selector.add_goal(2, Box::new(TransportItemsGoal::new(1.0)));
            goal_selector.add_goal(
                3,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(1.0)));
        }

        mob_arc
    }

    #[must_use]
    pub fn weather_state(&self) -> CopperWeatherState {
        self.weather_state.load()
    }

    fn set_weather_state(&self, state: CopperWeatherState) {
        self.weather_state.store(state);
    }

    /// `CopperGolem.updateWeathering`: advances the oxidation stage on a fixed game-time
    /// schedule, then, once fully oxidized, rolls `canTurnToStatue` every tick.
    async fn update_weathering(&self, world: &Arc<World>) {
        let next_tick = self.next_weathering_tick.load();
        if next_tick == IGNORE_WEATHERING_TICK {
            return;
        }

        let state = self.weather_state();
        let is_fully_oxidized = state == CopperWeatherState::Oxidized;

        if next_tick == UNSET_WEATHERING_TICK {
            let game_time = world.get_world_age().await;
            let delay = rand::rng().random_range(WEATHERING_TICK_FROM..=WEATHERING_TICK_TO);
            self.next_weathering_tick.store(game_time + delay);
        } else if !is_fully_oxidized {
            let game_time = world.get_world_age().await;
            if game_time >= next_tick {
                let new_state = state.next();
                self.set_weather_state(new_state);
                if new_state == CopperWeatherState::Oxidized {
                    self.next_weathering_tick.store(0);
                } else {
                    let delay = rand::rng().random_range(WEATHERING_TICK_FROM..=WEATHERING_TICK_TO);
                    self.next_weathering_tick.store(next_tick + delay);
                }
            }
        }

        if is_fully_oxidized && Self::can_turn_to_statue(world, &self.position()) {
            self.turn_to_statue(world).await;
        }
    }

    fn position(&self) -> BlockPos {
        self.mob_entity.living_entity.entity.block_pos.load()
    }

    fn can_turn_to_statue(world: &World, pos: &BlockPos) -> bool {
        let state_id = world.get_block_state_id(pos);
        pumpkin_data::block_properties::is_air(state_id)
            && rand::rng().random::<f32>() <= TURN_TO_STATUE_CHANCE
    }

    /// `CopperGolem.turnToStatue`: replaces the golem with an oxidized copper golem statue
    /// block, always at the fully-oxidized stage (only reachable once the golem itself is
    /// fully oxidized).
    async fn turn_to_statue(&self, world: &Arc<World>) {
        let pos = self.position();
        let entity = &self.mob_entity.living_entity.entity;
        let facing = entity.get_horizontal_facing();

        let mut props =
            CopperGolemStatueLikeProperties::default(&Block::OXIDIZED_COPPER_GOLEM_STATUE);
        props.r#copper_golem_pose = match rand::rng().random_range(0..4) {
            0 => CopperGolemPose::Standing,
            1 => CopperGolemPose::Sitting,
            2 => CopperGolemPose::Running,
            _ => CopperGolemPose::Star,
        };
        props.r#facing = facing;
        let state_id = props.to_state_id(&Block::OXIDIZED_COPPER_GOLEM_STATUE);

        world
            .set_block_state(&pos, state_id, BlockFlags::NOTIFY_ALL)
            .await;

        if let Some(block_entity) = world.get_block_entity(&pos)
            && let Some(statue) = block_entity
                .as_any()
                .downcast_ref::<CopperGolemStatueBlockEntity>()
        {
            statue.create_statue();
        }

        entity.remove().await;
    }

    /// `CopperGolem.mobInteract`: honeycomb waxes (freezes weathering), axe on a waxed golem
    /// un-waxes it, axe on a weathered-but-unwaxed golem scrapes back one stage.
    ///
    /// Deferred: none of `SoundEvents.COPPER_GOLEM_*`/`AXE_SCRAPE` are wired here -- the
    /// generated `Sound` enum (`pumpkin-data/src/generated/sound.rs`, off-limits to hand-edit)
    /// has no `COPPER_GOLEM_*` entries at all, so this is a codegen gap, not a logic gap.
    pub fn golem_interact(&self, player: &Player, item_stack: &mut ItemStack) -> bool {
        if item_stack.item.id == Item::HONEYCOMB.id
            && self.next_weathering_tick.load() != IGNORE_WEATHERING_TICK
        {
            self.next_weathering_tick.store(IGNORE_WEATHERING_TICK);
            item_stack.decrement_unless_creative(player.gamemode.load(), 1);
            return true;
        }

        if item_stack.item.has_tag(&tag::Item::MINECRAFT_AXES) {
            if self.next_weathering_tick.load() == IGNORE_WEATHERING_TICK {
                self.next_weathering_tick.store(UNSET_WEATHERING_TICK);
                let _ = item_stack.damage_item(1);
                return true;
            }

            let state = self.weather_state();
            if state != CopperWeatherState::Unaffected {
                self.next_weathering_tick.store(UNSET_WEATHERING_TICK);
                self.set_weather_state(state.previous());
                let _ = item_stack.damage_item(1);
                return true;
            }
        }

        false
    }
}

impl NBTStorage for CopperGolemEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_long("next_weather_age", self.next_weathering_tick.load());
            nbt.put_string("weather_state", self.weather_state().as_str().to_string());
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.next_weathering_tick.store(
                nbt.get_long("next_weather_age")
                    .unwrap_or(UNSET_WEATHERING_TICK),
            );
            self.set_weather_state(CopperWeatherState::from_name(
                nbt.get_string("weather_state").unwrap_or("unaffected"),
            ));
        })
    }
}

impl Mob for CopperGolemEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let world = self.mob_entity.living_entity.entity.world.load();
            self.update_weathering(&world).await;
        })
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> crate::entity::EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if self.golem_interact(player, item_stack) {
                return true;
            }
            self.get_mob_entity().mob_interact(player, item_stack).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CopperWeatherState, IGNORE_WEATHERING_TICK, UNSET_WEATHERING_TICK, WEATHERING_TICK_FROM,
        WEATHERING_TICK_TO,
    };

    #[test]
    fn weather_state_next_saturates_at_oxidized() {
        assert_eq!(
            CopperWeatherState::Unaffected.next(),
            CopperWeatherState::Exposed
        );
        assert_eq!(
            CopperWeatherState::Exposed.next(),
            CopperWeatherState::Weathered
        );
        assert_eq!(
            CopperWeatherState::Weathered.next(),
            CopperWeatherState::Oxidized
        );
        assert_eq!(
            CopperWeatherState::Oxidized.next(),
            CopperWeatherState::Oxidized
        );
    }

    #[test]
    fn weather_state_previous_saturates_at_unaffected() {
        assert_eq!(
            CopperWeatherState::Oxidized.previous(),
            CopperWeatherState::Weathered
        );
        assert_eq!(
            CopperWeatherState::Weathered.previous(),
            CopperWeatherState::Exposed
        );
        assert_eq!(
            CopperWeatherState::Exposed.previous(),
            CopperWeatherState::Unaffected
        );
        assert_eq!(
            CopperWeatherState::Unaffected.previous(),
            CopperWeatherState::Unaffected
        );
    }

    #[test]
    fn weather_state_name_round_trips() {
        for state in [
            CopperWeatherState::Unaffected,
            CopperWeatherState::Exposed,
            CopperWeatherState::Weathered,
            CopperWeatherState::Oxidized,
        ] {
            assert_eq!(CopperWeatherState::from_name(state.as_str()), state);
        }
    }

    #[test]
    fn weathering_sentinels_are_distinct_from_the_real_schedule_range() {
        const {
            assert!(UNSET_WEATHERING_TICK < 0);
            assert!(IGNORE_WEATHERING_TICK < 0);
            assert!(WEATHERING_TICK_FROM < WEATHERING_TICK_TO);
            assert!(WEATHERING_TICK_FROM > 0);
        }
        assert_ne!(UNSET_WEATHERING_TICK, IGNORE_WEATHERING_TICK);
    }
}
