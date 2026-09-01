use pumpkin_data::noise_router::OVERWORLD_BASE_NOISE_ROUTER;
use pumpkin_util::math::vector3::Vector3;

use crate::generation::GlobalRandomConfig;
use crate::generation::noise::router::chunk_density_function::{
    ChunkNoiseFunctionBuilderOptions, ChunkNoiseFunctionSampleOptions, SampleAction,
};
use crate::generation::noise::router::chunk_noise_router::ChunkNoiseRouter;
use crate::generation::noise::router::proto_noise_router::ProtoNoiseRouters;

fn fnv1a_hash_f32(values: impl Iterator<Item = f32>) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01B3;
    let mut hash = FNV_OFFSET;
    for value in values {
        for byte in value.to_bits().to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash
}

/// This just detects if a mass set of hashes is the same as it was previously, should reliably detect regressions.
#[test]
fn overworld_density_fingerprint_is_stable() {
    let mut results = Vec::new();

    for seed in [0u64, 1, 42, 1_779_920_288_596_261_407] {
        let random_config = GlobalRandomConfig::new(seed, false);
        let proto_routers =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);

        let builder_options = ChunkNoiseFunctionBuilderOptions::new(
            4,
            8,
            48,
            4,
            0,
            0,
            4,
            Vec::new(),
            Vec::new(),
            None,
        );
        let mut router = ChunkNoiseRouter::generate(&proto_routers.noise, &builder_options);

        let options =
            ChunkNoiseFunctionSampleOptions::new(false, SampleAction::SkipCellCaches, 0, 0, 0);

        for x in (-64..64).step_by(11) {
            for y in (-64..320).step_by(19) {
                for z in (-64..64).step_by(13) {
                    let pos = Vector3::new(x, y, z);

                    results.push(router.final_density(&pos, &options));
                    results.push(router.vein_toggle(&pos, &options));
                    results.push(router.vein_ridged(&pos, &options));
                    results.push(router.vein_gap(&pos, &options));
                }
            }
        }
    }

    let hash = fnv1a_hash_f32(results.into_iter());
    assert_eq!(
        hash, 9_544_564_030_852_513_724,
        "Overworld density fingerprint changed"
    );
}
