use pumpkin_data::Block;
use pumpkin_gametest::{GameTestHelper, GameTestResult};
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use uuid::Uuid;

use super::build_stone_floor;

pub(super) async fn test_item_entities_merge(helper: GameTestHelper) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;

    for x in 0..4 {
        for z in 0..4 {
            if x == 0 || x == 3 || z == 0 || z == 3 {
                helper
                    .set_block_default(&BlockPos::new(x, 1, z), &Block::STONE)
                    .await?;
            }
        }
    }

    helper
        .spawn_item_at("minecraft:diamond", 10, Vector3::new(1.5, 1.1, 1.5))
        .await?;
    helper
        .spawn_item_at("minecraft:diamond", 15, Vector3::new(1.5, 1.1, 1.5))
        .await?;

    let center = BlockPos::new(1, 1, 1);
    helper.succeed_when(move |helper| async move {
        helper.assert_entities_present("minecraft:item", 1).await?;
        helper
            .assert_item_entity_count("minecraft:diamond", &center, 2.0, 25)
            .await
    })?;
    Ok(())
}

pub(super) async fn test_zombie_pathfinds_to_villager(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 5, 8).await?;

    for x in 0..5 {
        for z in 0..8 {
            if x == 0 || x == 4 || z == 0 || z == 7 {
                helper
                    .set_block_default(&BlockPos::new(x, 1, z), &Block::STONE)
                    .await?;
                helper
                    .set_block_default(&BlockPos::new(x, 2, z), &Block::STONE)
                    .await?;
            }
        }
    }

    helper
        .spawn_entity_at("minecraft:villager", Vector3::new(2.5, 1.0, 6.0))
        .await?;
    helper
        .spawn_entity_at("minecraft:husk", Vector3::new(2.5, 1.0, 1.5))
        .await?;

    helper.succeed_when(move |helper| async move {
        helper
            .assert_entity_present_in_box(
                "minecraft:husk",
                BlockPos::new(1, 1, 4),
                BlockPos::new(4, 3, 7),
            )
            .await
    })?;
    Ok(())
}

pub(super) async fn test_creaking_accepts_protected_hits(
    helper: GameTestHelper,
) -> GameTestResult<()> {
    build_stone_floor(&helper, 4, 4).await?;
    let heart = BlockPos::new(0, 1, 0);
    let target = Uuid::new_v4();
    let mut heart_nbt = NbtCompound::new();
    heart_nbt.put_string("id", "minecraft:creaking_heart".to_owned());
    heart_nbt.put_uuid("creaking", target);
    helper
        .set_block_with_nbt(&heart, Block::CREAKING_HEART.default_state.id, &heart_nbt)
        .await?;

    let mut creaking_nbt = NbtCompound::new();
    creaking_nbt.put_uuid("UUID", target);
    creaking_nbt.put_bool("PersistenceRequired", true);
    creaking_nbt.put_bool("Invulnerable", true);
    let home = helper.absolute_pos(&heart);
    let mut home_nbt = NbtCompound::new();
    home_nbt.put_int("x", home.0.x);
    home_nbt.put_int("y", home.0.y);
    home_nbt.put_int("z", home.0.z);
    creaking_nbt.put_compound("home_pos", home_nbt);
    let position = BlockPos::new(1, 1, 1);
    helper
        .execute_command(
            &position,
            &format!("summon minecraft:creaking ~ ~ ~ {creaking_nbt}"),
        )
        .await?;
    helper
        .execute_command(&position, "summon minecraft:cow ~1 ~ ~ {NoAI:1b}")
        .await?;
    let health = format!("data get entity {target} Health");
    helper.assert_true(
        helper.execute_command(&position, &health).await? == 1,
        "expected a living protected creaking with one health",
    )?;

    let hit = format!(
        "damage {target} 1 minecraft:mob_attack by @e[type=minecraft:cow,distance=..2,sort=nearest,limit=1]"
    );
    helper.assert_true(
        helper.execute_command(&position, &hit).await.is_err(),
        "an explicitly invulnerable creaking must reject a living attack",
    )?;
    helper
        .execute_command(
            &position,
            &format!("data merge entity {target} {{Invulnerable:0b}}"),
        )
        .await?;
    helper.assert_true(
        helper
            .execute_command(&position, &format!("damage {target} 1 minecraft:generic"))
            .await
            .is_err(),
        "a protected creaking must reject damage without a direct attacker or player credit",
    )?;
    helper.execute_command(&position, &hit).await?;
    helper.assert_true(
        helper.execute_command(&position, &health).await? == 1,
        "a protected creaking must accept a living attack without losing health",
    )?;
    helper.assert_true(
        helper.execute_command(&position, &hit).await.is_err(),
        "the accepted attack must start the protection cooldown",
    )?;
    helper
        .execute_command(
            &position,
            &format!("damage {target} 1 minecraft:generic_kill"),
        )
        .await?;
    helper.succeed();
    Ok(())
}
