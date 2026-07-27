//! The point-of-interest type table, mirroring vanilla
//! `PoiTypes.bootstrap` (`/root/Vanilla/src/net/minecraft/world/entity/ai/village/poi/PoiTypes.java:90-112`).
//!
//! Each entry carries the two numbers vanilla's `PoiType` record holds
//! (`PoiType.java:11`): `maxTickets` — how many entities may claim the POI at
//! once — and `validRange` — how close an entity must path to count as "at" it.
//!
//! The tag membership tables mirror the vanilla datapack tags:
//! - `#minecraft:village` — `/root/Vanilla/resources/data/minecraft/tags/point_of_interest_type/village.json`
//! - `#minecraft:acquirable_job_site` — `.../acquirable_job_site.json`
//! - `#minecraft:bee_home` — `.../bee_home.json`

use pumpkin_data::block_properties::{BedPart, BlockProperties, WhiteBedLikeProperties};
use pumpkin_data::tag::Taggable;
use pumpkin_data::{Block, BlockStateId};

/// A registered point-of-interest type — vanilla `PoiType` (`PoiType.java:11`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoiType {
    /// Registry name, e.g. `minecraft:home`.
    pub name: &'static str,
    /// Vanilla `PoiType.maxTickets` — the number of simultaneous claims allowed.
    pub max_tickets: i32,
    /// Vanilla `PoiType.validRange` — pathfinding range that counts as reaching it.
    pub valid_range: i32,
    /// Member of `#minecraft:acquirable_job_site`.
    pub acquirable_job_site: bool,
    /// Member of `#minecraft:village`.
    pub village: bool,
    /// Member of `#minecraft:bee_home`.
    pub bee_home: bool,
}

impl PoiType {
    const fn job_site(name: &'static str) -> Self {
        // `PoiTypes.java:91-103`: every profession site is registered with
        // maxTickets = 1, validRange = 1. All 13 are in
        // `#minecraft:acquirable_job_site`, which `#minecraft:village` includes.
        Self {
            name,
            max_tickets: 1,
            valid_range: 1,
            acquirable_job_site: true,
            village: true,
            bee_home: false,
        }
    }

    /// A type nothing can claim (`maxTickets = 0`) and that no tag contains.
    const fn untagged(name: &'static str, max_tickets: i32, valid_range: i32) -> Self {
        Self {
            name,
            max_tickets,
            valid_range,
            acquirable_job_site: false,
            village: false,
            bee_home: false,
        }
    }
}

// `PoiTypes.java:91-103` — the thirteen villager workstation types.
pub static ARMORER: PoiType = PoiType::job_site("minecraft:armorer");
pub static BUTCHER: PoiType = PoiType::job_site("minecraft:butcher");
pub static CARTOGRAPHER: PoiType = PoiType::job_site("minecraft:cartographer");
pub static CLERIC: PoiType = PoiType::job_site("minecraft:cleric");
pub static FARMER: PoiType = PoiType::job_site("minecraft:farmer");
pub static FISHERMAN: PoiType = PoiType::job_site("minecraft:fisherman");
pub static FLETCHER: PoiType = PoiType::job_site("minecraft:fletcher");
pub static LEATHERWORKER: PoiType = PoiType::job_site("minecraft:leatherworker");
pub static LIBRARIAN: PoiType = PoiType::job_site("minecraft:librarian");
pub static MASON: PoiType = PoiType::job_site("minecraft:mason");
pub static SHEPHERD: PoiType = PoiType::job_site("minecraft:shepherd");
pub static TOOLSMITH: PoiType = PoiType::job_site("minecraft:toolsmith");
pub static WEAPONSMITH: PoiType = PoiType::job_site("minecraft:weaponsmith");

/// `PoiTypes.java:104` — `register(registry, HOME, BEDS, 1, 1)`. `BEDS` is the
/// head half of every bed block state (`PoiTypes.java:53`).
pub static HOME: PoiType = PoiType {
    name: "minecraft:home",
    max_tickets: 1,
    valid_range: 1,
    acquirable_job_site: false,
    // `village.json` lists `minecraft:home` directly.
    village: true,
    bee_home: false,
};

/// `PoiTypes.java:105` — `register(registry, MEETING, BELL, 32, 6)`.
pub static MEETING: PoiType = PoiType {
    name: "minecraft:meeting",
    max_tickets: 32,
    valid_range: 6,
    acquirable_job_site: false,
    // `village.json` lists `minecraft:meeting` directly.
    village: true,
    bee_home: false,
};

/// `PoiTypes.java:106` — `register(registry, BEEHIVE, BEEHIVE, 0, 1)`.
pub static BEEHIVE: PoiType = PoiType {
    name: "minecraft:beehive",
    max_tickets: 0,
    valid_range: 1,
    acquirable_job_site: false,
    village: false,
    bee_home: true,
};

/// `PoiTypes.java:107` — `register(registry, BEE_NEST, BEE_NEST, 0, 1)`.
pub static BEE_NEST: PoiType = PoiType {
    name: "minecraft:bee_nest",
    max_tickets: 0,
    valid_range: 1,
    acquirable_job_site: false,
    village: false,
    bee_home: true,
};

/// `PoiTypes.java:108` — `register(registry, NETHER_PORTAL, NETHER_PORTAL, 0, 1)`.
pub static NETHER_PORTAL: PoiType = PoiType::untagged("minecraft:nether_portal", 0, 1);
/// `PoiTypes.java:109` — `register(registry, LODESTONE, LODESTONE, 0, 1)`.
pub static LODESTONE: PoiType = PoiType::untagged("minecraft:lodestone", 0, 1);
/// `PoiTypes.java:110` — `register(registry, TEST_INSTANCE, TEST_INSTANCE_BLOCK, 0, 1)`.
pub static TEST_INSTANCE: PoiType = PoiType::untagged("minecraft:test_instance", 0, 1);
/// `PoiTypes.java:111` — `register(registry, LIGHTNING_ROD, LIGHTNING_RODS, 0, 1)`.
pub static LIGHTNING_ROD: PoiType = PoiType::untagged("minecraft:lightning_rod", 0, 1);

/// Every registered type, in `PoiTypes.bootstrap` order.
pub static ALL: &[&PoiType] = &[
    &ARMORER,
    &BUTCHER,
    &CARTOGRAPHER,
    &CLERIC,
    &FARMER,
    &FISHERMAN,
    &FLETCHER,
    &LEATHERWORKER,
    &LIBRARIAN,
    &MASON,
    &SHEPHERD,
    &TOOLSMITH,
    &WEAPONSMITH,
    &HOME,
    &MEETING,
    &BEEHIVE,
    &BEE_NEST,
    &NETHER_PORTAL,
    &LODESTONE,
    &TEST_INSTANCE,
    &LIGHTNING_ROD,
];

/// Look a type up by its registry name. Unknown names (from an older save, or a
/// type Pumpkin has not implemented) return `None`.
#[must_use]
pub fn by_name(name: &str) -> Option<&'static PoiType> {
    ALL.iter().copied().find(|t| t.name == name)
}

/// Vanilla `PoiTypes.forState` (`PoiTypes.java:82-84`): the type registered for
/// this block state, or `None` when the state is not a POI.
///
/// Vanilla keys the lookup on `BlockState`; Pumpkin keys it on `(block, state_id)`
/// because only beds distinguish states, via `BedBlock.PART == HEAD`
/// (`PoiTypes.java:53`).
#[must_use]
#[expect(
    clippy::cognitive_complexity,
    reason = "flat mirror of the PoiTypes.bootstrap table; splitting it would obscure the 1:1 mapping"
)]
pub fn for_state(block: &Block, state_id: BlockStateId) -> Option<&'static PoiType> {
    // `PoiTypes.java:53` — only the HEAD half of a bed is a HOME POI.
    if block.has_tag(&pumpkin_data::tag::Block::MINECRAFT_BEDS) {
        let props = WhiteBedLikeProperties::from_state_id(state_id, block);
        return if props.part == BedPart::Head {
            Some(&HOME)
        } else {
            None
        };
    }

    // `PoiTypes.java:91-103`, in registration order.
    if block == &Block::BLAST_FURNACE {
        Some(&ARMORER)
    } else if block == &Block::SMOKER {
        Some(&BUTCHER)
    } else if block == &Block::CARTOGRAPHY_TABLE {
        Some(&CARTOGRAPHER)
    } else if block == &Block::BREWING_STAND {
        Some(&CLERIC)
    } else if block == &Block::COMPOSTER {
        Some(&FARMER)
    } else if block == &Block::BARREL {
        Some(&FISHERMAN)
    } else if block == &Block::FLETCHING_TABLE {
        Some(&FLETCHER)
    // `PoiTypes.java:54` — CAULDRONS covers all four cauldron blocks.
    } else if block == &Block::CAULDRON
        || block == &Block::LAVA_CAULDRON
        || block == &Block::WATER_CAULDRON
        || block == &Block::POWDER_SNOW_CAULDRON
    {
        Some(&LEATHERWORKER)
    } else if block == &Block::LECTERN {
        Some(&LIBRARIAN)
    } else if block == &Block::STONECUTTER {
        Some(&MASON)
    } else if block == &Block::LOOM {
        Some(&SHEPHERD)
    } else if block == &Block::SMITHING_TABLE {
        Some(&TOOLSMITH)
    } else if block == &Block::GRINDSTONE {
        Some(&WEAPONSMITH)
    } else if block == &Block::BELL {
        Some(&MEETING)
    } else if block == &Block::BEEHIVE {
        Some(&BEEHIVE)
    } else if block == &Block::BEE_NEST {
        Some(&BEE_NEST)
    } else if block == &Block::NETHER_PORTAL {
        Some(&NETHER_PORTAL)
    } else if block == &Block::LODESTONE {
        Some(&LODESTONE)
    } else if block == &Block::TEST_INSTANCE_BLOCK {
        Some(&TEST_INSTANCE)
    } else if block == &Block::LIGHTNING_ROD {
        Some(&LIGHTNING_ROD)
    } else {
        None
    }
}

/// Vanilla `PoiTypes.hasPoi` (`PoiTypes.java:86-88`).
#[must_use]
pub fn has_poi(block: &Block, state_id: BlockStateId) -> bool {
    for_state(block, state_id).is_some()
}

/// `maxTickets` for a stored record whose type name may be unknown.
///
/// Unknown types get 0, which makes them permanently unclaimable rather than
/// silently claimable — the conservative reading of `PoiRecord.acquireTicket`
/// (`PoiRecord.java:51-58`).
#[must_use]
pub fn max_tickets_of(name: &str) -> i32 {
    by_name(name).map_or(0, |t| t.max_tickets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticket_and_range_numbers_match_poi_types_bootstrap() {
        // PoiTypes.java:91-103
        for job in [
            &ARMORER,
            &BUTCHER,
            &CARTOGRAPHER,
            &CLERIC,
            &FARMER,
            &FISHERMAN,
            &FLETCHER,
            &LEATHERWORKER,
            &LIBRARIAN,
            &MASON,
            &SHEPHERD,
            &TOOLSMITH,
            &WEAPONSMITH,
        ] {
            assert_eq!(job.max_tickets, 1, "{}", job.name);
            assert_eq!(job.valid_range, 1, "{}", job.name);
        }
        // PoiTypes.java:104
        assert_eq!(HOME.max_tickets, 1);
        assert_eq!(HOME.valid_range, 1);
        // PoiTypes.java:105
        assert_eq!(MEETING.max_tickets, 32);
        assert_eq!(MEETING.valid_range, 6);
        // PoiTypes.java:106-111
        for zero in [
            &BEEHIVE,
            &BEE_NEST,
            &NETHER_PORTAL,
            &LODESTONE,
            &TEST_INSTANCE,
            &LIGHTNING_ROD,
        ] {
            assert_eq!(zero.max_tickets, 0, "{}", zero.name);
            assert_eq!(zero.valid_range, 1, "{}", zero.name);
        }
    }

    #[test]
    fn village_tag_matches_the_datapack() {
        // village.json = #acquirable_job_site + home + meeting
        let village: Vec<&str> = ALL.iter().filter(|t| t.village).map(|t| t.name).collect();
        assert_eq!(village.len(), 15);
        assert!(village.contains(&"minecraft:home"));
        assert!(village.contains(&"minecraft:meeting"));
        assert!(village.contains(&"minecraft:farmer"));
        assert!(!village.contains(&"minecraft:nether_portal"));
        assert!(!village.contains(&"minecraft:beehive"));
    }

    #[test]
    fn acquirable_job_site_tag_has_the_thirteen_professions() {
        assert_eq!(ALL.iter().filter(|t| t.acquirable_job_site).count(), 13);
        assert!(!HOME.acquirable_job_site);
        assert!(!MEETING.acquirable_job_site);
    }

    #[test]
    fn bee_home_tag_has_hive_and_nest() {
        let bee: Vec<&str> = ALL.iter().filter(|t| t.bee_home).map(|t| t.name).collect();
        assert_eq!(bee, vec!["minecraft:beehive", "minecraft:bee_nest"]);
    }

    #[test]
    fn workstation_states_resolve_to_their_profession_type() {
        let cases: [(&Block, &PoiType); 16] = [
            (&Block::BLAST_FURNACE, &ARMORER),
            (&Block::SMOKER, &BUTCHER),
            (&Block::CARTOGRAPHY_TABLE, &CARTOGRAPHER),
            (&Block::BREWING_STAND, &CLERIC),
            (&Block::COMPOSTER, &FARMER),
            (&Block::BARREL, &FISHERMAN),
            (&Block::FLETCHING_TABLE, &FLETCHER),
            (&Block::CAULDRON, &LEATHERWORKER),
            (&Block::WATER_CAULDRON, &LEATHERWORKER),
            (&Block::LAVA_CAULDRON, &LEATHERWORKER),
            (&Block::POWDER_SNOW_CAULDRON, &LEATHERWORKER),
            (&Block::LECTERN, &LIBRARIAN),
            (&Block::STONECUTTER, &MASON),
            (&Block::LOOM, &SHEPHERD),
            (&Block::SMITHING_TABLE, &TOOLSMITH),
            (&Block::GRINDSTONE, &WEAPONSMITH),
        ];
        for (block, expected) in cases {
            let found = for_state(block, block.default_state.id)
                .unwrap_or_else(|| panic!("{} should be a POI", block.name));
            assert_eq!(found.name, expected.name, "{}", block.name);
        }
    }

    #[test]
    fn only_the_bed_head_is_a_home_poi() {
        let block = &Block::WHITE_BED;
        let mut heads = 0;
        let mut feet = 0;
        for state in block.states {
            let props = WhiteBedLikeProperties::from_state_id(state.id, block);
            match for_state(block, state.id) {
                Some(t) => {
                    assert_eq!(t.name, "minecraft:home");
                    assert_eq!(props.part, BedPart::Head);
                    heads += 1;
                }
                None => {
                    assert_eq!(props.part, BedPart::Foot);
                    feet += 1;
                }
            }
        }
        assert!(heads > 0 && feet > 0);
    }

    #[test]
    fn non_poi_blocks_resolve_to_nothing() {
        assert!(for_state(&Block::STONE, Block::STONE.default_state.id).is_none());
        assert!(!has_poi(&Block::STONE, Block::STONE.default_state.id));
        assert!(has_poi(&Block::BELL, Block::BELL.default_state.id));
    }

    #[test]
    fn lookup_by_name_round_trips_and_rejects_unknowns() {
        for t in ALL {
            assert_eq!(by_name(t.name).map(|f| f.name), Some(t.name));
        }
        assert!(by_name("minecraft:not_a_poi").is_none());
        assert_eq!(max_tickets_of("minecraft:home"), 1);
        assert_eq!(max_tickets_of("minecraft:meeting"), 32);
        assert_eq!(max_tickets_of("minecraft:nether_portal"), 0);
        // Unknown names are unclaimable rather than free-for-all.
        assert_eq!(max_tickets_of("minecraft:not_a_poi"), 0);
    }
}
