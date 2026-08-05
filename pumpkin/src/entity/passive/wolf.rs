use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering},
};

use pumpkin_data::entity::{EntityStatus, EntityType};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::particle::Particle;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::codec::var_int::VarInt;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::block::entities::sign::DyeColor;
use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, beg::BegGoal, breed::BreedGoal,
        escape_danger::EscapeDangerGoal, follow_owner::FollowOwnerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, non_tame_random_target::NonTameRandomTargetGoal,
        owner_hurt_by_target::OwnerHurtByTargetGoal, owner_hurt_target::OwnerHurtTargetGoal,
        revenge::RevengeGoal, sit::SitGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    living::LivingEntity,
    mob::{Mob, MobEntity},
    persistent_anger::PersistentAnger,
    player::Player,
};
use crate::world::World;

// Vanilla Wolf.java: `private static final DyeColor DEFAULT_COLLAR_COLOR = DyeColor.RED;`
const DEFAULT_COLLAR_COLOR: u8 = DyeColor::Red as u8;

/// Vanilla `Wolf.PREY_SELECTOR` target types (the selector predicate itself is trivial --
/// "is one of these three types" -- so it's folded directly into the type list here rather than
/// ported as a separate closure).
const WOLF_PREY_TYPES: &[&EntityType] =
    &[&EntityType::SHEEP, &EntityType::RABBIT, &EntityType::FOX];

/// Vanilla `AbstractSkeleton` covers `Skeleton`/`Stray`/`WitherSkeleton`; Pumpkin has no shared
/// class hierarchy for these three, so target-selector priority 7 registers one
/// `ActiveTargetGoal` per concrete type below rather than extending `ActiveTargetGoal` to
/// search multiple types at once.
const SKELETON_FAMILY: &[&EntityType] = &[
    &EntityType::SKELETON,
    &EntityType::STRAY,
    &EntityType::WITHER_SKELETON,
];

/// Vanilla Wolf.java `tick()`'s shake-splash particle count:
/// `(int)(Mth.sin((shakeAnim - 0.4F) * PI) * 7.0F)`, clamped to non-negative since a negative
/// Java int loop bound simply skips the loop.
#[must_use]
fn shake_particle_count(shake_anim: f32) -> i32 {
    (((shake_anim - 0.4) * std::f32::consts::PI).sin() * 7.0).max(0.0) as i32
}

pub struct WolfEntity {
    pub mob_entity: MobEntity,
    pub variant: AtomicU8,
    collar_color: AtomicU8,
    pub persistent_anger: PersistentAnger,
    // Vanilla Wolf.java shake/wet animation state (`isWet`/`isShaking`/`shakeAnim`). The
    // render-only companions (`interestedAngle`, `getTailAngle`, `getWetShade`, ...) are
    // deliberately not ported: they're pure client-side lerp values nothing server-side
    // branches on. `shake_anim` stores an `f32` bit pattern (see `entity::attributes` for the
    // same convention with `f64`).
    is_wet: AtomicBool,
    is_shaking: AtomicBool,
    shake_anim: AtomicU32,
}

impl WolfEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let wolf = Self {
            mob_entity,
            variant: AtomicU8::new(3), // Default to pale
            collar_color: AtomicU8::new(DEFAULT_COLLAR_COLOR),
            persistent_anger: PersistentAnger::default(),
            is_wet: AtomicBool::new(false),
            is_shaking: AtomicBool::new(false),
            shake_anim: AtomicU32::new(0),
        };
        let mob_arc = Arc::new(wolf);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, SitGoal::new());
            goal_selector.add_goal(4, EscapeDangerGoal::new(1.5));
            goal_selector.add_goal(5, BreedGoal::new(1.0));
            goal_selector.add_goal(6, FollowOwnerGoal::new(1.0, 10.0, 2.0));
            goal_selector.add_goal(8, Box::new(FollowParentGoal::new(1.1)));
            goal_selector.add_goal(9, BegGoal::new(8.0, &[&Item::BONE]));
            goal_selector.add_goal(
                10,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(10, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(12, Box::new(WanderAroundGoal::new(1.0)));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(1, OwnerHurtByTargetGoal::new());
            target_selector.add_goal(2, OwnerHurtTargetGoal::new());
            target_selector.add_goal(3, Box::new(RevengeGoal::new(true)));

            // Vanilla priority 4: `NearestAttackableTargetGoal<Player>(..., this::isAngryAt)`.
            // The predicate only sees the candidate, not the mob, so it closes over a weak
            // handle back to this wolf to consult its own `PersistentAnger` state.
            let angry_weak = mob_weak.clone();
            target_selector.add_goal(
                4,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::PLAYER,
                    10,
                    true,
                    false,
                    Some(move |target: Arc<LivingEntity>, _world: Arc<World>| {
                        let angry_weak = angry_weak.clone();
                        async move {
                            let Some(mob) = angry_weak.upgrade() else {
                                return false;
                            };
                            let Some(anger) = mob.persistent_anger() else {
                                return false;
                            };
                            anger.is_angry_at(target.entity.entity_uuid).await
                        }
                    }),
                )),
            );

            target_selector.add_goal(
                5,
                NonTameRandomTargetGoal::without_predicate(
                    &mob_arc.mob_entity,
                    WOLF_PREY_TYPES,
                    false,
                ),
            );
            target_selector.add_goal(
                6,
                NonTameRandomTargetGoal::new(
                    &mob_arc.mob_entity,
                    crate::entity::ai::goal::non_tame_random_target::TURTLE_TYPES,
                    false,
                    Some(crate::entity::ai::goal::non_tame_random_target::baby_turtle_on_land),
                ),
            );

            // Vanilla priority 7: one `ActiveTargetGoal` per skeleton-family type (see
            // `SKELETON_FAMILY` above).
            for skeleton_type in SKELETON_FAMILY {
                target_selector.add_goal(
                    7,
                    ActiveTargetGoal::with_default(&mob_arc.mob_entity, skeleton_type, false),
                );
            }

            // Vanilla priority 8, `ResetUniversalAngerTargetGoal(this, true)`, re-targets any
            // nearby player while "universally angry" (a targetless grudge gated behind the
            // `UNIVERSAL_ANGER` game rule) and separately expires the anger timer. Pumpkin has
            // no `UNIVERSAL_ANGER` game rule and `PersistentAnger::tick` already expires the
            // timer on its own, so only the universal-anger-only re-targeting behavior is
            // missing here; deferred until that game rule exists.
        };

        mob_arc
    }

    pub fn set_collar_color(&self, color: u8) {
        self.collar_color.store(color, Ordering::Relaxed);
        self.mob_entity.living_entity.entity.send_meta_data(
            &[Metadata::new(
                TrackedData::COLLAR_COLOR,
                MetaDataType::INT,
                VarInt(i32::from(color)),
            )],
            None,
        );
    }

    /// Ports Wolf.java's `aiStep`/`tick` shake-off-water state machine (server-observable
    /// subset only: sound, splash particles, `ShakeWetness`/`CancelShakeWetness` entity
    /// events). The rain half of vanilla's `isInWaterOrRain()` trigger is intentionally not
    /// ported: `isRainingAt` is a known-broken, unfixed check (see repo CLAUDE.md), so this
    /// only reacts to standing water.
    fn tick_shake_animation(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let touching_water = entity.touching_water.load(Ordering::SeqCst);
        let on_ground = entity.on_ground.load(Ordering::Relaxed);
        let is_pathfinding = !self
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_idle();

        // Vanilla's `aiStep` gates the shake start on the *latched* `isWet` flag (set by the
        // previous tick's `tick()` while in water and surviving until the shake cycle ends),
        // not on being in water right now -- the wolf actually starts shaking after it has
        // already left the water. Read the flag before this tick's `touching_water` update
        // below overwrites it.
        let was_wet = self.is_wet.load(Ordering::Relaxed);
        if was_wet && !self.is_shaking.load(Ordering::Relaxed) && !is_pathfinding && on_ground {
            self.is_shaking.store(true, Ordering::Relaxed);
            self.shake_anim.store(0f32.to_bits(), Ordering::Relaxed);
            world.send_entity_status(entity, EntityStatus::ShakeWetness);
        }

        if touching_water {
            self.is_wet.store(true, Ordering::Relaxed);
            if self.is_shaking.load(Ordering::Relaxed) {
                world.send_entity_status(entity, EntityStatus::CancelShakeWetness);
                self.is_shaking.store(false, Ordering::Relaxed);
                self.shake_anim.store(0f32.to_bits(), Ordering::Relaxed);
            }
            return;
        }

        if !self.is_shaking.load(Ordering::Relaxed) {
            return;
        }

        let shake_anim = f32::from_bits(self.shake_anim.load(Ordering::Relaxed));
        if shake_anim == 0.0 {
            entity.play_sound(Sound::EntityWolfShake);
        }

        let new_shake_anim = shake_anim + 0.05;

        if shake_anim >= 2.0 {
            self.is_wet.store(false, Ordering::Relaxed);
            self.is_shaking.store(false, Ordering::Relaxed);
            self.shake_anim.store(0f32.to_bits(), Ordering::Relaxed);
            return;
        }

        self.shake_anim
            .store(new_shake_anim.to_bits(), Ordering::Relaxed);

        if new_shake_anim > 0.4 {
            let width = entity.entity_dimension.load().width;
            let count = shake_particle_count(new_shake_anim);
            let pos = entity.pos.load();
            world.spawn_particle(
                Vector3::new(pos.x, pos.y + 0.8, pos.z),
                Vector3::new(width * 0.5, 0.0, width * 0.5),
                1.0,
                count,
                Particle::Splash,
            );
        }
    }
}

impl NBTStorage for WolfEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            // Vanilla Wolf.java persists collar color as a legacy dye-color id (0-15).
            nbt.put_byte(
                "CollarColor",
                self.collar_color.load(Ordering::Relaxed) as i8,
            );
            let variant_str = match self.variant.load(Ordering::Relaxed) {
                0 => "minecraft:ashen",
                1 => "minecraft:black",
                2 => "minecraft:chestnut",
                4 => "minecraft:rusty",
                5 => "minecraft:snowy",
                6 => "minecraft:spotted",
                7 => "minecraft:striped",
                8 => "minecraft:woods",
                _ => "minecraft:pale",
            };
            nbt.put_string("variant", variant_str.to_string());
            self.persistent_anger.write_nbt(nbt).await;
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.persistent_anger.read_nbt(nbt).await;
            if let Some(variant_str) = nbt.get_string("variant") {
                let variant = match variant_str
                    .strip_prefix("minecraft:")
                    .unwrap_or(variant_str)
                {
                    "ashen" => 0,
                    "black" => 1,
                    "chestnut" => 2,
                    "rusty" => 4,
                    "snowy" => 5,
                    "spotted" => 6,
                    "striped" => 7,
                    "woods" => 8,
                    _ => 3,
                };
                self.variant.store(variant, Ordering::Relaxed);
            }
            if let Some(color) = nbt.get_byte("CollarColor") {
                self.collar_color.store(color as u8, Ordering::Relaxed);
            }
        })
    }
}

impl Mob for WolfEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.persistent_anger.tick().await;

            // Simplified `NeutralMob::updatePersistentAnger(level, true)`: whenever this wolf
            // currently has a live target (set by e.g. `RevengeGoal`/`OwnerHurtByTargetGoal`),
            // adopt it as the anger target and (re)start the timer. Vanilla additionally
            // re-resolves a persisted target reference across entity reloads and clears anger
            // early for creative/spectator/peaceful targets; Pumpkin's `PersistentAnger::tick`
            // already handles timer expiry on its own.
            let current_target = self.mob_entity.target.lock().await.clone();
            if let Some(target) = current_target {
                let target_uuid = target.get_entity().entity_uuid;
                if !self.persistent_anger.is_angry_at(target_uuid).await {
                    self.persistent_anger.set_angry_at(Some(target_uuid)).await;
                    self.persistent_anger.start_timer();
                }
            }

            self.tick_shake_animation();
        })
    }

    fn persistent_anger(&self) -> Option<&PersistentAnger> {
        Some(&self.persistent_anger)
    }

    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            // Vanilla Wolf.java#mobInteract: a tamed wolf, held item in the collar-dye tag,
            // and owned by the interacting player recolors the collar instead of anything else.
            if self.mob_entity.is_tamed() {
                if tag::Item::MINECRAFT_WOLF_COLLAR_DYES
                    .1
                    .contains(&item_stack.item.id)
                    && self.mob_entity.owner.load() == Some(player.gameprofile.id)
                    && let Some(color_name) = item_stack.item.registry_key.strip_suffix("_dye")
                {
                    let new_color = DyeColor::from(color_name) as u8;
                    if new_color != self.collar_color.load(Ordering::Relaxed) {
                        self.set_collar_color(new_color);
                        item_stack.decrement_unless_creative(player.gamemode.load(), 1);
                        return true;
                    }
                }
                return false;
            }

            if item_stack.item.id != Item::BONE.id {
                return false;
            }

            item_stack.decrement_unless_creative(player.gamemode.load(), 1);

            let entity = &self.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let pos = entity.pos.load() + Vector3::new(0.0, f64::from(entity.height()), 0.0);

            if self.get_random().random_range(0..3) == 0 {
                self.mob_entity.set_owner(player.gameprofile.id);
                self.mob_entity
                    .living_entity
                    .set_health(self.mob_entity.living_entity.get_max_health());
                world.spawn_particle(pos, Vector3::new(0.5, 0.5, 0.5), 1.0, 7, Particle::Heart);
            } else {
                world.spawn_particle(pos, Vector3::new(0.5, 0.5, 0.5), 1.0, 7, Particle::Smoke);
            }

            true
        })
    }

    /// Vanilla `Wolf.wantsToAttack` (Wolf.java:638-648). The player/pvp branch checks only the
    /// server's pvp toggle, not scoreboard team allied-friendly-fire (`Player.canHarmPlayer`,
    /// Player.java:727-735) since Pumpkin has no team-alliance query wired to entities yet.
    /// The tamed-horse branch (`AbstractHorse.isTamed()`) is folded into the generic
    /// "any tamed `Mob`" check below since Pumpkin does not track horse taming separately.
    fn can_attack_with_owner(&self, target: &dyn EntityBase, owner: &dyn EntityBase) -> bool {
        let entity_type = target.get_entity().entity_type;

        if entity_type == &EntityType::CREEPER
            || entity_type == &EntityType::GHAST
            || entity_type == &EntityType::ARMOR_STAND
        {
            return false;
        }

        if entity_type == &EntityType::WOLF {
            let owner_uuid = owner.get_entity().entity_uuid;
            return target.get_mob().is_none_or(|target_mob| {
                !target_mob.get_mob_entity().is_tamed()
                    || target_mob.get_owner_uuid() != Some(owner_uuid)
            });
        }

        if target.get_player().is_some() && owner.get_player().is_some() {
            let world = self.get_entity().world.load_full();
            if let Some(server) = world.server.upgrade()
                && !server.advanced_config.pvp.enabled
            {
                return false;
            }
            return true;
        }

        !target
            .get_mob()
            .is_some_and(|target_mob| target_mob.get_mob_entity().is_tamed())
    }

    fn mob_set_variant_name(&self, name: &str) {
        let variant = match name.strip_prefix("minecraft:").unwrap_or(name) {
            "ashen" => 0,
            "black" => 1,
            "chestnut" => 2,
            "rusty" => 4,
            "snowy" => 5,
            "spotted" => 6,
            "striped" => 7,
            "woods" => 8,
            _ => 3,
        };
        self.variant.store(variant, Ordering::Relaxed);
    }

    fn mob_init_data_tracker(&self) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let entity = self.get_entity();
            let is_baby = entity.age.load(Ordering::Relaxed) < 0;
            if is_baby {
                entity.send_meta_data(
                    &[Metadata::new(
                        TrackedData::BABY_ID,
                        MetaDataType::BOOLEAN,
                        true,
                    )],
                    None,
                );
            }
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::WOLF_VARIANT_ID,
                    MetaDataType::WOLF_VARIANT,
                    VarInt(self.variant.load(Ordering::Relaxed) as i32),
                )],
                None,
            );
            entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::COLLAR_COLOR,
                    MetaDataType::INT,
                    VarInt(i32::from(self.collar_color.load(Ordering::Relaxed))),
                )],
                None,
            );
        })
    }
}

#[cfg(test)]
mod tests {
    use super::shake_particle_count;

    #[test]
    fn shake_particle_count_is_zero_at_threshold() {
        assert_eq!(shake_particle_count(0.4), 0);
    }

    #[test]
    fn shake_particle_count_peaks_near_quarter_cycle() {
        // sin((0.9 - 0.4) * PI) == sin(PI/2) == 1.0, so count should hit the full 7.
        assert_eq!(shake_particle_count(0.9), 7);
    }

    #[test]
    fn shake_particle_count_never_negative() {
        for i in 0..40 {
            let anim = i as f32 * 0.05;
            assert!(shake_particle_count(anim) >= 0);
        }
    }
}
