use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
};

use pumpkin_data::{Block, item::Item, item_stack::ItemStack, statistic::StatisticCategory};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{GameMode, Hand, math::vector3::Vector3};

use super::TestEventHandlers;

use crate::{
    entity::{
        Entity, EntityBase, item::ItemEntity, passive::snow_golem::SnowGolemEntity, player::Player,
    },
    plugin::{
        BoxFuture, EventRwHandler,
        api::events::{
            block::entity_block_form::EntityBlockFormEvent,
            player::{
                player_shear_entity::PlayerShearEntityEvent,
                player_unleash_entity::PlayerUnleashEntityEvent,
            },
        },
    },
    server::{Server, server_test_manager::server_world},
    world::{BlockFlags, World},
};

struct SnowGolemEvents {
    entity_id: i32,
    formations: AtomicUsize,
    hand: AtomicU8,
    shears: AtomicUsize,
    cancel_shear_or_form: AtomicBool,
    cancel_unleash: AtomicBool,
}

impl EventRwHandler<PlayerShearEntityEvent> for SnowGolemEvents {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerShearEntityEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                self.hand.store(event.hand, Ordering::Relaxed);
                self.shears.fetch_add(1, Ordering::Relaxed);
                event.cancelled = self.cancel_shear_or_form.load(Ordering::Relaxed);
            }
        })
    }
}

impl EventRwHandler<PlayerUnleashEntityEvent> for SnowGolemEvents {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerUnleashEntityEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                event.cancelled = self.cancel_unleash.load(Ordering::Relaxed);
            }
        })
    }
}

impl EventRwHandler<EntityBlockFormEvent> for SnowGolemEvents {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityBlockFormEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity.get_entity().entity_id == self.entity_id {
                self.formations.fetch_add(1, Ordering::Relaxed);
                event.cancelled = self.cancel_shear_or_form.load(Ordering::Relaxed);
                event.new_state_id = Block::MOSS_CARPET.default_state.id;
            }
        })
    }
}

pub(super) async fn test_snow_golem_shearing_leash_and_snow_formation(
    test: GameTestContext,
) -> GameTestResult<()> {
    test.set_dimensions(20, 5, 5).await?;
    test.build_floor(20, 5, &Block::STONE).await?;
    let player = test
        .spawn_simulated_player(
            "GolemTester",
            Vector3::new(2.5, 1.0, 2.5),
            GameMode::Survival,
        )
        .await?;
    let golem = test
        .spawn_entity("minecraft:snow_golem", Vector3::new(4.5, 1.0, 2.5))
        .await?;
    golem.set_no_ai(true)?;
    let world = server_world(&test)?;
    let server = world
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;
    let raw = world
        .get_entity_by_id(golem.entity_id())
        .ok_or_else(|| GameTestError::World("Snow golem was not spawned".to_string()))?;
    let snow_golem = raw
        .cast_any()
        .downcast_ref::<SnowGolemEntity>()
        .ok_or_else(|| GameTestError::World("Spawned entity is not a snow golem".to_string()))?;
    let real_player = world
        .get_player_by_name(player.name())
        .ok_or_else(|| GameTestError::World("Simulated player was not spawned".to_string()))?;
    let events = Arc::new(SnowGolemEvents {
        entity_id: golem.entity_id(),
        formations: AtomicUsize::new(0),
        hand: AtomicU8::new(0),
        shears: AtomicUsize::new(0),
        cancel_shear_or_form: AtomicBool::new(true),
        cancel_unleash: AtomicBool::new(false),
    });
    let handlers = TestEventHandlers::new(&server.plugin_manager, golem.entity_id());
    handlers.register::<PlayerShearEntityEvent>(events.clone());
    handlers.register::<PlayerUnleashEntityEvent>(events.clone());
    handlers.register::<EntityBlockFormEvent>(events.clone());

    check_shearing(&test, &player, &real_player, &golem, snow_golem, &events)?;
    check_leash(
        &test,
        &player,
        &real_player,
        &golem,
        snow_golem,
        &world,
        &events,
    )?;
    check_snow_formation(&test, snow_golem, &world, &events)?;
    test.succeed();
    Ok(())
}

fn check_shearing(
    test: &GameTestContext,
    player: &Arc<dyn pumpkin_gametest::SimulatedPlayer>,
    real_player: &Player,
    golem: &Arc<dyn pumpkin_gametest::GameTestEntity>,
    snow_golem: &SnowGolemEntity,
    events: &SnowGolemEvents,
) -> GameTestResult<()> {
    let mut shears = ItemStack::new(1, &Item::SHEARS);
    let last_damage = shears
        .get_max_damage()
        .ok_or_else(|| GameTestError::World("Shears have no maximum durability".to_string()))?
        - 1;
    shears.set_damage(last_damage);
    player.set_held_item(Hand::Right, ItemStack::new(1, &Item::DIAMOND))?;
    player.set_held_item(Hand::Left, shears)?;
    let broken_before = shears_stat(real_player, StatisticCategory::Broken);

    player.interact_with(golem, Hand::Left)?;
    test.assert_true(
        snow_golem.has_pumpkin()
            && player.held_item(Hand::Left).get_damage() == last_damage
            && events.hand.load(Ordering::Relaxed) == 1,
        "Cancelled offhand shearing changed the golem or tool, or reported the wrong hand",
    )?;
    events.cancel_shear_or_form.store(false, Ordering::Relaxed);
    player.interact_with(golem, Hand::Left)?;
    test.assert_true(
        !snow_golem.has_pumpkin()
            && player.held_item(Hand::Left).is_empty()
            && player.held_item(Hand::Right).item == &Item::DIAMOND
            && shears_stat(real_player, StatisticCategory::Broken) == broken_before + 1
            && events.shears.load(Ordering::Relaxed) == 2,
        "Offhand shearing did not remove the pumpkin and break only the offhand tool",
    )?;
    check_shearing_drop(test, snow_golem)?;
    player.set_held_item(Hand::Left, ItemStack::new(1, &Item::SHEARS))?;
    player.interact_with(golem, Hand::Left)?;
    test.assert_true(
        player.held_item(Hand::Left).get_damage() == 0
            && events.shears.load(Ordering::Relaxed) == 2,
        "A golem without a pumpkin was sheared again",
    )
}

fn check_leash(
    test: &GameTestContext,
    player: &Arc<dyn pumpkin_gametest::SimulatedPlayer>,
    real_player: &Arc<Player>,
    golem: &Arc<dyn pumpkin_gametest::GameTestEntity>,
    snow_golem: &SnowGolemEntity,
    world: &World,
    events: &SnowGolemEvents,
) -> GameTestResult<()> {
    snow_golem.set_has_pumpkin(true);
    let holder: Arc<dyn EntityBase> = real_player.clone();
    let entity = snow_golem.get_entity();
    entity.attach_leash(&holder);
    events.cancel_unleash.store(true, Ordering::Relaxed);
    player.interact_with(golem, Hand::Left)?;
    test.assert_true(
        entity.is_leashed()
            && snow_golem.has_pumpkin()
            && player.held_item(Hand::Left).get_damage() == 0,
        "Cancelled leash cutting changed the leash, pumpkin or shears",
    )?;

    let original = entity.pos.load();
    entity.set_pos(
        real_player
            .position()
            .add_raw(Entity::LEASH_SNAP_DISTANCE + 2.0, 0.0, 0.0),
    );
    let leads_before = item_drops(world, &Item::LEAD, None);
    let retained = (0..3).all(|_| {
        entity.tick_leash(snow_golem);
        entity.get_leash_holder().is_some_and(|holder| {
            holder.get_entity().entity_id == real_player.get_entity().entity_id
        }) && item_drops(world, &Item::LEAD, None) == leads_before
    });
    entity.set_pos(original);
    test.assert_true(
        retained,
        "Cancelled leash snapping must retain the connection without dropping leads",
    )?;

    events.cancel_unleash.store(false, Ordering::Relaxed);
    player.interact_with(golem, Hand::Left)?;
    test.assert_true(
        !entity.is_leashed()
            && snow_golem.has_pumpkin()
            && player.held_item(Hand::Left).get_damage() == 1,
        "Shearing must cut the leash before removing the pumpkin",
    )
}

fn check_shearing_drop(test: &GameTestContext, golem: &SnowGolemEntity) -> GameTestResult<()> {
    let entity = golem.get_entity();
    let eye_position = entity.pos.load().add_raw(0.0, entity.get_eye_height(), 0.0);
    test.assert_true(
        item_drops(
            &entity.world.load(),
            &Item::CARVED_PUMPKIN,
            Some(eye_position),
        ) == 1,
        "Shearing must drop one carved pumpkin at eye height",
    )
}

fn item_drops(world: &World, item: &Item, at: Option<Vector3<f64>>) -> usize {
    world
        .entities
        .load()
        .iter()
        .filter_map(|entity| entity.cast_any().downcast_ref::<ItemEntity>())
        .filter(|entity| at.is_none_or(|position| entity.get_entity().pos.load() == position))
        .map(|entity| {
            let stack = entity
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if stack.item == item {
                usize::from(stack.item_count)
            } else {
                0
            }
        })
        .sum()
}

fn shears_stat(player: &Player, category: StatisticCategory) -> i32 {
    player
        .stats
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(category, i32::from(Item::SHEARS.id))
}

fn check_snow_formation(
    test: &GameTestContext,
    golem: &SnowGolemEntity,
    world: &Arc<World>,
    events: &SnowGolemEvents,
) -> GameTestResult<()> {
    use crate::entity::mob::Mob;

    let entity = golem.get_entity();
    let original = entity.pos.load();
    let base = entity.block_pos.load();
    entity.set_pos(Vector3::new(
        f64::from(base.0.x),
        f64::from(base.0.y),
        f64::from(base.0.z),
    ));
    let positions = [base, base.west(), base.north(), base.west().north()];
    let saved: Vec<_> = positions
        .iter()
        .flat_map(|pos| [*pos, pos.down()])
        .map(|pos| (pos, world.get_block_state_id(&pos)))
        .collect();
    let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
    let result = (|| {
        for pos in positions {
            world.set_block_state(&pos.down(), Block::STONE.default_state.id, flags);
            world.set_block_state(&pos, Block::AIR.default_state.id, flags);
        }
        events.cancel_shear_or_form.store(true, Ordering::Relaxed);
        golem.mob_ai_step();
        test.assert_true(
            events.formations.load(Ordering::Relaxed) == 4
                && positions
                    .iter()
                    .all(|pos| world.get_block(pos) == &Block::AIR),
            "Cancelled snow formation must leave all four positions unchanged",
        )?;
        events.cancel_shear_or_form.store(false, Ordering::Relaxed);
        golem.mob_ai_step();
        test.assert_true(
            events.formations.load(Ordering::Relaxed) == 8
                && positions
                    .iter()
                    .all(|pos| world.get_block(pos) == &Block::MOSS_CARPET),
            "Snow formation must honor a plugin's replacement block state",
        )
    })();
    for (pos, state) in saved {
        world.set_block_state(&pos, state, flags);
    }
    entity.set_pos(original);
    events.cancel_shear_or_form.store(false, Ordering::Relaxed);
    result
}
