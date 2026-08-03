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

    pub async fn init_client(&self, client: &JavaClient) {
        client
            .enqueue_packet(&CInitializeWorldBorder::new(
                self.center_x,
                self.center_z,
                self.old_diameter,
                self.new_diameter,
                self.speed.into(),
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
        self.old_diameter = self.new_diameter;
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
        self.set_diameter(world, self.new_diameter + offset, speed);
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

    #[must_use]
    pub fn contains(&self, x: f64, z: f64) -> bool {
        let half = self.current_diameter / 2.0;
        let min_x = self.center_x - half;
        let max_x = self.center_x + half;
        let min_z = self.center_z - half;
        let max_z = self.center_z + half;
        x >= min_x && x < max_x && z >= min_z && z < max_z
    }

    #[must_use]
    pub fn contains_block(&self, x: i32, z: i32) -> bool {
        self.contains(f64::from(x), f64::from(z))
            && self.contains(f64::from(x + 1), f64::from(z + 1))
    }

    /// Signed distance from `(x, z)` to the nearest border edge; negative when outside.
    #[must_use]
    pub fn distance_to_border(&self, x: f64, z: f64) -> f64 {
        let half = self.current_diameter / 2.0;
        let min_x = self.center_x - half;
        let max_x = self.center_x + half;
        let min_z = self.center_z - half;
        let max_z = self.center_z + half;

        let from_west = x - min_x;
        let from_east = max_x - x;
        let from_north = z - min_z;
        let from_south = max_z - z;

        from_west.min(from_east).min(from_north).min(from_south)
    }

    #[must_use]
    pub fn clamp_block(&self, x: i32, z: i32) -> (i32, i32) {
        let half = self.current_diameter / 2.0;
        let min_x = (self.center_x - half).floor() as i32 - 1;
        let max_x = (self.center_x + half).floor() as i32 - 1;
        let min_z = (self.center_z - half).floor() as i32;
        let max_z = (self.center_z + half).floor() as i32 - 1;
        (x.clamp(min_x, max_x), z.clamp(min_z, max_z))
    }
}
