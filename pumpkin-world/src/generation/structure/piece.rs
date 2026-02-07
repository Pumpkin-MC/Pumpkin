#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StructurePieceType {
    // Mineshaft
    MineshaftCorridor,
    MineshaftCrossing,
    MineshaftRoom,
    MineshaftStairs,

    // Nether Fortress
    NetherFortressBridgeCrossing,
    NetherFortressBridgeEnd,
    NetherFortressBridge,
    NetherFortressCorridorStairs,
    NetherFortressCorridorBalcony,
    NetherFortressCorridorExit,
    NetherFortressCorridorCrossing,
    NetherFortressCorridorLeftTurn,
    NetherFortressSmallCorridor,
    NetherFortressCorridorRightTurn,
    NetherFortressCorridorNetherWartsRoom,
    NetherFortressBridgePlatform,
    NetherFortressBridgeSmallCrossing,
    NetherFortressBridgeStairs,
    NetherFortressStart,

    // Stronghold
    StrongholdChestCorridor,
    StrongholdSmallCorridor,
    StrongholdFiveWayCrossing,
    StrongholdLeftTurn,
    StrongholdLibrary,
    StrongholdPortalRoom,
    StrongholdPrisonHall,
    StrongholdRightTurn,
    StrongholdSquareRoom,
    StrongholdSpiralStaircase,
    StrongholdStart,
    StrongholdCorridor,
    StrongholdStairs,

    // Overworld/General
    JungleTemple,
    OceanTemple,
    Igloo,
    RuinedPortal,
    SwampHut,
    DesertTemple,

    // Ocean Monument
    OceanMonumentBase,
    OceanMonumentCoreRoom,
    OceanMonumentDoubleXRoom,
    OceanMonumentDoubleXYRoom,
    OceanMonumentDoubleYRoom,
    OceanMonumentDoubleYZRoom,
    OceanMonumentDoubleZRoom,
    OceanMonumentEntryRoom,
    OceanMonumentPenthouse,
    OceanMonumentSimpleRoom,
    OceanMonumentSimpleTopRoom,
    OceanMonumentWingRoom,

    // End / Other
    EndCity,
    WoodlandMansion,
    BuriedTreasure,
    Shipwreck,
    NetherFossil,
    Jigsaw,
    OceanRuin,
    PillagerOutpost,

    // Deep Dark / Archaeology
    AncientCity,
    TrailRuins,
    TrialChambers,

    // Nether
    BastionRemnant,
}
