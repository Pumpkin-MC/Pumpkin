// Entity
pub static DATA_SHARED_FLAGS_ID: u8 = 0;
pub static DATA_AIR_SUPPLY_ID: u8 = 1;
pub static DATA_CUSTOM_NAME: u8 = 2;
pub static DATA_CUSTOM_NAME_VISIBLE: u8 = 3;
pub static DATA_SILENT: u8 = 4;
pub static DATA_NO_GRAVITY: u8 = 5;
pub static DATA_POSE: u8 = 6;
pub static DATA_TICKS_FROZEN: u8 = 7;
// OminousItemSpawner
// DATA_ITEM
pub static DATA_ITEM_OMINOUS_ITEM_SPAWNER: u8 = 8;
// LivingEntity
pub static DATA_LIVING_ENTITY_FLAGS: u8 = 8;
pub static DATA_HEALTH_ID: u8 = 9;
pub static DATA_EFFECT_PARTICLES: u8 = 10;
pub static DATA_EFFECT_AMBIENCE_ID: u8 = 11;
pub static DATA_ARROW_COUNT_ID: u8 = 12;
pub static DATA_STINGER_COUNT_ID: u8 = 13;
pub static SLEEPING_POS_ID: u8 = 14;
// Mob
pub static DATA_MOB_FLAGS_ID: u8 = 15;
// PathfinderMob
// AgeableMob
// DATA_BABY_ID
pub static DATA_BABY_ID_AGEABLE_MOB: u8 = 16;
// Interaction
// DATA_WIDTH_ID
pub static DATA_WIDTH_ID_INTERACTION: u8 = 8;
// DATA_HEIGHT_ID
pub static DATA_HEIGHT_ID_INTERACTION: u8 = 9;
pub static DATA_RESPONSE_ID: u8 = 10;
// Animal
// TamableAnimal
// DATA_FLAGS_ID
pub static DATA_FLAGS_ID_TAMABLE_ANIMAL: u8 = 17;
pub static DATA_OWNERUUID_ID: u8 = 18;
// Display
pub static DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID: u8 = 8;
pub static DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID: u8 = 9;
pub static DATA_POS_ROT_INTERPOLATION_DURATION_ID: u8 = 10;
pub static DATA_TRANSLATION_ID: u8 = 11;
pub static DATA_SCALE_ID: u8 = 12;
pub static DATA_LEFT_ROTATION_ID: u8 = 13;
pub static DATA_RIGHT_ROTATION_ID: u8 = 14;
pub static DATA_BILLBOARD_RENDER_CONSTRAINTS_ID: u8 = 15;
pub static DATA_BRIGHTNESS_OVERRIDE_ID: u8 = 16;
pub static DATA_VIEW_RANGE_ID: u8 = 17;
pub static DATA_SHADOW_RADIUS_ID: u8 = 18;
pub static DATA_SHADOW_STRENGTH_ID: u8 = 19;
// DATA_WIDTH_ID
pub static DATA_WIDTH_ID_DISPLAY: u8 = 20;
// DATA_HEIGHT_ID
pub static DATA_HEIGHT_ID_DISPLAY: u8 = 21;
pub static DATA_GLOW_COLOR_OVERRIDE_ID: u8 = 22;
// Display.BlockDisplay
// DATA_BLOCK_STATE_ID
pub static DATA_BLOCK_STATE_ID_DISPLAY_BLOCK_DISPLAY: u8 = 0;
// Display.ItemDisplay
pub static DATA_ITEM_STACK_ID: u8 = 0;
pub static DATA_ITEM_DISPLAY_ID: u8 = 1;
// Display.TextDisplay
pub static DATA_TEXT_ID: u8 = 0;
pub static DATA_LINE_WIDTH_ID: u8 = 1;
pub static DATA_BACKGROUND_COLOR_ID: u8 = 2;
pub static DATA_TEXT_OPACITY_ID: u8 = 3;
pub static DATA_STYLE_FLAGS_ID: u8 = 4;
// AgeableWaterCreature
// Squid
// GlowSquid
pub static DATA_DARK_TICKS_REMAINING: u8 = 17;
// ExperienceOrb
pub static DATA_VALUE: u8 = 8;
// AreaEffectCloud
pub static DATA_RADIUS: u8 = 8;
pub static DATA_WAITING: u8 = 9;
pub static DATA_PARTICLE: u8 = 10;
// ArmorStand
pub static DATA_CLIENT_FLAGS: u8 = 15;
pub static DATA_HEAD_POSE: u8 = 16;
pub static DATA_BODY_POSE: u8 = 17;
pub static DATA_LEFT_ARM_POSE: u8 = 18;
pub static DATA_RIGHT_ARM_POSE: u8 = 19;
pub static DATA_LEFT_LEG_POSE: u8 = 20;
pub static DATA_RIGHT_LEG_POSE: u8 = 21;
// BlockAttachedEntity
// HangingEntity
// ItemFrame
// DATA_ITEM
pub static DATA_ITEM_ITEM_FRAME: u8 = 8;
pub static DATA_ROTATION: u8 = 9;
// Painting
pub static DATA_PAINTING_VARIANT_ID: u8 = 8;
// VehicleEntity
pub static DATA_ID_HURT: u8 = 8;
pub static DATA_ID_HURTDIR: u8 = 9;
pub static DATA_ID_DAMAGE: u8 = 10;
// AbstractBoat
pub static DATA_ID_PADDLE_LEFT: u8 = 11;
pub static DATA_ID_PADDLE_RIGHT: u8 = 12;
pub static DATA_ID_BUBBLE_TIME: u8 = 13;
// AbstractMinecart
pub static DATA_ID_CUSTOM_DISPLAY_BLOCK: u8 = 11;
pub static DATA_ID_DISPLAY_OFFSET: u8 = 12;
// MinecartFurnace
pub static DATA_ID_FUEL: u8 = 13;
// MinecartCommandBlock
pub static DATA_ID_COMMAND_NAME: u8 = 13;
pub static DATA_ID_LAST_OUTPUT: u8 = 14;
// AmbientCreature
// Bat
// DATA_ID_FLAGS
pub static DATA_ID_FLAGS_BAT: u8 = 16;
// EndCrystal
pub static DATA_BEAM_TARGET: u8 = 8;
pub static DATA_SHOW_BOTTOM: u8 = 9;
// EnderDragon
pub static DATA_PHASE: u8 = 16;
// Monster
// WitherBoss
pub static DATA_TARGET_A: u8 = 16;
pub static DATA_TARGET_B: u8 = 17;
pub static DATA_TARGET_C: u8 = 18;
pub static DATA_ID_INV: u8 = 19;
// Projectile
// FishingHook
pub static DATA_HOOKED_ENTITY: u8 = 8;
pub static DATA_BITING: u8 = 9;
// EyeOfEnder
// DATA_ITEM_STACK
pub static DATA_ITEM_STACK_EYE_OF_ENDER: u8 = 8;
// AbstractArrow
pub static ID_FLAGS: u8 = 8;
pub static PIERCE_LEVEL: u8 = 9;
pub static IN_GROUND: u8 = 10;
// ThrownTrident
pub static ID_LOYALTY: u8 = 11;
pub static ID_FOIL: u8 = 12;
// Arrow
pub static ID_EFFECT_COLOR: u8 = 11;
// AbstractHurtingProjectile
// WitherSkull
pub static DATA_DANGEROUS: u8 = 8;
// Fireball
// DATA_ITEM_STACK
pub static DATA_ITEM_STACK_FIREBALL: u8 = 8;
// FireworkRocketEntity
pub static DATA_ID_FIREWORKS_ITEM: u8 = 8;
pub static DATA_ATTACHED_TO_TARGET: u8 = 9;
pub static DATA_SHOT_AT_ANGLE: u8 = 10;
// ThrowableProjectile
// ThrowableItemProjectile
// DATA_ITEM_STACK
pub static DATA_ITEM_STACK_THROWABLE_ITEM_PROJECTILE: u8 = 8;
// FallingBlockEntity
pub static DATA_START_POS: u8 = 8;
// PrimedTnt
pub static DATA_FUSE_ID: u8 = 8;
// DATA_BLOCK_STATE_ID
pub static DATA_BLOCK_STATE_ID_PRIMED_TNT: u8 = 9;
// ItemEntity
// DATA_ITEM
pub static DATA_ITEM_ITEM_ENTITY: u8 = 8;
// PatrollingMonster
// Raider
pub static IS_CELEBRATING: u8 = 16;
// Rabbit
// DATA_TYPE_ID
pub static DATA_TYPE_ID_RABBIT: u8 = 17;
// ShoulderRidingEntity
// Parrot
// DATA_VARIANT_ID
pub static DATA_VARIANT_ID_PARROT: u8 = 19;
// Turtle
pub static HAS_EGG: u8 = 17;
pub static LAYING_EGG: u8 = 18;
// WaterAnimal
// AbstractFish
// FROM_BUCKET
pub static FROM_BUCKET_ABSTRACT_FISH: u8 = 16;
// Pufferfish
pub static PUFF_STATE: u8 = 17;
// AbstractGolem
// IronGolem
// DATA_FLAGS_ID
pub static DATA_FLAGS_ID_IRON_GOLEM: u8 = 16;
// Bee
// DATA_FLAGS_ID
pub static DATA_FLAGS_ID_BEE: u8 = 17;
// DATA_REMAINING_ANGER_TIME
pub static DATA_REMAINING_ANGER_TIME_BEE: u8 = 18;
// SnowGolem
pub static DATA_PUMPKIN_ID: u8 = 16;
// AbstractCow
// Cow
// DATA_VARIANT_ID
pub static DATA_VARIANT_ID_COW: u8 = 17;
// AbstractSchoolingFish
// Salmon
// DATA_TYPE
pub static DATA_TYPE_SALMON: u8 = 17;
// Cat
// DATA_VARIANT_ID
pub static DATA_VARIANT_ID_CAT: u8 = 19;
pub static IS_LYING: u8 = 20;
pub static RELAX_STATE_ONE: u8 = 21;
// DATA_COLLAR_COLOR
pub static DATA_COLLAR_COLOR_CAT: u8 = 22;
// PolarBear
pub static DATA_STANDING_ID: u8 = 17;
// TropicalFish
// DATA_ID_TYPE_VARIANT
pub static DATA_ID_TYPE_VARIANT_TROPICAL_FISH: u8 = 17;
// Fox
// DATA_TYPE_ID
pub static DATA_TYPE_ID_FOX: u8 = 17;
// DATA_FLAGS_ID
pub static DATA_FLAGS_ID_FOX: u8 = 18;
pub static DATA_TRUSTED_ID_0: u8 = 19;
pub static DATA_TRUSTED_ID_1: u8 = 20;
// Ocelot
pub static DATA_TRUSTING: u8 = 17;
// Chicken
// DATA_VARIANT_ID
pub static DATA_VARIANT_ID_CHICKEN: u8 = 17;
// Panda
pub static UNHAPPY_COUNTER: u8 = 17;
pub static SNEEZE_COUNTER: u8 = 18;
pub static EAT_COUNTER: u8 = 19;
pub static MAIN_GENE_ID: u8 = 20;
pub static HIDDEN_GENE_ID: u8 = 21;
// DATA_ID_FLAGS
pub static DATA_ID_FLAGS_PANDA: u8 = 22;
// MushroomCow
// DATA_TYPE
pub static DATA_TYPE_MUSHROOM_COW: u8 = 17;
// Pig
// DATA_BOOST_TIME
pub static DATA_BOOST_TIME_PIG: u8 = 17;
// DATA_VARIANT_ID
pub static DATA_VARIANT_ID_PIG: u8 = 18;
// Dolphin
pub static GOT_FISH: u8 = 17;
pub static MOISTNESS_LEVEL: u8 = 18;
// Sniffer
pub static DATA_STATE: u8 = 17;
pub static DATA_DROP_SEED_AT_TICK: u8 = 18;
// Allay
pub static DATA_DANCING: u8 = 16;
pub static DATA_CAN_DUPLICATE: u8 = 17;
// Sheep
pub static DATA_WOOL_ID: u8 = 17;
// AbstractHorse
// DATA_ID_FLAGS
pub static DATA_ID_FLAGS_ABSTRACT_HORSE: u8 = 17;
// Camel
pub static DASH: u8 = 18;
pub static LAST_POSE_CHANGE_TICK: u8 = 19;
// Goat
pub static DATA_IS_SCREAMING_GOAT: u8 = 17;
pub static DATA_HAS_LEFT_HORN: u8 = 18;
pub static DATA_HAS_RIGHT_HORN: u8 = 19;
// Wolf
pub static DATA_INTERESTED_ID: u8 = 19;
// DATA_COLLAR_COLOR
pub static DATA_COLLAR_COLOR_WOLF: u8 = 20;
// DATA_REMAINING_ANGER_TIME
pub static DATA_REMAINING_ANGER_TIME_WOLF: u8 = 21;
// DATA_VARIANT_ID
pub static DATA_VARIANT_ID_WOLF: u8 = 22;
pub static DATA_SOUND_VARIANT_ID: u8 = 23;
// Frog
// DATA_VARIANT_ID
pub static DATA_VARIANT_ID_FROG: u8 = 17;
pub static DATA_TONGUE_TARGET_ID: u8 = 18;
// Horse
// DATA_ID_TYPE_VARIANT
pub static DATA_ID_TYPE_VARIANT_HORSE: u8 = 18;
// AbstractChestedHorse
pub static DATA_ID_CHEST: u8 = 18;
// Llama
pub static DATA_STRENGTH_ID: u8 = 19;
// DATA_VARIANT_ID
pub static DATA_VARIANT_ID_LLAMA: u8 = 20;
// Axolotl
pub static DATA_VARIANT: u8 = 17;
pub static DATA_PLAYING_DEAD: u8 = 18;
// FROM_BUCKET
pub static FROM_BUCKET_AXOLOTL: u8 = 19;
// Armadillo
pub static ARMADILLO_STATE: u8 = 17;
// AbstractVillager
pub static DATA_UNHAPPY_COUNTER: u8 = 17;
// Villager
// DATA_VILLAGER_DATA
pub static DATA_VILLAGER_DATA_VILLAGER: u8 = 18;
// Vex
// DATA_FLAGS_ID
pub static DATA_FLAGS_ID_VEX: u8 = 16;
// FlyingMob
// Ghast
pub static DATA_IS_CHARGING: u8 = 16;
// Zoglin
// DATA_BABY_ID
pub static DATA_BABY_ID_ZOGLIN: u8 = 16;
// Zombie
// DATA_BABY_ID
pub static DATA_BABY_ID_ZOMBIE: u8 = 16;
pub static DATA_SPECIAL_TYPE_ID: u8 = 17;
pub static DATA_DROWNED_CONVERSION_ID: u8 = 18;
// Blaze
// DATA_FLAGS_ID
pub static DATA_FLAGS_ID_BLAZE: u8 = 16;
// Guardian
pub static DATA_ID_MOVING: u8 = 16;
pub static DATA_ID_ATTACK_TARGET: u8 = 17;
// Strider
// DATA_BOOST_TIME
pub static DATA_BOOST_TIME_STRIDER: u8 = 17;
pub static DATA_SUFFOCATING: u8 = 18;
// Spider
// DATA_FLAGS_ID
pub static DATA_FLAGS_ID_SPIDER: u8 = 16;
// Phantom
// ID_SIZE
pub static ID_SIZE_PHANTOM: u8 = 16;
// AbstractSkeleton
// Skeleton
pub static DATA_STRAY_CONVERSION_ID: u8 = 16;
// AbstractIllager
// SpellcasterIllager
pub static DATA_SPELL_CASTING_ID: u8 = 17;
// Witch
pub static DATA_USING_ITEM: u8 = 17;
// Bogged
pub static DATA_SHEARED: u8 = 16;
// Slime
// ID_SIZE
pub static ID_SIZE_SLIME: u8 = 16;
// Creeper
pub static DATA_SWELL_DIR: u8 = 16;
pub static DATA_IS_POWERED: u8 = 17;
pub static DATA_IS_IGNITED: u8 = 18;
// EnderMan
pub static DATA_CARRY_STATE: u8 = 16;
pub static DATA_CREEPY: u8 = 17;
pub static DATA_STARED_AT: u8 = 18;
// Pillager
pub static IS_CHARGING_CROSSBOW: u8 = 17;
// ZombieVillager
pub static DATA_CONVERTING_ID: u8 = 19;
// DATA_VILLAGER_DATA
pub static DATA_VILLAGER_DATA_ZOMBIE_VILLAGER: u8 = 20;
// Shulker
pub static DATA_ATTACH_FACE_ID: u8 = 16;
pub static DATA_PEEK_ID: u8 = 17;
pub static DATA_COLOR_ID: u8 = 18;
// Creaking
pub static CAN_MOVE: u8 = 16;
pub static IS_ACTIVE: u8 = 17;
pub static IS_TEARING_DOWN: u8 = 18;
pub static HOME_POS: u8 = 19;
// AbstractPiglin
// DATA_IMMUNE_TO_ZOMBIFICATION
pub static DATA_IMMUNE_TO_ZOMBIFICATION_ABSTRACT_PIGLIN: u8 = 16;
// Piglin
// DATA_BABY_ID
pub static DATA_BABY_ID_PIGLIN: u8 = 17;
pub static DATA_IS_CHARGING_CROSSBOW: u8 = 18;
pub static DATA_IS_DANCING: u8 = 19;
// Hoglin
// DATA_IMMUNE_TO_ZOMBIFICATION
pub static DATA_IMMUNE_TO_ZOMBIFICATION_HOGLIN: u8 = 17;
// Warden
pub static CLIENT_ANGER_LEVEL: u8 = 16;
// Player
pub static DATA_PLAYER_ABSORPTION_ID: u8 = 15;
pub static DATA_SCORE_ID: u8 = 16;
pub static DATA_PLAYER_MODE_CUSTOMISATION: u8 = 17;
pub static DATA_PLAYER_MAIN_HAND: u8 = 18;
pub static DATA_SHOULDER_LEFT: u8 = 19;
pub static DATA_SHOULDER_RIGHT: u8 = 20;
