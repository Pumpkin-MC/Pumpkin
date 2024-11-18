// TODO
#[derive(Clone, Copy)]
#[repr(i32)]
pub enum EntityType {
    AcaciaBoat = 0,
    AcaciaChestBoat = 1,
    Allay = 2,
    AreaEffectCloud = 3,
    Armadillo = 4,
    ArmorStand = 5,
    Arrow = 6,
    Axolotl = 7,
    BambooChestRaft = 8,
    BambooRaft = 9,
    Bat = 10,
    Bee = 11,
    BirchBoat = 12,
    BirchChestBoat = 13,
    Blaze = 14,
    BlockDisplay = 15,
    Bogged = 16,
    Breeze = 17,
    BreezeWindCharge = 18,
    Camel = 19,
    Cat = 20,
    CaveSpider = 21,
    CherryBoat = 22,
    CherryChestBoat = 23,
    ChestMinecart = 24,
    Chicken = 25,
    Cod = 26,
    CommandBlockMinecart = 27,
    Cow = 28,
    Creaking = 29,
    CreakingTransient = 30,
    Creeper = 31,
    DarkOakBoat = 32,
    DarkOakChestBoat = 33,
    Dolphin = 34,
    Donkey = 35,
    DragonFireball = 36,
    Drowned = 37,
    Egg = 38,
    ElderGuardian = 39,
    Enderman = 40,
    Endermite = 41,
    EnderDragon = 42,
    EnderPearl = 43,
    EndCrystal = 44,
    Evoker = 45,
    EvokerFangs = 46,
    ExperienceBottle = 47,
    ExperienceOrb = 48,
    EyeOfEnder = 49,
    FallingBlock = 50,
    Fireball = 51,
    FireworkRocket = 52,
    Fox = 53,
    Frog = 54,
    FurnaceMinecart = 55,
    Ghast = 56,
    Giant = 57,
    GlowItemFrame = 58,
    GlowSquid = 59,
    Goat = 60,
    Guardian = 61,
    Hoglin = 62,
    HopperMinecart = 63,
    Horse = 64,
    Husk = 65,
    Illusioner = 66,
    Interaction = 67,
    IronGolem = 68,
    Item = 69,
    ItemDisplay = 70,
    ItemFrame = 71,
    JungleBoat = 72,
    JungleChestBoat = 73,
    LeashKnot = 74,
    LightningBolt = 75,
    Llama = 76,
    LlamaSpit = 77,
    MagmaCube = 78,
    MangroveBoat = 79,
    MangroveChestBoat = 80,
    Marker = 81,
    Minecart = 82,
    Mooshroom = 83,
    Mule = 84,
    OakBoat = 85,
    OakChestBoat = 86,
    Ocelot = 87,
    OminousItemSpawner = 88,
    Painting = 89,
    PaleOakBoat = 90,
    PaleOakChestBoat = 91,
    Panda = 92,
    Parrot = 93,
    Phantom = 94,
    Pig = 95,
    Piglin = 96,
    PiglinBrute = 97,
    Pillager = 98,
    PolarBear = 99,
    Potion = 100,
    Pufferfish = 101,
    Rabbit = 102,
    Ravager = 103,
    Salmon = 104,
    Sheep = 105,
    Shulker = 106,
    ShulkerBullet = 107,
    Silverfish = 108,
    Skeleton = 109,
    SkeletonHorse = 110,
    Slime = 111,
    SmallFireball = 112,
    Sniffer = 113,
    Snowball = 114,
    SnowGolem = 115,
    SpawnerMinecart = 116,
    SpectralArrow = 117,
    Spider = 118,
    SpruceBoat = 119,
    SpruceChestBoat = 120,
    Squid = 121,
    Stray = 122,
    Strider = 123,
    Tadpole = 124,
    TextDisplay = 125,
    Tnt = 126,
    TntMinecart = 127,
    TraderLlama = 128,
    Trident = 129,
    TropicalFish = 130,
    Turtle = 131,
    Vex = 132,
    Villager = 133,
    Vindicator = 134,
    WanderingTrader = 135,
    Warden = 136,
    WindCharge = 137,
    Witch = 138,
    Wither = 139,
    WitherSkeleton = 140,
    WitherSkull = 141,
    Wolf = 142,
    Zoglin = 143,
    Zombie = 144,
    ZombieHorse = 145,
    ZombieVillager = 146,
    ZombifiedPiglin = 147,
    Player = 148,
    FishingBobber = 149,
}

impl EntityType {
    pub const fn gravity(&self) -> f64 {
        use EntityType::*;
        match self {
            Item => 0.04,
            _ => todo!(),
        }
    }
}