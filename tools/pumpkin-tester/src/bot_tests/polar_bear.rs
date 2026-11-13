use std::sync::Arc;

use pumpkin_core::{
    entity::{Entity, EntityBase, ageable::AgeableMob, passive::polar_bear::PolarBearEntity},
    server::Server,
};
use pumpkin_data::entity::EntityType;
use pumpkin_util::{GameMode, math::vector3::Vector3};

use crate::bot::HeadlessBot;

type DefenseCase = (
    &'static str,
    bool,
    Option<(bool, Vector3<f64>)>,
    f64,
    GameMode,
    bool,
);

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

pub fn run_polar_bear_defense_test(
    server: &Arc<Server>,
    bot: &HeadlessBot,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let player = server.get_player_by_name(&bot.name).ok_or("missing bot")?;
    let world = player.world();
    let original_position = player.position();
    let original_mode = player.gamemode.load();
    let position = original_position.add_raw(0.0, 64.0, 0.0);
    let other_modes: Vec<_> = server
        .get_all_players()
        .into_iter()
        .filter(|other| other.entity_id() != player.entity_id())
        .map(|other| {
            let mode = other.gamemode.load();
            other.set_gamemode(GameMode::Spectator);
            (other, mode)
        })
        .collect();

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
        // Exercise the registered target selector without movement or melee changing the fixture.
        bear.mob_entity.goals_selector.clear(bear.as_ref());
        if !world.spawn_entity(bear.clone()) {
            return Err("Could not spawn the polar bear defender".into());
        }
        let cub = cub_offset.map(|(baby, offset)| {
            let cub = PolarBearEntity::new(Entity::new(
                world.clone(),
                position + offset,
                &EntityType::POLAR_BEAR,
            ));
            cub.set_baby(baby);
            world.spawn_entity(cub.clone());
            cub
        });

        // The vanilla search is random; allow many selector passes, stopping at the first target.
        for _ in 0..512 {
            bear.mob_entity.server_ai_step(bear.as_ref(), bear.as_ref());
            if bear.mob_entity.get_target().is_some() {
                break;
            }
        }
        let target = bear.mob_entity.get_target();
        let selected_player = target
            .as_ref()
            .is_some_and(|target| target.get_entity().entity_id == player.entity_id());
        if selected_player != expected || (target.is_some() && !selected_player) {
            return Err(format!(
                "Polar bear defense ({name}): player targeted={selected_player}, expected={expected}"
            ));
        }

        if expected {
            if let Some(cub) = &cub {
                cub.get_entity().remove();
            }
            bear.mob_entity.server_ai_step(bear.as_ref(), bear.as_ref());
            if bear.mob_entity.get_target().is_none() {
                return Err("Polar bear forgot its target as soon as the cub left".into());
            }
            player
                .get_entity()
                .set_pos(position.add_raw(11.0, 0.0, 0.0));
            bear.mob_entity.server_ai_step(bear.as_ref(), bear.as_ref());
            if bear.mob_entity.get_target().is_some() {
                return Err("Polar bear retained its target outside the half follow range".into());
            }
        }
        bear.get_entity().remove();
        if let Some(cub) = cub {
            cub.get_entity().remove();
        }
    }

    player.get_entity().set_pos(original_position);
    player.set_gamemode(original_mode);
    for (other, mode) in other_modes {
        other.set_gamemode(mode);
    }
    summary_lines.push("[PASS] bot:polar_bear_cub_defense_and_target_lifecycle".into());
    Ok(())
}
