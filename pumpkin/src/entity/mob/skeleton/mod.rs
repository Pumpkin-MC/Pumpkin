use std::sync::{Arc, Weak};

use pumpkin_data::{
    data_component_impl::{EquipmentSlot, StatusEffectInstance},
    entity::EntityType,
    item::Item,
    item_stack::ItemStack,
};

use crate::entity::{
    Entity, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, flee_sun::FleeSunGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal,
        ranged_bow_attack::RangedBowAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use pumpkin_nbt::compound::NbtCompound;

pub mod bogged;
pub mod parched;
#[allow(clippy::module_inception)]
pub mod skeleton;
pub mod stray;
pub mod wither;

/// `AbstractSkeleton#getHardAttackInterval` (AbstractSkeleton.java). Pumpkin picks the goal
/// once at construction and does not re-run `reassessWeaponGoal`, so the hard-difficulty
/// value is used unconditionally, as it already was before Parched existed.
const SKELETON_ATTACK_INTERVAL: i32 = 20;
/// `Parched#getHardAttackInterval` (Parched.java).
const PARCHED_ATTACK_INTERVAL: i32 = 50;

/// `Bogged#getHardAttackInterval` (Bogged.java).
///
/// Same "hard value used unconditionally" simplification as `SKELETON_ATTACK_INTERVAL`/
/// `PARCHED_ATTACK_INTERVAL` above -- `Bogged#getAttackInterval` (70, the non-hard-difficulty
/// value) is not modeled since `reassessWeaponGoal` is never re-run here.
pub const BOGGED_ATTACK_INTERVAL: i32 = 50;

/// `Parched#getArrow` (Parched.java) attaches `new MobEffectInstance(MobEffects.WEAKNESS, 600)`
/// to every arrow it fires; that constructor defaults to amplifier 0, non-ambient, with
/// particles and icon shown.
const PARCHED_ARROW_EFFECTS: &[StatusEffectInstance] = &[StatusEffectInstance {
    effect_id: std::borrow::Cow::Borrowed("minecraft:weakness"),
    amplifier: 0,
    duration: 600,
    ambient: false,
    show_particles: true,
    show_icon: true,
}];

/// `Bogged#getArrow` (Bogged.java) attaches `new MobEffectInstance(MobEffects.POISON, 100)` to
/// every arrow it fires.
pub const BOGGED_ARROW_EFFECTS: &[StatusEffectInstance] = &[StatusEffectInstance {
    effect_id: std::borrow::Cow::Borrowed("minecraft:poison"),
    amplifier: 0,
    duration: 100,
    ambient: false,
    show_particles: true,
    show_icon: true,
}];

pub struct SkeletonEntityBase {
    pub mob_entity: MobEntity,
}

impl SkeletonEntityBase {
    pub fn new(entity: Entity) -> Arc<Self> {
        let uses_bow = entity.entity_type != &EntityType::WITHER_SKELETON;
        let is_parched = entity.entity_type == &EntityType::PARCHED;
        let is_bogged = entity.entity_type == &EntityType::BOGGED;
        let mob_entity = MobEntity::new(entity);
        let mob = Self { mob_entity };
        let mob_arc = Arc::new(mob);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        {
            // Vanilla `AbstractSkeleton#populateDefaultEquipmentSlots` equips a bow;
            // WitherSkeleton overrides that hook with a stone sword.
            let main_hand = if uses_bow {
                &Item::BOW
            } else {
                &Item::STONE_SWORD
            };
            mob_arc
                .mob_entity
                .living_entity
                .entity_equipment
                .try_lock()
                .expect("new skeleton equipment is uncontended")
                .equipment
                .insert(
                    EquipmentSlot::MAIN_HAND,
                    Arc::new(tokio::sync::Mutex::new(ItemStack::new(1, main_hand))),
                );

            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(3, FleeSunGoal::new(1.0));
            if uses_bow {
                // Vanilla `AbstractSkeleton#reassessWeaponGoal` selects this at priority 4.
                let (interval, arrow_effects) = if is_parched {
                    (PARCHED_ATTACK_INTERVAL, PARCHED_ARROW_EFFECTS)
                } else if is_bogged {
                    (BOGGED_ATTACK_INTERVAL, BOGGED_ARROW_EFFECTS)
                } else {
                    (SKELETON_ATTACK_INTERVAL, &[][..])
                };
                goal_selector.add_goal(
                    4,
                    Box::new(RangedBowAttackGoal::with_arrow_effects(
                        interval,
                        15.0,
                        arrow_effects,
                    )),
                );
            } else {
                goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.2, false)));
            }
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));

            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }
}

impl NBTStorage for SkeletonEntityBase {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        self.mob_entity.living_entity.write_nbt(nbt)
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        self.mob_entity.living_entity.read_nbt_non_mut(nbt)
    }
}

impl Mob for SkeletonEntityBase {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }
}

#[cfg(test)]
mod tests {
    use super::{PARCHED_ARROW_EFFECTS, PARCHED_ATTACK_INTERVAL, SKELETON_ATTACK_INTERVAL};
    use crate::item::potion::PotionContents;
    use pumpkin_data::data_component::DataComponent;
    use pumpkin_data::data_component_impl::{DataComponentImpl, PotionContentsImpl};
    use pumpkin_data::effect::StatusEffect;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;

    #[test]
    fn parched_shoots_slower_than_a_plain_skeleton() {
        assert_eq!(SKELETON_ATTACK_INTERVAL, 20);
        assert_eq!(PARCHED_ATTACK_INTERVAL, 50);
    }

    #[test]
    fn parched_arrows_carry_ten_second_weakness() {
        let mut arrow = ItemStack::new(1, &Item::ARROW);
        arrow.patch.push((
            DataComponent::PotionContents,
            Some(
                PotionContentsImpl {
                    potion_id: None,
                    custom_color: None,
                    custom_effects: PARCHED_ARROW_EFFECTS.to_vec(),
                    custom_name: None,
                }
                .to_dyn(),
            ),
        ));

        // `ArrowEntity::new_shot` stores `item_stack.copy_with_count(1)`, so the patch has
        // to survive that copy for the fired arrow to carry the effect.
        let arrow = arrow.copy_with_count(1);

        let effects = PotionContents::read_potion_effects(&arrow);
        assert_eq!(effects.len(), 1);
        let (effect_type, duration, amplifier, ambient, particles, icon) = effects[0];
        assert_eq!(effect_type.id, StatusEffect::WEAKNESS.id);
        assert_eq!(duration, 600);
        assert_eq!(amplifier, 0);
        assert!(!ambient);
        assert!(particles);
        assert!(icon);
    }
}
