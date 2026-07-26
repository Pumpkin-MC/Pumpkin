//! Structural checks that GoalSelector registrations match decompiled
//! MC 26.2 `registerGoals` (CFR from `server-26.2.jar`).
//!
//! These assert shipped source structure (not re-implement AI). Failures mean
//! invented goals crept back in.

#[cfg(test)]
mod tests {
    #[test]
    fn fox_matches_vanilla_register_goals_shape() {
        // Decompile Fox.java: Avoid Player/Wolf/PolarBear; land prey; no TemptGoal.
        let src = include_str!("../passive/fox.rs");
        assert!(
            !src.contains("tempt::TemptGoal") && !src.contains("TemptGoal::"),
            "vanilla Fox has no TemptGoal in registerGoals"
        );
        assert!(
            src.contains("EntityType::PLAYER") && src.contains("AvoidEntityGoal"),
            "fox must avoid players (trust filter stand-in)"
        );
        assert!(src.contains("EntityType::WOLF"));
        assert!(src.contains("EntityType::POLAR_BEAR"));
        assert!(src.contains("EntityType::CHICKEN"));
        assert!(src.contains("EntityType::RABBIT"));
        assert!(src.contains("LeapAtTargetGoal"));
    }

    #[test]
    fn sheep_has_no_predator_avoid() {
        let src = include_str!("../passive/sheep.rs");
        assert!(
            !src.contains("AvoidEntityGoal"),
            "sheep: vanilla has no predator AvoidEntity"
        );
        assert!(
            src.contains("EscapeDangerGoal"),
            "sheep: vanilla PanicGoal stand-in required"
        );
    }

    #[test]
    fn chicken_has_no_predator_avoid() {
        let src = include_str!("../passive/chicken.rs");
        assert!(
            !src.contains("AvoidEntityGoal"),
            "chicken: vanilla has no predator AvoidEntity"
        );
        assert!(
            src.contains("EscapeDangerGoal"),
            "chicken: vanilla PanicGoal stand-in required"
        );
    }

    #[test]
    fn vex_targets_player_only_not_villager_golem() {
        // Decompile Vex.java: NearestAttackableTarget(Player) only.
        let src = include_str!("../mob/vex.rs");
        assert!(src.contains("EntityType::PLAYER"));
        assert!(
            !src.contains("EntityType::VILLAGER"),
            "vex must not NearestAttackableTarget villagers"
        );
        assert!(
            !src.contains("EntityType::IRON_GOLEM"),
            "vex must not NearestAttackableTarget iron golems"
        );
    }

    #[test]
    fn iron_golem_has_no_float_goal() {
        // Decompile IronGolem.java: no FloatGoal; decreaseAirSupply no-op.
        let src = include_str!("../passive/iron_golem.rs");
        assert!(
            !src.contains("SwimGoal"),
            "iron golem must not register Float/SwimGoal"
        );
    }

    #[test]
    fn dolphin_avoids_guardians_does_not_hunt_them() {
        let src = include_str!("../passive/dolphin.rs");
        assert!(src.contains("AvoidEntityGoal") && src.contains("GUARDIAN"));
        assert!(
            !src.contains(
                "ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::GUARDIAN"
            ),
            "dolphin must not hunt guardians"
        );
    }

    #[test]
    fn dolphin_swims_with_nearby_swimming_players() {
        let src = include_str!("../passive/dolphin.rs");
        assert!(src.contains("DolphinSwimWithPlayerGoal::new(4.0)"));
        assert!(
            !src.contains("fn mob_tick"),
            "Dolphin's Grace must be owned by the swim-with-player goal"
        );
    }

    #[test]
    fn piglin_brute_does_not_flee_zombified() {
        // ZOMBIFIED_PIGLIN may appear as the zombification target; only the
        // avoid goal would be un-vanilla.
        let src = include_str!("../mob/piglin_brute.rs");
        assert!(
            !src.contains("AvoidEntityGoal::new(&EntityType::ZOMBIFIED_PIGLIN"),
            "vanilla piglin brutes do not flee zombified piglins"
        );
    }

    #[test]
    fn cod_avoids_player_not_axolotl() {
        let src = include_str!("../passive/cod.rs");
        assert!(
            src.contains("EntityType::PLAYER") && src.contains("AvoidEntityGoal"),
            "AbstractFish avoids players"
        );
        assert!(
            !src.contains("EntityType::AXOLOTL"),
            "AbstractFish does not AvoidEntity axolotl"
        );
    }

    #[test]
    fn salmon_avoids_player_not_axolotl() {
        let src = include_str!("../passive/salmon.rs");
        assert!(src.contains("EntityType::PLAYER") && src.contains("AvoidEntityGoal"));
        assert!(!src.contains("EntityType::AXOLOTL"));
    }

    #[test]
    fn tropical_fish_avoids_player_not_axolotl() {
        let src = include_str!("../passive/tropical_fish.rs");
        assert!(src.contains("EntityType::PLAYER") && src.contains("AvoidEntityGoal"));
        assert!(!src.contains("EntityType::AXOLOTL"));
    }

    #[test]
    fn bee_has_breed_tempt_and_pack_anger() {
        // Decompile Bee.java: Breed + Tempt(BEE_FOOD) + HurtBy.setAlertOthers.
        let src = include_str!("../passive/bee.rs");
        assert!(src.contains("BreedGoal"));
        assert!(src.contains("TemptGoal") && src.contains("DANDELION"));
        assert!(src.contains("JoinAngerGoal") && src.contains("BEE"));
        assert!(src.contains("MeleeAttackGoal"));
    }

    #[test]
    fn strider_no_zombified_flee_looks_at_strider() {
        let src = include_str!("../passive/strider.rs");
        assert!(!src.contains("ZOMBIFIED_PIGLIN"));
        assert!(src.contains("EntityType::STRIDER"));
    }

    #[test]
    fn shulker_pack_anger_and_player_target() {
        let src = include_str!("../mob/shulker.rs");
        assert!(src.contains("JoinAngerGoal") && src.contains("SHULKER"));
        assert!(src.contains("EntityType::PLAYER"));
    }
}
