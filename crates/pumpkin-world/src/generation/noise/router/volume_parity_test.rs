use pumpkin_data::noise_router::OVERWORLD_BASE_NOISE_ROUTER;

use crate::generation::GlobalRandomConfig;
use crate::generation::noise::router::chunk_density_function::{
    ChunkNoiseFunctionBuilderOptions, ChunkSpecificNoiseFunctionComponent,
};
use crate::generation::noise::router::chunk_noise_router::{
    ChunkNoiseFunctionComponent, ChunkNoiseRouter,
};
use crate::generation::noise::router::density_function::PassThrough;
use crate::generation::noise::router::density_volume::{DensityBuffer, DensityVolume};
use crate::generation::noise::router::proto_noise_router::{
    ProtoNoiseFunctionComponent, ProtoNoiseRouters,
};

fn pass_through_stack(
    base: &[ProtoNoiseFunctionComponent],
) -> Vec<ChunkNoiseFunctionComponent<'_>> {
    base.iter()
        .map(|component| match component {
            ProtoNoiseFunctionComponent::Dependent(dependent) => {
                ChunkNoiseFunctionComponent::Dependent(dependent)
            }
            ProtoNoiseFunctionComponent::Independent(independent) => {
                ChunkNoiseFunctionComponent::Independent(independent)
            }
            ProtoNoiseFunctionComponent::PassThrough(pass_through) => {
                ChunkNoiseFunctionComponent::PassThrough(pass_through.clone())
            }
            ProtoNoiseFunctionComponent::Beardifier(beardifier) => {
                ChunkNoiseFunctionComponent::Chunk(ChunkSpecificNoiseFunctionComponent::Beardifier(
                    beardifier.clone(),
                ))
            }
            ProtoNoiseFunctionComponent::Wrapper(wrapper) => {
                ChunkNoiseFunctionComponent::PassThrough(PassThrough::new(
                    wrapper.input_index,
                    0.0,
                    0.0,
                ))
            }
        })
        .collect()
}

fn assert_volume_matches(
    stack: &mut [ChunkNoiseFunctionComponent],
    volume: &DensityVolume,
    label: &str,
) {
    let mut actual = DensityBuffer::acquire(volume);
    ChunkNoiseFunctionComponent::sample_volume_from_stack(stack, &mut actual, volume);
    let mut expected = DensityBuffer::acquire(volume);
    volume.fill_with(&mut expected, |pos| {
        ChunkNoiseFunctionComponent::sample_from_stack(stack, pos)
    });
    for (index, (a, b)) in actual.iter().zip(expected.iter()).enumerate() {
        assert!(
            a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()),
            "{label}: volume {volume:?} index {index}: got {a}, expected {b}"
        );
    }
}

fn small_volumes() -> [DensityVolume; 5] {
    [
        DensityVolume::with_block_step(8, 24, 8, 0, -16, 0),
        DensityVolume::new(3, 7, 3, 16, -64, 16, 4, 8, 4),
        DensityVolume::with_block_step(4, 1, 4, -13, 0, 21),
        DensityVolume::with_block_step(1, 40, 1, 7, 60, 3),
        DensityVolume::new(2, 3, 2, -13, 100, 5, 1, 4, 16),
    ]
}

#[test]
fn every_component_samples_volumes_like_values() {
    let random_config = GlobalRandomConfig::new(42, false);
    let proto_routers = ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);

    for (name, base) in [
        ("noise", &proto_routers.noise.full_component_stack),
        (
            "surface",
            &proto_routers.surface_estimator.full_component_stack,
        ),
        (
            "multi_noise",
            &proto_routers.multi_noise.full_component_stack,
        ),
    ] {
        let mut stack = pass_through_stack(base);
        for index in 0..stack.len() {
            for volume in &small_volumes() {
                assert_volume_matches(
                    &mut stack[..=index],
                    volume,
                    &format!("{name} component {index}"),
                );
            }
        }
    }
}

#[test]
fn chunk_router_samples_volumes_like_values() {
    let random_config = GlobalRandomConfig::new(42, false);
    let proto_routers = ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);
    let builder_options = ChunkNoiseFunctionBuilderOptions::new(Vec::new(), Vec::new(), None);
    let mut router = ChunkNoiseRouter::generate(&proto_routers.noise, &builder_options);
    let chunk = DensityVolume::with_block_step(16, 384, 16, 0, -64, 0);
    let cells = DensityVolume::new(5, 49, 5, 0, -64, 0, 4, 8, 4);
    let unaligned = DensityVolume::with_block_step(7, 20, 5, -13, -61, 3);
    let column = DensityVolume::with_block_step(1, 384, 1, -35, -64, 18);

    for volume in [chunk, cells, unaligned, column] {
        let mut actual = DensityBuffer::acquire(&volume);
        router.final_density_volume(&mut actual, &volume);
        let mut expected = DensityBuffer::acquire(&volume);
        volume.fill_with(&mut expected, |pos| router.final_density(pos));
        for (index, (a, b)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!(
                a.to_bits() == b.to_bits(),
                "final_density volume {volume:?} index {index}: got {a}, expected {b}"
            );
        }
    }
}
