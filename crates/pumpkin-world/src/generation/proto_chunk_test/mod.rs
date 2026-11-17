#[cfg(test)]
pub mod badlands;
#[cfg(test)]
pub mod biome_hashes;
#[cfg(test)]
pub mod dumps;
#[cfg(test)]
pub mod resume;
#[cfg(test)]
pub mod stage_parity;
#[cfg(test)]
pub mod structure_starts;

#[cfg(test)]
pub(crate) fn surface_biomes(
    world_gen: &crate::generation::generator::WorldGenerator,
    center_x: i32,
    center_z: i32,
) -> crate::chunk_system::generation_cache::SurfaceBiomeNeighborhood {
    let crate::generation::generator::WorldGenerator::Noise(generator) = world_gen else {
        unreachable!()
    };
    let mut neighborhood =
        crate::chunk_system::generation_cache::SurfaceBiomeNeighborhood::new(center_x, center_z);
    for chunk_x in center_x - 1..=center_x + 1 {
        for chunk_z in center_z - 1..=center_z + 1 {
            let mut chunk =
                crate::generation::proto_chunk::ProtoChunk::new(chunk_x, chunk_z, world_gen);
            chunk.step_to_biomes(generator);
            assert!(neighborhood.push_chunk(&crate::chunk_system::Chunk::Proto(Box::new(chunk))));
        }
    }
    assert!(neighborhood.is_complete());
    neighborhood
}
