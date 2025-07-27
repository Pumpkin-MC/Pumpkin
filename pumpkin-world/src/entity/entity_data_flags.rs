// Entity
pub const DATA_SHARED_FLAGS_ID: u8 = 0;
pub const DATA_AIR_SUPPLY_ID: u8 = 1;
pub const DATA_CUSTOM_NAME: u8 = 2;
pub const DATA_CUSTOM_NAME_VISIBLE: u8 = 3;
pub const DATA_SILENT: u8 = 4;
pub const DATA_NO_GRAVITY: u8 = 5;
pub const DATA_POSE: u8 = 6;
pub const DATA_TICKS_FROZEN: u8 = 7;
// OminousItemSpawner
// DATA_ITEM
pub const DATA_ITEM_OMINOUS_ITEM_SPAWNER: u8 = 8;
// LivingEntity
pub const DATA_LIVING_ENTITY_FLAGS: u8 = 8;
pub const DATA_HEALTH_ID: u8 = 9;
pub const DATA_EFFECT_PARTICLES: u8 = 10;
pub const DATA_EFFECT_AMBIENCE_ID: u8 = 11;
pub const DATA_ARROW_COUNT_ID: u8 = 12;
pub const DATA_STINGER_COUNT_ID: u8 = 13;
pub const SLEEPING_POS_ID: u8 = 14;
// Mob
pub const DATA_MOB_FLAGS_ID: u8 = 15;
// PathfinderMob
// AgeableMob
// DATA_BABY_ID
pub const DATA_BABY_ID_AGEABLE_MOB: u8 = 16;
// Interaction
// DATA_WIDTH_ID
pub const DATA_WIDTH_ID_INTERACTION: u8 = 8;
// DATA_HEIGHT_ID
pub const DATA_HEIGHT_ID_INTERACTION: u8 = 9;
pub const DATA_RESPONSE_ID: u8 = 10;
// Animal
// TamableAnimal
// DATA_FLAGS_ID
pub const DATA_FLAGS_ID_TAMABLE_ANIMAL: u8 = 17;
pub const DATA_OWNERUUID_ID: u8 = 18;
// Display
pub const DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID: u8 = 8;
pub const DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID: u8 = 9;
pub const DATA_POS_ROT_INTERPOLATION_DURATION_ID: u8 = 10;
pub const DATA_TRANSLATION_ID: u8 = 11;
pub const DATA_SCALE_ID: u8 = 12;
pub const DATA_LEFT_ROTATION_ID: u8 = 13;
pub const DATA_RIGHT_ROTATION_ID: u8 = 14;
pub const DATA_BILLBOARD_RENDER_CONSTRAINTS_ID: u8 = 15;
pub const DATA_BRIGHTNESS_OVERRIDE_ID: u8 = 16;
pub const DATA_VIEW_RANGE_ID: u8 = 17;
pub const DATA_SHADOW_RADIUS_ID: u8 = 18;
pub const DATA_SHADOW_STRENGTH_ID: u8 = 19;
// DATA_WIDTH_ID
pub const DATA_WIDTH_ID_DISPLAY: u8 = 20;
// DATA_HEIGHT_ID
pub const DATA_HEIGHT_ID_DISPLAY: u8 = 21;
pub const DATA_GLOW_COLOR_OVERRIDE_ID: u8 = 22;
// Display.BlockDisplay
// DATA_BLOCK_STATE_ID
pub const DATA_BLOCK_STATE_ID_DISPLAY_BLOCK_DISPLAY: u8 = 0;
// Display.ItemDisplay
pub const DATA_ITEM_STACK_ID: u8 = 0;
pub const DATA_ITEM_DISPLAY_ID: u8 = 1;
// Display.TextDisplay
pub const DATA_TEXT_ID: u8 = 0;
pub const DATA_LINE_WIDTH_ID: u8 = 1;
pub const DATA_BACKGROUND_COLOR_ID: u8 = 2;
pub const DATA_TEXT_OPACITY_ID: u8 = 3;
pub const DATA_STYLE_FLAGS_ID: u8 = 4;
// AgeableWaterCreature
// Squid
// GlowSquid
pub const DATA_DARK_TICKS_REMAINING: u8 = 17;
// ExperienceOrb
pub const DATA_VALUE: u8 = 8;
// AreaEffectCloud
pub const DATA_RADIUS: u8 = 8;
pub const DATA_WAITING: u8 = 9;
pub const DATA_PARTICLE: u8 = 10;
// ArmorStand
pub const DATA_CLIENT_FLAGS: u8 = 15;
pub const DATA_HEAD_POSE: u8 = 16;
pub const DATA_BODY_POSE: u8 = 17;
pub const DATA_LEFT_ARM_POSE: u8 = 18;
pub const DATA_RIGHT_ARM_POSE: u8 = 19;
pub const DATA_LEFT_LEG_POSE: u8 = 20;
pub const DATA_RIGHT_LEG_POSE: u8 = 21;
// BlockAttachedEntity
// HangingEntity
// ItemFrame
// DATA_ITEM
pub const DATA_ITEM_ITEM_FRAME: u8 = 8;
pub const DATA_ROTATION: u8 = 9;
// Painting
pub const DATA_PAINTING_VARIANT_ID: u8 = 8;
// VehicleEntity
pub const DATA_ID_HURT: u8 = 8;
pub const DATA_ID_HURTDIR: u8 = 9;
pub const DATA_ID_DAMAGE: u8 = 10;
// AbstractBoat
pub const DATA_ID_PADDLE_LEFT: u8 = 11;
pub const DATA_ID_PADDLE_RIGHT: u8 = 12;
pub const DATA_ID_BUBBLE_TIME: u8 = 13;
// AbstractMinecart
pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: u8 = 11;
pub const DATA_ID_DISPLAY_OFFSET: u8 = 12;
// MinecartFurnace
pub const DATA_ID_FUEL: u8 = 13;
// MinecartCommandBlock
pub const DATA_ID_COMMAND_NAME: u8 = 13;
pub const DATA_ID_LAST_OUTPUT: u8 = 14;
// AmbientCreature
// Bat
// DATA_ID_FLAGS
pub const DATA_ID_FLAGS_BAT: u8 = 16;
// EndCrystal
pub const DATA_BEAM_TARGET: u8 = 8;
pub const DATA_SHOW_BOTTOM: u8 = 9;
// EnderDragon
pub const DATA_PHASE: u8 = 16;
// Monster
// WitherBoss
pub const DATA_TARGET_A: u8 = 16;
pub const DATA_TARGET_B: u8 = 17;
pub const DATA_TARGET_C: u8 = 18;
pub const DATA_ID_INV: u8 = 19;
// Projectile
// FishingHook
pub const DATA_HOOKED_ENTITY: u8 = 8;
pub const DATA_BITING: u8 = 9;
// EyeOfEnder
// DATA_ITEM_STACK
pub const DATA_ITEM_STACK_EYE_OF_ENDER: u8 = 8;
// AbstractArrow
pub const ID_FLAGS: u8 = 8;
pub const PIERCE_LEVEL: u8 = 9;
pub const IN_GROUND: u8 = 10;
// ThrownTrident
pub const ID_LOYALTY: u8 = 11;
pub const ID_FOIL: u8 = 12;
// Arrow
pub const ID_EFFECT_COLOR: u8 = 11;
// AbstractHurtingProjectile
// WitherSkull
pub const DATA_DANGEROUS: u8 = 8;
// Fireball
// DATA_ITEM_STACK
pub const DATA_ITEM_STACK_FIREBALL: u8 = 8;
// FireworkRocketEntity
pub const DATA_ID_FIREWORKS_ITEM: u8 = 8;
pub const DATA_ATTACHED_TO_TARGET: u8 = 9;
pub const DATA_SHOT_AT_ANGLE: u8 = 10;
// ThrowableProjectile
// ThrowableItemProjectile
// DATA_ITEM_STACK
pub const DATA_ITEM_STACK_THROWABLE_ITEM_PROJECTILE: u8 = 8;
// FallingBlockEntity
pub const DATA_START_POS: u8 = 8;
// PrimedTnt
pub const DATA_FUSE_ID: u8 = 8;
// DATA_BLOCK_STATE_ID
pub const DATA_BLOCK_STATE_ID_PRIMED_TNT: u8 = 9;
// ItemEntity
// DATA_ITEM
pub const DATA_ITEM_ITEM_ENTITY: u8 = 8;
// PatrollingMonster
// Raider
pub const IS_CELEBRATING: u8 = 16;
// Rabbit
// DATA_TYPE_ID
pub const DATA_TYPE_ID_RABBIT: u8 = 17;
// ShoulderRidingEntity
// Parrot
// DATA_VARIANT_ID
pub const DATA_VARIANT_ID_PARROT: u8 = 19;
// Turtle
pub const HAS_EGG: u8 = 17;
pub const LAYING_EGG: u8 = 18;
// WaterAnimal
// AbstractFish
// FROM_BUCKET
pub const FROM_BUCKET_ABSTRACT_FISH: u8 = 16;
// Pufferfish
pub const PUFF_STATE: u8 = 17;
// AbstractGolem
// IronGolem
// DATA_FLAGS_ID
pub const DATA_FLAGS_ID_IRON_GOLEM: u8 = 16;
// Bee
// DATA_FLAGS_ID
pub const DATA_FLAGS_ID_BEE: u8 = 17;
// DATA_REMAINING_ANGER_TIME
pub const DATA_REMAINING_ANGER_TIME_BEE: u8 = 18;
// SnowGolem
pub const DATA_PUMPKIN_ID: u8 = 16;
// AbstractCow
// Cow
// DATA_VARIANT_ID
pub const DATA_VARIANT_ID_COW: u8 = 17;
// AbstractSchoolingFish
// Salmon
// DATA_TYPE
pub const DATA_TYPE_SALMON: u8 = 17;
// Cat
// DATA_VARIANT_ID
pub const DATA_VARIANT_ID_CAT: u8 = 19;
pub const IS_LYING: u8 = 20;
pub const RELAX_STATE_ONE: u8 = 21;
// DATA_COLLAR_COLOR
pub const DATA_COLLAR_COLOR_CAT: u8 = 22;
// PolarBear
pub const DATA_STANDING_ID: u8 = 17;
// TropicalFish
// DATA_ID_TYPE_VARIANT
pub const DATA_ID_TYPE_VARIANT_TROPICAL_FISH: u8 = 17;
// Fox
// DATA_TYPE_ID
pub const DATA_TYPE_ID_FOX: u8 = 17;
// DATA_FLAGS_ID
pub const DATA_FLAGS_ID_FOX: u8 = 18;
pub const DATA_TRUSTED_ID_0: u8 = 19;
pub const DATA_TRUSTED_ID_1: u8 = 20;
// Ocelot
pub const DATA_TRUSTING: u8 = 17;
// Chicken
// DATA_VARIANT_ID
pub const DATA_VARIANT_ID_CHICKEN: u8 = 17;
// Panda
pub const UNHAPPY_COUNTER: u8 = 17;
pub const SNEEZE_COUNTER: u8 = 18;
pub const EAT_COUNTER: u8 = 19;
pub const MAIN_GENE_ID: u8 = 20;
pub const HIDDEN_GENE_ID: u8 = 21;
// DATA_ID_FLAGS
pub const DATA_ID_FLAGS_PANDA: u8 = 22;
// MushroomCow
// DATA_TYPE
pub const DATA_TYPE_MUSHROOM_COW: u8 = 17;
// Pig
// DATA_BOOST_TIME
pub const DATA_BOOST_TIME_PIG: u8 = 17;
// DATA_VARIANT_ID
pub const DATA_VARIANT_ID_PIG: u8 = 18;
// Dolphin
pub const GOT_FISH: u8 = 17;
pub const MOISTNESS_LEVEL: u8 = 18;
// Sniffer
pub const DATA_STATE: u8 = 17;
pub const DATA_DROP_SEED_AT_TICK: u8 = 18;
// Allay
pub const DATA_DANCING: u8 = 16;
pub const DATA_CAN_DUPLICATE: u8 = 17;
// Sheep
pub const DATA_WOOL_ID: u8 = 17;
// AbstractHorse
// DATA_ID_FLAGS
pub const DATA_ID_FLAGS_ABSTRACT_HORSE: u8 = 17;
// Camel
pub const DASH: u8 = 18;
pub const LAST_POSE_CHANGE_TICK: u8 = 19;
// Goat
pub const DATA_IS_SCREAMING_GOAT: u8 = 17;
pub const DATA_HAS_LEFT_HORN: u8 = 18;
pub const DATA_HAS_RIGHT_HORN: u8 = 19;
// Wolf
pub const DATA_INTERESTED_ID: u8 = 19;
// DATA_COLLAR_COLOR
pub const DATA_COLLAR_COLOR_WOLF: u8 = 20;
// DATA_REMAINING_ANGER_TIME
pub const DATA_REMAINING_ANGER_TIME_WOLF: u8 = 21;
// DATA_VARIANT_ID
pub const DATA_VARIANT_ID_WOLF: u8 = 22;
pub const DATA_SOUND_VARIANT_ID: u8 = 23;
// Frog
// DATA_VARIANT_ID
pub const DATA_VARIANT_ID_FROG: u8 = 17;
pub const DATA_TONGUE_TARGET_ID: u8 = 18;
// Horse
// DATA_ID_TYPE_VARIANT
pub const DATA_ID_TYPE_VARIANT_HORSE: u8 = 18;
// AbstractChestedHorse
pub const DATA_ID_CHEST: u8 = 18;
// Llama
pub const DATA_STRENGTH_ID: u8 = 19;
// DATA_VARIANT_ID
pub const DATA_VARIANT_ID_LLAMA: u8 = 20;
// Axolotl
pub const DATA_VARIANT: u8 = 17;
pub const DATA_PLAYING_DEAD: u8 = 18;
// FROM_BUCKET
pub const FROM_BUCKET_AXOLOTL: u8 = 19;
// Armadillo
pub const ARMADILLO_STATE: u8 = 17;
// AbstractVillager
pub const DATA_UNHAPPY_COUNTER: u8 = 17;
// Villager
// DATA_VILLAGER_DATA
pub const DATA_VILLAGER_DATA_VILLAGER: u8 = 18;
// Vex
// DATA_FLAGS_ID
pub const DATA_FLAGS_ID_VEX: u8 = 16;
// FlyingMob
// Ghast
pub const DATA_IS_CHARGING: u8 = 16;
// Zoglin
// DATA_BABY_ID
pub const DATA_BABY_ID_ZOGLIN: u8 = 16;
// Zombie
// DATA_BABY_ID
pub const DATA_BABY_ID_ZOMBIE: u8 = 16;
pub const DATA_SPECIAL_TYPE_ID: u8 = 17;
pub const DATA_DROWNED_CONVERSION_ID: u8 = 18;
// Blaze
// DATA_FLAGS_ID
pub const DATA_FLAGS_ID_BLAZE: u8 = 16;
// Guardian
pub const DATA_ID_MOVING: u8 = 16;
pub const DATA_ID_ATTACK_TARGET: u8 = 17;
// Strider
// DATA_BOOST_TIME
pub const DATA_BOOST_TIME_STRIDER: u8 = 17;
pub const DATA_SUFFOCATING: u8 = 18;
// Spider
// DATA_FLAGS_ID
pub const DATA_FLAGS_ID_SPIDER: u8 = 16;
// Phantom
// ID_SIZE
pub const ID_SIZE_PHANTOM: u8 = 16;
// AbstractSkeleton
// Skeleton
pub const DATA_STRAY_CONVERSION_ID: u8 = 16;
// AbstractIllager
// SpellcasterIllager
pub const DATA_SPELL_CASTING_ID: u8 = 17;
// Witch
pub const DATA_USING_ITEM: u8 = 17;
// Bogged
pub const DATA_SHEARED: u8 = 16;
// Slime
// ID_SIZE
pub const ID_SIZE_SLIME: u8 = 16;
// Creeper
pub const DATA_SWELL_DIR: u8 = 16;
pub const DATA_IS_POWERED: u8 = 17;
pub const DATA_IS_IGNITED: u8 = 18;
// EnderMan
pub const DATA_CARRY_STATE: u8 = 16;
pub const DATA_CREEPY: u8 = 17;
pub const DATA_STARED_AT: u8 = 18;
// Pillager
pub const IS_CHARGING_CROSSBOW: u8 = 17;
// ZombieVillager
pub const DATA_CONVERTING_ID: u8 = 19;
// DATA_VILLAGER_DATA
pub const DATA_VILLAGER_DATA_ZOMBIE_VILLAGER: u8 = 20;
// Shulker
pub const DATA_ATTACH_FACE_ID: u8 = 16;
pub const DATA_PEEK_ID: u8 = 17;
pub const DATA_COLOR_ID: u8 = 18;
// Creaking
pub const CAN_MOVE: u8 = 16;
pub const IS_ACTIVE: u8 = 17;
pub const IS_TEARING_DOWN: u8 = 18;
pub const HOME_POS: u8 = 19;
// AbstractPiglin
// DATA_IMMUNE_TO_ZOMBIFICATION
pub const DATA_IMMUNE_TO_ZOMBIFICATION_ABSTRACT_PIGLIN: u8 = 16;
// Piglin
// DATA_BABY_ID
pub const DATA_BABY_ID_PIGLIN: u8 = 17;
pub const DATA_IS_CHARGING_CROSSBOW: u8 = 18;
pub const DATA_IS_DANCING: u8 = 19;
// Hoglin
// DATA_IMMUNE_TO_ZOMBIFICATION
pub const DATA_IMMUNE_TO_ZOMBIFICATION_HOGLIN: u8 = 17;
// Warden
pub const CLIENT_ANGER_LEVEL: u8 = 16;
// Player
pub const DATA_PLAYER_ABSORPTION_ID: u8 = 15;
pub const DATA_SCORE_ID: u8 = 16;
pub const DATA_PLAYER_MODE_CUSTOMISATION: u8 = 17;
pub const DATA_PLAYER_MAIN_HAND: u8 = 18;
pub const DATA_SHOULDER_LEFT: u8 = 19;
pub const DATA_SHOULDER_RIGHT: u8 = 20;
