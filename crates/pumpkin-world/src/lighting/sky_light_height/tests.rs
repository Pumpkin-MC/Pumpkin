//! tests for the sky light cut height.

use super::{SPREAD_SCALES, SkyLightHeight, SkyLightHeightMigration, SkyLightTier};
use crate::ProtoChunk;
use crate::chunk::format::anvil::SingleChunkDataSerializer;
use crate::chunk::{ChunkData, ChunkSections};
use pumpkin_data::{Block, BlockState, BlockStateId};
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::math::vector2::Vector2;
use std::sync::atomic::Ordering;

/// Namespace and key are part of the on-disk contract, spelled out in concept #5.
const NAMESPACE: &str = "pumpkin:optimization";
const KEY: &str = "sky_light_height_v1";

const SECTIONS: usize = 24;
const MIN_Y: i32 = -64;
const HEIGHT: i32 = SECTIONS as i32 * 16;

/// One test terrain, described once and built twice: as a `ChunkData` for the runtime path
/// and as a `ProtoChunk` for the worldgen path.
struct Shape {
    name: &'static str,
    /// Top of the solid crust per column, or `None` for a chunk with no blocks at all.
    top: Option<fn(i32, i32) -> i32>,
    /// A transparent roof across the whole chunk: above the crust in the heightmap, but
    /// sky light falls straight through it.
    glass_roof: Option<i32>,
    /// A column carved open down to the world floor (ravine).
    shaft: Option<(usize, usize)>,
    /// A single column raised to this height.
    tower: Option<(usize, usize, i32)>,
}

const SHAPES: &[Shape] = &[
    Shape {
        name: "flat",
        top: Some(|_, _| 60),
        glass_roof: None,
        shaft: None,
        tower: None,
    },
    Shape {
        name: "gentle slope",
        top: Some(|x, _| 60 + x / 4),
        glass_roof: None,
        shaft: None,
        tower: None,
    },
    Shape {
        name: "rough",
        top: Some(|x, z| 60 + (x * z) % 20),
        glass_roof: None,
        shaft: None,
        tower: None,
    },
    Shape {
        name: "tower in NE",
        top: Some(|_, _| 60),
        glass_roof: None,
        shaft: None,
        tower: Some((12, 3, 200)),
    },
    Shape {
        name: "shaft to the floor in SW",
        top: Some(|_, _| 60),
        glass_roof: None,
        shaft: Some((3, 12)),
        tower: None,
    },
    Shape {
        name: "glass roof over stone",
        top: Some(|_, _| 60),
        glass_roof: Some(90),
        shaft: None,
        tower: None,
    },
    Shape {
        name: "no blocks at all",
        top: None,
        glass_roof: None,
        shaft: None,
        tower: None,
    },
];

fn stone() -> BlockStateId {
    Block::STONE.default_state.id
}

fn chunk_with_geometry(sections: usize, min_y: i32) -> ChunkData {
    let mut chunk = ChunkData::empty(0, 0);
    chunk.section = ChunkSections::new(sections, min_y);
    chunk
}

/// Builds the shape as a finished chunk, then rebuilds the heightmaps the way a freshly
/// loaded chunk would have them.
fn build_chunk(shape: &Shape) -> ChunkData {
    let chunk = chunk_with_geometry(SECTIONS, MIN_Y);
    let mut updates = Vec::new();

    if let Some(top) = shape.top {
        for x in 0..16i32 {
            for z in 0..16i32 {
                let column_top = top(x, z);
                for y in (column_top - 5)..=column_top {
                    updates.push((x as usize, y, z as usize, stone()));
                }
            }
        }
    }
    if let Some((x, z, top)) = shape.tower {
        for y in 61..=top {
            updates.push((x, y, z, stone()));
        }
    }
    if let Some(y) = shape.glass_roof {
        for x in 0..16usize {
            for z in 0..16usize {
                updates.push((x, y, z, Block::GLASS.default_state.id));
            }
        }
    }
    if let Some((x, z)) = shape.shaft {
        for y in MIN_Y..=200 {
            updates.push((x, y, z, Block::AIR.default_state.id));
        }
    }

    chunk.set_blocks_batch(updates);
    *chunk
        .heightmap
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = chunk.calculate_heightmap();
    chunk
}

fn shape(name: &str) -> ChunkData {
    build_chunk(
        SHAPES
            .iter()
            .find(|s| s.name == name)
            .expect("unknown shape"),
    )
}

/// "Light-blocking" is `opacity > 0` -> the predicate the spec names when it corrects the
/// `WorldSurface` caveat: glass is not air, but it does not stop sky light.
fn true_ceiling(chunk: &ChunkData, x: usize, z: usize) -> i32 {
    let min_y = chunk.section.min_y;
    let mut y = min_y + SkyLightHeight::chunk_height(chunk) - 1;
    while y >= min_y {
        if let Some(id) = chunk.section.get_block_absolute_y(x, y, z)
            && BlockState::from_id(id).opacity > 0
        {
            return y;
        }
        y -= 1;
    }
    min_y - 1
}

fn tier_at(chunk: &ChunkData, height: SkyLightHeight, x: i32, z: i32, y: i32) -> SkyLightTier {
    height.tier(
        y,
        x,
        z,
        chunk.section.min_y,
        SkyLightHeight::chunk_height(chunk),
    )
}

/// The bit layout is the on-disk format: every stored world depends on it.
/// Bits 24-31 stay clear because the persistence layer keeps the geometry tag there.
#[test]
fn the_bit_layout_is_frozen() {
    let half = MIN_Y + HEIGHT / 2;
    for y in MIN_Y..(MIN_Y + HEIGHT) {
        let raw = SkyLightHeight::encode(y, MIN_Y, HEIGHT).raw();
        assert_eq!(
            raw >> 24,
            0,
            "encode({y}) spilled into the geometry-tag bits"
        );
        assert_eq!(
            raw & (1 << 16) != 0,
            y >= half,
            "bit 16 is not the half flag at y={y}"
        );
    }

    let base = SkyLightHeight::encode(80, MIN_Y, HEIGHT);
    assert_eq!(
        base.with_surface_water(true).raw() ^ base.raw(),
        1 << 17,
        "surface water is not bit 17"
    );
    for (x, z, bit, name) in [
        (0, 0, 18, "NW"),
        (8, 0, 19, "NE"),
        (0, 8, 20, "SW"),
        (8, 8, 21, "SE"),
    ] {
        assert_eq!(
            base.with_quadrant_diverged(x, z).raw() ^ base.raw(),
            1 << bit,
            "quadrant {name} is not bit {bit}"
        );
    }

    for shape in SHAPES {
        let height = SkyLightHeight::compute_from_chunk(&build_chunk(shape));
        let index = ((height.raw() >> 22) & 0b11) as usize;
        assert_eq!(
            SPREAD_SCALES[index],
            height.spread(),
            "bits 22-23 do not index SPREAD_SCALES on {}",
            shape.name
        );
    }
}

/// The fraction is lossy by design and the tier logic budgets for exactly one block of it.
/// Pinning the error to `[y-1, y]` is what allows the band edges to carry a margin of 1.
#[test]
fn decoding_loses_at_most_one_block_and_never_rounds_up() {
    for (min_y, height) in [(-64, 384), (0, 256), (-64, 256), (0, 128)] {
        for y in min_y..(min_y + height) {
            let decoded = SkyLightHeight::encode(y, min_y, height).decode(min_y, height);
            assert!(
                decoded <= y && decoded >= y - 1,
                "encode/decode of y={y} in ({min_y}, {height}) returned {decoded}, outside \
                 the one-block error the band margin is built for"
            );
        }
    }
}

/// `raw() == 0` is the "nothing cached yet" sentinel of `sky_light_height_cache`, and the
/// lowest encodable value collides with it. The spec resolves that in `ensure_lazy`,
/// before the value is cached or persisted.
#[test]
fn the_cache_sentinel_is_never_stored_as_a_real_value() {
    assert_eq!(
        SkyLightHeight::encode(MIN_Y, MIN_Y, HEIGHT).raw(),
        0,
        "the collision this test guards against is gone, the sentinel fix may be dead code"
    );

    let chunk = chunk_with_geometry(SECTIONS, MIN_Y);
    let stored = SkyLightHeightMigration::ensure_lazy(&chunk, || {
        SkyLightHeight::encode(MIN_Y, MIN_Y, HEIGHT)
    });

    assert_ne!(stored.raw(), 0, "the sentinel value was handed out as-is");
    assert_ne!(
        chunk.sky_light_height_cache.load(Ordering::Relaxed),
        0,
        "the sentinel was written into the cache, where it means 'not cached'"
    );
    assert_eq!(
        stored.decode(MIN_Y, HEIGHT),
        MIN_Y,
        "the sentinel correction moved the cut further than the fraction it bumped"
    );
}

/// The four flags partition the chunk into 8x8 quadrants, and divergence means "fall back
/// to the real check"
#[test]
fn a_diverged_quadrant_costs_exactly_its_own_64_columns() {
    let chunk = shape("flat");
    let base = SkyLightHeight::compute_from_chunk(&chunk);

    for (fx, fz, name) in [(0, 0, "NW"), (8, 0, "NE"), (0, 8, "SW"), (8, 8, "SE")] {
        let height = base.with_quadrant_diverged(fx, fz);
        let mut diverged = 0;
        for x in 0..16 {
            for z in 0..16 {
                let same_quadrant = (x < 8) == (fx < 8) && (z < 8) == (fz < 8);
                assert_eq!(
                    !height.quadrant_uses_limit(x, z),
                    same_quadrant,
                    "flag {name} leaked into column ({x}, {z})"
                );
                diverged += i32::from(same_quadrant);
            }
        }
        assert_eq!(diverged, 64, "quadrant {name} is not an 8x8 block");

        for y in MIN_Y..(MIN_Y + HEIGHT) {
            assert_eq!(
                tier_at(&chunk, height, fx, fz, y),
                SkyLightTier::Unknown,
                "diverged quadrant {name} still gave a fast answer at y={y}"
            );
        }
    }
}

/// `OpenSky` claims full sunlight and burns light 15 into the cell; `NoOpenSky` claims the
/// column is covered. Both are checked against ground truth recomputed from raw block
/// states, for every column and every height of every shape.
#[test]
fn tier_never_lies_about_open_sky() {
    for shape in SHAPES {
        let chunk = build_chunk(shape);
        let height = SkyLightHeight::compute_from_chunk(&chunk);
        for x in 0..16i32 {
            for z in 0..16i32 {
                let ceiling = true_ceiling(&chunk, x as usize, z as usize);
                for y in MIN_Y..(MIN_Y + HEIGHT) {
                    match tier_at(&chunk, height, x, z, y) {
                        SkyLightTier::OpenSky => assert!(
                            y > ceiling,
                            "{}: claimed open sky at ({x}, {y}, {z}), but the column is \
                             covered up to {ceiling}",
                            shape.name
                        ),
                        SkyLightTier::NoOpenSky => assert!(
                            y <= ceiling,
                            "{}: claimed no open sky at ({x}, {y}, {z}), but the column \
                             ends at {ceiling}",
                            shape.name
                        ),
                        SkyLightTier::Unknown => {}
                    }
                }
            }
        }
    }
}

/// The other half of the promise: a quadrant only keeps the fast path if every one of its
/// ceilings really is inside `[cut, cut + spread]`.
#[test]
fn a_quadrant_that_uses_the_cut_keeps_every_one_of_its_ceilings_in_band() {
    for shape in SHAPES {
        let chunk = build_chunk(shape);
        let height = SkyLightHeight::compute_from_chunk(&chunk);
        for x in 0..16i32 {
            for z in 0..16i32 {
                if !height.quadrant_uses_limit(x, z) {
                    continue;
                }
                let ceiling = true_ceiling(&chunk, x as usize, z as usize);
                assert!(
                    height.ceiling_within_band(ceiling, MIN_Y, HEIGHT),
                    "{}: column ({x}, {z}) has ceiling {ceiling} outside the band of a \
                     quadrant that still promises the fast path (cut={}, spread={})",
                    shape.name,
                    height.decode(MIN_Y, HEIGHT),
                    height.spread()
                );
            }
        }
    }
}

/// flat terrain lands on the narrowest scale, and terrain that
/// still fits a scale widens the band instead of paying with a quadrant.
#[test]
fn the_band_is_as_narrow_as_the_terrain_allows() {
    let flat = SkyLightHeight::compute_from_chunk(&shape("flat"));
    assert_eq!(flat.spread(), SPREAD_SCALES[0]);

    let rough = SkyLightHeight::compute_from_chunk(&shape("rough"));
    assert!(
        rough.spread() > SPREAD_SCALES[0],
        "a 20-block ceiling range fit into the narrowest band"
    );

    for (name, height) in [("flat", flat), ("rough", rough)] {
        for x in 0..16 {
            for z in 0..16 {
                assert!(
                    height.quadrant_uses_limit(x, z),
                    "{name} terrain diverged at ({x}, {z}) although it fits a scale"
                );
            }
        }
    }
}

/// A single column open to the floor is the case the quadrant flags exist for: it must
/// cost its own quadrant, not the chunk.
#[test]
fn a_column_open_to_the_floor_degrades_only_its_own_quadrant() {
    let height = SkyLightHeight::compute_from_chunk(&shape("shaft to the floor in SW"));

    assert!(
        !height.quadrant_uses_limit(3, 12),
        "the shaft's own quadrant kept the fast path"
    );
    for (x, z) in [(3, 3), (12, 3), (12, 12)] {
        assert!(
            height.quadrant_uses_limit(x, z),
            "the shaft in SW degraded the untouched quadrant at ({x}, {z})"
        );
    }
}

/// `WorldSurface` counts glass as surface, but sky light falls straight through it. A cut
/// taken from the heightmap alone would sit at the glass and declare the lit room below it
/// covered.
#[test]
fn a_glass_roof_does_not_raise_the_cut() {
    let chunk = shape("glass roof over stone");
    let height = SkyLightHeight::compute_from_chunk(&chunk);

    let cut = height.decode(MIN_Y, HEIGHT);
    assert!(
        cut <= 61,
        "the cut followed the glass roof to {cut} instead of the stone at 60"
    );
    assert_eq!(
        tier_at(&chunk, height, 4, 4, 75),
        SkyLightTier::OpenSky,
        "a cell under glass and above stone was not treated as sunlit"
    );
}

/// A vanilla chunk has no key at all: compute once, cache, persist and never compute
/// again while the chunk stays loaded.
#[test]
fn a_chunk_without_the_key_computes_once_and_keeps_the_result() {
    let chunk = shape("flat");
    assert!(!SkyLightHeightMigration::fast_load_flag(&chunk));

    let mut computed = 0;
    let first = SkyLightHeightMigration::ensure_lazy(&chunk, || {
        computed += 1;
        SkyLightHeight::compute_from_chunk(&chunk)
    });
    let second = SkyLightHeightMigration::ensure_lazy(&chunk, || {
        computed += 1;
        SkyLightHeight::compute_from_chunk(&chunk)
    });

    assert_eq!(computed, 1, "the cached value was recomputed");
    assert_eq!(first, second);
    assert!(
        SkyLightHeightMigration::fast_load_flag(&chunk),
        "the computed value was never persisted"
    );
}

/// Every unusable persisted value takes the same route -> recompute and overwrite. No
/// repair path per failure mode: a wrongly converted cut does not crash, it shows up as
/// permanently wrong light.
#[test]
fn every_unusable_persisted_value_is_recomputed_the_same_way() {
    /// A way to leave the persisted value unusable, paired with what it is called in the
    /// failure message.
    type Case<'a> = (&'a str, &'a dyn Fn(&ChunkData));

    let cases: [Case; 3] = [
        ("wrong tag type", &|chunk: &ChunkData| {
            chunk.set_custom_data(NAMESPACE, KEY, NbtTag::String("not a cut height".into()));
        }),
        ("key of another format version", &|chunk: &ChunkData| {
            chunk.set_custom_data(NAMESPACE, "sky_light_height_v0", NbtTag::Int(0x0BAD_F00D));
        }),
        (
            "value tagged for another world height",
            &|chunk: &ChunkData| {
                let tall = shape("flat");
                SkyLightHeightMigration::ensure_lazy(&tall, || {
                    SkyLightHeight::compute_from_chunk(&tall)
                });
                let stored = tall
                    .get_custom_data(NAMESPACE, KEY)
                    .expect("the reference value was persisted");
                chunk.set_custom_data(NAMESPACE, KEY, stored);
            },
        ),
    ];

    for (name, corrupt) in cases {
        // A 256-block chunk, so the value written above for a 384-block world is rejected
        // on its geometry tag instead of silently decoding to a far too low cut.
        let chunk = chunk_with_geometry(16, MIN_Y);
        corrupt(&chunk);

        let mut computed = 0;
        let height = SkyLightHeightMigration::ensure_lazy(&chunk, || {
            computed += 1;
            SkyLightHeight::compute_from_chunk(&chunk)
        });

        assert_eq!(computed, 1, "{name} was accepted as a cut height");
        assert_eq!(
            SkyLightHeightMigration::load_persisted(&chunk),
            Some(height),
            "{name} was not overwritten with the recomputed value"
        );
    }
}

/// The cut is derived data, computed on the read path. If deriving it dirtied the chunk,
/// walking through a world would rewrite every chunk it touched
#[test]
fn deriving_the_cut_does_not_dirty_the_chunk() {
    let chunk = shape("flat");
    chunk.dirty.store(false, Ordering::Relaxed);

    SkyLightHeightMigration::ensure_lazy(&chunk, || SkyLightHeight::compute_from_chunk(&chunk));

    assert!(
        !chunk.dirty.load(Ordering::Relaxed),
        "a read-path computation forced a full chunk rewrite"
    );
}

/// A geometry whose tag cannot be represented gets no persisted value at all:
/// Such a chunk still uses the RAM cache.
#[test]
fn unrepresentable_geometry_is_not_persisted_at_all() {
    let chunk = chunk_with_geometry(SECTIONS, -60);
    let height =
        SkyLightHeightMigration::ensure_lazy(&chunk, || SkyLightHeight::compute_from_chunk(&chunk));

    assert!(
        !SkyLightHeightMigration::fast_load_flag(&chunk),
        "a value was persisted under a geometry whose tag cannot be encoded"
    );
    assert_eq!(
        chunk.sky_light_height_cache.load(Ordering::Relaxed),
        height.raw(),
        "the chunk lost the in-memory cache as well"
    );
}

/// Divergence is discovered at runtime, has to survive the save, and only ever accumulates
/// -> a later discovery elsewhere in the chunk must not disprove an earlier one.
#[test]
fn divergence_reaches_cache_and_nbt_and_only_accumulates() {
    let chunk = shape("flat");
    SkyLightHeightMigration::ensure_lazy(&chunk, || SkyLightHeight::compute_from_chunk(&chunk));

    SkyLightHeightMigration::mark_quadrant_diverged(&chunk, 2, 2);
    SkyLightHeightMigration::mark_quadrant_diverged(&chunk, 12, 2);

    let cached = SkyLightHeight::from_raw(chunk.sky_light_height_cache.load(Ordering::Relaxed));
    assert!(
        !cached.quadrant_uses_limit(2, 2),
        "the first flag was lost when the second was set"
    );
    assert!(
        !cached.quadrant_uses_limit(12, 2),
        "the second flag is missing"
    );
    assert!(
        cached.quadrant_uses_limit(2, 12),
        "an untouched quadrant was flagged"
    );
    assert_eq!(
        SkyLightHeightMigration::load_persisted(&chunk),
        Some(cached),
        "the divergence would be lost on the next load"
    );
}

/// Divergences discovered at the same moment must all survive: a dropped flag is a
/// quadrant that goes on promising a fast answer it can no longer honour.
#[test]
fn divergences_discovered_at_the_same_moment_all_survive() {
    const QUADRANTS: [(i32, i32); 4] = [(2, 2), (12, 2), (2, 12), (12, 12)];

    for round in 0..16 {
        let chunk = shape("flat");
        SkyLightHeightMigration::ensure_lazy(&chunk, || SkyLightHeight::compute_from_chunk(&chunk));

        std::thread::scope(|scope| {
            for (x, z) in QUADRANTS {
                let chunk = &chunk;
                scope.spawn(move || {
                    SkyLightHeightMigration::mark_quadrant_diverged(chunk, x, z);
                });
            }
        });

        let cached = SkyLightHeight::from_raw(chunk.sky_light_height_cache.load(Ordering::Relaxed));
        for (x, z) in QUADRANTS {
            assert!(
                !cached.quadrant_uses_limit(x, z),
                "round {round}: the flag for the quadrant at ({x}, {z}) was lost to a \
                 concurrent one"
            );
        }
    }
}

/// A value that only ever lived in the RAM cache must still reach the disk when the chunk
/// is saved for other reasons, otherwise the cache is rebuilt on every single load.
#[test]
fn saving_a_chunk_persists_a_value_that_only_lived_in_the_cache() {
    let chunk = shape("flat");
    let height = SkyLightHeight::compute_from_chunk(&chunk);
    chunk
        .sky_light_height_cache
        .store(height.raw(), Ordering::Relaxed);
    chunk.remove_custom_data(NAMESPACE, KEY);

    let bytes = chunk.to_bytes().expect("chunk serializes");
    let reloaded = ChunkData::from_bytes(&bytes, Vector2::new(0, 0)).expect("chunk parses");

    assert_eq!(
        SkyLightHeightMigration::load_persisted(&reloaded),
        Some(height),
        "the cached cut height did not survive the save"
    );
}

/// The full round trip, geometry tag included: what comes back has to be usable without
/// recomputing, and it must not have displaced anyone else's custom data on the way.
#[test]
fn the_cut_height_survives_a_round_trip_beside_other_custom_data() {
    let chunk = shape("flat");
    chunk.set_custom_data("my_plugin", "counter", NbtTag::Int(7));
    let height =
        SkyLightHeightMigration::ensure_lazy(&chunk, || SkyLightHeight::compute_from_chunk(&chunk));

    let bytes = chunk.to_bytes().expect("chunk serializes");
    let reloaded = ChunkData::from_bytes(&bytes, Vector2::new(0, 0)).expect("chunk parses");

    let mut computed = 0;
    let after = SkyLightHeightMigration::ensure_lazy(&reloaded, || {
        computed += 1;
        SkyLightHeight::compute_from_chunk(&reloaded)
    });

    assert_eq!(computed, 0, "the reloaded chunk recomputed a stored value");
    assert_eq!(after, height);
    assert_eq!(
        reloaded.get_custom_data("my_plugin", "counter"),
        Some(NbtTag::Int(7)),
        "persisting the cut height displaced other custom data"
    );
}

/// Worldgen
fn build_proto(shape: &Shape) -> ProtoChunk {
    use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::world_seed::Seed;

    let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
        Seed(42),
        Dimension::OVERWORLD,
    )));
    let mut proto = ProtoChunk::new(0, 0, &world_gen);

    if let Some(top) = shape.top {
        for x in 0..16i32 {
            for z in 0..16i32 {
                let column_top = top(x, z);
                for y in (column_top - 5)..=column_top {
                    proto.set_block_state(x, y, z, Block::STONE.default_state);
                }
            }
        }
    }
    if let Some((x, z, top)) = shape.tower {
        for y in 61..=top {
            proto.set_block_state(x as i32, y, z as i32, Block::STONE.default_state);
        }
    }
    if let Some(y) = shape.glass_roof {
        for x in 0..16i32 {
            for z in 0..16i32 {
                proto.set_block_state(x, y, z, Block::GLASS.default_state);
            }
        }
    }
    if let Some((x, z)) = shape.shaft {
        for y in MIN_Y..=200 {
            proto.set_block_state(x as i32, y, z as i32, Block::AIR.default_state);
        }
    }

    proto
}

/// Worldgen derives the value from the `ProtoChunk` heightmap, the runtime from the
/// finished chunk. They have to agree on every shape, otherwise a chunk starts life with
/// a different band than it would recompute, and the difference surfaces much later as
/// wrong light.
#[test]
fn worldgen_and_runtime_derive_the_same_value() {
    for shape in SHAPES {
        // A generated chunk is never blockless; that shape exists for the runtime only.
        if shape.top.is_none() {
            continue;
        }
        let from_proto = SkyLightHeight::compute_from_proto(&build_proto(shape));
        let from_chunk = SkyLightHeight::compute_from_chunk(&build_chunk(shape));

        assert_eq!(
            from_proto.decode(MIN_Y, HEIGHT),
            from_chunk.decode(MIN_Y, HEIGHT),
            "{}: worldgen and runtime disagree about the cut",
            shape.name
        );
        assert_eq!(
            from_proto.spread(),
            from_chunk.spread(),
            "{}: worldgen and runtime chose different bands",
            shape.name
        );
        for x in 0..16 {
            for z in 0..16 {
                assert_eq!(
                    from_proto.quadrant_uses_limit(x, z),
                    from_chunk.quadrant_uses_limit(x, z),
                    "{}: worldgen and runtime disagree about quadrant ({x}, {z})",
                    shape.name
                );
            }
        }
    }
}

/// The generated value has to reach the level chunk, so the first sky light access in the
/// running world finds it instead of deriving it again, and the upgrade must not drop
/// the rest of the chunk's custom data on the way. Unlike the cut height, plugin data is
/// the only copy.
#[test]
fn the_worldgen_value_reaches_the_level_chunk_with_the_rest_of_the_custom_data() {
    use crate::chunk_system::chunk_state::Chunk;
    use pumpkin_config::lighting::LightingEngineConfig;
    use pumpkin_data::dimension::Dimension;

    let mut proto = build_proto(&SHAPES[0]);
    proto.sky_light_height = SkyLightHeight::encode(60, MIN_Y, HEIGHT).raw();
    proto.custom_data.put("my_plugin", {
        let mut namespace = pumpkin_nbt::compound::NbtCompound::new();
        namespace.put("counter", NbtTag::Int(7));
        NbtTag::Compound(namespace)
    });

    let mut chunk = Chunk::Proto(Box::new(proto));
    chunk.upgrade_to_level_chunk(&Dimension::OVERWORLD, &LightingEngineConfig::Default);
    let Chunk::Level(level) = chunk else {
        panic!("upgrade did not produce a level chunk");
    };

    assert_eq!(
        level.sky_light_height_cache.load(Ordering::Relaxed),
        SkyLightHeight::encode(60, MIN_Y, HEIGHT).raw(),
        "the generated cut height was dropped during the upgrade"
    );
    assert!(
        SkyLightHeightMigration::fast_load_flag(&level),
        "the generated cut height was not persisted"
    );
    assert_eq!(
        level.get_custom_data("my_plugin", "counter"),
        Some(NbtTag::Int(7)),
        "plugin data was erased by the upgrade"
    );
}
