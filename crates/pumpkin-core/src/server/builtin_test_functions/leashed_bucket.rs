use std::sync::Arc;

use pumpkin_data::{Block, entity::EntityType, item::Item, item_stack::ItemStack};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult, SimulatedPlayer};
use pumpkin_util::{
    GameMode, Hand,
    math::{position::BlockPos, vector3::Vector3},
};

use super::TestEventHandlers;

use crate::{
    entity::{Entity, EntityBase, decoration::leash_knot::LeashKnotEntity},
    plugin::{BoxFuture, EventRwHandler, api::events::entity::entity_unleash::EntityUnleashEvent},
    server::{Server, server_test_manager::server_world},
    world::World,
};

const PICKUP_POS: Vector3<f64> = Vector3::new(4.5, 1.0, 3.5);
const PICKUP_BLOCK: BlockPos = BlockPos::new(4, 1, 3);

pub(super) async fn test_leashed_bucket_pickup_respects_unleash(
    test: GameTestContext,
) -> GameTestResult<()> {
    let world = server_world(&test)?;
    let (player, cow) = setup_bucket_pickup(&test, &world).await?;

    for (holder_missing, expected_leads) in [(false, 1), (true, 2)] {
        let axolotl = test.spawn_entity("minecraft:axolotl", PICKUP_POS).await?;
        axolotl.set_no_ai(true)?;
        let raw = world
            .get_entity_by_id(axolotl.entity_id())
            .ok_or_else(|| GameTestError::World("Axolotl was not spawned".to_string()))?;
        let entity = raw.get_entity();
        if holder_missing {
            let knot: Arc<dyn EntityBase> = Arc::new(LeashKnotEntity::new(Entity::new(
                world.clone(),
                test.absolute_vec(PICKUP_POS),
                &EntityType::LEASH_KNOT,
            )));
            entity.attach_leash(&knot);
        } else {
            entity.attach_leash(&cow);
        }

        player.set_held_item(Hand::Right, ItemStack::new(1, &Item::WATER_BUCKET))?;
        player.interact_with(&axolotl, Hand::Right)?;
        test.assert_true(
            entity.is_removed() && player.held_item(Hand::Right).item == &Item::AXOLOTL_BUCKET,
            format!("Bucket pickup failed with missing leash holder set to {holder_missing}"),
        )?;
        test.assert_item_entity_count("minecraft:lead", &PICKUP_BLOCK, 1.0, expected_leads)
            .await?;
    }

    test.succeed();
    Ok(())
}

struct CancelUnleash {
    entity_id: i32,
}

impl EventRwHandler<EntityUnleashEvent> for CancelUnleash {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut EntityUnleashEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if event.entity_id == self.entity_id {
                event.cancelled = true;
            }
        })
    }
}

pub(super) async fn test_leashed_bucket_pickup_cancelled_unleash(
    test: GameTestContext,
) -> GameTestResult<()> {
    let world = server_world(&test)?;
    let (player, cow) = setup_bucket_pickup(&test, &world).await?;
    let axolotl = test.spawn_entity("minecraft:axolotl", PICKUP_POS).await?;
    axolotl.set_no_ai(true)?;
    let raw = world
        .get_entity_by_id(axolotl.entity_id())
        .ok_or_else(|| GameTestError::World("Axolotl was not spawned".to_string()))?;
    let entity = raw.get_entity();
    entity.attach_leash(&cow);
    let server = world
        .server
        .upgrade()
        .ok_or_else(|| GameTestError::World("Server stopped".to_string()))?;
    let hooks = Arc::new(CancelUnleash {
        entity_id: axolotl.entity_id(),
    });
    let handlers = TestEventHandlers::new(&server.plugin_manager, axolotl.entity_id());
    handlers.register::<EntityUnleashEvent>(hooks);

    player.set_held_item(Hand::Right, ItemStack::new(1, &Item::WATER_BUCKET))?;
    player.interact_with(&axolotl, Hand::Right)?;
    test.assert_true(
        !entity.is_removed()
            && entity.get_leash_holder().is_some()
            && player.held_item(Hand::Right).item == &Item::WATER_BUCKET,
        "A cancelled unleash still bucketed the mob or changed the leash",
    )?;
    test.assert_item_entity_count("minecraft:lead", &PICKUP_BLOCK, 1.0, 0)
        .await?;
    test.succeed();
    Ok(())
}

async fn setup_bucket_pickup(
    test: &GameTestContext,
    world: &World,
) -> GameTestResult<(Arc<dyn SimulatedPlayer>, Arc<dyn EntityBase>)> {
    test.set_dimensions(7, 5, 7).await?;
    test.build_floor(7, 7, &Block::STONE).await?;
    let player = test
        .spawn_simulated_player(
            "BucketTester",
            Vector3::new(2.5, 1.0, 3.5),
            GameMode::Survival,
        )
        .await?;
    let cow = test
        .spawn_entity("minecraft:cow", Vector3::new(5.5, 1.0, 3.5))
        .await?;
    cow.set_no_ai(true)?;
    let cow = world
        .get_entity_by_id(cow.entity_id())
        .ok_or_else(|| GameTestError::World("Cow was not spawned".to_string()))?;
    Ok((player, cow))
}
