//! Driver for the sculk-patch configured feature.
//!
//! This module keeps the public struct / field layout required by the
//! codegen (`configured_features_generated.rs`) and the two generation entry
//! points (`generate` / `generate_in_proto_chunk`). The actual spreading
//! algorithm lives in the [`sculk`] sub-module.

use pumpkin_data::Block;
use pumpkin_util::math::int_provider::IntProvider;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::RandomGenerator;
use pumpkin_util::random::RandomImpl;

use crate::generation::feature::features::sculk;
use crate::generation::feature::features::sculk::ProtoChunkSculkView;
use crate::generation::feature::features::sculk::SculkLevel;
use crate::generation::feature::features::sculk::spreader::SculkSpreader;
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
    /// Post-terrain generation entry point. `T: GenerationCache` also
    /// implements `SculkLevel` via the blanket impl.
    pub fn generate<T: GenerationCache>(
        &self,
        _block_registry: &dyn WorldPortalExt,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !sculk::can_spread_from(chunk, pos) {
            return false;
        }

        let mut spreader = SculkSpreader::new_world_gen_spreader();
        let total_rounds = self.spread_rounds + self.growth_rounds;

        for round in 0..total_rounds {
            for _ in 0..self.charge_count {
                spreader.add_cursors(pos, self.amount_per_charge);
            }

            for _ in 0..self.spread_attempts {
                spreader.update_cursors(chunk, pos, random, round < self.spread_rounds);
            }

            spreader.clear();
        }

        // Catalyst placement (vanilla: block below must be a full cube).
        if random.next_f32() <= self.catalyst_chance && chunk.sculk_is_full_cube(pos.down()) {
            chunk.sculk_set(pos, Block::SCULK_CATALYST.default_state);
        }

        // Extra-rare growths (shriekers, only for ancient cities).
        self.place_extra_rare_growths(chunk, random, pos);

        true
    }

    /// In-chunk (proto-chunk) generation entry point used during terrain.
    pub fn generate_in_proto_chunk(
        &self,
        chunk: &mut crate::ProtoChunk,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut view = ProtoChunkSculkView::new(chunk);
        if !sculk::can_spread_from(&view, pos) {
            return false;
        }

        let mut spreader = SculkSpreader::new_world_gen_spreader();
        let total_rounds = self.spread_rounds + self.growth_rounds;
        for round in 0..total_rounds {
            for _ in 0..self.charge_count {
                spreader.add_cursors(pos, self.amount_per_charge);
            }
            for _ in 0..self.spread_attempts {
                spreader.update_cursors(&mut view, pos, random, round < self.spread_rounds);
            }
            spreader.clear();
        }

        if random.next_f32() <= self.catalyst_chance && view.sculk_is_full_cube(pos.down()) {
            view.sculk_set(pos, Block::SCULK_CATALYST.default_state);
        }

        self.place_extra_rare_growths(&mut view, random, pos);

        true
    }

    /// Places extra-rare growths (sculk shriekers) around the origin.
    fn place_extra_rare_growths(
        &self,
        level: &mut dyn SculkLevel,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) {
        let extra_growths = self.extra_rare_growths.get(random);
        for _ in 0..extra_growths {
            let candidate = pos.offset(Vector3::new(
                random.next_bounded_i32(5) - 2,
                0,
                random.next_bounded_i32(5) - 2,
            ));
            if level.sculk_is_air(candidate)
                && level.sculk_is_face_sturdy(candidate.down(), pumpkin_data::BlockDirection::Up)
            {
                level.sculk_set(candidate, sculk::shrieker_state(true));
            }
        }
    }
}
