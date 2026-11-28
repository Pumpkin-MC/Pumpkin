use pumpkin_protocol::java::client::play::{
    CInitializeWorldBorder, CSetBorderCenter, CSetBorderLerpSize, CSetBorderSize,
    CSetBorderWarningDelay, CSetBorderWarningDistance,
};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::Entity;
use crate::net::java::JavaClient;

use super::World;

pub struct Worldborder {
    pub center_x: f64,
    pub center_z: f64,
    pub old_diameter: f64,
    pub new_diameter: f64,
    pub speed: i64,
    pub portal_teleport_boundary: i32,
    pub warning_blocks: i32,
    pub warning_time: i32,
    pub damage_per_block: f32,
    pub buffer: f32,
    lerp_ticks_remaining: i64,
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
            speed,
            portal_teleport_boundary: 29_999_984,
            warning_blocks,
            warning_time,
            damage_per_block: 0.0,
            buffer: 0.0,
            lerp_ticks_remaining: 0,
        }
    }

    pub fn init_client(&self, client: &JavaClient) {
        if let Ok(data) = client.serialize_packet(&CInitializeWorldBorder::new(
            self.center_x,
            self.center_z,
            self.current_diameter(),
            self.new_diameter,
            (self.lerp_ticks_remaining * 50).into(),
            self.portal_teleport_boundary.into(),
            self.warning_blocks.into(),
            self.warning_time.into(),
        )) {
            client.try_enqueue_packet(data);
        }
    }

    pub fn set_center(&mut self, world: &World, x: f64, z: f64) {
        self.center_x = x;
        self.center_z = z;

        world.broadcast_packet_all(&CSetBorderCenter::new(self.center_x, self.center_z));
    }

    pub fn set_diameter(&mut self, world: &World, diameter: f64, speed: Option<i64>) {
        self.old_diameter = self.current_diameter();
        self.new_diameter = diameter;
        self.lerp_ticks_remaining = speed.unwrap_or(0).max(0) / 50;
        self.speed = self.lerp_ticks_remaining * 50;
        if self.lerp_ticks_remaining > 0 {
            world.broadcast_packet_all(&CSetBorderLerpSize::new(
                self.old_diameter,
                self.new_diameter,
                self.speed.into(),
            ));
        } else {
            world.broadcast_packet_all(&CSetBorderSize::new(self.new_diameter));
        }
    }

    pub fn add_diameter(&mut self, world: &World, offset: f64, speed: Option<i64>) {
        self.set_diameter(world, self.current_diameter() + offset, speed);
    }

    pub(super) const fn tick(&mut self) {
        if self.lerp_ticks_remaining > 0 {
            self.lerp_ticks_remaining -= 1;
        }
    }

    pub(crate) const fn remaining_lerp_ticks(&self) -> i64 {
        self.lerp_ticks_remaining
    }

    #[must_use]
    pub fn current_diameter(&self) -> f64 {
        self.diameter_at(self.lerp_ticks_remaining)
    }

    fn diameter_at(&self, remaining: i64) -> f64 {
        let duration = self.speed / 50;
        if remaining <= 0 || duration <= 0 {
            return self.new_diameter;
        }
        let progress = (duration - remaining.min(duration)) as f64 / duration as f64;
        self.old_diameter + progress * (self.new_diameter - self.old_diameter)
    }

    fn collision_diameter(&self) -> f64 {
        // Java's border geometry uses a partial-tick value of zero for collisions.
        if self.lerp_ticks_remaining == 0 {
            self.new_diameter
        } else {
            self.diameter_at(self.lerp_ticks_remaining + 1)
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

    pub const fn set_damage_buffer(&mut self, buffer: f32) {
        self.buffer = buffer;
    }

    pub const fn set_damage_per_block(&mut self, damage: f32) {
        self.damage_per_block = damage;
    }

    pub fn reset(&mut self, world: &World) {
        self.center_x = 0.0;
        self.center_z = 0.0;
        self.old_diameter = 29_999_984.0;
        self.new_diameter = 29_999_984.0;
        self.speed = 0;
        self.lerp_ticks_remaining = 0;
        self.portal_teleport_boundary = 29_999_984;
        self.warning_blocks = 5;
        self.warning_time = 15;
        self.damage_per_block = 0.2;
        self.buffer = 5.0;

        world.broadcast_packet_all(&CInitializeWorldBorder::new(
            self.center_x,
            self.center_z,
            self.old_diameter,
            self.new_diameter,
            self.speed.into(),
            self.portal_teleport_boundary.into(),
            self.warning_blocks.into(),
            self.warning_time.into(),
        ));
    }

    #[must_use]
    pub fn get_distance_to_border(&self, x: f64, z: f64) -> f64 {
        let [min_x, max_x, min_z, max_z] = self.limits();
        (x - min_x).min(max_x - x).min(z - min_z).min(max_z - z)
    }

    /// Pulls `x` and `z` just inside the border.
    #[must_use]
    pub fn clamp_to_bounds(&self, position: Vector3<f64>) -> Vector3<f64> {
        let [min_x, max_x, min_z, max_z] = self.limits();
        let inset = f64::from(1.0e-5f32);
        Vector3::new(
            position.x.clamp(min_x, (max_x - inset).max(min_x)),
            position.y,
            position.z.clamp(min_z, (max_z - inset).max(min_z)),
        )
    }

    #[must_use]
    pub fn is_within_bounds(&self, x: f64, z: f64, margin: f64) -> bool {
        let [min_x, max_x, min_z, max_z] = self.limits();
        x >= min_x - margin && x < max_x + margin && z >= min_z - margin && z < max_z + margin
    }

    #[must_use]
    pub fn is_inside_close_to_border(&self, entity: &Entity, bounding_box: &BoundingBox) -> bool {
        let size = (bounding_box.max.x - bounding_box.min.x)
            .abs()
            .max((bounding_box.max.z - bounding_box.min.z).abs())
            .max(1.0);
        let pos = entity.pos.load();
        self.get_distance_to_border(pos.x, pos.z) < size * 2.0
            && self.is_within_bounds(pos.x, pos.z, size)
    }

    #[must_use]
    pub fn collision_shape_contains(&self, bounding_box: &BoundingBox) -> bool {
        let [min_x, max_x, min_z, max_z] = self.collision_limits();
        bounding_box.min.x >= min_x
            && bounding_box.max.x <= max_x
            && bounding_box.min.z >= min_z
            && bounding_box.max.z <= max_z
    }

    fn collision_limits(&self) -> [f64; 4] {
        let [min_x, max_x, min_z, max_z] = self.limits();
        [min_x.floor(), max_x.ceil(), min_z.floor(), max_z.ceil()]
    }

    fn limits(&self) -> [f64; 4] {
        let half = self.collision_diameter() / 2.0;
        let limit = f64::from(self.portal_teleport_boundary);
        [
            (self.center_x - half).clamp(-limit, limit),
            (self.center_x + half).clamp(-limit, limit),
            (self.center_z - half).clamp(-limit, limit),
            (self.center_z + half).clamp(-limit, limit),
        ]
    }

    pub(super) fn collision_walls(&self) -> [BoundingBox; 4] {
        let [min_x, max_x, min_z, max_z] = self.collision_limits();
        let low = f64::NEG_INFINITY;
        let high = f64::INFINITY;
        [
            BoundingBox::new_array([low, low, low], [min_x, high, high]),
            BoundingBox::new_array([max_x, low, low], [high, high, high]),
            BoundingBox::new_array([low, low, low], [high, high, min_z]),
            BoundingBox::new_array([low, low, max_z], [high, high, high]),
        ]
    }

    #[must_use]
    pub fn contains(&self, x: f64, z: f64) -> bool {
        self.is_within_bounds(x, z, 0.0)
    }

    #[must_use]
    pub fn contains_block(&self, x: i32, z: i32) -> bool {
        self.contains(f64::from(x), f64::from(z))
            && self.contains(f64::from(x + 1), f64::from(z + 1))
    }

    #[must_use]
    pub fn clamp_block(&self, x: i32, z: i32) -> (i32, i32) {
        let [min_x, max_x, min_z, max_z] = self.limits();
        // A border narrower than one block spans no block boundary, leaving `max`
        // below `min`. `Ord::clamp` panics on an inverted range, so collapse the
        // range onto the single block that holds the centre instead.
        let min_x = min_x.floor() as i32;
        let max_x = (max_x.floor() as i32 - 1).max(min_x);
        let min_z = min_z.floor() as i32;
        let max_z = (max_z.floor() as i32 - 1).max(min_z);
        (x.clamp(min_x, max_x), z.clamp(min_z, max_z))
    }

    /// Clamps a position into the border on the horizontal axes.
    #[must_use]
    pub fn clamp_to_bound(&self, x: f64, z: f64) -> (f64, f64) {
        // Unlike `f64::clamp`, an inverted range does not panic.
        let clamp = |value: f64, min: f64, max: f64| if value < min { min } else { value.min(max) };
        let [min_x, max_x, min_z, max_z] = self.limits();
        let margin = f64::from(1.0E-5f32);
        (
            clamp(x, min_x, (max_x - margin).max(min_x)),
            clamp(z, min_z, (max_z - margin).max(min_z)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{BoundingBox, Worldborder};

    fn centered_border(diameter: f64) -> Worldborder {
        Worldborder::new(0.0, 0.0, diameter, 0, 5, 300)
    }

    /// `find_safe_location` only falls through to the `clamp_block` fallback
    /// because a zero-width border rejects every candidate it scans: `contains`
    /// reduces to `x >= center && x < center`, which no coordinate satisfies.
    #[test]
    fn a_zero_width_border_contains_no_block() {
        let border = centered_border(0.0);

        assert!(!border.contains_block(0, 0));
        assert!(
            (-32..=32).all(|x| (-32..=32).all(|z| !border.contains_block(x, z))),
            "a zero-width border should contain no block in the portal search area"
        );
    }

    /// `/worldborder set 0` is accepted, because the command's consumer is
    /// `BoundedNumArgumentConsumer::new().min(0.0)` and that bound is inclusive.
    #[test]
    fn clamp_block_handles_a_zero_width_border() {
        let border = centered_border(0.0);

        assert_eq!(border.clamp_block(0, 0), (0, 0));
        assert_eq!(border.clamp_block(100, -100), (0, 0));
    }

    /// A centre off the block grid puts both edges inside the same block, so the
    /// border spans no block boundary even though its diameter is not zero.
    #[test]
    fn clamp_block_handles_a_border_narrower_than_one_block() {
        let border = Worldborder::new(0.5, 0.5, 0.5, 0, 5, 300);

        assert_eq!(border.clamp_block(100, -100), (0, 0));
    }

    #[test]
    fn clamp_block_still_clamps_a_normal_border() {
        let border = centered_border(10.0);

        assert_eq!(border.clamp_block(100, 100), (4, 4));
        assert_eq!(border.clamp_block(-100, -100), (-5, -5));
        assert_eq!(border.clamp_block(2, -3), (2, -3));
    }

    #[test]
    fn collision_walls_respect_the_absolute_border_limit() {
        let border = Worldborder::new(25_000_000.0, 0.0, 20_000_000.0, 0, 5, 300);
        let inside = BoundingBox::new_array([29_999_982.0, 64.0, 0.0], [29_999_984.0, 65.0, 1.0]);
        let crossing = BoundingBox::new_array([29_999_983.0, 64.0, 0.0], [29_999_985.0, 65.0, 1.0]);
        assert!(border.collision_shape_contains(&inside));
        assert!(!border.collision_shape_contains(&crossing));
        assert_eq!(border.collision_walls()[1].min.x, 29_999_984.0);
    }
}
