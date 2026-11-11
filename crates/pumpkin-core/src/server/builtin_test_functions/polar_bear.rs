use pumpkin_data::entity::EntityType;
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{GameMode, math::vector3::Vector3};

use crate::{
    entity::{Entity, EntityBase, ageable::AgeableMob, passive::polar_bear::PolarBearEntity},
    server::server_test_manager::ServerSimulatedPlayer,
};

type DefenseCase = (
    &'static str,
    bool,
    Option<(bool, Vector3<f64>)>,
    f64,
    GameMode,
    bool,
);

const MAX_TARGET_SELECTION_PASSES: usize = 512;
const DEFENSE_CASES: [DefenseCase; 10] = [
    ("no cub", false, None, 5.0, GameMode::Survival, false),
    (
        "adult companion",
        false,
        Some((false, Vector3::new(1.0, 0.0, 0.0))),
        5.0,
        GameMode::Survival,
        false,
    ),
    (
        "nearby cub",
        false,
        Some((true, Vector3::new(1.0, 0.0, 0.0))),
        5.0,
        GameMode::Survival,
        true,
    ),
    (
        "cub at box corner",
        false,
        Some((true, Vector3::new(8.0, 4.0, 8.0))),
        5.0,
        GameMode::Survival,
        true,
    ),
    (
        "cub outside horizontal box",
        false,
        Some((true, Vector3::new(10.0, 0.0, 0.0))),
        5.0,
        GameMode::Survival,
        false,
    ),
    (
        "cub outside vertical box",
        false,
        Some((true, Vector3::new(0.0, 6.0, 0.0))),
        5.0,
        GameMode::Survival,
        false,
    ),
    (
        "baby defender",
        true,
        Some((true, Vector3::new(1.0, 0.0, 0.0))),
        5.0,
        GameMode::Survival,
        false,
    ),
    (
        "player outside half range",
        false,
        Some((true, Vector3::new(1.0, 0.0, 0.0))),
        11.0,
        GameMode::Survival,
        false,
    ),
    (
        "creative player",
        false,
        Some((true, Vector3::new(1.0, 0.0, 0.0))),
        5.0,
        GameMode::Creative,
        false,
    ),
    (
        "spectator player",
        false,
        Some((true, Vector3::new(1.0, 0.0, 0.0))),
        5.0,
        GameMode::Spectator,
        false,
    ),
];

pub(super) async fn test_polar_bear_cub_defense_and_target_lifecycle(
    test: GameTestContext,
) -> GameTestResult<()> {
    let simulated = test
        .spawn_simulated_player(
            "BearTester",
            Vector3::new(17.5, 2.0, 12.5),
            GameMode::Survival,
        )
        .await?;
    let player = simulated
        .as_any()
        .downcast_ref::<ServerSimulatedPlayer>()
        .ok_or_else(|| GameTestError::World("Expected server simulated player".to_string()))?
        .player();
    let world = player.world();
    let position = test.absolute_vec(Vector3::new(12.5, 2.0, 12.5));

    for (name, baby, cub_offset, distance, mode, expected) in DEFENSE_CASES {
        player.set_gamemode(mode);
        player
            .get_entity()
            .set_pos(position.add_raw(distance, 0.0, 0.0));
        let bear = PolarBearEntity::new(Entity::new(
            world.clone(),
            position,
            &EntityType::POLAR_BEAR,
        ));
        bear.set_baby(baby);
        bear.mob_entity.goals_selector.clear(bear.as_ref());
        test.assert_true(
            world.spawn_entity(bear.clone()),
            "Could not spawn polar bear",
        )?;
        let cub = if let Some((baby, offset)) = cub_offset {
            let cub = PolarBearEntity::new(Entity::new(
                world.clone(),
                position + offset,
                &EntityType::POLAR_BEAR,
            ));
            cub.set_baby(baby);
            test.assert_true(world.spawn_entity(cub.clone()), "Could not spawn cub")?;
            Some(cub)
        } else {
            None
        };

        for _ in 0..MAX_TARGET_SELECTION_PASSES {
            bear.mob_entity.server_ai_step(bear.as_ref(), bear.as_ref());
            if bear.mob_entity.get_target().is_some() {
                break;
            }
        }
        let target = bear.mob_entity.get_target();
        let selected_player = target
            .as_ref()
            .is_some_and(|target| target.get_entity().entity_id == player.entity_id());
        test.assert_true(
            selected_player == expected && (target.is_none() || selected_player),
            format!("Polar bear defense ({name}): player targeted={selected_player}, expected={expected}"),
        )?;

        if expected {
            if let Some(cub) = &cub {
                cub.get_entity().remove();
            }
            bear.mob_entity.server_ai_step(bear.as_ref(), bear.as_ref());
            test.assert_true(
                bear.mob_entity.get_target().is_some(),
                "Polar bear forgot its target as soon as the cub left",
            )?;
            player
                .get_entity()
                .set_pos(position.add_raw(11.0, 0.0, 0.0));
            bear.mob_entity.server_ai_step(bear.as_ref(), bear.as_ref());
            test.assert_true(
                bear.mob_entity.get_target().is_none(),
                "Polar bear retained its target outside the half follow range",
            )?;
        }
        bear.get_entity().remove();
        if let Some(cub) = cub {
            cub.get_entity().remove();
        }
    }

    test.succeed();
    Ok(())
}
