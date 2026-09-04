//! TEMPORARY -- development scaffolding for the light engine work, not meant to ship.
//!
//! End-to-end tests for the sky light cut height inside the running light engine.
//!
//! The unit tests in `lighting::sky_light_height` prove the cached value is consistent
//! with the chunk it was derived from. These three prove the part that only exists once
//! blocks actually move: that a stale value is invalidated before it is used, and that the
//! cheap tier answers never suppress propagation that has to happen.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use pumpkin_config::world::LevelConfig;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::level::Level;
use pumpkin_world::lighting::sky_light_height::SkyLightTier;
use pumpkin_world::lighting::{DynamicLightEngine, SkyLightHeight, SkyLightHeightMigration};
use std::sync::Arc;
use tempfile::TempDir;

const SECTIONS: usize = 24;
const MIN_Y: i32 = -64;
const HEIGHT: i32 = SECTIONS as i32 * 16;
/// Top of the solid ground. Everything from y=0 up to here is stone.
const SURFACE: i32 = 60;

struct World {
    level: Arc<Level>,
    chunks: Vec<(Vector2<i32>, Arc<ChunkData>)>,
    engine: DynamicLightEngine,
    _dir: TempDir,
}

impl World {
    /// A single loaded chunk with solid ground up to `SURFACE`, plus the sky light state a
    /// converged engine would have left behind: 15 above every column's ceiling, 0 below.
    ///
    /// `carve` runs before the lighting is derived, so anything it opens is part of the
    /// terrain the cut height is computed from.
    fn new(carve: impl Fn(&mut Vec<(usize, i32, usize, BlockStateId)>)) -> Self {
        Self::spanning(&[(0, 0)], |_, _, updates| carve(updates))
    }

    /// The same, over several loaded chunks. `carve` is called per chunk with its chunk
    /// coordinates -> test can open a shaft in one and a tunnel in the next.
    ///
    /// Everything outside the given list stays unloaded
    fn spanning(
        positions: &[(i32, i32)],
        carve: impl Fn(i32, i32, &mut Vec<(usize, i32, usize, BlockStateId)>),
    ) -> Self {
        let dir = TempDir::new().unwrap();
        let level = Level::from_root_folder(
            &LevelConfig::default(),
            dir.path().to_path_buf(),
            42,
            Dimension::OVERWORLD,
        );

        let chunks = positions
            .iter()
            .map(|&(cx, cz)| {
                let chunk = Self::build_chunk(cx, cz, &carve);
                level
                    .loaded_chunks
                    .insert(Vector2::new(cx, cz), chunk.clone());
                (Vector2::new(cx, cz), chunk)
            })
            .collect();

        Self {
            level,
            chunks,
            engine: DynamicLightEngine::new(),
            _dir: dir,
        }
    }

    fn build_chunk(
        cx: i32,
        cz: i32,
        carve: &impl Fn(i32, i32, &mut Vec<(usize, i32, usize, BlockStateId)>),
    ) -> Arc<ChunkData> {
        let chunk = ChunkData::empty(cx, cz);
        let mut updates = Vec::new();
        for x in 0..16usize {
            for z in 0..16usize {
                for y in 0..=SURFACE {
                    updates.push((x, y, z, Block::STONE.default_state.id));
                }
            }
        }
        carve(cx, cz, &mut updates);
        chunk.set_blocks_batch(updates);
        *chunk
            .heightmap
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = chunk.calculate_heightmap();

        let mut light = chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        light.sky_light = (0..SECTIONS)
            .map(|_| LightContainer::new_empty(0))
            .collect();
        light.block_light = (0..SECTIONS)
            .map(|_| LightContainer::new_empty(0))
            .collect();
        for x in 0..16usize {
            for z in 0..16usize {
                let mut y = MIN_Y + HEIGHT - 1;
                while y >= MIN_Y {
                    let solid = chunk
                        .section
                        .get_block_absolute_y(x, y, z)
                        .is_some_and(|id| id != Block::AIR.default_state.id);
                    if solid {
                        break;
                    }
                    let relative = (y - MIN_Y) as usize;
                    light.sky_light[relative / 16].set(x, relative % 16, z, 15);
                    y -= 1;
                }
            }
        }
        drop(light);

        Arc::new(chunk)
    }

    fn chunk(&self) -> &Arc<ChunkData> {
        &self.chunks[0].1
    }

    fn chunk_at(&self, cx: i32, cz: i32) -> &Arc<ChunkData> {
        &self
            .chunks
            .iter()
            .find(|(pos, _)| pos.x == cx && pos.y == cz)
            .expect("that chunk was never loaded")
            .1
    }

    /// Changes the block without telling the engine, as the caller does when
    /// [`LightEngine::has_different_light_properties`] says no.
    fn set_block_unannounced(&self, pos: BlockPos, id: BlockStateId) {
        let chunk = self.chunk_at(pos.0.x >> 4, pos.0.z >> 4);
        chunk.set_block_absolute_y(
            (pos.0.x & 15) as usize,
            pos.0.y,
            (pos.0.z & 15) as usize,
            id,
        );
    }

    fn set_block(&self, pos: BlockPos, id: BlockStateId) {
        let chunk = self.chunk_at(pos.0.x >> 4, pos.0.z >> 4);
        chunk.set_block_absolute_y(
            (pos.0.x & 15) as usize,
            pos.0.y,
            (pos.0.z & 15) as usize,
            id,
        );
        self.engine.update_lighting_at(&self.level, pos);
    }

    fn settle(&self) {
        let converged = (0..2000).any(|_| !self.engine.drain_queued(&self.level).leftover);
        assert!(converged, "light updates did not converge");
    }

    fn sky(&self, x: i32, y: i32, z: i32) -> u8 {
        self.engine
            .get_sky_light_level(&self.level, &BlockPos::new(x, y, z))
    }

    fn cut(&self) -> SkyLightHeight {
        SkyLightHeightMigration::get(self.chunk())
    }
}

fn air() -> BlockStateId {
    Block::AIR.default_state.id
}

/// Digging under an intact surface must not invalidate anything, and the tunnel must end
/// up correctly lit from the side.
#[tokio::test]
async fn digging_under_an_intact_surface_keeps_the_cut_and_still_lights_the_tunnel() {
    // A shaft in NW, deep enough that its quadrant diverges from the start. The tunnel is
    // then dug into NE, which keeps the fast path.
    let world = World::new(|updates| {
        for y in 19..=SURFACE {
            updates.push((2, y, 2, air()));
        }
    });

    let before = world.cut();
    assert!(
        !before.quadrant_uses_limit(2, 2),
        "the shaft's own quadrant should have diverged during the initial derivation"
    );
    assert!(
        before.quadrant_uses_limit(10, 2),
        "NE was untouched and must keep the fast path"
    );
    assert_eq!(
        before.tier(19, 10, 2, MIN_Y, HEIGHT),
        SkyLightTier::NoOpenSky,
        "the tunnel level in NE has to sit in tier 1, or this test proves nothing"
    );
    assert_eq!(world.sky(2, 19, 2), 15, "the shaft is open to the sky");

    for x in 3..=10 {
        world.set_block(BlockPos::new(x, 19, 2), air());
    }
    world.settle();

    let after = world.cut();
    assert_eq!(
        after, before,
        "a tunnel under intact ceilings changed the cached cut height"
    );

    for x in 3..=10 {
        assert_eq!(
            world.sky(x, 19, 2),
            (15 - (x - 2)) as u8,
            "wrong sky light in the tunnel at x={x}: light entering from the shaft has to \
             lose exactly one level per block, whatever the cut height claims down here"
        );
    }
}

/// Opening a column to the sky is the change that really does invalidate: its ceiling
/// leaves the band. It has to cost exactly one quadrant, and the light has to arrive.
#[tokio::test]
async fn opening_a_column_to_the_sky_invalidates_only_its_own_quadrant() {
    let world = World::new(|_| {});

    let before = world.cut();
    for x in [2, 10] {
        for z in [2, 10] {
            assert!(
                before.quadrant_uses_limit(x, z),
                "flat terrain diverged at ({x}, {z}) before anything was dug"
            );
        }
    }

    // A trench in SW, deep enough to drop its ceiling out of the band.
    for y in (40..=SURFACE).rev() {
        world.set_block(BlockPos::new(3, y, 12), air());
    }
    world.settle();

    let after = world.cut();
    assert!(
        !after.quadrant_uses_limit(3, 12),
        "the trench's ceiling left the band, but its quadrant still promises the fast path"
    );
    for (x, z) in [(2, 2), (10, 2), (10, 10)] {
        assert!(
            after.quadrant_uses_limit(x, z),
            "the trench in SW degraded the untouched quadrant at ({x}, {z})"
        );
    }
    assert_eq!(
        world.sky(3, 45, 12),
        15,
        "the trench is open to the sky and must be fully lit"
    );

    let lone = BlockPos::new(10, 65, 2);
    assert_eq!(
        after.tier(65, 10, 2, MIN_Y, HEIGHT),
        SkyLightTier::OpenSky,
        "the cell picked for the relight check is not on the fast path"
    );
    for y in 61..(MIN_Y + HEIGHT) {
        world
            .engine
            .set_sky_light_level(&world.level, &BlockPos::new(10, y, 2), 0)
            .expect("the column is inside the loaded chunk");
    }
    world.engine.update_lighting_at(&world.level, lone);
    world.settle();
    assert_eq!(
        world.sky(10, 65, 2),
        15,
        "a blanked column under open sky came back short; the fast open-sky answer was \
         not the one the engine acted on"
    );
}

/// A newly built roof is the sharp case, and the reason the divergence flag has to exist.
#[tokio::test]
async fn a_new_roof_is_not_treated_as_open_sky() {
    let world = World::new(|_| {});
    let before = world.cut();
    assert_eq!(
        world.sky(8, 65, 8),
        15,
        "the room is open sky to begin with"
    );

    for x in 0..16 {
        for z in 0..16 {
            world.set_block(BlockPos::new(x, 70, z), Block::STONE.default_state.id);
        }
    }
    world.settle();

    assert_eq!(
        world.sky(8, 65, 8),
        0,
        "the room under the new roof is still sunlit"
    );

    let after = world.cut();
    assert!(
        !after.quadrant_uses_limit(8, 8),
        "the roof moved every ceiling out of the band, but the quadrant was not flagged"
    );
    assert_eq!(
        before.tier(65, 8, 8, MIN_Y, HEIGHT),
        SkyLightTier::OpenSky,
        "without the flag the stale value would not have claimed open sky here, so this \
         test would not prove the flag is load-bearing"
    );
    assert_eq!(
        after.tier(65, 8, 8, MIN_Y, HEIGHT),
        SkyLightTier::Unknown,
        "the flagged value still hands out a fast answer inside the sealed room"
    );
}

/// Light spreads into all six neighbours, not only the one direction the other tests
/// use.
///
/// The four propagation loops share a single neighbour walk. Every other test here is
/// essentially one-dimensional.
#[tokio::test]
async fn a_point_light_reaches_all_six_neighbours() {
    // A hollow pocket deep in the rock, so nothing but the source can light it.
    let world = World::new(|updates| {
        for x in 7..=9usize {
            for z in 7..=9usize {
                for y in 29..=31 {
                    updates.push((x, y, z, air()));
                }
            }
        }
    });

    let source = BlockPos::new(8, 30, 8);
    world.set_block(source, Block::GLOWSTONE.default_state.id);
    world.settle();

    let luminance = Block::GLOWSTONE.default_state.luminance;
    for (dx, dy, dz) in [
        (1, 0, 0),
        (-1, 0, 0),
        (0, 1, 0),
        (0, -1, 0),
        (0, 0, 1),
        (0, 0, -1),
    ] {
        let neighbor = BlockPos::new(8 + dx, 30 + dy, 8 + dz);
        assert_eq!(
            world
                .engine
                .get_block_light_level(&world.level, &neighbor)
                .expect("the pocket is inside the loaded chunk"),
            luminance - 1,
            "no light reached the neighbour at offset ({dx}, {dy}, {dz})"
        );
    }

    assert_eq!(
        world
            .engine
            .get_block_light_level(&world.level, &BlockPos::new(7, 29, 8))
            .expect("the pocket is inside the loaded chunk"),
        luminance - 2,
        "light did not carry on past the first ring"
    );
}

/// Light crosses a chunk border, and the edge of the loaded area does not stop it.
#[tokio::test]
async fn light_crosses_a_chunk_border_and_the_edge_of_the_loaded_area() {
    let world = World::spanning(&[(0, 0), (1, 0)], |cx, _cz, updates| {
        if cx == 0 {
            // A shaft down to the tunnel level, part of the terrain from the start.
            for y in 19..=SURFACE {
                updates.push((13, y, 0, air()));
            }
        }
    });

    assert_eq!(world.sky(13, 19, 0), 15, "the shaft is open to the sky");

    for x in (14..=20).rev() {
        world.set_block(BlockPos::new(x, 19, 0), air());
    }
    world.settle();

    for x in 14..=20 {
        assert_eq!(
            world.sky(x, 19, 0),
            (15 - (x - 13)) as u8,
            "sky light did not reach x={x}; it has to flow one block per step, across the \
             chunk border at x=16 and past the unloaded chunk to the north"
        );
    }

    let neighbour = SkyLightHeightMigration::get(world.chunk_at(1, 0));
    assert!(
        neighbour.quadrant_uses_limit(0, 0),
        "a tunnel under the intact ceilings of the next chunk invalidated its cut"
    );
}

/// The same block changes, by one thread or by four, have to reach the same fixpoint.
///
/// What keeps the lock-free check honest is not exclusion but the queue: every write is
/// followed by a queue entry, and the still-serialised flood re-derives from there. Each
/// thread owns whole chunks, so only the engine is shared.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_block_changes_converge_to_the_same_light() {
    const CHUNKS: i32 = 4;

    fn tunnel(cx: i32) -> Vec<BlockPos> {
        (3..=10)
            .rev()
            .map(|x| BlockPos::new(cx * 16 + x, 19, 0))
            .collect()
    }

    fn build() -> World {
        let positions: Vec<(i32, i32)> = (0..CHUNKS).map(|cx| (cx, 0)).collect();
        World::spanning(&positions, |_cx, _cz, updates| {
            for y in 19..=SURFACE {
                updates.push((2, y, 0, air()));
            }
        })
    }

    fn readings(world: &World) -> Vec<u8> {
        (0..CHUNKS)
            .flat_map(|cx| {
                (2..=10)
                    .map(|x| world.sky(cx * 16 + x, 19, 0))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    let sequential = {
        let world = build();
        for cx in 0..CHUNKS {
            for pos in tunnel(cx) {
                world.set_block(pos, air());
            }
        }
        world.settle();
        readings(&world)
    };

    let concurrent = {
        let world = build();
        std::thread::scope(|scope| {
            for cx in 0..CHUNKS {
                let world = &world;
                scope.spawn(move || {
                    for pos in tunnel(cx) {
                        world.set_block(pos, air());
                    }
                });
            }
        });
        world.settle();
        readings(&world)
    };

    assert_eq!(
        concurrent, sequential,
        "eight threads reached a different fixpoint than one"
    );
    assert!(
        sequential.iter().any(|&light| light > 0),
        "the workload lit nothing, so the comparison proves nothing"
    );
}

#[tokio::test]
async fn skipping_a_light_neutral_change_leaves_the_same_light() {
    // Under the floor of a lit tunnel: another opaque block changes nothing, air lets
    // the light drop in.
    const WALL: BlockPos = BlockPos::new(5, 18, 0);

    fn lit_world() -> World {
        let world = World::spanning(&[(0, 0)], |_, _, updates| {
            for y in 19..=SURFACE {
                updates.push((2, y, 0, air()));
            }
        });
        for x in (3..=10).rev() {
            world.set_block(BlockPos::new(x, 19, 0), air());
        }
        world.settle();
        world
    }

    fn readings(world: &World) -> Vec<u8> {
        (2..=10)
            .flat_map(|x| [world.sky(x, 19, 0), world.sky(x, 18, 0)])
            .collect()
    }

    for (name, replacement, must_match) in [
        ("stone -> dirt", Block::DIRT.default_state.id, true),
        ("stone -> air", air(), false),
    ] {
        let announced = {
            let world = lit_world();
            world.set_block(WALL, replacement);
            world.settle();
            readings(&world)
        };
        let skipped = {
            let world = lit_world();
            world.set_block_unannounced(WALL, replacement);
            world.settle();
            readings(&world)
        };

        if must_match {
            assert_eq!(
                skipped, announced,
                "{name} is light neutral, so skipping the engine must change nothing"
            );
        } else {
            assert_ne!(
                skipped, announced,
                "{name} does move light, so the two worlds have to differ -- otherwise \
                 this comparison could not tell the neutral case apart either"
            );
        }
    }
}
