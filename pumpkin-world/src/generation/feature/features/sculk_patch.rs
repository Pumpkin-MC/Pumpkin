use pumpkin_data::BlockStateId;
use pumpkin_data::tag::Block::MINECRAFT_SCULK_REPLACEABLE_WORLD_GEN;

use pumpkin_data::{
    Block, BlockId, BlockState,
    block_properties::{BlockProperties, GlowLichenLikeProperties, SculkShriekerLikeProperties},
};

use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;

pub struct SculkPatchFeature {
    pub charge_count: i32,
    pub amount_per_charge: i32,
    pub spread_attempts: i32,
    pub growth_rounds: i32,
    pub spread_rounds: i32,
    pub extra_rare_growths: IntProvider,
    pub catalyst_chance: f32,
}

impl SculkPatchFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        _block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !Self::can_spread_from(chunk, pos) {
            return false;
        }

        let mut spreader = SculkSpreader::new();
        let total_rounds = self.spread_rounds + self.growth_rounds;

        for round in 0..total_rounds {
            for _ in 0..self.charge_count {
                spreader.add_cursor(pos, self.amount_per_charge);
            }

            let spread_veins = round < self.spread_rounds;

            for _ in 0..self.spread_attempts {
                spreader.update_cursors(chunk, random, spread_veins);
            }

            spreader.clear();
        }

        let below = pos.down();
        if random.next_f32() <= self.catalyst_chance
            && GenerationCache::get_block_state(chunk, &below.0)
                .to_state()
                .is_solid()
        {
            chunk.set_block_state(&pos.0, Block::SCULK_CATALYST.default_state);
        }

        let extra_growths = self.extra_rare_growths.get(random);
        for _ in 0..extra_growths {
            let candidate = pos.offset(Vector3::new(
                random.next_bounded_i32(5) - 2,
                0,
                random.next_bounded_i32(5) - 2,
            ));
            let state = GenerationCache::get_block_state(chunk, &candidate.0).to_state();
            let below_candidate = candidate.down();
            let below_state =
                GenerationCache::get_block_state(chunk, &below_candidate.0).to_state();

            if state.is_air() && below_state.is_side_solid(pumpkin_data::BlockDirection::Up) {
                chunk.set_block_state(&candidate.0, ancient_city_shrieker_state());
            }
        }

        true
    }

    pub fn generate_in_proto_chunk(
        &self,
        chunk: &mut crate::ProtoChunk,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !can_spread_from_proto_chunk(chunk, pos) {
            return false;
        }

        let mut spreader = SculkSpreader::new();
        let total_rounds = self.spread_rounds + self.growth_rounds;
        for round in 0..total_rounds {
            for _ in 0..self.charge_count {
                spreader.add_cursor(pos, self.amount_per_charge);
            }
            for _ in 0..self.spread_attempts {
                spreader.update_cursors_in_proto_chunk(chunk, random, round < self.spread_rounds);
            }
            spreader.clear();
        }

        let below = pos.down();
        if random.next_f32() <= self.catalyst_chance
            && proto_chunk_state(chunk, below).is_some_and(|state| state.to_state().is_solid())
        {
            set_proto_chunk_state(chunk, pos, Block::SCULK_CATALYST.default_state);
        }

        for _ in 0..self.extra_rare_growths.get(random) {
            let candidate = pos.offset(Vector3::new(
                random.next_bounded_i32(5) - 2,
                0,
                random.next_bounded_i32(5) - 2,
            ));
            let below = candidate.down();
            if proto_chunk_state(chunk, candidate).is_some_and(|state| state.to_state().is_air())
                && proto_chunk_state(chunk, below).is_some_and(|state| {
                    state
                        .to_state()
                        .is_side_solid(pumpkin_data::BlockDirection::Up)
                })
            {
                set_proto_chunk_state(chunk, candidate, ancient_city_shrieker_state());
            }
        }

        true
    }

    fn can_spread_from<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
        let state = GenerationCache::get_block_state(chunk, &pos.0);
        let block_id = state.to_block_id();
        if is_sculk_behaviour(block_id) {
            true
        } else {
            if !chunk.is_air(&pos.0) && block_id != Block::WATER.id {
                return false;
            }
            // Check if any neighbor is solid
            for neighbor in [
                pos.offset(Vector3::new(1, 0, 0)),
                pos.offset(Vector3::new(-1, 0, 0)),
                pos.offset(Vector3::new(0, 1, 0)),
                pos.offset(Vector3::new(0, -1, 0)),
                pos.offset(Vector3::new(0, 0, 1)),
                pos.offset(Vector3::new(0, 0, -1)),
            ] {
                if GenerationCache::get_block_state(chunk, &neighbor.0)
                    .to_state()
                    .is_solid()
                {
                    return true;
                }
            }
            false
        }
    }
}

const fn is_sculk_behaviour(id: BlockId) -> bool {
    matches!(
        id,
        BlockId::SCULK
            | BlockId::SCULK_VEIN
            | BlockId::SCULK_CATALYST
            | BlockId::SCULK_SHRIEKER
            | BlockId::SCULK_SENSOR
            | BlockId::CALIBRATED_SCULK_SENSOR
    )
}

fn is_sculk_replaceable(id: BlockId) -> bool {
    id.has_tag(MINECRAFT_SCULK_REPLACEABLE_WORLD_GEN)
}

fn ancient_city_shrieker_state() -> &'static BlockState {
    let mut properties = SculkShriekerLikeProperties::default(&Block::SCULK_SHRIEKER);
    properties.r#can_summon = true;
    BlockState::from_id(properties.to_state_id(&Block::SCULK_SHRIEKER))
}

/// Resolves the sculk-vein block state to place at a position so that it clings to a sturdy
/// neighbour on `face` (the direction from the vein position toward that neighbour). Existing
/// sculk-vein states are merged; replaceable/air/water blocks become a fresh vein.
fn sculk_vein_state_with_face(
    existing: BlockStateId,
    face: pumpkin_data::BlockDirection,
) -> Option<&'static BlockState> {
    let existing_block_id = existing.to_block_id();
    let is_vein = existing_block_id == BlockId::SCULK_VEIN;
    if !(is_vein || is_sculk_replaceable(existing_block_id) || existing_block_id == Block::WATER.id)
    {
        return None;
    }

    let mut properties = if is_vein {
        GlowLichenLikeProperties::from_state_id(existing, &Block::SCULK_VEIN)
    } else {
        let mut properties = GlowLichenLikeProperties::default(&Block::SCULK_VEIN);
        properties.r#waterlogged = existing_block_id == BlockId::WATER;
        properties
    };

    match face {
        pumpkin_data::BlockDirection::Down => properties.r#down = true,
        pumpkin_data::BlockDirection::Up => properties.r#up = true,
        pumpkin_data::BlockDirection::North => properties.r#north = true,
        pumpkin_data::BlockDirection::South => properties.r#south = true,
        pumpkin_data::BlockDirection::West => properties.r#west = true,
        pumpkin_data::BlockDirection::East => properties.r#east = true,
    }

    Some(BlockState::from_id(
        properties.to_state_id(&Block::SCULK_VEIN),
    ))
}

struct Cursor {
    pos: BlockPos,
    charge: i32,
}

struct SculkSpreader {
    cursors: Vec<Cursor>,
}

impl SculkSpreader {
    const fn new() -> Self {
        Self {
            cursors: Vec::new(),
        }
    }

    fn add_cursor(&mut self, pos: BlockPos, charge: i32) {
        self.cursors.push(Cursor { pos, charge });
    }

    fn clear(&mut self) {
        self.cursors.clear();
    }

    fn update_cursors<T: GenerationCache>(
        &mut self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        spread_veins: bool,
    ) {
        let mut next_cursors = Vec::new();
        for mut cursor in self.cursors.drain(..) {
            if cursor.charge <= 0 {
                continue;
            }

            // In world gen, it picks one of 26 neighbors
            let dx = random.next_bounded_i32(3) - 1;
            let dy = random.next_bounded_i32(3) - 1;
            let dz = random.next_bounded_i32(3) - 1;
            let target_pos = cursor.pos.offset(Vector3::new(dx, dy, dz));

            let target_state = GenerationCache::get_block_state(chunk, &target_pos.0);
            let target_block_id = target_state.to_block_id();

            if is_sculk_replaceable(target_block_id) {
                chunk.set_block_state(&target_pos.0, Block::SCULK.default_state);
                if spread_veins {
                    grow_sculk_veins(chunk, target_pos);
                }
                cursor.pos = target_pos;
                cursor.charge -= 1;
            } else if target_block_id == Block::SCULK.id {
                cursor.pos = target_pos;
                cursor.charge -= 1;
            }

            if cursor.charge > 0 {
                next_cursors.push(cursor);
            }
        }
        self.cursors = next_cursors;
    }

    fn update_cursors_in_proto_chunk(
        &mut self,
        chunk: &mut crate::ProtoChunk,
        random: &mut RandomGenerator,
        spread_veins: bool,
    ) {
        let mut next_cursors = Vec::new();
        for mut cursor in self.cursors.drain(..) {
            if cursor.charge <= 0 {
                continue;
            }

            let target_pos = cursor.pos.offset(Vector3::new(
                random.next_bounded_i32(3) - 1,
                random.next_bounded_i32(3) - 1,
                random.next_bounded_i32(3) - 1,
            ));
            let Some(target_state) = proto_chunk_state(chunk, target_pos) else {
                continue;
            };
            let target_block_id = target_state.to_block_id();
            if is_sculk_replaceable(target_block_id) {
                set_proto_chunk_state(chunk, target_pos, Block::SCULK.default_state);
                if spread_veins {
                    grow_sculk_veins_in_proto_chunk(chunk, target_pos);
                }
                cursor.pos = target_pos;
                cursor.charge -= 1;
            } else if target_block_id == BlockId::SCULK {
                cursor.pos = target_pos;
                cursor.charge -= 1;
            }

            if cursor.charge > 0 {
                next_cursors.push(cursor);
            }
        }
        self.cursors = next_cursors;
    }
}

fn proto_chunk_state(chunk: &crate::ProtoChunk, pos: BlockPos) -> Option<BlockStateId> {
    ((pos.0.x >> 4) == chunk.x && (pos.0.z >> 4) == chunk.z).then(|| chunk.get_block_state(&pos.0))
}

fn set_proto_chunk_state(
    chunk: &mut crate::ProtoChunk,
    pos: BlockPos,
    state: &'static pumpkin_data::BlockState,
) {
    if (pos.0.x >> 4) == chunk.x && (pos.0.z >> 4) == chunk.z {
        chunk.set_block_state(pos.0.x, pos.0.y, pos.0.z, state);
    }
}

fn grow_sculk_veins<T: GenerationCache>(chunk: &mut T, sculk_pos: BlockPos) {
    for dir in pumpkin_data::BlockDirection::all() {
        let vein_pos = sculk_pos.offset(dir.to_offset());
        let existing = GenerationCache::get_block_state(chunk, &vein_pos.0);
        if let Some(state) = sculk_vein_state_with_face(existing, dir.opposite()) {
            chunk.set_block_state(&vein_pos.0, state);
        }
    }
}

fn grow_sculk_veins_in_proto_chunk(chunk: &mut crate::ProtoChunk, sculk_pos: BlockPos) {
    for dir in pumpkin_data::BlockDirection::all() {
        let vein_pos = sculk_pos.offset(dir.to_offset());
        let Some(existing) = proto_chunk_state(chunk, vein_pos) else {
            continue;
        };
        if let Some(state) = sculk_vein_state_with_face(existing, dir.opposite()) {
            set_proto_chunk_state(chunk, vein_pos, state);
        }
    }
}

fn can_spread_from_proto_chunk(chunk: &crate::ProtoChunk, pos: BlockPos) -> bool {
    let Some(state) = proto_chunk_state(chunk, pos) else {
        return false;
    };
    let block_id = state.to_block_id();
    if is_sculk_behaviour(block_id) {
        return true;
    }
    if !state.to_state().is_air() && block_id != Block::WATER.id {
        return false;
    }

    [
        Vector3::new(1, 0, 0),
        Vector3::new(-1, 0, 0),
        Vector3::new(0, 1, 0),
        Vector3::new(0, -1, 0),
        Vector3::new(0, 0, 1),
        Vector3::new(0, 0, -1),
    ]
    .into_iter()
    .any(|offset| {
        proto_chunk_state(chunk, pos.offset(offset))
            .is_some_and(|state| state.to_state().is_solid())
    })
}
