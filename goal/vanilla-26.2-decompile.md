# Vanilla 26.2 Decompile Inventory (ground truth)

Source: official `META-INF/versions/26.2/server-26.2.jar` via CFR 0.152
Output: `/tmp/mc26.2-src` — **637** `.java` files
Method extracts: `/tmp/mc26.2-src/VANILLA_METHODS/`
Index: `/tmp/mc26.2-src/VANILLA_EXTRACT.md`

## Package counts (top)

- `net/minecraft/world/entity/ai`: 277
- `net/minecraft/world/entity/animal`: 130
- `net/minecraft/world/entity/monster`: 84
- `net/minecraft/world/entity/projectile`: 37
- `net/minecraft/world/entity/boss`: 24
- `net/minecraft/world/entity/npc`: 15
- `net/minecraft/world/entity/player`: 14
- `VANILLA_METHODS`: 13
- `net/minecraft/world/entity/raid`: 4
- `net/minecraft/world/damagesource/CombatEntry.java`: 1
- `net/minecraft/world/damagesource/CombatRules.java`: 1
- `net/minecraft/world/damagesource/CombatTracker.java`: 1
- `net/minecraft/world/damagesource/DamageEffects.java`: 1
- `net/minecraft/world/damagesource/DamageScaling.java`: 1
- `net/minecraft/world/damagesource/DamageSource.java`: 1
- `net/minecraft/world/damagesource/DamageSources.java`: 1
- `net/minecraft/world/damagesource/DamageType.java`: 1
- `net/minecraft/world/damagesource/DamageTypes.java`: 1
- `net/minecraft/world/damagesource/DeathMessageType.java`: 1
- `net/minecraft/world/damagesource/FallLocation.java`: 1
- `net/minecraft/world/damagesource/package-info.java`: 1
- `net/minecraft/world/entity/Entity.java`: 1
- `net/minecraft/world/entity/EntityAttachment.java`: 1
- `net/minecraft/world/entity/EntityAttachments.java`: 1
- `net/minecraft/world/entity/EntityDimensions.java`: 1
- `net/minecraft/world/entity/EntityEquipment.java`: 1
- `net/minecraft/world/entity/EntityEvent.java`: 1
- `net/minecraft/world/entity/EntityFluidInteraction.java`: 1
- `net/minecraft/world/entity/EntityProcessor.java`: 1
- `net/minecraft/world/entity/EntityReference.java`: 1
- `net/minecraft/world/entity/EntitySelector.java`: 1
- `net/minecraft/world/entity/EntitySpawnReason.java`: 1
- `net/minecraft/world/entity/EntitySpawnRequest.java`: 1
- `net/minecraft/world/entity/EntityType.java`: 1
- `net/minecraft/world/entity/EntityTypeIds.java`: 1
- `net/minecraft/world/entity/EntityTypes.java`: 1
- `net/minecraft/world/entity/LivingEntity.java`: 1
- `net/minecraft/world/entity/Mob.java`: 1
- `net/minecraft/world/entity/MobCategory.java`: 1
- `net/minecraft/world/entity/NeutralMob.java`: 1

## Critical classes (line counts)

- OK `net/minecraft/world/entity/LivingEntity.java` (3946 lines)
- OK `net/minecraft/world/entity/Mob.java` (1562 lines)
- OK `net/minecraft/world/entity/player/Player.java` (1974 lines)
- OK `net/minecraft/world/entity/animal/golem/IronGolem.java` (359 lines)
- OK `net/minecraft/world/entity/monster/illager/Pillager.java` (294 lines)
- OK `net/minecraft/world/entity/ai/goal/RangedCrossbowAttackGoal.java` (151 lines)
- OK `net/minecraft/world/entity/ai/goal/MeleeAttackGoal.java` (159 lines)
- OK `net/minecraft/world/entity/ai/goal/MoveTowardsTargetGoal.java` (68 lines)
- OK `net/minecraft/world/entity/ai/goal/RangedBowAttackGoal.java` (140 lines)
- OK `net/minecraft/world/entity/ai/control/MoveControl.java` (175 lines)
- OK `net/minecraft/world/entity/ai/control/LookControl.java` (111 lines)
- OK `net/minecraft/world/entity/ai/navigation/PathNavigation.java` (447 lines)
- OK `net/minecraft/world/entity/ai/navigation/GroundPathNavigation.java` (154 lines)
- OK `net/minecraft/world/entity/monster/CrossbowAttackMob.java` (40 lines)
- OK `net/minecraft/world/item/CrossbowItem.java` (311 lines)
- OK `net/minecraft/world/item/BowItem.java` (112 lines)

## Ground-truth facts (from decompiled source, not inference)

### IronGolem (`IronGolem.java`)
- `createAttributes`: MAX_HEALTH 100, MOVEMENT_SPEED 0.25, **KNOCKBACK_RESISTANCE 1.0**, ATTACK_DAMAGE 15, STEP_HEIGHT 1.0
- `registerGoals`: MeleeAttack 1.0 true; MoveTowardsTarget 0.9 / 32; MoveBackToVillage; GolemRandomStroll; OfferFlower; LookAt; RandomLook
- `doHurtTarget`: **no horizontal knockback**; only `deltaMovement += (0, 0.4 * (1-kbRes), 0)` on target; sound always

### LivingEntity.knockback
```
power *= (1.0 - getAttributeValue(KNOCKBACK_RESISTANCE));
if (power <= 0) return;
// normalize xz, scale power, half current horizontal + vertical min(0.4, ...)
```
Also `dealDefaultKnockback` uses base power **0.4f** then same method.

### Player.attack (26.2)
- Damage via `entity.hurtOrSimulate`
- On success: `causeExtraKnockback(entity, getKnockback(...) + (sprint full ? 0.5 : 0), ...)`
- `causeExtraKnockback` → `livingTarget.knockback(...)` which **applies resistance inside**
- Sweep: nearby `knockback(0.4f, ...)` also through LivingEntity.knockback

### Pillager
- `registerGoals`: Float; Avoid Creaking; HoldGroundAttack; **RangedCrossbowAttackGoal(1.0, 8.0f)**; RandomStroll 0.6; LookAt
- Equipment: MAINHAND CROSSBOW
- Attributes: MOVEMENT_SPEED 0.35, MAX_HEALTH 24, ATTACK_DAMAGE 5, FOLLOW_RANGE 32
- Arm pose: CROSSBOW_CHARGE if charging flag; else CROSSBOW_HOLD if holding crossbow

### RangedCrossbowAttackGoal (full state machine — real code)
States: UNCHARGED → CHARGING → CHARGED → READY_TO_ATTACK → UNCHARGED

1. **UNCHARGED**: if in range (`!needsToMove`): `startUsingItem(crossbow hand)` + `setChargingCrossbow(true)` → CHARGING
2. **CHARGING**: wait until `getTicksUsingItem() >= CrossbowItem.getChargeDuration` (base **1.25s = 25 ticks**); then `releaseUsingItem()`, CHARGED, `attackDelay = 20 + random(20)`, `setChargingCrossbow(false)`
3. **CHARGED**: countdown attackDelay → READY_TO_ATTACK
4. **READY_TO_ATTACK** + LOS: `performRangedAttack` → UNCHARGED

Movement:
- `needsToMove` = distance > attackRadius² OR seeTime < 5, AND attackDelay==0
- if needsToMove: `moveTo(target, canRun() ? speed : speed*0.5)` where `canRun()` only when **UNCHARGED**
- else: `navigation.stop()`
- Look at target 30/30 always

**Important correction vs earlier notes:** vanilla **does** call `startUsingItem` for crossbow charge. Use animation is `ItemUseAnimation.CROSSBOW` (not bow). Charging also sets entity data `IS_CHARGING_CROSSBOW` for illager arm pose.

### CrossbowItem.getChargeDuration
```
float duration = EnchantmentHelper.modifyCrossbowChargingTime(crossbow, user, 1.25f);
return floor(duration * 20);  // 25 ticks default
```

## Corrections to prior Pumpkin notes (hallucination audit)

| Prior claim | Vanilla 26.2 fact |
|---|---|
| Crossbow goal does not use startUsingItem | **False** — it does, plus setChargingCrossbow |
| Crossbow charge ~25 ticks silent timer only | Partially true duration, but uses real use-item ticks |
| Iron golem attributes KB=1.0 | **True** (confirmed in createAttributes) |
| Player KB ignores resistance | Was a **Pumpkin bug**; vanilla always multiplies in LivingEntity.knockback |
| MeleeAttack createPath then moveTo | **True** (MeleeAttackGoal) |
| MoveTowardsTarget uses getPosTowards 16/7/π/2 | **True** |

## How to re-decompile
```bash
# extract classes then CFR single files (directory mode incomplete)
java -jar /tmp/cfr.jar path/to/Class.class --outputdir /tmp/mc26.2-src --silent true
```
