//! Vanilla `ServerPlayer.pushEntities` for all players of a world.

use std::sync::{Arc, OnceLock};

use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::entity::EntityBase;
use crate::entity::living::{AskEntity, PushLookup};
use crate::entity::player::Player;
use crate::plugin::events_handled;
use crate::world::scoreboard::CollisionRule;

use super::World;

/// What the pass read of a player the first time one met it.
#[derive(Default)]
struct Met {
    pushable: OnceLock<bool>,
    team_collision: OnceLock<Option<(String, CollisionRule)>>,
}

/// Remembers the answers about players. In a crowd every player is met by many others,
/// and both answers take a lock all threads share: the block a player stands in and the
/// scoreboard.
struct MetPlayers(FxHashMap<i32, Met>);

impl PushLookup for MetPlayers {
    fn is_pushable(&self, entity: &dyn EntityBase) -> bool {
        let ask = || AskEntity.is_pushable(entity);
        self.0
            .get(&entity.get_entity().entity_id)
            .map_or_else(ask, |met| *met.pushable.get_or_init(ask))
    }

    fn team_collision(&self, entity: &dyn EntityBase) -> Option<(String, CollisionRule)> {
        let ask = || AskEntity.team_collision(entity);
        self.0
            .get(&entity.get_entity().entity_id)
            .map_or_else(ask, |met| met.team_collision.get_or_init(ask).clone())
    }
}

/// One player after another in list order, like vanilla. The pass moves nobody, so the
/// overlaps are looked up for all players at once; deaths from earlier turns are checked
/// on each turn.
pub(super) fn push_entities(world: &World, players: &[Arc<Player>]) {
    let entities = world.entities.load_full();
    let handled = events_handled();
    let met = MetPlayers(
        players
            .iter()
            .map(|player| (player.entity_id(), Met::default()))
            .collect(),
    );
    let pushable: Vec<_> = players
        .par_iter()
        .map(|player| player.pushable_entities(&met))
        .collect();
    for (player, pushable) in players.iter().zip(pushable) {
        // A plugin handler may have moved anybody. A death drop, a parrot leaving a hurt
        // player's shoulder or a player joining are new neighbours. Then the turn looks again.
        let as_found = events_handled() == handled
            && Arc::ptr_eq(&entities, &world.entities.load())
            && std::ptr::eq(players.as_ptr(), world.players.load().as_ptr());
        if as_found {
            player.push_entities_among(pushable);
        } else {
            player.push_entities();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use pumpkin_data::entity::EntityType;
    use pumpkin_data::game_rules::{GameRule, GameRuleValue};
    use pumpkin_protocol::java::client::play::CEntityVelocity;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_util::text::TextComponent;
    use pumpkin_util::text::color::NamedColor;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::passive::cow::CowEntity;
    use crate::plugin::api::events::entity::entity_damage::EntityDamageEvent;
    use crate::plugin::player::player_velocity::PlayerVelocityEvent;

    use crate::test_support::{Client, TestResult, count, server, shutdown};
    use crate::world::scoreboard::{NameTagVisibility, Team};

    fn velocities(entities: &[&Entity]) -> Vec<Vector3<f64>> {
        entities
            .iter()
            .map(|entity| entity.velocity.swap(Vector3::default()))
            .collect()
    }

    #[tokio::test]
    async fn a_crowd_is_pushed_like_one_player_after_another() -> TestResult {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.worlds.load()[0].clone();
        // Forty players on three blocks by three: everybody overlaps a few others, nowhere
        // enough of them for cramming, which rolls a die.
        let mut seed = 7u64;
        let mut spot = || {
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            (seed >> 33) as f64 / f64::from(1u32 << 31) * 3.0
        };
        let crowd: Vec<Client> = (1..=40)
            .map(|id| Client::join(&server, id, Vector3::new(spot(), 100.0, spot())))
            .collect();
        let cow: Arc<dyn EntityBase> = CowEntity::new(Entity::new(
            world.clone(),
            Vector3::new(1.5, 100.0, 1.5),
            &EntityType::COW,
        ));
        world.add_entity_silent(cow.clone());
        let players = world.players.load_full();
        let mut bodies: Vec<&Entity> = players.iter().map(|player| player.get_entity()).collect();
        bodies.push(cow.get_entity());

        for player in players.iter() {
            player.push_entities();
        }
        let one_after_another = velocities(&bodies);
        push_entities(&world, &players);
        let at_once = velocities(&bodies);

        assert!(one_after_another.iter().any(|v| *v != Vector3::default()));
        assert_eq!(at_once, one_after_another);
        drop(crowd);
        shutdown(&server).await;
        Ok(())
    }

    #[tokio::test]
    async fn a_player_killed_by_cramming_is_left_alone_for_the_rest_of_the_pass() -> TestResult {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.worlds.load()[0].clone();
        // The first player overlaps the second, the second only the first.
        let first = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
        let second = Client::join(&server, 2, Vector3::new(0.85, 100.0, 0.5));
        let players = world.players.load_full();
        let pushed = second.player.get_entity();

        first.player.push_entities();
        let one_push = pushed.velocity.swap(Vector3::default());
        assert_ne!(one_push, Vector3::default());

        // Now every overlap can hurt, and one hit kills the first player.
        world.set_game_rule(&GameRule::MaxEntityCramming, GameRuleValue::Int(1));
        let living = &first.player.living_entity;
        let mut died = false;
        for _ in 0..400 {
            living.health.store(1.0);
            living.dead.store(false, Ordering::Relaxed);
            living.hurt_cooldown.store(0, Ordering::Relaxed);
            first.player.set_client_loaded(true);
            first.player.get_entity().velocity.store(Vector3::default());
            pushed.velocity.store(Vector3::default());

            push_entities(&world, &players);
            if living.health.load() <= 0.0 {
                died = true;
                break;
            }
        }
        assert!(died, "cramming never hit in 400 passes");
        // It pushed on its own turn. On the second player's turn it was dead: no pair, so
        // no push back and no cramming for the second player.
        assert_eq!(pushed.velocity.load(), one_push);
        assert!(second.player.living_entity.health.load() > 1.0);
        shutdown(&server).await;
        Ok(())
    }

    #[tokio::test]
    async fn a_team_that_never_collides_neither_pushes_nor_is_pushed() -> TestResult {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.worlds.load()[0].clone();
        let first = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
        let second = Client::join(&server, 2, Vector3::new(0.85, 100.0, 0.5));
        let players = world.players.load_full();
        let bodies = [first.player.get_entity(), second.player.get_entity()];

        push_entities(&world, &players);
        assert!(velocities(&bodies).iter().all(|v| *v != Vector3::default()));

        world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .add_team(
                &world,
                Team {
                    name: "quiet".to_string(),
                    display_name: TextComponent::text("quiet"),
                    options: 0,
                    nametag_visibility: NameTagVisibility::Always,
                    collision_rule: CollisionRule::Never,
                    color: NamedColor::White,
                    player_prefix: TextComponent::text(""),
                    player_suffix: TextComponent::text(""),
                    players: vec![first.player.gameprofile.name.clone()],
                },
            );
        push_entities(&world, &players);
        assert!(velocities(&bodies).iter().all(|v| *v == Vector3::default()));
        shutdown(&server).await;
        Ok(())
    }

    struct SeesVelocity;
    impl crate::plugin::EventHandler<PlayerVelocityEvent> for SeesVelocity {}

    #[tokio::test(flavor = "multi_thread")]
    async fn watchers_get_the_velocity_of_every_pushed_player_once() -> TestResult {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.worlds.load()[0].clone();
        // Uneven gaps, so no player's pushes cancel out.
        let pushed: Vec<Client> = [0.0, 0.2, 0.5, 0.9, 1.0, 1.35]
            .into_iter()
            .zip(1..)
            .map(|(x, id)| Client::join(&server, id, Vector3::new(0.5 + x, 100.0, 0.5)))
            .collect();
        let mut watcher = Client::join(&server, 7, Vector3::new(8.5, 100.0, 0.5));
        let players = world.players.load_full();
        world.entity_tracker.update_all(&world);
        watcher.received().await?;

        // In parallel, then for a plugin that wants `PlayerVelocityEvent` in list order.
        for with_plugin in [false, true] {
            if with_plugin {
                server.plugin_manager.register(
                    Arc::new(SeesVelocity),
                    crate::plugin::EventPriority::Normal,
                    false,
                );
            }
            push_entities(&world, &players);
            World::send_velocity_changes(&players, &server);
            assert_eq!(
                count::<CEntityVelocity>(&watcher.received().await?),
                pushed.len()
            );
        }
        shutdown(&server).await;
        Ok(())
    }

    /// Stands in for a plugin that moves the second player away when somebody is hurt.
    struct MovesAway(Arc<Player>);

    impl crate::plugin::EventHandler<EntityDamageEvent> for MovesAway {
        fn handle_blocking<'a>(
            &'a self,
            _server: &'a Arc<crate::server::Server>,
            _event: &'a mut EntityDamageEvent,
        ) -> crate::plugin::BoxFuture<'a, ()> {
            self.0
                .get_entity()
                .set_pos(Vector3::new(100.85, 100.0, 0.5));
            Box::pin(async {})
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn later_turns_see_what_a_plugin_did_during_the_pass() -> TestResult {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.worlds.load()[0].clone();
        let first = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
        let second = Client::join(&server, 2, Vector3::new(0.85, 100.0, 0.5));
        let players = world.players.load_full();
        world.set_game_rule(&GameRule::MaxEntityCramming, GameRuleValue::Int(1));
        server.plugin_manager.register(
            Arc::new(MovesAway(second.player.clone())),
            crate::plugin::EventPriority::Normal,
            true,
        );
        let living = &first.player.living_entity;
        let bodies = [first.player.get_entity(), second.player.get_entity()];
        first.player.set_client_loaded(true);

        // The speeds after a pass in which cramming hurt the first player.
        let after_a_hit = |pass: &dyn Fn()| -> Option<Vec<Vector3<f64>>> {
            (0..400).find_map(|_| {
                living.health.store(20.0);
                living.hurt_cooldown.store(0, Ordering::Relaxed);
                bodies[1].set_pos(Vector3::new(0.85, 100.0, 0.5));
                velocities(&bodies);
                pass();
                let speeds = velocities(&bodies);
                (living.health.load() < 20.0).then_some(speeds)
            })
        };
        let one_after_another = after_a_hit(&|| {
            for player in players.iter() {
                player.push_entities();
            }
        });
        let at_once = after_a_hit(&|| push_entities(&world, &players));

        assert!(
            one_after_another.is_some(),
            "cramming never hit in 400 passes"
        );
        // The second player was moved out of reach before its turn: it pushes nobody.
        assert_eq!(at_once, one_after_another);
        shutdown(&server).await;
        Ok(())
    }
}
