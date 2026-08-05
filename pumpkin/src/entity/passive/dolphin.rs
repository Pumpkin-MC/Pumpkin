use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use pumpkin_data::Block;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        avoid_entity::AvoidEntityGoal, breath_air::BreathAirGoal,
        dolphin_hurt_by_target::DolphinHurtByTargetGoal, dolphin_jump::DolphinJumpGoal,
        dolphin_swim_to_treasure::DolphinSwimToTreasureGoal,
        dolphin_swim_with_player::DolphinSwimWithPlayerGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        try_find_water::TryFindWaterGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    player::Player,
};

/// `Dolphin.TOTAL_MOISTNESS_LEVEL`.
const TOTAL_MOISTNESS_LEVEL: i32 = 2400;
/// `Dolphin.tick`: dry-out damage dealt once moistness hits 0, applied every tick while dry.
const DRY_OUT_DAMAGE: f32 = 1.0;
/// `Dolphin.TOTAL_AIR_SUPPLY` / `Dolphin.getMaxAirSupply` (`Dolphin.java:77,194-196`): dolphins
/// hold far more air than the generic `LivingEntity` default of 300.
const MAX_AIR_SUPPLY: i32 = 4800;
/// `LivingEntity.baseTick` drowning threshold (`LivingEntity.java:444`): once air supply reaches
/// this, air is clamped to 0 and drown damage is dealt.
const DROWN_AIR_THRESHOLD: i32 = -20;
/// `LivingEntity.baseTick` (`LivingEntity.java:447`): `hurtServer(level, damageSources().drown(), 2.0F)`.
const DROWN_DAMAGE: f32 = 2.0;

/// Represents a Dolphin, a neutral aquatic mob that can give players the Dolphin's Grace effect.
///
/// Wiki: <https://minecraft.wiki/w/Dolphin>
pub struct DolphinEntity {
    pub mob_entity: MobEntity,
    got_fish: AtomicBool,
    moistness_level: AtomicI32,
    air_supply: AtomicI32,
}

impl DolphinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let dolphin = Self {
            mob_entity,
            got_fish: AtomicBool::new(false),
            moistness_level: AtomicI32::new(TOTAL_MOISTNESS_LEVEL),
            air_supply: AtomicI32::new(MAX_AIR_SUPPLY),
        };
        let mob_arc = Arc::new(dolphin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // `Dolphin.registerGoals` (`Dolphin.java:157-171`). No `FloatGoal`/`SwimGoal` is
            // registered in vanilla for Dolphin (it swims via `WaterBoundPathNavigation` +
            // `SmoothSwimmingMoveControl` instead), so the previous `SwimGoal` registration at
            // priority 0 is removed rather than kept as an approximation.
            goal_selector.add_goal(0, BreathAirGoal::new());
            goal_selector.add_goal(0, TryFindWaterGoal::new());
            goal_selector.add_goal(1, DolphinSwimToTreasureGoal::new(1.3));
            goal_selector.add_goal(2, DolphinSwimWithPlayerGoal::new(4.0));
            goal_selector.add_goal(4, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(4, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(
                5,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(5, Box::new(DolphinJumpGoal::new(10)));
            goal_selector.add_goal(6, Box::new(MeleeAttackGoal::new(1.2, true)));
            // `AvoidEntityGoal<Guardian>(8.0F, 1.0, 1.0)` at priority 9 (`Dolphin.java:170`).
            // Vanilla's search is `getEntitiesOfClass(Guardian.class, ...)`, which also matches
            // `ElderGuardian extends Guardian`; this codebase's `AvoidEntityGoal` only matches
            // one exact `EntityType`, so both are registered at the same priority as an
            // approximation. They share `Controls::MOVE`, so only one runs at a time (whichever
            // finds a threat first), unlike vanilla's single goal instance covering both types.
            goal_selector.add_goal(
                9,
                Box::new(AvoidEntityGoal::new(&EntityType::GUARDIAN, 8.0, 1.0, 1.0)),
            );
            goal_selector.add_goal(
                9,
                Box::new(AvoidEntityGoal::new(
                    &EntityType::ELDER_GUARDIAN,
                    8.0,
                    1.0,
                    1.0,
                )),
            );

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(1, DolphinHurtByTargetGoal::new());
        };

        mob_arc
    }

    #[must_use]
    pub fn get_air_supply(&self) -> i32 {
        self.air_supply.load(Relaxed)
    }

    /// `Entity::isEyeInFluid(FluidTags.WATER)` minus the bubble-column exclusion
    /// (`LivingEntity.java:436`), ported locally for Dolphin rather than generalizing
    /// `breath.rs`'s player-only `is_eye_in_water` (see the report for why the diff stays
    /// scoped to Dolphin).
    fn eye_in_breathable_water(entity: &Entity) -> bool {
        let pos = entity.pos.load();
        let eye_y = entity.get_eye_y();
        let bp = BlockPos::new(
            pos.x.floor() as i32,
            eye_y.floor() as i32,
            pos.z.floor() as i32,
        );
        let world = entity.world.load();

        if world.get_block(&bp) == &Block::BUBBLE_COLUMN {
            return false;
        }

        let (fluid, state) = world.get_fluid_and_fluid_state(&bp);
        let mut in_water_fluid = fluid.has_tag(&tag::Fluid::MINECRAFT_WATER);

        if !in_water_fluid {
            let state_here = world.get_block_state(&bp);
            if !state_here.is_solid() {
                let above = BlockPos::new(bp.0.x, bp.0.y + 1, bp.0.z);
                let fluid_above_x = world.get_fluid(&above);
                if fluid_above_x.has_tag(&tag::Fluid::MINECRAFT_WATER) {
                    in_water_fluid = true;
                }
            }
        }

        if !in_water_fluid {
            return false;
        }

        let above = BlockPos::new(bp.0.x, bp.0.y + 1, bp.0.z);
        let fluid_above = world.get_fluid(&above);

        let surface_y = if fluid_above.has_tag(&tag::Fluid::MINECRAFT_WATER) || state.is_still {
            f64::from(bp.0.y as f32 + 1.0)
        } else {
            let lvl = i32::from(state.level);
            let height: f32 = if lvl >= 8 {
                1.0
            } else {
                (8 - lvl).clamp(1, 8) as f32 / 8.0
            };
            f64::from(bp.0.y as f32 + height)
        };

        surface_y > eye_y
    }

    /// `LivingEntity::decreaseAirSupply`/`increaseAirSupply` driving `LivingEntity.baseTick`
    /// (`LivingEntity.java:436-452,588-601`), with `Dolphin.increaseAirSupply` overridden to
    /// restore to full instantly (`Dolphin.java:198-201`) and `Dolphin.handleAirSupply`
    /// (`Dolphin.java:116-117`) a no-op, so the `AgeableWaterCreature`-specific extra drain
    /// (`AgeableWaterCreature.java:37-47`) never runs for Dolphin.
    async fn tick_air_supply(&self) {
        let entity = &self.mob_entity.living_entity.entity;

        if Self::eye_in_breathable_water(entity) {
            if self
                .mob_entity
                .living_entity
                .has_effect(&StatusEffect::WATER_BREATHING)
                .await
            {
                if self.air_supply.load(Relaxed) < MAX_AIR_SUPPLY {
                    self.air_supply.store(MAX_AIR_SUPPLY, Relaxed);
                }
                return;
            }

            let new_air = self.air_supply.fetch_sub(1, Relaxed) - 1;
            if new_air <= DROWN_AIR_THRESHOLD {
                self.air_supply.store(0, Relaxed);
                self.damage_with_context(self, DROWN_DAMAGE, DamageType::DROWN, None, None, None)
                    .await;
            }
        } else if self.air_supply.load(Relaxed) < MAX_AIR_SUPPLY {
            self.air_supply.store(MAX_AIR_SUPPLY, Relaxed);
        }
    }

    #[must_use]
    pub fn got_fish(&self) -> bool {
        self.got_fish.load(Relaxed)
    }

    pub fn set_got_fish(&self, got_fish: bool) {
        self.got_fish.store(got_fish, Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::GOT_FISH,
                MetaDataType::BOOLEAN,
                got_fish,
            )],
            None,
        );
    }

    fn send_moistness(&self, level: i32) {
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::MOISTNESS_LEVEL,
                MetaDataType::INT,
                VarInt(level),
            )],
            None,
        );
    }

    /// `Dolphin.tick`: moistness reset in water/rain, otherwise drained by 1/tick, dealing
    /// `dryOut` damage every tick once depleted; while out of water and grounded, dolphins
    /// flop with a random horizontal impulse and upward hop.
    async fn tick_moistness(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let in_water_or_rain = entity.touching_water.load(Relaxed) || world.is_raining().await;

        if in_water_or_rain {
            if self.moistness_level.load(Relaxed) != TOTAL_MOISTNESS_LEVEL {
                self.moistness_level.store(TOTAL_MOISTNESS_LEVEL, Relaxed);
                self.send_moistness(TOTAL_MOISTNESS_LEVEL);
            }
        } else {
            let new_level = self.moistness_level.fetch_sub(1, Relaxed) - 1;
            self.send_moistness(new_level);
            if new_level <= 0 {
                self.damage_with_context(
                    self,
                    DRY_OUT_DAMAGE,
                    DamageType::DRY_OUT,
                    None,
                    None,
                    None,
                )
                .await;
            }
        }

        if !in_water_or_rain && entity.on_ground.load(Relaxed) {
            let mut rng = rand::rng();
            let velocity = entity.velocity.load();
            entity.velocity.store(
                Vector3::new(
                    f64::from(rng.random_range(-0.2f32..=0.2f32)),
                    0.5,
                    f64::from(rng.random_range(-0.2f32..=0.2f32)),
                ) + Vector3::new(0.0, velocity.y.max(0.0), 0.0),
            );
            entity.yaw.store(rng.random_range(0.0f32..360.0f32));
            entity.on_ground.store(false, Relaxed);
        }
    }
}

impl NBTStorage for DolphinEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            nbt.put_bool("GotFish", self.got_fish());
            nbt.put_int("Moistness", self.moistness_level.load(Relaxed));
            // `Entity.saveWithoutId` (`Entity.java:2090`): `output.putShort("Air",
            // (short)this.getAirSupply())`. Saved here rather than in the generic
            // `LivingEntity`/`Entity` NBT path since no other mob in this codebase tracks air
            // supply yet.
            nbt.put_int("Air", self.air_supply.load(Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.got_fish
                .store(nbt.get_bool("GotFish").unwrap_or(false), Relaxed);
            self.moistness_level.store(
                nbt.get_int("Moistness").unwrap_or(TOTAL_MOISTNESS_LEVEL),
                Relaxed,
            );
            self.air_supply
                .store(nbt.get_int("Air").unwrap_or(MAX_AIR_SUPPLY), Relaxed);
        })
    }
}

impl Mob for DolphinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            self.send_moistness(self.moistness_level.load(Relaxed));
            self.mob_entity.living_entity.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::GOT_FISH,
                    MetaDataType::BOOLEAN,
                    self.got_fish(),
                )],
                None,
            );
        })
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.mob_entity.living_entity.dead.load(Relaxed) {
                return;
            }
            self.tick_moistness().await;
            self.tick_air_supply().await;
        })
    }

    /// `Dolphin.mobInteract`: feeding a tagged fish item plays the eat sound and sets
    /// `gotFish`, consuming one item. Vanilla also ages up a baby dolphin instead of setting
    /// `gotFish` in that case; `DolphinEntity` does not implement `AgeableMob` (no baby/age
    /// state is tracked for Dolphin in this pass, see the report), so that branch is not
    /// ported and every feed currently takes the adult path.
    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if !item_stack.item.has_tag(&tag::Item::MINECRAFT_FISHES) {
                return self
                    .get_mob_entity()
                    .mob_interact(player, item_stack, self.can_be_leashed())
                    .await;
            }

            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();
            world.play_sound(
                Sound::EntityDolphinEat,
                SoundCategory::Neutral,
                &entity.pos.load(),
            );

            self.set_got_fish(true);
            if player.gamemode.load() != pumpkin_util::GameMode::Creative {
                item_stack.decrement(1);
            }
            true
        })
    }
}
