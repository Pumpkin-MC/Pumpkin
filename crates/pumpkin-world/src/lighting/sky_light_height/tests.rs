use super::*;
use crate::ProtoChunk;
use crate::chunk::ChunkData;
use pumpkin_data::Block;
use pumpkin_nbt::tag::NbtTag;
use std::sync::atomic::Ordering;

#[test]
fn encode_decode_round_trip_lower_half() {
    let height = SkyLightHeight::encode(-32, -64, 384);
    assert_eq!(height.raw() & SkyLightHeight::FLAG_HALF, 0);
    let decoded = height.decode(-64, 384);
    assert!((decoded - -32).abs() <= 1);
}

#[test]
fn encode_decode_round_trip_upper_half() {
    let height = SkyLightHeight::encode(200, -64, 384);
    assert!(height.raw() & SkyLightHeight::FLAG_HALF != 0);
    let decoded = height.decode(-64, 384);
    assert!((decoded - 200).abs() <= 1);
}

#[test]
fn surface_water_flag_round_trips() {
    let height = SkyLightHeight::encode(64, -64, 384).with_surface_water(true);
    assert!(height.has_surface_water());
    let height = height.with_surface_water(false);
    assert!(!height.has_surface_water());
}

#[test]
fn quadrant_flags_are_independent() {
    let height = SkyLightHeight::encode(64, -64, 384);
    assert!(height.quadrant_uses_limit(0, 0));
    assert!(height.quadrant_uses_limit(15, 15));

    let height = height.with_quadrant_diverged(3, 3);
    assert!(!height.quadrant_uses_limit(0, 0));
    assert!(height.quadrant_uses_limit(12, 0));
    assert!(height.quadrant_uses_limit(0, 12));
    assert!(height.quadrant_uses_limit(12, 12));
}

#[test]
fn sentinel_zero_is_never_produced_by_ensure_lazy() {
    let chunk = ChunkData::empty(0, 0);
    let height = SkyLightHeightMigration::ensure_lazy(&chunk, || SkyLightHeight::from_raw(0));
    assert_ne!(height.raw(), 0);
    assert_eq!(
        chunk.sky_light_height_cache.load(Ordering::Relaxed),
        height.raw()
    );
}

#[test]
fn ensure_lazy_persists_and_reloads() {
    let chunk = ChunkData::empty(0, 0);
    assert!(!SkyLightHeightMigration::fast_load_flag(&chunk));

    let computed = SkyLightHeight::encode(10, -64, 384);
    let height = SkyLightHeightMigration::ensure_lazy(&chunk, || computed);
    assert!(SkyLightHeightMigration::fast_load_flag(&chunk));

    // Reset the in-memory cache to force the NBT-backed reload path.
    chunk.sky_light_height_cache.store(0, Ordering::Relaxed);
    let reloaded = SkyLightHeightMigration::ensure_lazy(&chunk, || {
        panic!("should not recompute once persisted")
    });
    assert_eq!(reloaded, height);
}

/// Fills every column of the chunk with stone from `min_y` up to and including `top`.
fn fill_terrain(chunk: &ChunkData, top: i32) {
    let min_y = chunk.section.min_y;
    for local_z in 0..16usize {
        for local_x in 0..16usize {
            for y in min_y..=top {
                chunk.set_block_absolute_y(local_x, y, local_z, Block::STONE.default_state.id);
            }
        }
    }
}

fn tier_at(chunk: &ChunkData, height: SkyLightHeight, y: i32, x: i32, z: i32) -> SkyLightTier {
    height.tier(
        y,
        x,
        z,
        chunk.section.min_y,
        SkyLightHeight::chunk_height(chunk),
    )
}

#[test]
fn flat_terrain_splits_into_three_tiers() {
    let chunk = ChunkData::empty(0, 0);
    fill_terrain(&chunk, 60);
    let height = SkyLightHeight::compute_from_chunk(&chunk);

    // Flat: no quadrant diverges, everything can use the chunk cut.
    assert!(height.quadrant_uses_limit(0, 0));
    assert!(height.quadrant_uses_limit(15, 15));

    assert_eq!(tier_at(&chunk, height, 20, 8, 8), SkyLightTier::NoOpenSky);
    assert_eq!(tier_at(&chunk, height, 60, 8, 8), SkyLightTier::Unknown);
    assert_eq!(tier_at(&chunk, height, 200, 8, 8), SkyLightTier::OpenSky);
}

/// Flat terrain has no spread, so the expensive tier 3 band must shrink to the
/// smallest step
#[test]
fn flat_terrain_picks_the_tightest_band() {
    let chunk = ChunkData::empty(0, 0);
    fill_terrain(&chunk, 60);
    let height = SkyLightHeight::compute_from_chunk(&chunk);

    assert_eq!(height.spread(), SPREAD_SCALES[0]);
    // Just above the surface is already tier 2, no longer the band.
    assert_eq!(tier_at(&chunk, height, 67, 8, 8), SkyLightTier::OpenSky);
}

/// Swiss-cheese terrain needs a wider band, otherwise the quadrants would
/// all be marked as diverged
#[test]
fn rough_terrain_widens_the_band_instead_of_diverging() {
    let chunk = ChunkData::empty(0, 0);
    fill_terrain(&chunk, 60);
    // Pillars up to y=72: spread 12, fits into neither of the two smallest steps.
    for local_z in (0..16usize).step_by(4) {
        for local_x in (0..16usize).step_by(4) {
            for y in 61..=72 {
                chunk.set_block_absolute_y(local_x, y, local_z, Block::STONE.default_state.id);
            }
        }
    }

    let height = SkyLightHeight::compute_from_chunk(&chunk);
    assert!(height.spread() >= 12, "band {} too narrow", height.spread());
    assert!(
        height.quadrant_uses_limit(2, 2),
        "widening the band must keep the quadrants usable"
    );
}

#[test]
fn spread_survives_a_round_trip_through_nbt() {
    let chunk = ChunkData::empty(0, 0);
    fill_terrain(&chunk, 60);
    let height = SkyLightHeightMigration::get(&chunk);
    let spread = height.spread();

    chunk.sky_light_height_cache.store(0, Ordering::Relaxed);
    assert_eq!(SkyLightHeightMigration::get(&chunk).spread(), spread);
}

/// The cut must follow the highest light-blocking block, not the `WorldSurface`
/// heightmap: glass is "not air" but transmits, so a surface-derived cut would
/// trivially reject positions under glass that really do see the sky.
#[test]
fn glass_does_not_raise_the_cut() {
    let chunk = ChunkData::empty(0, 0);
    fill_terrain(&chunk, 60);
    for local_z in 0..16usize {
        for local_x in 0..16usize {
            chunk.set_block_absolute_y(local_x, 100, local_z, Block::GLASS.default_state.id);
        }
    }

    let height = SkyLightHeight::compute_from_chunk(&chunk);
    let cut = height.decode(chunk.section.min_y, SkyLightHeight::chunk_height(&chunk));
    assert!(
        cut <= 61,
        "cut {cut} follows the glass at y=100 instead of the stone ceiling at y=60"
    );
    assert_ne!(tier_at(&chunk, height, 80, 8, 8), SkyLightTier::NoOpenSky);
}

#[test]
fn shaft_only_degrades_its_own_quadrant() {
    let chunk = ChunkData::empty(0, 0);
    fill_terrain(&chunk, 60);
    // Dig a 1x1 shaft in the NW quadrant from the surface down to the bottom.
    for y in chunk.section.min_y..=60 {
        chunk.set_block_absolute_y(2, y, 2, Block::AIR.default_state.id);
    }

    let height = SkyLightHeight::compute_from_chunk(&chunk);
    assert!(
        !height.quadrant_uses_limit(2, 2),
        "the shaft's own quadrant must lose its fast path"
    );
    for (x, z) in [(12, 2), (2, 12), (12, 12)] {
        assert!(
            height.quadrant_uses_limit(x, z),
            "quadrant ({x},{z}) must keep the fast path"
        );
    }

    // The shaft column must never be trivially rejected; the untouched ones still are.
    assert_eq!(tier_at(&chunk, height, 10, 2, 2), SkyLightTier::Unknown);
    assert_eq!(tier_at(&chunk, height, 10, 12, 12), SkyLightTier::NoOpenSky);
}

/// AND gate: the fast path holds at a border only if both
/// quadrants carry it. NAND (one diverges) -> real check.
#[test]
fn border_gate_needs_both_sides() {
    let flat = SkyLightHeight::encode(56, -64, 384);
    let diverged = flat.with_quadrant_diverged(15, 8);

    // Our east edge (x=15) meets the neighbour's west edge (x=0).
    assert!(flat.border_uses_limit(flat, 15, 8, 0, 8), "schneller Pfad");
    assert!(
        !flat.border_uses_limit(diverged.with_quadrant_diverged(0, 8), 15, 8, 0, 8),
        "Nachbar -> echter Check"
    );
    assert!(
        !flat
            .with_quadrant_diverged(15, 8)
            .border_uses_limit(flat, 15, 8, 0, 8),
        "master -> echter Check"
    );

    // Only the neighbour's near-border quadrant counts: a divergence on its
    // opposite side (x=15) must not slow us down.
    assert!(
        flat.border_uses_limit(flat.with_quadrant_diverged(15, 8), 15, 8, 0, 8),
        "Abweichung auf der fernen Seite des Nachbarn ist irrelevant"
    );
}

#[test]
fn marking_a_quadrant_diverged_writes_through_to_nbt() {
    let chunk = ChunkData::empty(0, 0);
    fill_terrain(&chunk, 60);
    let height = SkyLightHeightMigration::get(&chunk);
    assert!(height.quadrant_uses_limit(2, 2));

    SkyLightHeightMigration::mark_quadrant_diverged(&chunk, 2, 2);

    let updated =
        SkyLightHeight::from_raw(chunk.sky_light_height_cache.load(Ordering::Relaxed));
    assert!(!updated.quadrant_uses_limit(2, 2));
    assert!(updated.quadrant_uses_limit(12, 12));

    // Drop the in-memory cache: the divergence must survive in NBT.
    chunk.sky_light_height_cache.store(0, Ordering::Relaxed);
    let reloaded = SkyLightHeightMigration::get(&chunk);
    assert!(!reloaded.quadrant_uses_limit(2, 2));
}

// ---- Phase 4: persistence --------------------------------------------------

/// The persisted `Int` carries value (bits 0-23) and geometry tag (bits 24-31);
/// what is read back is the plain 24-bit value again.
#[test]
fn persisted_value_carries_the_geometry_tag_out_of_band() {
    let chunk = ChunkData::empty(0, 0);
    let height = SkyLightHeight::encode(60, -64, 384).with_spread_index(2);
    SkyLightHeightMigration::persist(&chunk, height);

    let Some(NbtTag::Int(stored)) =
        chunk.get_custom_data("pumpkin:optimization", "sky_light_height_v1")
    else {
        panic!("nothing persisted");
    };
    let stored = stored as u32;
    assert_ne!(stored >> 24, 0, "Geometrie-Tag fehlt");
    assert_eq!(stored & 0x00FF_FFFF, height.raw(), "Wert veraendert");
    assert_eq!(
        SkyLightHeightMigration::load_persisted(&chunk),
        Some(height)
    );
}

/// The core of phase 4: the cut is encoded relative to `min_y`/chunk height. After a
/// world height change the same raw value decodes to a different Y — a cut that is too
/// low would treat blocks under solid rock as "open sky". The tag has to catch that
/// instead of reusing the old number.
#[test]
fn a_changed_world_height_invalidates_the_persisted_value() {
    let chunk = ChunkData::empty(0, 0);
    let height = SkyLightHeight::encode(60, -64, 384);
    SkyLightHeightMigration::persist(&chunk, height);
    assert!(SkyLightHeightMigration::load_persisted(&chunk).is_some());

    // The same raw value, but written under a different geometry.
    let foreign = SkyLightHeightMigration::geometry_tag(0, 256).expect("darstellbar");
    let current = SkyLightHeightMigration::geometry_tag(-64, 384).expect("darstellbar");
    assert_ne!(
        foreign, current,
        "die beiden Geometrien muessen sich unterscheiden"
    );
    chunk.set_custom_data(
        "pumpkin:optimization",
        "sky_light_height_v1",
        NbtTag::Int((height.raw() | (u32::from(foreign) << 24)) as i32),
    );

    assert_eq!(
        SkyLightHeightMigration::load_persisted(&chunk),
        None,
        "fremde Geometrie muss verworfen werden"
    );

    // ensure_lazy falls back cleanly to recomputation instead of adopting the garbage.
    fill_terrain(&chunk, 60);
    let fresh = SkyLightHeightMigration::get(&chunk);
    assert_eq!(SkyLightHeightMigration::load_persisted(&chunk), Some(fresh));
}

/// Unrepresentable geometry: better to persist nothing at all than to leave a value
/// whose validity nobody can check later on.
#[test]
fn unrepresentable_geometry_is_not_persisted() {
    assert!(SkyLightHeightMigration::geometry_tag(-64, 384).is_some());
    assert!(SkyLightHeightMigration::geometry_tag(0, 256).is_some());
    assert!(
        SkyLightHeightMigration::geometry_tag(-64, 100).is_none(),
        "Hoehe kein Vielfaches von 16"
    );
    assert!(
        SkyLightHeightMigration::geometry_tag(-1024, 384).is_none(),
        "min_y ausserhalb des darstellbaren Bereichs"
    );
}

/// Exactly one supported version: the key name must match [`SkyLightHeightMigration::VERSION`]
/// or writer and reader drift apart.
#[test]
fn the_key_name_matches_the_version_constant() {
    assert_eq!(
        SkyLightHeightMigration::KEY,
        format!("sky_light_height_v{}", SkyLightHeightMigration::VERSION)
    );
}

/// A key from another (older) version is not interpreted: recompute and
/// write the current version.
#[test]
fn an_older_version_key_is_ignored_and_overwritten() {
    let chunk = ChunkData::empty(0, 0);
    fill_terrain(&chunk, 60);

    // This is what a predecessor format might look like — other key, arbitrary content.
    chunk.set_custom_data(
        "pumpkin:optimization",
        "sky_light_height_v0",
        NbtTag::Int(0x0BAD_F00Du32 as i32),
    );

    assert_eq!(
        SkyLightHeightMigration::load_persisted(&chunk),
        None,
        "ein fremder/alter Key darf nicht als Wert durchgehen"
    );

    let height = SkyLightHeightMigration::get(&chunk);
    assert_eq!(height, SkyLightHeight::compute_from_chunk(&chunk));
    assert_eq!(
        SkyLightHeightMigration::load_persisted(&chunk),
        Some(height)
    );
    assert!(SkyLightHeightMigration::fast_load_flag(&chunk));
}

/// Broken or foreign NBT must not require its own repair path.
#[test]
fn corrupt_nbt_falls_back_to_recompute() {
    let chunk = ChunkData::empty(0, 0);
    chunk.set_custom_data(
        "pumpkin:optimization",
        "sky_light_height_v1",
        NbtTag::String("nonsense".into()),
    );
    assert_eq!(SkyLightHeightMigration::load_persisted(&chunk), None);

    fill_terrain(&chunk, 60);
    let height = SkyLightHeightMigration::get(&chunk);
    assert_ne!(height.raw(), 0);
    assert_eq!(
        SkyLightHeightMigration::load_persisted(&chunk),
        Some(height)
    );
}

/// A pure read access must not dirty the chunk — otherwise merely walking through a
/// world rewrites every chunk it touches in full.
#[test]
fn lazily_computing_the_cut_does_not_dirty_the_chunk() {
    let chunk = ChunkData::empty(0, 0);
    fill_terrain(&chunk, 60);
    chunk.dirty.store(false, Ordering::Relaxed);

    let _ = SkyLightHeightMigration::get(&chunk);

    assert!(
        !chunk.dirty.load(Ordering::Relaxed),
        "abgeleiteter Cache darf keinen Save erzwingen"
    );
    assert!(
        SkyLightHeightMigration::fast_load_flag(&chunk),
        "trotzdem muss der Wert im custom_data stehen und beim naechsten Save mitfahren"
    );
}

/// Save path: a value sitting only in the RAM cache must be picked up on serialization
/// and survive a full disk round trip.
#[test]
fn cached_value_survives_a_full_serialization_round_trip() {
    use crate::chunk::format::anvil::SingleChunkDataSerializer;
    use pumpkin_util::math::vector2::Vector2;

    let chunk = ChunkData::empty(0, 0);
    fill_terrain(&chunk, 60);
    let height = SkyLightHeight::compute_from_chunk(&chunk);

    // Only set the cache, deliberately do not call `persist`.
    chunk
        .sky_light_height_cache
        .store(height.raw(), Ordering::Relaxed);
    assert!(!SkyLightHeightMigration::fast_load_flag(&chunk));

    let bytes = chunk.to_bytes().expect("serialize");
    let reloaded = ChunkData::from_bytes(&bytes, Vector2::new(0, 0)).expect("deserialize");

    assert_eq!(
        SkyLightHeightMigration::load_persisted(&reloaded),
        Some(height),
        "der Save-Pfad muss den gecachten Wert mitschreiben"
    );
    assert_eq!(
        reloaded.sky_light_height_cache.load(Ordering::Relaxed),
        0,
        "frisch geladen ist der RAM-Cache leer; der Wert kommt erst beim ersten Zugriff"
    );
    assert_eq!(SkyLightHeightMigration::get(&reloaded), height);
}

/// A column where nothing has changed must never count as "left the band"
/// -> or its quadrant degrades to tier 3 permanently.
#[test]
fn unchanged_columns_never_leave_the_band() {
    let mut spurious = Vec::new();

    for top in 0..120i32 {
        let chunk = ChunkData::empty(0, 0);
        fill_terrain(&chunk, top);
        let height = SkyLightHeightMigration::get(&chunk);
        let min_y = chunk.section.min_y;
        let chunk_height = SkyLightHeight::chunk_height(&chunk);

        for (x, z) in [(2, 2), (12, 2), (2, 12), (12, 12)] {
            let ceiling = SkyLightHeight::column_ceiling_at(&chunk, x, z);
            if !height.ceiling_within_band(ceiling, min_y, chunk_height) {
                let cut = height.decode(min_y, chunk_height);
                spurious.push(format!(
                    "top={top} ({x},{z}): Decke {ceiling} ausserhalb [{}, {}]",
                    cut,
                    cut + height.spread()
                ));
            }
        }
    }

    assert!(
        spurious.is_empty(),
        "unveraenderte Spalten wurden als divergiert gewertet ({} Faelle):\n{}",
        spurious.len(),
        spurious.join("\n")
    );
}

/// The widened band check must not undermine the tier promise
#[test]
fn band_tolerance_never_contradicts_the_tier_promise() {
    let (min_y, chunk_height) = (-64, 384);

    for cut_y in (min_y + 1)..(min_y + chunk_height - 40) {
        for spread_index in 0..SPREAD_SCALES.len() {
            let height =
                SkyLightHeight::encode(cut_y, min_y, chunk_height).with_spread_index(spread_index);
            let cut = height.decode(min_y, chunk_height);

            for ceiling in (cut - 2)..=(cut + height.spread() + 2) {
                if !height.ceiling_within_band(ceiling, min_y, chunk_height) {
                    continue;
                }
                for y in (cut - 4)..=(cut + height.spread() + 4) {
                    match height.tier(y, 0, 0, min_y, chunk_height) {
                        SkyLightTier::NoOpenSky => assert!(
                            y <= ceiling,
                            "Tier 1 bei y={y}, aber die Decke liegt bei {ceiling} darunter"
                        ),
                        SkyLightTier::OpenSky => assert!(
                            y > ceiling,
                            "Tier 2 bei y={y}, aber die Decke {ceiling} verdeckt es"
                        ),
                        SkyLightTier::Unknown => {}
                    }
                }
            }
        }
    }
}

///`ProtoChunk` without any generation run.
fn proto_chunk() -> ProtoChunk {
    use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::world_seed::Seed;

    let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
        Seed(42),
        Dimension::OVERWORLD,
    )));
    ProtoChunk::new(0, 0, &world_gen)
}

/// Fills every column up to `top` with stone and updates the `WorldSurface` heightmap.
fn fill_proto_terrain(proto: &mut ProtoChunk, top: i32) {
    let min_y = i32::from(proto.bottom_y());
    for local_z in 0..16 {
        for local_x in 0..16 {
            for y in min_y..=top {
                proto.set_block_state(local_x, y, local_z, Block::STONE.default_state);
            }
        }
    }
}

#[test]
fn worldgen_flat_terrain_is_usable_in_all_quadrants() {
    let mut proto = proto_chunk();
    fill_proto_terrain(&mut proto, 60);

    let height = SkyLightHeight::compute_from_proto(&proto);
    let min_y = i32::from(proto.bottom_y());
    let cut = height.decode(min_y, i32::from(proto.height()));

    // The cut is the lower edge of the band, not the ceiling height itself: all
    // ceilings (here 60) must lie in [cut, cut + spread].
    assert!(
        cut - DECODE_SAFETY_MARGIN <= 60 && 60 <= cut + height.spread(),
        "Decke 60 liegt nicht im Band [{cut}, {}]",
        cut + height.spread()
    );
    for (x, z) in [(2, 2), (12, 2), (2, 12), (12, 12)] {
        assert!(
            height.quadrant_uses_limit(x, z),
            "flaches Terrain: Quadrant ({x},{z}) muss den schnellen Pfad behalten"
        );
    }
    // Flat means the narrowest band.
    assert_eq!(height.spread(), SPREAD_SCALES[0]);
    assert_ne!(height.raw(), 0, "Sentinel darf nie entstehen");
}

/// A shaft (carver/ravine case) may only degrade its own 8x8 quadrant.
#[test]
fn worldgen_shaft_only_degrades_its_own_quadrant() {
    let mut proto = proto_chunk();
    fill_proto_terrain(&mut proto, 60);

    // Column (2,2) cleared out to well below the cut.
    let min_y = i32::from(proto.bottom_y());
    for y in min_y..=60 {
        proto.set_block_state(2, y, 2, Block::AIR.default_state);
    }

    let height = SkyLightHeight::compute_from_proto(&proto);
    assert!(
        !height.quadrant_uses_limit(2, 2),
        "das Quadrant des Schachts muss den schnellen Pfad verlieren"
    );
    for (x, z) in [(12, 2), (2, 12), (12, 12)] {
        assert!(
            height.quadrant_uses_limit(x, z),
            "Quadrant ({x},{z}) muss den schnellen Pfad behalten"
        );
    }
}

/// Glass is not air but transmits light: `WorldSurface` sits high, yet the cut
/// must not follow, or a lit column would count as "no sky".
#[test]
fn worldgen_glass_does_not_raise_the_cut() {
    let mut proto = proto_chunk();
    fill_proto_terrain(&mut proto, 60);
    for local_z in 0..16 {
        for local_x in 0..16 {
            proto.set_block_state(local_x, 80, local_z, Block::GLASS.default_state);
        }
    }

    let height = SkyLightHeight::compute_from_proto(&proto);
    let cut = height.decode(i32::from(proto.bottom_y()), i32::from(proto.height()));
    assert!(
        cut + height.spread() < 80,
        "Band [{cut}, {}] folgt dem Glas auf 80 statt dem Stein auf 60",
        cut + height.spread()
    );
    assert!(
        cut - DECODE_SAFETY_MARGIN <= 60 && 60 <= cut + height.spread(),
        "der Stein auf 60 muss weiterhin die Decke sein"
    );
}

/// survive the upgrade to a level chunk -> in cache and NBT, without recomputation.
#[test]
fn worldgen_value_survives_upgrade_to_level_chunk() {
    use crate::chunk_system::chunk_state::Chunk;
    use pumpkin_config::lighting::LightingEngineConfig;
    use pumpkin_data::dimension::Dimension;

    let mut proto = proto_chunk();
    fill_proto_terrain(&mut proto, 60);
    let computed = SkyLightHeight::compute_from_proto(&proto);
    proto.sky_light_height = computed.raw();

    let mut chunk = Chunk::Proto(Box::new(proto));
    chunk.upgrade_to_level_chunk(&Dimension::OVERWORLD, &LightingEngineConfig::Default);
    let Chunk::Level(level) = chunk else {
        panic!("upgrade did not produce a level chunk");
    };

    assert_eq!(
        level.sky_light_height_cache.load(Ordering::Relaxed),
        computed.raw(),
        "der Worldgen-Wert muss im Cache ankommen"
    );
    assert!(
        SkyLightHeightMigration::fast_load_flag(&level),
        "und direkt persistiert sein, ohne ersten Lazy-Zugriff"
    );
    assert_eq!(SkyLightHeightMigration::get(&level), computed);
}
