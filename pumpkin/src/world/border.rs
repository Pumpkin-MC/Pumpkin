use pumpkin_protocol::java::client::play::{
    CInitializeWorldBorder, CSetBorderCenter, CSetBorderLerpSize, CSetBorderSize,
    CSetBorderWarningDelay, CSetBorderWarningDistance,
};

use crate::net::java::JavaClient;

use super::World;

pub struct Worldborder {
    pub center_x: f64,
    pub center_z: f64,
    pub old_diameter: f64,
    pub new_diameter: f64,
    /// The actual size used for containment/damage/clamping checks, interpolated
    /// each tick from `old_diameter` toward `new_diameter` (vanilla
    /// `WorldBorder.MovingBorderExtent`). `old_diameter`/`new_diameter` stay pure
    /// lerp endpoints for the client packets, matching vanilla's `from`/`to`.
    current_diameter: f64,
    lerp_ticks_total: i64,
    lerp_ticks_remaining: i64,
    pub speed: i64,
    pub portal_teleport_boundary: i32,
    pub warning_blocks: i32,
    pub warning_time: i32,
    pub damage_per_block: f32,
    pub buffer: f32,
}

impl Worldborder {
    #[must_use]
    pub const fn new(
        x: f64,
        z: f64,
        diameter: f64,
        speed: i64,
        warning_blocks: i32,
        warning_time: i32,
    ) -> Self {
        Self {
            center_x: x,
            center_z: z,
            old_diameter: diameter,
            new_diameter: diameter,
            current_diameter: diameter,
            lerp_ticks_total: 0,
            lerp_ticks_remaining: 0,
            speed,
            portal_teleport_boundary: 29_999_984,
            warning_blocks,
            warning_time,
            damage_per_block: 0.2,
            buffer: 5.0,
        }
    }

    /// Vanilla `WorldBorder.getSize()`: the current (interpolated) diameter.
    #[must_use]
    pub const fn size(&self) -> f64 {
        self.current_diameter
    }

    /// Vanilla `WorldBorder.getLerpTime()`: ticks left in the active interpolation.
    #[must_use]
    pub const fn lerp_time(&self) -> i64 {
        self.lerp_ticks_remaining
    }

    pub async fn init_client(&self, client: &JavaClient) {
        client
            .enqueue_packet(&CInitializeWorldBorder::new(
                self.center_x,
                self.center_z,
                self.current_diameter,
                self.new_diameter,
                self.lerp_ticks_remaining.into(),
                self.portal_teleport_boundary.into(),
                self.warning_blocks.into(),
                self.warning_time.into(),
            ))
            .await;
    }

    pub fn set_center(&mut self, world: &World, x: f64, z: f64) {
        self.center_x = x;
        self.center_z = z;

        world.broadcast_packet_all(&CSetBorderCenter::new(self.center_x, self.center_z));
    }

    pub fn set_diameter(&mut self, world: &World, diameter: f64, speed: Option<i64>) {
        // Vanilla `WorldBorderCommand.setSize` lerps from `border.getSize()`, the
        // current interpolated size, not from the previous lerp target.
        self.old_diameter = self.current_diameter;
        self.new_diameter = diameter;

        // A zero (or negative) tick duration has nothing to interpolate over --
        // vanilla's own `calculateSize` degenerates to `to` immediately in that case
        // (`(duration - progress) / duration` is NaN, which fails the `< 1.0` check).
        if let Some(ticks) = speed.filter(|ticks| *ticks > 0) {
            self.lerp_ticks_total = ticks;
            self.lerp_ticks_remaining = ticks;
            self.current_diameter = self.old_diameter;
            world.broadcast_packet_all(&CSetBorderLerpSize::new(
                self.old_diameter,
                self.new_diameter,
                ticks.into(),
            ));
        } else {
            self.lerp_ticks_total = 0;
            self.lerp_ticks_remaining = 0;
            self.current_diameter = self.new_diameter;
            if speed.is_some() {
                world.broadcast_packet_all(&CSetBorderLerpSize::new(
                    self.old_diameter,
                    self.new_diameter,
                    0i64.into(),
                ));
            } else {
                world.broadcast_packet_all(&CSetBorderSize::new(self.new_diameter));
            }
        }
    }

    pub fn add_diameter(&mut self, world: &World, offset: f64, speed: Option<i64>) {
        self.set_diameter(world, self.current_diameter + offset, speed);
    }

    /// Per-tick lerp update, mirroring vanilla `WorldBorder.MovingBorderExtent::update`.
    /// A no-op once the lerp has completed (`lerp_ticks_remaining == 0`).
    pub fn tick(&mut self, _world: &World) {
        if self.lerp_ticks_remaining > 0 {
            self.lerp_ticks_remaining -= 1;
            self.current_diameter = if self.lerp_ticks_remaining > 0 {
                let progress = (self.lerp_ticks_total - self.lerp_ticks_remaining) as f64
                    / self.lerp_ticks_total as f64;
                self.old_diameter + (self.new_diameter - self.old_diameter) * progress
            } else {
                self.new_diameter
            };
        }
    }

    pub fn set_warning_delay(&mut self, world: &World, delay: i32) {
        self.warning_time = delay;

        world.broadcast_packet_all(&CSetBorderWarningDelay::new(self.warning_time.into()));
    }

    pub fn set_warning_distance(&mut self, world: &World, distance: i32) {
        self.warning_blocks = distance;

        world.broadcast_packet_all(&CSetBorderWarningDistance::new(self.warning_blocks.into()));
    }

    /// `(min_x, max_x, min_z, max_z)`, matching vanilla's `BorderExtent` getters:
    /// the half-diameter offsets from the center, each clamped to
    /// `±absoluteMaxSize` (`WorldBorder.StaticBorderExtent.updateBox`).
    fn bounds(&self) -> (f64, f64, f64, f64) {
        let half = self.current_diameter / 2.0;
        let limit = f64::from(self.portal_teleport_boundary);
        (
            clamp(self.center_x - half, -limit, limit),
            clamp(self.center_x + half, -limit, limit),
            clamp(self.center_z - half, -limit, limit),
            clamp(self.center_z + half, -limit, limit),
        )
    }

    #[must_use]
    pub fn contains(&self, x: f64, z: f64) -> bool {
        let (min_x, max_x, min_z, max_z) = self.bounds();
        x >= min_x && x < max_x && z >= min_z && z < max_z
    }

    /// Vanilla `WorldBorder.isWithinBounds(BlockPos)`, which tests the single
    /// `(getX(), getZ())` pair. The two-corner form belongs to the `ChunkPos`
    /// overload.
    #[must_use]
    pub fn contains_block(&self, x: i32, z: i32) -> bool {
        self.contains(f64::from(x), f64::from(z))
    }

    /// Signed distance from `(x, z)` to the nearest border edge; negative when outside.
    #[must_use]
    pub fn distance_to_border(&self, x: f64, z: f64) -> f64 {
        let (min_x, max_x, min_z, max_z) = self.bounds();

        let from_west = x - min_x;
        let from_east = max_x - x;
        let from_north = z - min_z;
        let from_south = max_z - z;

        from_west.min(from_east).min(from_north).min(from_south)
    }

    /// Vanilla `WorldBorder.clampToBounds`: clamp into
    /// `[min, max - 1.0E-5]` on each axis, then `BlockPos.containing` (floor).
    #[must_use]
    pub fn clamp_block(&self, x: i32, z: i32) -> (i32, i32) {
        let (min_x, max_x, min_z, max_z) = self.bounds();
        (
            clamp(f64::from(x), min_x, max_x - 1.0e-5).floor() as i32,
            clamp(f64::from(z), min_z, max_z - 1.0e-5).floor() as i32,
        )
    }
}

/// `Mth.clamp`: the lower bound wins when the range is inverted, unlike
/// `f64::clamp`, which panics.
fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::Worldborder;

    fn border(center: f64, diameter: f64) -> Worldborder {
        Worldborder::new(center, center, diameter, 0, 5, 300)
    }

    #[test]
    fn contains_block_tests_a_single_point() {
        let border = border(0.0, 16.0);
        // The border spans [-8, 8); block 7 is the last one inside.
        assert!(border.contains_block(7, 7));
        assert!(!border.contains_block(8, 8));
        assert!(border.contains_block(-8, -8));
    }

    #[test]
    fn clamp_block_matches_clamp_to_bounds() {
        let border = border(0.0, 16.0);
        assert_eq!(border.clamp_block(3, -3), (3, -3));
        // floor(clamp(100, -8, 8 - 1e-5)) == 7
        assert_eq!(border.clamp_block(100, 100), (7, 7));
        assert_eq!(border.clamp_block(-100, -100), (-8, -8));
    }

    #[test]
    fn bounds_are_clamped_to_the_absolute_max_size() {
        let border = border(2.0e7, 3.0e7);
        // max = 2e7 + 1.5e7 = 3.5e7, clamped down to 29_999_984.
        assert!(!border.contains(3.0e7, 2.0e7));
        assert!(border.contains(2.999_998_3E7, 2.0e7));
    }
}
