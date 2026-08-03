//! Dense-grid block-light propagation, the shared workload for the experimental
//! GPU compute path.
//!
//! [`LightVolume::propagate_cpu`] ports the block-light rule from
//! [`crate::lighting::engine::LightPropagator::propagate`] onto a flat 3D array:
//! same BFS, same falloff, same `visited`/`skip_direction` early exits. It is
//! deliberately not the shipping call path, since the live engine works through
//! `Cache`, whose per-chunk nibble arrays sit behind mutexes.

use std::collections::VecDeque;

/// Per-voxel input properties for a light volume.
#[derive(Clone, Copy, Default)]
pub struct VoxelProps {
    /// Levels absorbed when light enters this voxel (0..=15).
    pub opacity: u8,
    /// Light emitted by this voxel (0..=15).
    pub luminance: u8,
}

/// The six block directions in `BlockDirection::all()` order. `i ^ 1` is the opposite.
const NEIGHBORS: [(i32, i32, i32); 6] = [
    (0, -1, 0),
    (0, 1, 0),
    (0, 0, -1),
    (0, 0, 1),
    (-1, 0, 0),
    (1, 0, 0),
];

/// A dense cuboid of voxels carrying block-light state.
pub struct LightVolume {
    pub size_x: u32,
    pub size_y: u32,
    pub size_z: u32,
    /// One entry per voxel: bits 0..4 opacity, bits 4..8 luminance.
    pub props: Vec<u32>,
    /// Resulting light levels, one per voxel.
    pub light: Vec<u8>,
}

impl LightVolume {
    /// Creates an unlit volume of the given dimensions.
    #[must_use]
    pub fn new(size_x: u32, size_y: u32, size_z: u32, props: &[VoxelProps]) -> Self {
        let total = (size_x as usize) * (size_y as usize) * (size_z as usize);
        assert_eq!(props.len(), total, "props length must match volume size");
        let packed = props
            .iter()
            .map(|p| u32::from(p.opacity.min(15)) | (u32::from(p.luminance.min(15)) << 4))
            .collect();
        Self {
            size_x,
            size_y,
            size_z,
            props: packed,
            light: vec![0; total],
        }
    }

    #[must_use]
    pub const fn voxel_count(&self) -> usize {
        (self.size_x as usize) * (self.size_y as usize) * (self.size_z as usize)
    }

    #[must_use]
    const fn index(&self, x: u32, y: u32, z: u32) -> usize {
        ((y as usize) * (self.size_z as usize) + (z as usize)) * (self.size_x as usize)
            + (x as usize)
    }

    const fn coords(&self, idx: usize) -> (u32, u32, u32) {
        let sx = self.size_x as usize;
        let sz = self.size_z as usize;
        #[expect(clippy::cast_possible_truncation)]
        {
            (
                (idx % sx) as u32,
                (idx / (sx * sz)) as u32,
                (idx / sx % sz) as u32,
            )
        }
    }

    #[must_use]
    fn opacity_at(&self, idx: usize) -> u8 {
        (self.props[idx] & 15) as u8
    }

    #[must_use]
    fn luminance_at(&self, idx: usize) -> u8 {
        ((self.props[idx] >> 4) & 15) as u8
    }

    const fn neighbor(&self, idx: usize, dir: usize) -> Option<usize> {
        let (x, y, z) = self.coords(idx);
        let (dx, dy, dz) = NEIGHBORS[dir];
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        let nz = z as i32 + dz;
        if nx < 0
            || ny < 0
            || nz < 0
            || nx >= self.size_x as i32
            || ny >= self.size_y as i32
            || nz >= self.size_z as i32
        {
            return None;
        }
        #[expect(clippy::cast_sign_loss)]
        Some(self.index(nx as u32, ny as u32, nz as u32))
    }

    /// Clears all computed light back to zero.
    pub fn reset_light(&mut self) {
        self.light.fill(0);
    }

    /// CPU block-light propagation - a direct port of the engine's BFS.
    pub fn propagate_cpu(&mut self) {
        let total = self.voxel_count();
        let mut visited = vec![false; total];
        let mut queue: VecDeque<(usize, u8)> = VecDeque::with_capacity(4096);

        for (idx, seen) in visited.iter_mut().enumerate() {
            let emission = ((self.props[idx] >> 4) & 15) as u8;
            if emission > 0 {
                self.light[idx] = self.light[idx].max(emission);
                *seen = true;
                // 6 means nothing to skip, mirroring `skip_direction: None`.
                queue.push_back((idx, 6));
            }
        }

        while let Some((idx, skip)) = queue.pop_front() {
            let current_light = self.light[idx];
            if current_light <= 1 {
                continue;
            }
            for dir in 0..6 {
                if dir == skip as usize {
                    continue;
                }
                let Some(nidx) = self.neighbor(idx, dir) else {
                    continue;
                };
                if visited[nidx] {
                    continue;
                }
                let new_level = current_light.saturating_sub(self.opacity_at(nidx).max(1));
                if new_level > self.light[nidx] {
                    self.light[nidx] = new_level;
                    if new_level > 1 {
                        visited[nidx] = true;
                        #[expect(clippy::cast_possible_truncation)]
                        queue.push_back((nidx, (dir ^ 1) as u8));
                    }
                }
            }
        }
    }

    /// Ground truth: relax to the fixed point
    /// `light[v] = max(luminance[v], max_n(light[n]) - max(opacity[v], 1))`.
    /// Slow, but order-independent, so it validates both the BFS and the shader.
    pub fn propagate_reference(&mut self) {
        let total = self.voxel_count();
        for idx in 0..total {
            self.light[idx] = self.luminance_at(idx);
        }
        loop {
            let mut changed = false;
            for idx in 0..total {
                let falloff = self.opacity_at(idx).max(1);
                let mut best = self.light[idx];
                for dir in 0..6 {
                    if let Some(nidx) = self.neighbor(idx, dir) {
                        best = best.max(self.light[nidx].saturating_sub(falloff));
                    }
                }
                if best > self.light[idx] {
                    self.light[idx] = best;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{LightVolume, VoxelProps};

    fn empty_props(n: usize) -> Vec<VoxelProps> {
        vec![VoxelProps::default(); n]
    }

    #[test]
    fn single_source_matches_reference() {
        let (sx, sy, sz) = (16u32, 16u32, 16u32);
        let mut props = empty_props((sx * sy * sz) as usize);
        let center = ((8 * sz as usize) + 8) * sx as usize + 8;
        props[center].luminance = 15;

        let mut cpu = LightVolume::new(sx, sy, sz, &props);
        cpu.propagate_cpu();
        let mut reference = LightVolume::new(sx, sy, sz, &props);
        reference.propagate_reference();

        assert_eq!(cpu.light, reference.light);
        assert_eq!(cpu.light[center], 15);
    }

    #[test]
    fn mixed_luminance_under_lights_vs_reference() {
        // engine.rs skips an already-`visited` neighbour before it can be raised, so
        // a dim source reaching a voxel first locks in a level below the fixed point.
        // Uniform luminance hides this; mixed sources do not. Documented, not fixed:
        // it is a live engine bug and belongs in its own change.
        let (sx, sy, sz) = (24u32, 8u32, 8u32);
        let mut props = empty_props((sx * sy * sz) as usize);
        let idx = |x: usize, y: usize, z: usize| ((y * sz as usize) + z) * sx as usize + x;
        props[idx(4, 4, 4)].luminance = 8;
        props[idx(12, 4, 4)].luminance = 15;
        props[idx(19, 4, 4)].luminance = 12;

        let mut cpu = LightVolume::new(sx, sy, sz, &props);
        cpu.propagate_cpu();
        let mut reference = LightVolume::new(sx, sy, sz, &props);
        reference.propagate_reference();

        assert_ne!(cpu.light, reference.light);
        assert!(
            cpu.light.iter().zip(&reference.light).all(|(c, r)| c <= r),
            "BFS should only ever under-light relative to the fixed point"
        );
    }

    #[test]
    fn opaque_wall_blocks_light() {
        let (sx, sy, sz) = (8u32, 4u32, 4u32);
        let mut props = empty_props((sx * sy * sz) as usize);
        let idx = |x: usize, y: usize, z: usize| ((y * sz as usize) + z) * sx as usize + x;
        props[idx(0, 1, 1)].luminance = 15;
        for y in 0..sy as usize {
            for z in 0..sz as usize {
                props[idx(3, y, z)].opacity = 15;
            }
        }
        let mut cpu = LightVolume::new(sx, sy, sz, &props);
        cpu.propagate_cpu();
        assert_eq!(cpu.light[idx(4, 1, 1)], 0);
        assert!(cpu.light[idx(2, 1, 1)] > 0);
    }
}
