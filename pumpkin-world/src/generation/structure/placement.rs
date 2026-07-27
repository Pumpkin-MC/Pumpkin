use pumpkin_data::structures::{
    ConcentricRingsStructurePlacement, FrequencyReductionMethod, RandomSpreadStructurePlacement,
    SpreadType, StructurePlacement, StructurePlacementCalculator, StructurePlacementType,
};
use pumpkin_util::{
    math::floor_div,
    random::{RandomGenerator, RandomImpl, get_region_seed, legacy_rand::LegacyRand},
};
use std::f64::consts::PI;
use std::hash::Hash;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crate::ProtoChunk;
use dashmap::DashMap;
use pumpkin_data::structures::StructureKeys;

use super::structures::{StructurePosition, create_chunk_random};

/// How many memoized structure starts may stay resident.
///
/// # Deviation from vanilla
///
/// Vanilla has no global structure-start memo at all: starts are owned by the
/// `ChunkHolder` and die with the chunk, and `StructureCheck.loadedChunks`
/// (`/root/Vanilla/src/net/minecraft/world/level/levelgen/structure/StructureCheck.java:74`)
/// caches only per-chunk reference *counts* (`Object2IntMap<Structure>`), never
/// piece lists. Pumpkin memoizes the starts themselves because its reference
/// scan recomputes a start for every neighbouring chunk that overlaps it, which
/// would re-run the full jigsaw expansion many times over.
///
/// The bound is therefore Pumpkin's own, not a vanilla constant. A start is only
/// useful while chunks within 8 of it are still generating
/// (`ChunkGenerator.createReferences`, the +-8 window mirrored in
/// `proto_chunk/structures.rs`), i.e. one 17x17 chunk neighbourhood per start,
/// so a few hundred entries covers every start any in-flight chunk can reach.
const MAX_STRUCTURE_STARTS: usize = 512;

/// How many entries a trim leaves behind.
///
/// Trimming below the bound rather than exactly to it amortises the cost: the
/// next `MAX_STRUCTURE_STARTS - KEEP_STRUCTURE_STARTS` insertions are free.
const KEEP_STRUCTURE_STARTS: usize = MAX_STRUCTURE_STARTS * 3 / 4;

/// Returns the stamp to retain from, keeping the `keep` newest entries.
///
/// `stamps` is reordered in place. `None` means nothing needs evicting.
fn retain_threshold(stamps: &mut [u64], keep: usize) -> Option<u64> {
    if stamps.len() <= keep {
        return None;
    }
    // The keep-th newest stamp sits at `len - keep` once partially ordered.
    let index = stamps.len() - keep;
    let (_, threshold, _) = stamps.select_nth_unstable(index);
    Some(*threshold)
}

/// A capacity-bounded concurrent memo that discards its oldest entries.
///
/// Entries are stamped with a monotonic counter on insert; a trim keeps the
/// newest [`KEEP_STRUCTURE_STARTS`] and drops the rest. Reads never block on the
/// trim beyond a single `DashMap` shard, which matters because this is hit from
/// the rayon generation pool.
struct BoundedCache<K, V> {
    entries: DashMap<K, (u64, V)>,
    clock: AtomicU64,
    /// Guards the trim so concurrent inserters do not all scan the map.
    trimming: AtomicBool,
}

impl<K: Eq + Hash, V> Default for BoundedCache<K, V> {
    fn default() -> Self {
        Self {
            entries: DashMap::new(),
            clock: AtomicU64::new(0),
            trimming: AtomicBool::new(false),
        }
    }
}

impl<K: Eq + Hash, V: Clone> BoundedCache<K, V> {
    /// Returns the memoized value for `key`, computing and storing it on a miss.
    ///
    /// Two threads racing on the same missing key may both run `compute`; the
    /// last write wins. That is harmless here because the computation is a pure
    /// function of the key and the world seed.
    fn get_or_insert_with(&self, key: K, compute: impl FnOnce() -> V) -> V {
        // Clone out of the guard immediately: holding a `Ref` across the trim
        // below would deadlock against our own `retain`.
        let cached = self.entries.get(&key).map(|entry| entry.value().1.clone());
        if let Some(value) = cached {
            return value;
        }

        let computed = compute();
        let stamp = self.clock.fetch_add(1, Ordering::Relaxed);
        self.entries.insert(key, (stamp, computed.clone()));
        self.trim();
        computed
    }

    fn trim(&self) {
        if self.entries.len() <= MAX_STRUCTURE_STARTS {
            return;
        }
        // Another thread is already trimming; the bound is soft, so skip.
        if self.trimming.swap(true, Ordering::Acquire) {
            return;
        }
        let mut stamps: Vec<u64> = self.entries.iter().map(|entry| entry.value().0).collect();
        if let Some(threshold) = retain_threshold(&mut stamps, KEEP_STRUCTURE_STARTS) {
            self.entries.retain(|_, (stamp, _)| *stamp >= threshold);
        }
        self.trimming.store(false, Ordering::Release);
    }
}

/// A thread-safe global cache for structures that require world-wide placement calculations
/// rather than localized chunk-based math (e.g., Strongholds using Concentric Rings).
///
/// This prevents chunk generation deadlocks by allowing chunks to query a pre-calculated
/// mathematical layout in `O(1)` time instead of triggering cascading chunk loads.
pub struct GlobalStructureCache {
    /// A cached list of mathematically predicted (`chunk_x`, `chunk_z`) coordinates.
    stronghold_chunks: OnceLock<Vec<(i32, i32)>>,
    /// Memoized structure starts, keyed by (structure, start chunk x, start chunk z).
    ///
    /// A jigsaw structure's placement is fully determined by its start chunk and the
    /// world seed, so it is computed once here instead of being recomputed for every
    /// surrounding chunk whose structure references overlap it.
    ///
    /// Bounded at [`MAX_STRUCTURE_STARTS`] — each value owns a
    /// `StructurePiecesCollector` with up to hundreds of boxed pieces, and this
    /// cache lives as long as the generator (i.e. the whole server run). See the
    /// constant for why vanilla needs no such bound.
    structure_starts: OnceLock<BoundedCache<(StructureKeys, i32, i32), Option<StructurePosition>>>,
}
impl GlobalStructureCache {
    /// Creates a new, empty global structure cache.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            stronghold_chunks: OnceLock::new(),
            structure_starts: OnceLock::new(),
        }
    }

    pub fn get_stronghold_chunks(&self) -> &[(i32, i32)] {
        self.stronghold_chunks
            .get()
            .map_or(&[], std::vec::Vec::as_slice)
    }

    /// Returns the memoized structure start for the given structure and start chunk,
    /// computing it via `compute` on the first request and caching the result.
    ///
    /// Because a structure's placement depends only on its start chunk and the world
    /// seed, every chunk whose references overlap that structure can reuse the cached
    /// result instead of re-running the expensive jigsaw expansion.
    pub fn get_or_compute_structure_start(
        &self,
        key: StructureKeys,
        chunk_x: i32,
        chunk_z: i32,
        compute: impl FnOnce() -> Option<StructurePosition>,
    ) -> Option<StructurePosition> {
        self.structure_starts
            .get_or_init(BoundedCache::default)
            .get_or_insert_with((key, chunk_x, chunk_z), compute)
    }

    /// Retrieves the list of chunk coordinates for Concentric Ring structures.
    /// If the cache is empty, it calculates the 128 ring positions mathematically.
    #[allow(clippy::cast_precision_loss)]
    pub fn get_or_calculate_strongholds(
        &self,
        seed: i64,
        placement: &ConcentricRingsStructurePlacement,
        biome_supplier: &ProtoChunk,
        allowed_biomes: &[u16],
    ) -> &[(i32, i32)] {
        self.stronghold_chunks.get_or_init(|| {
            let mut chunks = Vec::with_capacity(placement.count as usize);

            let distance_param = f64::from(placement.distance); // Usually 32
            let mut spread = placement.spread; // Usually 3
            let count = placement.count; // Usually 128

            // The random generator for stronghold placement is based on the world seed and a fixed salt.
            // This ensures that the stronghold layout is consistent across all worlds with the same seed.
            let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(seed as u64));

            // The initial angle includes a random rotation jitter for the whole world
            let mut angle = random.next_f64() * PI * 2.0;
            let mut position_in_circle = 0;
            let mut circle = 0;

            for i in 0..count {
                // 1. Distance Formula
                // dist = (4 * spacing) + (spacing * ring_index * 6) + (random_jitter)
                // The jitter is +/- (spacing * 1.25)
                let dist = 4.0 * distance_param
                    + distance_param * f64::from(circle) * 6.0
                    + (random.next_f64() - 0.5) * (distance_param * 2.5);

                let initial_x = (angle.cos() * dist + 0.5).floor() as i32;
                let initial_z = (angle.sin() * dist + 0.5).floor() as i32;

                // 2. RNG Forking
                // We must fork/split the random generator for the biome search.
                // This keeps the main angle/distance sequence identical across all worlds.
                let fork_seed = random.next_i64();

                let mut biome_search_generator =
                    RandomGenerator::Legacy(LegacyRand::from_seed(fork_seed as u64));

                // 3. Reservoir Sampling Biome Search
                // Strongholds search the entire 112-block square
                // and pick one valid location at random (Reservoir Sampling).
                let mut found_pos = None;
                let mut found_count = 0;

                let center_block_x = (initial_x << 4) + 8;
                let center_block_z = (initial_z << 4) + 8;
                let step = 4;
                let search_radius = 112;

                for dz in (-search_radius..=search_radius).step_by(step as usize) {
                    for dx in (-search_radius..=search_radius).step_by(step as usize) {
                        let test_x = center_block_x + dx;
                        let test_z = center_block_z + dz;

                        let biome = biome_supplier.get_biome(test_x, 0, test_z);

                        if allowed_biomes.contains(&(biome.id as u16)) {
                            found_count += 1;
                            // Reservoir sampling: Pick the Nth valid biome with 1/N probability
                            if found_pos.is_none()
                                || biome_search_generator.next_bounded_i32(found_count) == 0
                            {
                                found_pos = Some((test_x >> 4, test_z >> 4));
                            }
                        }
                    }
                }

                let (final_chunk_x, final_chunk_z) = found_pos.unwrap_or((initial_x, initial_z));
                chunks.push((final_chunk_x, final_chunk_z));

                // 4. Dynamic Ring Progression
                angle += (PI * 2.0) / f64::from(spread);
                position_in_circle += 1;

                if position_in_circle == spread {
                    circle += 1;
                    position_in_circle = 0;

                    spread += 2 * spread / (circle + 1);
                    spread = spread.min(count - i);
                    angle += random.next_f64() * PI * 2.0;
                }
            }
            chunks
        })
    }
}

impl Default for GlobalStructureCache {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
// #[expect(clippy::too_many_arguments)]
pub fn should_generate_structure(
    placement: &StructurePlacement,
    calculator: &StructurePlacementCalculator,
    chunk_x: i32,
    chunk_z: i32,
    global_cache: &GlobalStructureCache,
    biome_supplier: &ProtoChunk,
    allowed_biomes: &[u16],
) -> bool {
    is_start_chunk(
        &placement.placement_type,
        calculator,
        chunk_x,
        chunk_z,
        placement.salt,
        global_cache,
        biome_supplier,
        allowed_biomes,
    ) && apply_frequency_reduction(
        placement.frequency_reduction_method,
        calculator.seed,
        chunk_x,
        chunk_z,
        placement.salt,
        placement.frequency.unwrap_or(1.0),
    )
}

fn apply_frequency_reduction(
    method: Option<FrequencyReductionMethod>,
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
    frequency: f32,
) -> bool {
    if frequency >= 1.0 {
        return true;
    }

    let method = method.unwrap_or(FrequencyReductionMethod::Default);
    should_generate_frequency(method, seed, chunk_x, chunk_z, salt, frequency)
}

/// `WorldgenRandom.setLargeFeatureWithSalt` (WorldgenRandom.java l.77-80):
/// `x * 341873128712 + z * 132897987541 + seed + blend`, fed to
/// `LegacyRandomSource.setSeed` (the `from_seed` scramble).
const fn large_feature_with_salt_seed(seed: i64, x: i64, z: i64, blend: i64) -> u64 {
    x.wrapping_mul(341_873_128_712)
        .wrapping_add(z.wrapping_mul(132_897_987_541))
        .wrapping_add(seed)
        .wrapping_add(blend) as u64
}

/// The four vanilla frequency reducers (StructurePlacement.java l.97-129). All
/// of them run on `WorldgenRandom(new LegacyRandomSource(0))` — a Java LCG —
/// never on Xoroshiro, regardless of the world's RNG algorithm.
fn should_generate_frequency(
    method: FrequencyReductionMethod,
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
    frequency: f32,
) -> bool {
    match method {
        FrequencyReductionMethod::Default => {
            // probabilityReducer (l.97-101): setLargeFeatureWithSalt(seed, salt,
            // chunkX, chunkZ) — vanilla passes the SALT as the x multiplicand,
            // chunkX as z, and chunkZ as the additive blend term.
            let mut random = LegacyRand::from_seed(large_feature_with_salt_seed(
                seed,
                i64::from(salt),
                i64::from(chunk_x),
                i64::from(chunk_z),
            ));
            random.next_f32() < frequency
        }
        FrequencyReductionMethod::LegacyType1 => {
            // legacyPillagerOutpostReducer (l.115-122): cx = chunkX >> 4,
            // cz = chunkZ >> 4; setSeed((cx ^ cz << 4) ^ seed); nextInt();
            // nextInt(1 / frequency) == 0.
            let x = chunk_x >> 4;
            let z = chunk_z >> 4;
            let mut random = LegacyRand::from_seed((i64::from(x ^ z << 4) ^ seed) as u64);
            random.next_i32();
            random.next_bounded_i32((1.0 / frequency) as i32) == 0
        }
        FrequencyReductionMethod::LegacyType2 => {
            // legacyArbitrarySaltProbabilityReducer (l.109-113):
            // setLargeFeatureWithSalt(seed, chunkX, chunkZ, 10387320); nextFloat.
            let mut random = LegacyRand::from_seed(large_feature_with_salt_seed(
                seed,
                i64::from(chunk_x),
                i64::from(chunk_z),
                10_387_320,
            ));
            random.next_f32() < frequency
        }
        FrequencyReductionMethod::LegacyType3 => {
            // legacyProbabilityReducerWithDouble (l.103-107):
            // setLargeFeatureSeed(seed, chunkX, chunkZ) — the multiplier-XOR
            // form (WorldgenRandom.java l.69-75), identical to
            // create_chunk_random — then nextDouble() < frequency. Used by
            // minecraft:mineshafts (frequency 0.004).
            let mut random = create_chunk_random(seed, chunk_x, chunk_z);
            random.next_f64() < f64::from(frequency)
        }
    }
}

#[expect(clippy::too_many_arguments)]
fn is_start_chunk(
    placement_type: &StructurePlacementType,
    calculator: &StructurePlacementCalculator,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
    global_cache: &GlobalStructureCache,
    biome_supplier: &ProtoChunk,
    allowed_biomes: &[u16],
) -> bool {
    match placement_type {
        StructurePlacementType::RandomSpread(placement) => {
            is_start_chunk_random_spread(placement, calculator, chunk_x, chunk_z, salt)
        }
        StructurePlacementType::ConcentricRings(placement) => {
            let strongholds = global_cache.get_or_calculate_strongholds(
                calculator.seed,
                placement,
                biome_supplier,
                allowed_biomes,
            );
            strongholds.contains(&(chunk_x, chunk_z))
        }
    }
}

/// Predicts the exact chunk (X, Z) where a structure will attempt to spawn in a given Region (rx, rz).
#[must_use]
pub fn get_structure_chunk_in_region(
    placement: &RandomSpreadStructurePlacement,
    seed: i64,
    rx: i32,
    rz: i32,
    salt: u32,
) -> (i32, i32) {
    let region_seed = get_region_seed(seed as u64, rx, rz, salt);
    let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(region_seed));

    let bound = placement.spacing - placement.separation;
    let spread_type = placement.spread_type.unwrap_or(SpreadType::Linear);

    let rand_x = spread_type.get(&mut random, bound);
    let rand_z = spread_type.get(&mut random, bound);

    (
        rx * placement.spacing + rand_x,
        rz * placement.spacing + rand_z,
    )
}

fn get_start_chunk_random_spread(
    placement: &RandomSpreadStructurePlacement,
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
) -> (i32, i32) {
    // 1. Find the region
    let rx = floor_div(chunk_x, placement.spacing);
    let rz = floor_div(chunk_z, placement.spacing);

    // 2. Get the structure chunk for that region
    get_structure_chunk_in_region(placement, seed, rx, rz, salt)
}

fn is_start_chunk_random_spread(
    placement: &RandomSpreadStructurePlacement,
    calculator: &StructurePlacementCalculator,
    chunk_x: i32,
    chunk_z: i32,
    salt: u32,
) -> bool {
    let pos = get_start_chunk_random_spread(placement, calculator.seed, chunk_x, chunk_z, salt);
    (chunk_x == pos.0) && (chunk_z == pos.1)
}
#[cfg(test)]
mod tests {
    use pumpkin_data::structures::RandomSpreadStructurePlacement;
    use pumpkin_util::random::{
        RandomGenerator, RandomImpl, get_region_seed, legacy_rand::LegacyRand,
    };

    use crate::generation::structure::placement::{
        BoundedCache, KEEP_STRUCTURE_STARTS, MAX_STRUCTURE_STARTS, get_start_chunk_random_spread,
        retain_threshold,
    };

    #[test]
    fn retain_threshold_is_none_below_the_bound() {
        let mut stamps = [1, 2, 3];
        assert_eq!(retain_threshold(&mut stamps, 3), None);
        assert_eq!(retain_threshold(&mut stamps, 4), None);
    }

    #[test]
    fn retain_threshold_keeps_the_newest_entries() {
        let mut stamps = [5, 1, 4, 2, 3];
        // Keep 2 -> threshold must admit exactly {4, 5}.
        let threshold = retain_threshold(&mut stamps, 2).expect("over the bound");
        assert_eq!(threshold, 4);
        assert_eq!(
            [5, 1, 4, 2, 3].iter().filter(|s| **s >= threshold).count(),
            2
        );
    }

    #[test]
    fn bounded_cache_memoizes_and_computes_once() {
        let cache: BoundedCache<u32, u32> = BoundedCache::default();
        let mut calls = 0;
        assert_eq!(cache.get_or_insert_with(7, || 70), 70);
        cache.get_or_insert_with(7, || {
            calls += 1;
            0
        });
        assert_eq!(calls, 0, "second lookup must hit the memo");
    }

    #[test]
    fn bounded_cache_respects_its_capacity() {
        let cache: BoundedCache<u32, u32> = BoundedCache::default();
        let total = u32::try_from(MAX_STRUCTURE_STARTS * 3).expect("fits in u32");
        for key in 0..total {
            cache.get_or_insert_with(key, || key);
        }
        assert!(
            cache.entries.len() <= MAX_STRUCTURE_STARTS,
            "cache grew to {} entries",
            cache.entries.len()
        );
    }

    #[test]
    fn bounded_cache_evicts_the_oldest_keys_first() {
        let cache: BoundedCache<u32, u32> = BoundedCache::default();
        let total = u32::try_from(MAX_STRUCTURE_STARTS + 1).expect("fits in u32");
        for key in 0..total {
            cache.get_or_insert_with(key, || key);
        }
        // One trim has run, leaving the KEEP newest insertions.
        assert_eq!(cache.entries.len(), KEEP_STRUCTURE_STARTS);
        assert!(!cache.entries.contains_key(&0), "oldest key survived");
        assert!(
            cache.entries.contains_key(&(total - 1)),
            "newest key was evicted"
        );
    }

    #[test]
    fn get_start_chunk_random() {
        let region_seed = get_region_seed(123, 1, 1, 14357620);
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(region_seed));
        assert_eq!(random.next_bounded_i32(32 - 8), 8);
    }

    #[test]
    fn get_start_chunk() {
        let random = RandomSpreadStructurePlacement {
            spacing: 32,
            separation: 8,
            spread_type: None,
        };
        let (x, z) = get_start_chunk_random_spread(&random, 123, 1, 1, 14357620);
        assert_eq!(x, 5);
        assert_eq!(z, 4);
    }
}
