use pumpkin_registry::{Registry, RegistryBuilder, bootstrap::RegistryEntry, bootstrap_provider};
use pumpkin_util::identifier::Identifier;
use std::sync::Arc;

use crate::structures::StructureSet;

static STRUCTURE_SET_IDENTIFIERS: [Identifier; 20] = [
    Identifier::vanilla_static("ancient_cities"),
    Identifier::vanilla_static("buried_treasures"),
    Identifier::vanilla_static("desert_pyramids"),
    Identifier::vanilla_static("end_cities"),
    Identifier::vanilla_static("igloos"),
    Identifier::vanilla_static("jungle_temples"),
    Identifier::vanilla_static("mineshafts"),
    Identifier::vanilla_static("nether_complexes"),
    Identifier::vanilla_static("nether_fossils"),
    Identifier::vanilla_static("ocean_monuments"),
    Identifier::vanilla_static("ocean_ruins"),
    Identifier::vanilla_static("pillager_outposts"),
    Identifier::vanilla_static("ruined_portals"),
    Identifier::vanilla_static("shipwrecks"),
    Identifier::vanilla_static("strongholds"),
    Identifier::vanilla_static("swamp_huts"),
    Identifier::vanilla_static("trail_ruins"),
    Identifier::vanilla_static("trial_chambers"),
    Identifier::vanilla_static("villages"),
    Identifier::vanilla_static("woodland_mansions"),
];

bootstrap_provider! {
    WORLDGEN_REGISTRY: Arc<dyn Registry> => "minecraft:root",
    || {
        vec![RegistryEntry::new(
            Identifier::vanilla_static("worldgen"),
            RegistryBuilder::<Arc<dyn Registry>>::frozen(
                &Identifier::vanilla_static("worldgen"),
            )
            .unwrap()
            .arc_dyn(),
        )]
    }
}

bootstrap_provider! {
    STRUCTURE_SET_REGISTRY: Arc<dyn Registry> => "minecraft:worldgen",
    || {
        vec![RegistryEntry::new(
            Identifier::vanilla_static("structure_set"),
            RegistryBuilder::<StructureSet>::new_static(
                &Identifier::parse_static("minecraft:worldgen/structure_set"),
                StructureSet::ALL,
                &STRUCTURE_SET_IDENTIFIERS,
            )
            .unwrap()
            .arc_dyn(),
        )]
    }
}
