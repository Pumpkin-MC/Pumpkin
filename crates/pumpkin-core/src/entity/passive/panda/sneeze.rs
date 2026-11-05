//! Vanilla `Panda` sneezing: the sneeze counter flow of `Panda.tick` and `afterSneeze`.

use std::sync::atomic::Ordering::Relaxed;

use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_protocol::codec::var_int::VarInt;
use rand::RngExt;

use super::PandaEntity;
use crate::entity::EntityBase;
use crate::entity::ageable::AgeableMob;
use crate::entity::mob::Mob;
use crate::world::loot::{LootContextParameters, generate_loot_from_handle};

/// `Panda.tick` ends the sneeze once the sneeze counter has passed this value.
const SNEEZE_COUNTER_END: i32 = 20;
/// `Panda.afterSneeze` makes the pandas inside the bounding box inflated by this much jump.
const SNEEZE_JUMP_RANGE: f64 = 10.0;
/// `BuiltInLootTables.PANDA_SNEEZE`.
const PANDA_SNEEZE_LOOT_TABLE: &str = "minecraft:gameplay/panda_sneeze";

impl PandaEntity {
    /// Vanilla `Panda.sneeze`: starts or ends the sneeze flag; ending it resets the counter.
    pub fn sneeze(&self, value: bool) {
        self.set_flag(super::FLAG_SNEEZE, value);
        if !value {
            self.set_sneeze_counter(0);
        }
    }

    /// Vanilla `getSneezeCounter`.
    #[must_use]
    pub fn get_sneeze_counter(&self) -> i32 {
        self.sneeze_counter.load(Relaxed)
    }

    /// Vanilla `setSneezeCounter`; the counter is synched so the client can animate the sneeze.
    pub fn set_sneeze_counter(&self, value: i32) {
        self.sneeze_counter.store(value, Relaxed);
        self.get_entity().set_synced_data(
            pumpkin_data::tracked_data::panda::SNEEZE_COUNTER,
            VarInt(value),
        );
    }

    /// The sneeze part of vanilla `Panda.tick`: while the flag is set the counter grows by one
    /// per tick, plays `PANDA_PRE_SNEEZE` at 1 and ends the sneeze with `afterSneeze` once it
    /// passes 20.
    pub(super) fn tick_sneeze(&self) {
        if !self.is_sneezing() {
            return;
        }
        let counter = self.get_sneeze_counter() + 1;
        self.set_sneeze_counter(counter);
        if counter > SNEEZE_COUNTER_END {
            self.sneeze(false);
            self.after_sneeze();
        } else if counter == 1 {
            self.play_entity_sound(Sound::EntityPandaPreSneeze, 1.0, 1.0);
        }
    }

    /// Vanilla `Panda.afterSneeze`: plays `PANDA_SNEEZE`, makes the adult pandas within 10
    /// blocks jump and may drop the sneeze gift. The sneeze particle is added by the client
    /// from the synched sneeze counter, so no particle packet is sent.
    ///
    /// This reads and changes other entities up to 10 blocks away from the panda's tick, which
    /// the conservative entity island reach covers only because `has_audited_tick` is false for
    /// pandas; audit it before ever setting that.
    fn after_sneeze(&self) {
        self.play_entity_sound(Sound::EntityPandaSneeze, 1.0, 1.0);

        let entity = self.get_entity();
        let world = entity.world.load_full();
        let area = entity.bounding_box.load().expand(
            SNEEZE_JUMP_RANGE,
            SNEEZE_JUMP_RANGE,
            SNEEZE_JUMP_RANGE,
        );
        for other in world.get_entities_at_box(&area) {
            if let Some(panda) = other.cast_any().downcast_ref::<Self>()
                && !panda.is_baby()
                && panda.get_entity().on_ground.load(Relaxed)
                && !panda.get_entity().is_in_water()
                && panda.can_perform_action()
            {
                panda
                    .mob_entity
                    .living_entity
                    .jump_from_ground(panda.jump_power_multiplier());
            }
        }

        self.drop_sneeze_gift(self.get_random().random());
    }

    /// The loot rolled by `dropFromGiftLootTable` with `PANDA_SNEEZE` for a loot seed; empty if
    /// the table is unknown.
    fn roll_sneeze_gift(&self, seed: i64) -> Vec<ItemStack> {
        let world = self.get_entity().world.load_full();
        let Some(table) = world.get_loot_table(PANDA_SNEEZE_LOOT_TABLE) else {
            return Vec::new();
        };
        let params = LootContextParameters {
            world: Some(&world),
            position: Some(self.get_entity().pos.load()),
            this_entity: Some(self),
            ..Default::default()
        };
        generate_loot_from_handle(&table, seed, &params)
    }

    /// The end of vanilla `afterSneeze`: with `mob_drops` on, drops the `PANDA_SNEEZE` gift at
    /// the panda's feet. A plugin can cancel each stack through `EntityDropItemEvent`. Returns
    /// whether an item was dropped.
    fn drop_sneeze_gift(&self, seed: i64) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        if !world.level_info.load().game_rules.mob_drops {
            return false;
        }
        let mut dropped = false;
        for stack in self.roll_sneeze_gift(seed) {
            dropped |= self.spawn_stack_at_feet(stack);
        }
        dropped
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::Ordering::Relaxed;

    use pumpkin_data::Block;
    use pumpkin_data::entity::EntityType;
    use pumpkin_data::item::Item;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_util::math::vector3::Vector3;
    use pumpkin_world::chunk::ChunkData;
    use pumpkin_world::world::BlockFlags;

    use super::*;
    use crate::entity::Entity;
    use crate::entity::ai::test_support::world;
    use crate::world::World;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    /// A panda standing on the stone floor at `(x, z)`.
    fn spawn_panda(world: &Arc<World>, x: f64, z: f64) -> Arc<PandaEntity> {
        let panda = PandaEntity::new(Entity::new(
            world.clone(),
            Vector3::new(x, 64.0, z),
            &EntityType::PANDA,
        ));
        panda.get_entity().on_ground.store(true, Relaxed);
        world.spawn_entity(panda.clone());
        panda
    }

    fn stone_floor(world: &Arc<World>) {
        for chunk_x in -2..=1 {
            for chunk_z in -2..=1 {
                world.level.loaded_chunks.insert(
                    Vector2::new(chunk_x, chunk_z),
                    ChunkData::empty_sync(chunk_x, chunk_z),
                );
            }
        }
        let flags = BlockFlags::SKIP_BLOCK_ADDED_CALLBACK | BlockFlags::UPDATE_KNOWN_SHAPE;
        for x in -20..=20 {
            for z in -20..=20 {
                world.set_block_state(
                    &BlockPos::new(x, 63, z),
                    Block::STONE.default_state.id,
                    flags,
                );
            }
        }
    }

    fn vertical_speed(panda: &PandaEntity) -> f64 {
        panda.get_entity().velocity.load().y
    }

    #[tokio::test]
    async fn sneeze_counter_runs_for_twenty_one_ticks_then_resets() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = spawn_panda(&world, 0.5, 0.5);
        panda.mob_init_data_tracker();
        let synched = &panda.get_entity().synched_data;

        panda.tick_sneeze();
        assert_eq!(panda.get_sneeze_counter(), 0, "counted without a sneeze");

        synched.clear_dirty();
        panda.sneeze(true);
        assert!(panda.is_sneezing());
        assert!(synched.is_dirty(), "the sneeze flag was not synched");
        for tick in 1..=SNEEZE_COUNTER_END {
            synched.clear_dirty();
            panda.tick_sneeze();
            assert!(synched.is_dirty(), "the counter was not synched at {tick}");
            assert!(panda.is_sneezing(), "sneeze ended at tick {tick}");
            assert_eq!(panda.get_sneeze_counter(), tick);
        }
        panda.tick_sneeze();
        assert!(!panda.is_sneezing());
        assert_eq!(panda.get_sneeze_counter(), 0);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn sneeze_goes_on_when_the_baby_grows_up_or_sits_down() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let neighbour = spawn_panda(&world, 4.5, 0.5);
        panda.set_age(-24000);

        panda.sneeze(true);
        for _ in 0..10 {
            panda.tick_sneeze();
        }
        panda.set_age(0);
        panda.set_sitting(true);
        for _ in 0..10 {
            panda.tick_sneeze();
        }
        assert!(panda.is_sneezing());
        assert_eq!(panda.get_sneeze_counter(), 20);
        assert!(vertical_speed(&neighbour).abs() < 1e-9);

        panda.tick_sneeze();
        assert!(!panda.is_sneezing());
        assert!(
            vertical_speed(&neighbour) > 0.3,
            "the sneeze did not reach the neighbour"
        );
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn sneeze_only_makes_grounded_adults_that_can_act_jump_within_ten_blocks() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        stone_floor(&world);
        let sneezer = spawn_panda(&world, 0.5, 0.5);
        let near = spawn_panda(&world, 5.5, 0.5);
        let edge = spawn_panda(&world, 10.5, 0.5);
        let far = spawn_panda(&world, 13.5, 0.5);
        let baby = spawn_panda(&world, 2.5, 3.5);
        baby.set_age(-24000);
        let sitting = spawn_panda(&world, 2.5, -3.5);
        sitting.set_sitting(true);
        let airborne = spawn_panda(&world, -3.5, 3.5);
        airborne.get_entity().on_ground.store(false, Relaxed);
        let swimming = spawn_panda(&world, -3.5, -3.5);
        swimming.get_entity().touching_water.store(true, Relaxed);

        sneezer.sneeze(true);
        for _ in 0..=SNEEZE_COUNTER_END {
            sneezer.tick_sneeze();
        }

        for (name, jumped, panda) in [
            ("near adult", true, &near),
            ("adult at the edge of the box", true, &edge),
            ("adult out of range", false, &far),
            ("baby", false, &baby),
            ("sitting adult", false, &sitting),
            ("airborne adult", false, &airborne),
            ("adult in water", false, &swimming),
        ] {
            assert_eq!(vertical_speed(panda) > 0.3, jumped, "{name}");
        }
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn sneeze_gift_drops_at_the_panda_only_with_mob_drops_on() -> TestResult {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let panda = spawn_panda(&world, 0.5, 0.5);
        let winning_seed = (0..100_000)
            .find(|seed| !panda.roll_sneeze_gift(*seed).is_empty())
            .ok_or("no seed wins the gift")?;
        let gift = panda.roll_sneeze_gift(winning_seed);
        assert_eq!(gift.len(), 1);
        assert_eq!(gift[0].item.id, Item::SLIME_BALL.id);
        assert_eq!(gift[0].item_count, 1);
        let items_near = || {
            let area = panda.get_entity().bounding_box.load().expand(2.0, 2.0, 2.0);
            world
                .get_entities_at_box(&area)
                .iter()
                .filter(|entity| entity.get_item_entity().is_some())
                .count()
        };

        let mut level_info = (**world.level_info.load()).clone();
        level_info.game_rules.mob_drops = false;
        world.level_info.store(Arc::new(level_info.clone()));
        assert!(!panda.drop_sneeze_gift(winning_seed));
        assert_eq!(items_near(), 0, "dropped with mob drops off");

        level_info.game_rules.mob_drops = true;
        world.level_info.store(Arc::new(level_info));
        assert!(panda.drop_sneeze_gift(winning_seed));
        assert_eq!(items_near(), 1);
        world.level.shutdown().await;
        Ok(())
    }
}
