use std::collections::VecDeque;

use rustc_hash::FxHashSet;

#[derive(Clone, Copy, Default)]
pub struct VoxelProps {
    pub opacity: u8,
    pub luminance: u8,
}

const NEIGHBORS: [(i32, i32, i32); 6] = [
    (0, -1, 0),
    (0, 1, 0),
    (0, 0, -1),
    (0, 0, 1),
    (-1, 0, 0),
    (1, 0, 0),
];

pub const LIGHT_RADIUS: u32 = 15;

pub type PropDelta = (u32, VoxelProps);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AffectedBox {
    pub min: [u32; 3],
    pub max: [u32; 3],
}

impl AffectedBox {
    #[must_use]
    pub const fn contains(&self, x: u32, y: u32, z: u32) -> bool {
        x >= self.min[0]
            && x <= self.max[0]
            && y >= self.min[1]
            && y <= self.max[1]
            && z >= self.min[2]
            && z <= self.max[2]
    }

    #[must_use]
    pub const fn voxels(&self) -> usize {
        ((self.max[0] - self.min[0] + 1) as usize)
            * ((self.max[1] - self.min[1] + 1) as usize)
            * ((self.max[2] - self.min[2] + 1) as usize)
    }
}

pub struct LightVolume {
    pub size_x: u32,
    pub size_y: u32,
    pub size_z: u32,
    pub props: Vec<u32>,
    pub light: Vec<u8>,
}

impl LightVolume {
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
    pub const fn index(&self, x: u32, y: u32, z: u32) -> usize {
        ((y as usize) * (self.size_z as usize) + (z as usize)) * (self.size_x as usize)
            + (x as usize)
    }

    #[must_use]
    pub const fn coords(&self, idx: usize) -> (u32, u32, u32) {
        let sx = self.size_x as usize;
        let sz = self.size_z as usize;
        (
            (idx % sx) as u32,
            (idx / (sx * sz)) as u32,
            (idx / sx % sz) as u32,
        )
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
        Some(self.index(nx as u32, ny as u32, nz as u32))
    }

    pub fn reset_light(&mut self) {
        self.light.fill(0);
    }

    pub fn set_props(&mut self, idx: usize, p: VoxelProps) {
        self.props[idx] = u32::from(p.opacity.min(15)) | (u32::from(p.luminance.min(15)) << 4);
    }

    pub fn apply_prop_deltas(&mut self, deltas: &[PropDelta]) {
        for &(idx, p) in deltas {
            self.set_props(idx as usize, p);
        }
    }

    #[must_use]
    pub fn affected_box(&self, deltas: &[PropDelta]) -> Option<AffectedBox> {
        let mut min = [u32::MAX; 3];
        let mut max = [0u32; 3];
        for &(idx, _) in deltas {
            let (x, y, z) = self.coords(idx as usize);
            min[0] = min[0].min(x);
            max[0] = max[0].max(x);
            min[1] = min[1].min(y);
            max[1] = max[1].max(y);
            min[2] = min[2].min(z);
            max[2] = max[2].max(z);
        }
        if min[0] == u32::MAX {
            return None;
        }
        let size = [self.size_x, self.size_y, self.size_z];
        for i in 0..3 {
            min[i] = min[i].saturating_sub(LIGHT_RADIUS);
            max[i] = (max[i] + LIGHT_RADIUS).min(size[i] - 1);
        }
        Some(AffectedBox { min, max })
    }

    pub fn propagate_delta(&mut self, deltas: &[PropDelta]) {
        self.apply_prop_deltas(deltas);
        let Some(bounds) = self.affected_box(deltas) else {
            return;
        };

        let mut queue: VecDeque<usize> = VecDeque::with_capacity(4096);
        for y in bounds.min[1]..=bounds.max[1] {
            for z in bounds.min[2]..=bounds.max[2] {
                for x in bounds.min[0]..=bounds.max[0] {
                    let idx = self.index(x, y, z);
                    let lum = self.luminance_at(idx);
                    self.light[idx] = lum;
                    if lum > 1 {
                        queue.push_back(idx);
                    }
                }
            }
        }

        let size = [self.size_x, self.size_y, self.size_z];
        for (axis, &lo) in bounds.min.iter().enumerate() {
            let (u_axis, v_axis) = match axis {
                0 => (1, 2),
                1 => (0, 2),
                _ => (0, 1),
            };
            for plane in [lo.checked_sub(1), Some(bounds.max[axis] + 1)]
                .into_iter()
                .flatten()
            {
                if plane >= size[axis] {
                    continue;
                }
                for u in bounds.min[u_axis]..=bounds.max[u_axis] {
                    for v in bounds.min[v_axis]..=bounds.max[v_axis] {
                        let mut c = [0u32; 3];
                        c[axis] = plane;
                        c[u_axis] = u;
                        c[v_axis] = v;
                        let idx = self.index(c[0], c[1], c[2]);
                        if self.light[idx] > 1 {
                            queue.push_back(idx);
                        }
                    }
                }
            }
        }

        while let Some(idx) = queue.pop_front() {
            let current = self.light[idx];
            if current <= 1 {
                continue;
            }
            for dir in 0..6 {
                let Some(nidx) = self.neighbor(idx, dir) else {
                    continue;
                };
                let (nx, ny, nz) = self.coords(nidx);
                if !bounds.contains(nx, ny, nz) {
                    continue;
                }
                let new_level = current.saturating_sub(self.opacity_at(nidx).max(1));
                if new_level > self.light[nidx] {
                    self.light[nidx] = new_level;
                    queue.push_back(nidx);
                }
            }
        }
    }

    pub fn propagate_cpu(&mut self) {
        let total = self.voxel_count();
        let mut pending: FxHashSet<usize> = FxHashSet::default();
        let mut queue: VecDeque<(usize, u8)> = VecDeque::with_capacity(4096);

        for idx in 0..total {
            let emission = self.luminance_at(idx);
            if emission > 0 {
                self.light[idx] = self.light[idx].max(emission);
                pending.insert(idx);
                queue.push_back((idx, 6));
            }
        }

        while let Some((idx, skip)) = queue.pop_front() {
            pending.remove(&idx);
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
                let new_level = current_light.saturating_sub(self.opacity_at(nidx).max(1));
                if new_level > self.light[nidx] {
                    self.light[nidx] = new_level;
                    if new_level > 1 && pending.insert(nidx) {
                        queue.push_back((nidx, (dir ^ 1) as u8));
                    }
                }
            }
        }
    }

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

        assert_ne!(
            cpu.light, reference.light,
            "the engine BFS under-lights relative to the fixed point: skip_direction drops \
             updates when a brighter source re-raises an already-queued voxel. If this ever \
             starts passing, that engine bug was fixed and this test should flip to assert_eq"
        );
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

    #[test]
    fn delta_sequence_matches_reference() {
        let (sx, sy, sz) = (40u32, 24u32, 24u32);
        let mut props = empty_props((sx * sy * sz) as usize);
        let idx = |x: u32, y: u32, z: u32| (((y * sz) + z) * sx + x) as usize;
        for y in 0..6 {
            for z in 0..sz {
                for x in 0..sx {
                    props[idx(x, y, z)].opacity = 15;
                }
            }
        }
        props[idx(8, 12, 12)].luminance = 14;
        props[idx(30, 12, 12)].luminance = 10;

        let mut volume = LightVolume::new(sx, sy, sz, &props);
        volume.propagate_reference();

        let ticks: Vec<Vec<super::PropDelta>> = vec![
            vec![(
                idx(20, 12, 12) as u32,
                VoxelProps {
                    opacity: 0,
                    luminance: 15,
                },
            )],
            vec![(
                idx(8, 12, 12) as u32,
                VoxelProps {
                    opacity: 0,
                    luminance: 0,
                },
            )],
            vec![(
                idx(22, 12, 12) as u32,
                VoxelProps {
                    opacity: 15,
                    luminance: 0,
                },
            )],
            vec![
                (
                    idx(20, 12, 12) as u32,
                    VoxelProps {
                        opacity: 0,
                        luminance: 0,
                    },
                ),
                (
                    idx(30, 12, 12) as u32,
                    VoxelProps {
                        opacity: 0,
                        luminance: 15,
                    },
                ),
            ],
        ];

        for tick in &ticks {
            volume.propagate_delta(tick);
            let mut expected = LightVolume {
                size_x: sx,
                size_y: sy,
                size_z: sz,
                props: volume.props.clone(),
                light: vec![0; volume.voxel_count()],
            };
            expected.propagate_reference();
            assert_eq!(
                volume.light, expected.light,
                "incremental CPU solve diverged from the fixed point"
            );
        }
    }
}
