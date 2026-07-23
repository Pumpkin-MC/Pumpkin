use std::sync::{Arc, Weak};

use pumpkin_nbt::compound::NbtCompound;

use crate::entity::{
    Entity, NBTStorage, NbtFuture,
    ai::{
        goal::{
            active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
            look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
            wander_around::WanderAroundGoal,
        },
        pathfinder::node::PathType,
        vanilla_enemy::{IRON_GOLEM_ENEMY_EXCLUDES, IRON_GOLEM_TARGET_CHANCE},
    },
    mob::{Mob, MobEntity},
};
use pumpkin_data::entity::EntityType;

/// Iron golem — protects villagers; attacks hostile monsters (vanilla `Enemy`).
///
/// # Vanilla targeting (`IronGolem` constructor)
/// ```text
/// targetSelector:
///   1 HurtByTarget (alert others)
///   2 NearestAttackableTarget(Mob, 5, false, false,
///       e -> e instanceof Enemy && !(e instanceof Creeper))
/// ```
/// Pumpkin uses [`MobCategory::MONSTER`] as `Enemy` and excludes only creeper.
/// **Warden is a valid target** (monster / Enemy).
///
/// # Attack (`IronGolem.doHurtTarget`)
/// Handled in `MobEntity::try_attack`: arm-raise status 4, random damage,
/// vertical knockback `0.4 * (1 - knockbackResistance)` (warden res=1 → no fling).
///
/// Pathfinding: water is impassable so golems stay on the bank when prey is in water.
///
/// Wiki: <https://minecraft.wiki/w/Iron_Golem>
pub struct IronGolemEntity {
    pub mob_entity: MobEntity,
}

impl IronGolemEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);

        // Never path through water (malus < 0 = blocked in WalkNodeEvaluator).
        {
            let mut nav = mob_entity.navigator.lock().unwrap();
            nav.set_pathfinding_malus(PathType::Water, -1.0);
            nav.set_pathfinding_malus(PathType::WaterBorder, -1.0);
        }

        let iron_golem = Self { mob_entity };
        let mob_arc = Arc::new(iron_golem);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            // pause_when_mob_idle=true: if path fails (e.g. target mid-lake), stop
            // thrashing instead of direct-walking into the water.
            goal_selector.add_goal(1, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new(0.6)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            // Vanilla HurtByTargetGoal(this)
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            // Vanilla Enemy && !Creeper (includes warden, zombies, …)
            target_selector.add_goal(
                2,
                ActiveTargetGoal::for_enemies(
                    &mob_arc.mob_entity,
                    IRON_GOLEM_ENEMY_EXCLUDES,
                    IRON_GOLEM_TARGET_CHANCE,
                    false, // checkVisibility = false in vanilla golem target goal
                ),
            );
        };

        mob_arc
    }
}

impl NBTStorage for IronGolemEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        self.mob_entity.living_entity.write_nbt(nbt)
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        self.mob_entity.living_entity.read_nbt_non_mut(nbt)
    }
}

impl Mob for IronGolemEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}
