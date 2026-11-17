use pumpkin_data::{Block, item::Item, item_stack::ItemStack};
use pumpkin_gametest::{GameTestContext, GameTestError, GameTestResult};
use pumpkin_util::{GameMode, Hand, math::vector3::Vector3};

#[expect(
    clippy::too_many_lines,
    reason = "covers the allay interaction sequence"
)]
pub(super) async fn test_allay_interaction(test: GameTestContext) -> GameTestResult<()> {
    test.set_dimensions(5, 5, 5).await?;
    test.build_floor(5, 5, &Block::STONE).await?;

    let player = test
        .spawn_simulated_player(
            "AllayTester",
            Vector3::new(1.5, 1.0, 1.5),
            GameMode::Survival,
        )
        .await?;
    let allay = test
        .spawn_entity("minecraft:allay", Vector3::new(2.5, 1.0, 1.5))
        .await?;
    allay.set_no_ai(true)?;

    player.look_at_entity(&allay).await?;

    let sword = ItemStack::new(1, &Item::DIAMOND_SWORD);
    let diamond = ItemStack::new(1, &Item::DIAMOND);
    player.set_held_item(Hand::Right, diamond.clone())?;
    player.set_held_item(Hand::Left, sword.clone())?;

    let give_diamond = (player, allay, sword, diamond);
    let try_offhand = give_diamond.clone();
    let take_sample = give_diamond.clone();
    let name_allay = give_diamond.clone();
    let leash_allay = give_diamond.clone();
    let unleash_allay = give_diamond.clone();
    let dying_allay = give_diamond.clone();
    let dying_allay_again = give_diamond.clone();
    test.start_sequence()
        .then_idle(2)
        .then_execute(move |test| {
            let (player, allay, sword, diamond) = give_diamond.clone();
            async move {
                player.interact_with(&allay, Hand::Right)?;
                test.assert_true(
                    allay.held_item(Hand::Right)?.are_equal(&diamond)
                        && player.held_item(Hand::Left).are_equal(&sword)
                        && player.held_item(Hand::Right).is_empty(),
                    "Allay did not take the diamond while preserving the player's offhand item",
                )
            }
        })
        .then_idle(2)
        .then_execute(move |test| {
            let (player, allay, sword, diamond) = try_offhand.clone();
            async move {
                player.interact_with(&allay, Hand::Left)?;
                test.assert_true(
                    allay.held_item(Hand::Right)?.are_equal(&diamond)
                        && player.held_item(Hand::Left).are_equal(&sword),
                    "Allay replaced its diamond sample with the player's offhand item",
                )
            }
        })
        .then_idle(2)
        .then_execute(move |test| {
            let (player, allay, _, diamond) = take_sample.clone();
            async move {
                player.set_held_item(Hand::Right, ItemStack::EMPTY.clone())?;
                let first_free_slot = player.first_empty_inventory_slot().ok_or_else(|| {
                    GameTestError::World("Simulated player inventory is full".to_string())
                })?;
                player.interact_with(&allay, Hand::Right)?;
                test.assert_true(
                    player.inventory_item(first_free_slot)?.are_equal(&diamond)
                        && allay.held_item(Hand::Right)?.is_empty(),
                    "Taking the sample did not return the diamond to the first free slot",
                )
            }
        })
        .then_idle(2)
        .then_execute(move |test| {
            let (player, allay, _, _) = name_allay.clone();
            async move {
                let mut name_tag = ItemStack::new(1, &Item::NAME_TAG);
                name_tag.set_custom_name("Named Allay".to_string());
                player.set_held_item(Hand::Right, name_tag)?;
                player.interact_with(&allay, Hand::Right)?;
                test.assert_true(
                    allay.custom_name()?.as_deref() == Some("Named Allay")
                        && allay.held_item(Hand::Right)?.is_empty(),
                    "Allay took the name tag instead of being named",
                )
            }
        })
        .then_idle(2)
        .then_execute(move |test| {
            let (player, allay, _, _) = leash_allay.clone();
            async move {
                player.set_held_item(Hand::Right, ItemStack::new(1, &Item::LEAD))?;
                player.interact_with(&allay, Hand::Right)?;
                test.assert_true(
                    allay.is_leashed()? && allay.held_item(Hand::Right)?.is_empty(),
                    "Allay took the lead instead of being leashed",
                )
            }
        })
        .then_idle(2)
        .then_execute(move |test| {
            let (player, allay, _, _) = unleash_allay.clone();
            async move {
                player.set_held_item(Hand::Right, ItemStack::EMPTY.clone())?;
                player.interact_with(&allay, Hand::Right)?;
                test.assert_true(
                    !allay.is_leashed()? && allay.held_item(Hand::Right)?.is_empty(),
                    "Empty hand did not unleash the Allay",
                )
            }
        })
        .then_idle(2)
        .then_execute(move |test| {
            let (player, allay, sword, _) = dying_allay.clone();
            async move {
                allay.set_health(0.0)?;
                player.set_held_item(Hand::Right, sword.clone())?;
                player.interact_with(&allay, Hand::Right)?;
                test.assert_true(
                    player.held_item(Hand::Right).are_equal(&sword),
                    "Dying Allay accepted an item",
                )
            }
        })
        .then_idle(2)
        .then_execute(move |test| {
            let (player, allay, sword, _) = dying_allay_again.clone();
            async move {
                player.interact_with(&allay, Hand::Right)?;
                test.assert_true(
                    allay.held_item(Hand::Right)?.is_empty()
                        && player.held_item(Hand::Right).are_equal(&sword),
                    "Dying Allay kept its sample or changed the player's item",
                )
            }
        })
        .then_succeed();
    Ok(())
}
